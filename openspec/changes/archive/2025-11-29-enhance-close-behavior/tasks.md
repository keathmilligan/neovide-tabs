## 1. Implementation

### 1.1 Add request_close method to NeovideProcess
- [x] 1.1.1 Add `request_close()` method to `NeovideProcess` in `src/process.rs`
- [x] 1.1.2 Use `PostMessageW(hwnd, WM_CLOSE, ...)` to send close message to Neovide window
- [x] 1.1.3 Return `true` if window handle exists and message was sent, `false` otherwise
- [x] 1.1.4 Keep `terminate()` method for fallback when window not ready

### 1.2 Update TabManager for graceful close
- [x] 1.2.1 Add `request_close_tab()` method to `TabManager` in `src/tabs.rs`
- [x] 1.2.2 Implementation calls `process.request_close()` to send WM_CLOSE
- [x] 1.2.3 If `request_close()` returns false (window not ready), fall back to existing `close_tab()` behavior
- [x] 1.2.4 Do NOT remove tab in this method - let process polling handle removal when process exits

### 1.3 Add request_close_all method to TabManager
- [x] 1.3.1 Add `request_close_all()` method to `TabManager` in `src/tabs.rs`
- [x] 1.3.2 Iterate all tabs and call `request_close()` on each process
- [x] 1.3.3 For any tab where window is not ready, call `terminate()` as fallback

### 1.4 Update window.rs tab close button handler
- [x] 1.4.1 Update `WM_LBUTTONDOWN` handler for `TabHitResult::TabClose` case
- [x] 1.4.2 Call `request_close_tab()` instead of `close_tab()`
- [x] 1.4.3 Do NOT remove tab or change selection - process polling handles this when process exits
- [x] 1.4.4 Do NOT post WM_CLOSE to app - process polling closes app when last tab's process exits

### 1.5 Update application close handler
- [x] 1.5.1 Modify `WM_CLOSE` handler in `src/window.rs`
- [x] 1.5.2 Call `request_close_all()` to send WM_CLOSE to all Neovide windows
- [x] 1.5.3 Return without destroying window - let process polling detect exits and close app
- [x] 1.5.4 Existing process polling already handles: remove tabs when processes exit, close app when last tab removed

## 2. Testing

- [ ] 2.1 Manual test: Click tab close on a tab with unsaved file, verify save prompt appears
- [ ] 2.2 Manual test: Cancel save prompt, verify tab remains open
- [ ] 2.3 Manual test: Confirm save/discard, verify tab closes
- [ ] 2.4 Manual test: Close app with multiple tabs with unsaved files
- [ ] 2.5 Manual test: Cancel one close prompt, verify app stays open with that tab
- [ ] 2.6 Manual test: Close last tab with unsaved file, cancel, verify app stays open
- [ ] 2.7 Verify existing process exit polling still works (`:q` in Neovide removes tab)
- [ ] 2.8 Manual test: Close tab before Neovide window is ready, verify forced termination works

## 3. Validation

- [x] 3.1 Run `cargo build` to verify compilation
- [x] 3.2 Run `cargo clippy -- -D warnings` to verify no linting issues
- [x] 3.3 Run `cargo test` to verify existing tests pass
