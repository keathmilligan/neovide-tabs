## Context

neovide-tabs loads configuration at startup and stores it in `WindowState`. This includes:
- `background_color` - Used for title bar rendering and resize fill
- `profiles` - Tab profile definitions with icons, working directories, hotkeys, and title formats
- `hotkeys` - Tab switching hotkey mappings

The Win32 window procedure uses this configuration throughout the application lifecycle. Configuration changes currently require an application restart.

## Goals

- Enable live config updates without losing session state (open tabs, positions)
- Apply changes that make sense to update dynamically
- Handle transient invalid states gracefully (e.g., user mid-edit)
- Keep implementation simple and maintainable

## Non-Goals

- Hot-reload of window dimensions or initial position
- Automatic migration of open tabs when profiles are renamed/removed
- Real-time preview while editing (debouncing is acceptable)

## Decisions

### Decision: Use the `notify` crate for file watching
- **What**: Add `notify` as a dependency for cross-platform file system events
- **Why**: It's the de facto standard for file watching in Rust, actively maintained, and handles platform quirks (ReadDirectoryChangesW on Windows)
- **Alternatives**:
  - Raw Win32 `ReadDirectoryChangesW`: More complex, Windows-only, less abstraction
  - Polling: Simple but inefficient, doesn't scale to frequent saves

### Decision: Debounce file change events
- **What**: Wait 250-500ms after the last file change event before reloading
- **Why**: Editors often perform multiple writes when saving (write temp, rename, etc.), and users may save frequently while editing
- **Trade-off**: Slight delay vs. avoiding rapid re-parsing and UI churn

### Decision: Use Windows message queue for cross-thread communication
- **What**: Post a custom `WM_APP` message when config changes are detected
- **Why**: The file watcher runs on a separate thread; Win32 message queue is the standard way to communicate with the UI thread safely
- **Trade-off**: Requires message ID coordination (use `WM_APP + 10` to avoid conflicts with existing dropdown/overflow messages)

### Decision: Preserve open tabs when profiles change
- **What**: Open tabs keep their current profile settings unless explicitly refreshed
- **Why**: Avoids disrupting active work; icon/title changes apply on next title refresh cycle
- **Trade-off**: Tab may show stale profile info until next refresh, but this is acceptable

### Decision: Handle invalid config gracefully
- **What**: If the new config fails to parse, keep the existing config and log a warning
- **Why**: Users often save mid-edit; crashing or showing errors for temporary invalid states is poor UX
- **Trade-off**: User won't see immediate feedback that config is invalid, but existing behavior is preserved

### Decision: Re-register hotkeys on config change
- **What**: Unregister all existing hotkeys and re-register based on new config
- **Why**: Hotkey mappings may have changed (different keys, different profiles, disabled tabs)
- **Trade-off**: Brief period where hotkeys are unavailable during re-registration, but this is imperceptible

## Risks / Trade-offs

- **Risk**: File watcher thread panics or stops
  - Mitigation: Log errors but don't crash; user can restart app if watching fails
- **Risk**: Config file locked by editor during save
  - Mitigation: Retry read with short delay; log warning if still fails
- **Risk**: Hot-reload of background color causes visible flicker
  - Mitigation: Only repaint affected areas; use existing double-buffering

## Open Questions

- Should we show a subtle notification when config is reloaded successfully? (Probably not - silent is better)
- Should we validate profile icon paths exist before updating? (Yes - fall back gracefully)
