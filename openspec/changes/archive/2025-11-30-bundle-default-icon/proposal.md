# Change: Bundle default tab icon into executable

## Why
The application cannot reliably find the default tab icon (`neovide.png`) because it expects icons in a specific directory that users must manually populate. This creates a poor first-run experience and increases setup friction.

## What Changes
- Bundle `neovide-tabs.png` into the executable using Rust's `include_bytes!` macro
- Extract the bundled icon to `~/.local/share/neovide-tabs/` at runtime if not already present
- Change default icon behavior: the default icon is always loaded from `~/.local/share/neovide-tabs/neovide-tabs.png`
- Change user-defined icon behavior: users must specify a full path to custom icons (not just a filename)
- Remove the `~/.config/neovide-tabs/icons/` directory convention
- Set `neovide-tabs.png` as the application window icon (taskbar/title bar icon)

## Impact
- Affected specs: `app-config`
- Affected code: `src/config.rs`, `src/icons.rs`, `src/window.rs`, `src/main.rs`
- **BREAKING**: User-defined icons now require full paths instead of filenames
- New runtime behavior: Icon extraction on first launch
- New directory created at runtime: `~/.local/share/neovide-tabs/`
- Removed: `icons_dir_path()` function and icons directory convention
