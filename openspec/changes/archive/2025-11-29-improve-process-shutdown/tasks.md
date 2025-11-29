## 1. Process Exit Detection Infrastructure

- [x] 1.1 Add process ID tracking to `Tab` struct in `src/tabs.rs` (already tracked via Child handle)
- [x] 1.2 Create a mechanism to check if a Neovide process is still running (poll-based via `try_wait()`)
- [x] 1.3 Add a method to `TabManager` to check all tabs for exited processes and return indices of dead tabs

## 2. Tab Close Button Process Termination

- [x] 2.1 Ensure `close_tab()` in `TabManager` calls `process.terminate()` (verified - already implemented)
- [x] 2.2 Verify the tab close button handler in `WM_LBUTTONDOWN` uses `close_tab()` correctly

## 3. Process Exit Monitoring

- [x] 3.1 Add a Windows timer (`WM_TIMER`) to periodically poll for exited Neovide processes
- [x] 3.2 In the timer handler, check each tab's process status using `try_wait()`
- [x] 3.3 Remove tabs whose processes have exited and repaint the tab bar

## 4. Last Process Exit Handling

- [x] 4.1 After removing dead tabs, check if `tab_manager.is_empty()`
- [x] 4.2 If no tabs remain, post `WM_CLOSE` to shut down the application

## 5. Main Window Close Behavior

- [x] 5.1 Verify `WM_CLOSE` handler calls `terminate_all()` which only affects tracked processes
- [x] 5.2 Ensure only processes spawned by neovide-tabs (tracked via `Child` handles) are terminated

## 6. Testing

- [ ] 6.1 Manual test: Close tab via X button - verify Neovide process is killed
- [ ] 6.2 Manual test: Close Neovide via `:q` - verify tab is removed automatically
- [ ] 6.3 Manual test: Close last Neovide - verify app shuts down
- [ ] 6.4 Manual test: Close main window - verify all spawned Neovide instances are killed
- [ ] 6.5 Manual test: Start Neovide externally - verify it is NOT killed when closing neovide-tabs
