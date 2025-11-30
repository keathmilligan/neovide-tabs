# Change: Add Global Hotkey Support

## Why

Users need to quickly switch tabs or open profiles without needing to focus the neovide-tabs window first. Global hotkeys allow keyboard-driven workflow while working in other applications, improving productivity for power users who frequently switch between Neovide instances.

## What Changes

- Add system-wide global hotkey registration using Win32 `RegisterHotKey` API
- Default hotkeys:
  - `Ctrl+Shift+1` through `Ctrl+Shift+0` activate tabs 1-10 (if they exist)
  - Profile hotkeys defined per-profile via `hotkey` field in profile configuration
  - By default, the generated "Default" profile has `Ctrl+Shift+F1` as its hotkey
- Profile hotkeys:
  - Defined in each profile's configuration with a `hotkey` field
  - Only registered for profiles that exist in the configuration
  - When triggered, open a new tab with that profile OR activate an existing tab with that profile
- Tab hotkeys configurable via separate `hotkeys.tab` config section
- Hotkeys work regardless of which application is in the foreground

## Impact

- Affected specs: `app-config`, `window-management`
- Affected code:
  - `src/config.rs` - hotkey configuration parsing, profile hotkey field
  - `src/window.rs` - hotkey registration and WM_HOTKEY handling
  - New module: `src/hotkeys.rs` - hotkey constants and helpers
- Dependencies: Uses existing `windows` crate features (Win32 RegisterHotKey, UnregisterHotKey, WM_HOTKEY)
