# Change: Enhance Close Behavior for Graceful Neovide Shutdown

## Why

Currently, when the tab close button or application close button is clicked, the Neovide process is terminated immediately using `kill()`. This prevents Neovide from prompting the user to save unsaved files, potentially causing data loss. Users expect the same save-prompt behavior they would get when closing Neovide directly.

## What Changes

- **Tab close button**: Instead of killing the Neovide process, send a close message to the Neovide window (WM_CLOSE). This allows Neovide to handle the close gracefully, prompting to save unsaved files if needed.
- **Application close button**: Instead of immediately terminating all processes, send WM_CLOSE to each Neovide window sequentially. Wait for each Neovide process to exit before proceeding. If the user cancels a close (to save a file), that tab remains open and the application close is cancelled.
- **Close cancellation handling**: If the user cancels the close prompt in Neovide, the Neovide window remains open and the tab stays active. For application close, if any Neovide refuses to close, the entire close operation is cancelled.

## Impact

- **Affected specs**: `window-management` (Process Lifecycle Management, Tab Closing requirements)
- **Affected code**: 
  - `src/tabs.rs`: `close_tab()`, `terminate_all()` methods
  - `src/process.rs`: `terminate()` method, add new `request_close()` method
  - `src/window.rs`: `WM_CLOSE` handler, tab close button handler
- **Breaking changes**: None - this is a behavior improvement that makes close behavior more user-friendly
