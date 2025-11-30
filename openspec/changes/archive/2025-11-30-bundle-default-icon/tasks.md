## 1. Update Configuration Constants and Paths
- [x] 1.1 Change `DEFAULT_ICON` constant from `neovide.png` to `neovide-tabs.png` in `src/config.rs`
- [x] 1.2 Add function `data_dir_path()` to return `~/.local/share/neovide-tabs/` path
- [x] 1.3 Remove `icons_dir_path()` function (no longer needed)

## 2. Bundle and Extract Icon
- [x] 2.1 Add `include_bytes!` to embed `neovide-tabs.png` at compile time in `src/icons.rs`
- [x] 2.2 Create `ensure_default_icon_extracted()` function that:
  - Creates `~/.local/share/neovide-tabs/` directory if it doesn't exist
  - Writes the bundled icon bytes to `neovide-tabs.png` if the file doesn't exist
- [x] 2.3 Call `ensure_default_icon_extracted()` early in `main()` before config loading

## 3. Update Icon Loading Logic
- [x] 3.1 Modify `IconCache` to handle two icon types:
  - Default icon: loaded from `~/.local/share/neovide-tabs/neovide-tabs.png`
  - User icons: loaded from full path specified in config
- [x] 3.2 Update `IconCache::load_icon()` to treat the icon string as a full path for user-defined icons
- [x] 3.3 Add special handling for the default icon filename to load from data directory
- [x] 3.4 Remove icons directory logic from `IconCache::new()`

## 4. Set Application Window Icon
- [x] 4.1 Load the bundled icon and convert to HICON format for window icon
- [x] 4.2 Set window icon in `register_window_class()` using the `hIcon` field in `WNDCLASSW`
- [x] 4.3 Optionally set small icon (16x16) for taskbar using `WM_SETICON`

## 5. Testing and Validation
- [x] 5.1 Verify icon is extracted on first run to `~/.local/share/neovide-tabs/neovide-tabs.png`
- [x] 5.2 Verify icon is NOT overwritten on subsequent runs
- [x] 5.3 Verify tabs display the default icon correctly
- [x] 5.4 Verify user-defined icons work with full paths (e.g., `C:\Users\me\icons\custom.png`)
- [x] 5.5 Verify application window shows the icon in taskbar and Alt-Tab
- [x] 5.6 Run `cargo clippy -- -D warnings` and `cargo fmt -- --check`
