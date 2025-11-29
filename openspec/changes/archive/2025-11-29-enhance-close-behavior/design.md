## Context

The current implementation forcefully kills Neovide processes when tabs or the application are closed. This bypasses Neovide's built-in save prompt functionality, which can lead to data loss if users have unsaved work. The change requires coordinating graceful shutdown across the existing window management and process polling systems.

## Goals / Non-Goals

### Goals
- Allow Neovide to prompt users about unsaved files when closing tabs or the application
- Support close cancellation when users choose not to close
- Maintain existing behavior for quick close when no save prompts are needed
- Keep the implementation minimal and leverage existing process polling infrastructure

### Non-Goals
- Adding timeout/force-kill behavior (users should always have control)
- Adding UI feedback during close operations (Neovide provides this)
- Changing the process discovery or window embedding logic

## Decisions

### Decision: Use WM_CLOSE instead of process termination
- **What**: Send `WM_CLOSE` to Neovide windows instead of calling `Child::kill()`
- **Why**: WM_CLOSE is the standard Windows mechanism for graceful window closure. Neovide handles this by checking for unsaved files and prompting the user.
- **Alternative considered**: Sending custom messages or using Neovim RPC to trigger `:qa` - rejected as overly complex and requiring Neovim protocol knowledge.

### Decision: Fully stateless approach using existing process polling
- **What**: After sending WM_CLOSE, rely entirely on the existing 250ms process poll timer to detect when Neovide exits. No additional state tracking is needed.
- **Why**: The process polling infrastructure already handles tab removal and UI updates. When a process exits (for any reason), the tab is removed. When the last tab is removed, the app closes. This naturally handles both successful closes and user cancellations without any special state.
- **How it works**:
  - Tab close button: Send WM_CLOSE to Neovide window, do nothing else. Process polling will remove the tab when/if the process exits.
  - App close button: Send WM_CLOSE to all Neovide windows. If user cancels any, those processes stay running and tabs remain. If all exit, process polling removes all tabs and closes the app.
- **Alternative considered**: Adding `close_in_progress` state flag - rejected because it creates edge cases where the flag could get stuck if the user cancels, and requires timeout/reset logic.

### Decision: Fallback to terminate() when window not ready
- **What**: If the Neovide window handle hasn't been discovered yet, use the existing `terminate()` method
- **Why**: The window discovery happens asynchronously after spawn. During the discovery period, we only have the process handle, so `kill()` is the only option.
- **Alternative considered**: Wait for window discovery before allowing close - rejected as it could leave zombie tabs.

## Risks / Trade-offs

### Risk: Neovide window may not respond to WM_CLOSE
- **Mitigation**: The existing process polling will detect if the process exits for any reason (crash, external kill). Users can always force-close via Task Manager if needed.

### Risk: User confusion if close appears to do nothing
- **Mitigation**: Neovide itself shows the save dialog, providing immediate feedback. The tab remains visible, indicating the close is pending user action.

### Trade-off: No explicit close timeout
- **Accepted**: Users maintain full control. If they walk away with a save dialog open, the app waits. This matches native application behavior.

## Migration Plan

No migration needed - this is a behavior change that improves UX without changing APIs or data formats.

## Open Questions

None - the approach is straightforward and leverages existing Windows messaging patterns.
