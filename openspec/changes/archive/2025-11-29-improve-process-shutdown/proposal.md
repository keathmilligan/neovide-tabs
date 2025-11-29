# Change: Improve Process Management and Shutdown Behavior

## Why

The current implementation lacks proper lifecycle coordination between tabs and their associated Neovide processes. When a Neovide instance exits unexpectedly (user closes it, crash, or normal exit via `:q`), the corresponding tab remains in the UI with no process backing it. Additionally, the tab close button does not terminate the process, and closing the main window should only terminate processes spawned by neovide-tabs.

## What Changes

- Tab close button (x) now kills the associated Neovide process before removing the tab
- Detect when a Neovide process exits (via any means) and automatically remove its tab
- When the last Neovide process exits, automatically close the application
- Main window close button terminates only Neovide instances that were spawned by neovide-tabs (tracked via process ID)
- Process monitoring runs in background to detect unexpected exits

## Impact

- Affected specs: `window-management` (Process Lifecycle Management, Tab Closing requirements)
- Affected code: `src/process.rs` (process monitoring, termination), `src/tabs.rs` (tab removal on process exit), `src/window.rs` (window message handling, close behavior)
