## 1. Implementation

- [x] 1.1 Update DEFAULT_CONFIG_TEMPLATE to include an uncommented "Neovim" profile as the first profile
- [x] 1.2 Update `parse_profiles()` to only insert "Default" profile when profiles list is empty (not when "Default" is missing from user profiles)
- [x] 1.3 Update `Config::default_profile()` documentation to clarify it returns the first profile
- [x] 1.4 Update tests to reflect new behavior
- [x] 1.5 Run `cargo test` to verify all tests pass
- [x] 1.6 Run `cargo clippy -- -D warnings` to verify no linting issues
