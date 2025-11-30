# Change: Generate Default Config File

## Why
New users currently have no guidance on available configuration options. By generating a commented-out default config file on first run, users can easily discover and customize settings without referring to external documentation.

## What Changes
- On startup, if no config file exists at `~/.config/neovide-tabs/config.json`, generate one with all current defaults shown as comments
- The generated file includes commented-out examples of custom profiles to help users understand the profile structure
- The config directory is created if it does not exist

## Impact
- Affected specs: `app-config`
- Affected code: `src/config.rs` (add generation logic)
