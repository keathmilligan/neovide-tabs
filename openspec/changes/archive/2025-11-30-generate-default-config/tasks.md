## 1. Implementation

- [x] 1.1 Add `generate_default_config()` function in `src/config.rs` that creates a JSONC template string
- [x] 1.2 Add `ensure_config_file()` function that checks for config file existence and generates if missing
- [x] 1.3 Create helper function to ensure config directory exists with proper permissions
- [x] 1.4 Call `ensure_config_file()` at the start of `Config::load()`

## 2. Default Config Template

- [x] 2.1 Define the default config template as a multi-line string constant
- [x] 2.2 Include commented `background_color` with default value `#1a1b26`
- [x] 2.3 Include commented `hotkeys.tab` with all default tab hotkey mappings
- [x] 2.4 Include commented `profiles` array with example profiles:
  - Full example with all fields (name, icon, working_directory, hotkey)
  - Minimal example with name only
- [x] 2.5 Add descriptive comments explaining each configuration option

## 3. JSONC Support

- [x] 3.1 Generate config as `.jsonc` file extension (preferred format)
- [x] 3.2 Support both `.jsonc` and `.json` file extensions for loading
- [x] 3.3 Add `strip_jsonc_comments()` function to strip `//` comments before parsing
- [x] 3.4 Update `Config::load()` to use JSONC parsing for both file types
- [x] 3.5 Add `find_config_file()` to discover config file with priority (.jsonc > .json)

## 4. Testing

- [x] 4.1 Add unit test for `generate_default_config()` output format
- [x] 4.2 Add integration test verifying config file is created on first run
- [x] 4.3 Add test verifying existing config files are not overwritten
- [x] 4.4 Add test verifying generated content is valid JSONC (parses after stripping comments)
- [x] 4.5 Add tests for `strip_jsonc_comments()` function
- [x] 4.6 Add tests for comment preservation in strings

## 5. Validation

- [x] 5.1 Run `cargo build` and verify compilation
- [x] 5.2 Run `cargo test` and verify all tests pass
- [x] 5.3 Run `cargo clippy -- -D warnings` and fix any warnings
- [ ] 5.4 Manual test: delete config file, run app, verify config.jsonc is generated
