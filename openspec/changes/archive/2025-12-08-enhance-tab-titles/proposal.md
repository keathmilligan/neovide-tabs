# Change: Enhance Tab Titles with Dynamic String Expansion

## Why

Currently, tab titles display only the profile name, which is static and doesn't provide visibility into what the user is working on in each tab. Users want tabs to show contextual information like the current working directory or the Neovide window title (which reflects the active file/buffer in Neovim).

## What Changes

- Add a `title` setting to profile definitions that supports string expansion tokens
- Implement a string expansion mechanism with the following tokens:
  - `%p` - Profile name
  - `%w` - Working directory (displayed in `~/xxx` form for paths under home)
  - `%t` - Neovide window title (queried from the actual Neovide window)
- Query the Neovide window title on tab open, tab activation, and periodically to keep it in sync
- Strip leading/trailing whitespace, tabs, and dash (`-`) characters from the final expanded title
- Default the profile `title` setting to `%t` (Neovide window title)

## Impact

- Affected specs: `app-config` (profile configuration)
- Affected code:
  - `src/config.rs` - Add `title` field to profile definition
  - `src/tabs.rs` - Store and compute expanded tab titles
  - `src/process.rs` - Add method to query current Neovide window title
  - `src/window.rs` - Integrate periodic title refresh and update tab display
