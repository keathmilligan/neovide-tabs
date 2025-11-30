# Change: Update initial profile behavior and default config generation

## Why
The generated config file currently shows commented-out example profiles, which means users must manually edit the config to get started with profiles. Additionally, the initial tab always uses the hard-coded "Default" profile regardless of what profiles are defined.

## What Changes
- Generate config.jsonc with an uncommented "Neovim" profile as the first profile
- Remove the automatic insertion of a hard-coded "Default" profile when user profiles exist
- Use the first profile in the config for the initial tab when profiles are defined
- Only fall back to the internal hard-coded "Default" profile when no profiles are defined in an existing config

## Impact
- Affected specs: app-config
- Affected code: `src/config.rs` (DEFAULT_CONFIG_TEMPLATE, parse_profiles, Default Profile Generation logic)
