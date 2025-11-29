# Change: Add Configuration Capability

## Why
Users need the ability to customize application behavior, starting with background color to match their preferred Neovim theme and prevent visual jarring (white flash) during window resize operations.

## What Changes
- Add configuration file support: `~/.config/neovide-tabs/config.json` (cross-platform path)
- Load and parse configuration at startup
- Apply background color from config (default: `#1a1b26`)
- Use configured background color as the window class brush to prevent flash during resize

## Impact
- Affected specs: New `app-config` capability
- Affected code: `src/main.rs`, `src/window.rs`, new `src/config.rs`
- Dependencies: Add `serde` and `serde_json` for JSON parsing, `dirs` for cross-platform config path
