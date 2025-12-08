# Change: Add Configuration File Hot-Reload

## Why

Currently, configuration changes require restarting the entire application. Users who want to tweak their config (background color, tab profiles, hotkeys) must close all tabs, edit the config file, and restart neovide-tabs. This interrupts workflow and forces users to lose their current session state.

## What Changes

- Add file system watching for the config file (`config.jsonc` or `config.json`)
- When the config file changes, reload and re-parse the configuration
- Apply relevant changes dynamically without requiring a restart:
  - Update background color for title bar and resize operations
  - Update tab icons and titles for currently-open tabs when their profile changes
  - Re-register global hotkeys when hotkey configuration changes
  - Make new profiles available in the dropdown menu
- Handle edge cases gracefully (file temporarily invalid, profile used by open tab is removed, etc.)

## Impact

- Affected specs: `app-config`
- Affected code:
  - `src/config.rs` - Add file watcher setup and config reload logic
  - `src/window.rs` - Handle config reload events, update UI state
  - `src/hotkeys.rs` - Add hotkey re-registration support
  - `src/tabs.rs` - May need profile refresh logic for open tabs
- Dependencies: Will need a file watching crate (e.g., `notify`)
