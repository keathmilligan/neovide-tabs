## 1. Dependencies
- [x] 1.1 Add `serde`, `serde_json` dependencies to Cargo.toml
- [x] 1.2 Add `dirs` crate for cross-platform home directory resolution

## 2. Configuration Module
- [x] 2.1 Create `src/config.rs` module with `Config` struct
- [x] 2.2 Implement config file path resolution (`~/.config/neovide-tabs/config.json`)
- [x] 2.3 Implement config loading with JSON parsing
- [x] 2.4 Implement hex color parsing (with/without `#` prefix)
- [x] 2.5 Define default values (background_color: `#1a1b26`)

## 3. Window Integration
- [x] 3.1 Load config in `main.rs` before window creation
- [x] 3.2 Pass background color to `register_window_class`
- [x] 3.3 Create solid brush from parsed color for `hbrBackground`
- [x] 3.4 Verify resize no longer flashes white

## 4. Testing
- [x] 4.1 Add unit tests for hex color parsing
- [x] 4.2 Add unit tests for config loading (missing file, invalid JSON, valid config)
- [x] 4.3 Manual test: verify background color applies on startup
- [x] 4.4 Manual test: verify no white flash on resize
