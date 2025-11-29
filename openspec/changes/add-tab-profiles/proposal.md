# Change: Add Tab Profiles

## Why

Users often work in multiple project directories and want quick access to open new tabs with specific working directories and visual identifiers. Currently, all new tabs open with the same default working directory, requiring manual navigation each time.

## What Changes

- Add profile configuration to `config.json` with name, icon (optional, defaults to `neovide.png`), and working directory (optional, defaults to user's home directory)
- Generate a "Default" profile automatically at startup if no profiles are defined
- Use the Default profile for the initial tab on application launch
- Display profile icon and name in each tab (icon sized appropriately for the tab bar)
- Clicking the + button opens a new tab using the Default profile
- Add a dropdown button (downward caret) to the right of the + button that shows all available profiles with their icon and name
- Selecting a profile from the dropdown opens a new tab with that profile's configuration

## Impact

- Affected specs: `app-config`, `window-management`
- Affected code:
  - `src/config.rs` - Profile data structures and loading
  - `src/tabs.rs` - Tab struct to store profile reference
  - `src/window.rs` - Profile dropdown UI, tab rendering with icons/names
  - `src/process.rs` - Working directory support for Neovide spawning
