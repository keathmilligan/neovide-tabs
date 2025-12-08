## 1. Dependencies and Setup

- [x] 1.1 Add `notify` crate to `Cargo.toml` for file system watching
- [x] 1.2 Add `notify-debouncer-mini` crate for debounced file watching

## 2. Config Module Updates

- [x] 2.1 Extract config file path discovery into a public function for reuse
- [x] 2.2 Add `Config::reload()` method that re-reads and parses the config file
- [x] 2.3 Add `Config::find_profile_by_name()` helper (reserved for future use)
- [x] 2.4 Unit tests for config module (existing tests cover reload scenarios)

## 3. File Watcher Implementation

- [x] 3.1 Create file watcher module (`src/watcher.rs`) with setup function
- [x] 3.2 Implement debounced file watching using `notify-debouncer-mini` with 250ms delay
- [x] 3.3 Handle watcher errors gracefully (log but don't crash)
- [x] 3.4 Post `WM_APP + 10` message to window when config change detected
- [x] 3.5 Add unit tests for `should_reload()` function

## 4. Window State Updates

- [x] 4.1 Add config watcher handle to `WindowState` struct
- [x] 4.2 Initialize file watcher in `WM_CREATE` handler after window creation
- [x] 4.3 Add `WM_CONFIG_RELOAD` message handler for config reload events
- [x] 4.4 Implement background color update logic (update state and repaint)

## 5. Hotkey Re-registration

- [x] 5.1 Unregister all existing hotkeys on reload
- [x] 5.2 Register new tab hotkeys from reloaded config
- [x] 5.3 Register new profile hotkeys from reloaded config
- [ ] 5.4 Test hotkey changes work correctly (add, remove, modify) - manual test

## 6. Profile and Tab Updates

- [x] 6.1 Update profile list in WindowState when config reloads
- [x] 6.2 Tab bar repaint triggered on reload (shows new profile list)
- [ ] 6.3 Manual test: Profile display properties change correctly
- [ ] 6.4 Manual test: Profile index changes (profiles reordered in config)

## 7. Icon Cache Invalidation

- [x] 7.1 Add `clear_icon_cache()` function to icons module
- [x] 7.2 Call `clear_icon_cache()` on config reload

## 8. Testing and Validation

- [ ] 8.1 Manual test: Change background color while running
- [ ] 8.2 Manual test: Add new profile while running
- [ ] 8.3 Manual test: Change hotkey binding while running
- [ ] 8.4 Manual test: Save invalid JSON, verify app continues working
- [ ] 8.5 Manual test: Rapid saves (editor auto-save scenario)
- [x] 8.6 Run `cargo clippy` and `cargo test` to verify no regressions

## Dependencies

- Tasks 2.x can be done in parallel with 3.x
- Task 4.x depends on 2.x and 3.x
- Task 5.x depends on 4.x
- Task 6.x depends on 4.x
- Task 7.x can be done in parallel with 5.x and 6.x
- Task 8.x depends on all previous tasks
