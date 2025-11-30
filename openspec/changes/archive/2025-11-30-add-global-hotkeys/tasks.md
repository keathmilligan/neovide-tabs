## 1. Configuration Parsing

- [x] 1.1 Add `hotkey` optional field to `ProfileFile` struct for JSON deserialization
- [x] 1.2 Add `hotkey` optional field to `Profile` struct (parsed hotkey string)
- [x] 1.3 Add `HotkeyConfig` struct with `tab` hotkey map to `config.rs`
- [x] 1.4 Add `hotkeys` field to `ConfigFile` struct for tab hotkey configuration
- [x] 1.5 Implement hotkey string parser (e.g., "Ctrl+Shift+1" -> modifiers + virtual key)
- [x] 1.6 Set default hotkey `Ctrl+Shift+F1` for generated Default profile
- [x] 1.7 Add default tab hotkey configuration when no `hotkeys.tab` field is present
- [x] 1.8 Add unit tests for hotkey string parsing (valid formats, edge cases, invalid inputs)
- [x] 1.9 Add unit tests for profile hotkey loading (with/without hotkey field)
- [x] 1.10 Add unit tests for tab hotkey configuration (defaults, custom, empty)

## 2. Hotkey Module

- [x] 2.1 Create `src/hotkeys.rs` module with hotkey constants and ID ranges
- [x] 2.2 Define hotkey ID constants (TAB_BASE = 1, PROFILE_BASE = 101)
- [x] 2.3 Implement `register_hotkey` wrapper function that handles errors gracefully
- [x] 2.4 Implement `unregister_all_hotkeys` function for cleanup
- [x] 2.5 Add function to convert parsed hotkey config to Win32 RegisterHotKey parameters
- [x] 2.6 Add `src/hotkeys.rs` to `mod` declarations in `main.rs`

## 3. Window Integration

- [x] 3.1 Add `registered_hotkeys: Vec<i32>` to `WindowState` to track registered hotkey IDs
- [x] 3.2 Register tab hotkeys in `WM_CREATE` handler (IDs 1-10)
- [x] 3.3 Register profile hotkeys in `WM_CREATE` for each profile with a hotkey (IDs 101+)
- [x] 3.4 Add `WM_HOTKEY` message handler in `window_proc`
- [x] 3.5 Implement tab activation logic for tab hotkey IDs (1-10)
- [x] 3.6 Implement profile activation logic for profile hotkey IDs (101 + profile_index)
- [x] 3.7 Call `UnregisterHotKey` for all registered hotkeys in `WM_DESTROY` handler

## 4. Tab Manager Extensions

- [x] 4.1 Add `find_tab_by_profile_index` method to `TabManager` for profile activation
- [x] 4.2 Ensure `select_tab` properly handles window activation when called from hotkey

## 5. Window Focus Handling

- [x] 5.1 Add helper function to restore and bring wrapper window to foreground
- [x] 5.2 Handle minimized state (restore before activate)
- [x] 5.3 Ensure Neovide window is activated after tab selection

## 6. Testing and Validation

- [ ] 6.1 Test default tab hotkeys work out of box
- [ ] 6.2 Test default profile hotkey (Ctrl+Shift+F1 for Default) works out of box
- [ ] 6.3 Test custom tab hotkey configuration overrides defaults
- [ ] 6.4 Test profile with custom hotkey field registers correctly
- [ ] 6.5 Test profile without hotkey field does not register hotkey
- [ ] 6.6 Test empty `hotkeys.tab` config disables tab hotkeys
- [ ] 6.7 Test hotkey conflict handling (graceful degradation)
- [ ] 6.8 Test profile hotkeys activate existing tab vs create new
- [ ] 6.9 Test tab hotkeys with non-existent tab indices
- [ ] 6.10 Test window focus restoration from background/minimized

## 7. Documentation

- [x] 7.1 Update README with hotkey documentation
- [x] 7.2 Add example config.json showing profile hotkey and tab hotkey configuration
