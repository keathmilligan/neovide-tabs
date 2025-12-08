## 1. Configuration

- [x] 1.1 Add `title` field to `ProfileFile` struct in `config.rs` (optional String)
- [x] 1.2 Add `title` field to `Profile` struct with default value `%t`
- [x] 1.3 Update `parse_profiles` to propagate the title setting
- [x] 1.4 Update `DEFAULT_CONFIG_TEMPLATE` to document the `title` option with examples
- [x] 1.5 Add unit tests for profile title parsing

## 2. Title Expansion Engine

- [x] 2.1 Create `expand_title` function that takes a format string and context (profile name, working directory, window title)
- [x] 2.2 Implement `%p` expansion (profile name)
- [x] 2.3 Implement `%w` expansion (working directory with `~` substitution)
- [x] 2.4 Implement `%t` expansion (Neovide window title)
- [x] 2.5 Implement stripping of leading/trailing space, tab, and dash characters
- [x] 2.6 Add unit tests for title expansion (all tokens, edge cases, stripping)

## 3. Window Title Query

- [x] 3.1 Add `get_window_title` method to `NeovideProcess` that queries the current window title via Win32 `GetWindowTextW`
- [x] 3.2 Handle cases where window is not yet ready (return empty string or None)
- [x] 3.3 Add unit test/integration consideration for title query

## 4. Tab Title Integration

- [x] 4.1 Add `title_format` field to `Tab` struct (stores the profile's title setting)
- [x] 4.2 Add `cached_title` field to `Tab` struct (stores the last computed title)
- [x] 4.3 Update `TabManager::create_tab` to set `title_format` from profile
- [x] 4.4 Add `TabManager::update_tab_title` method that computes and caches the expanded title
- [x] 4.5 Update `TabManager::get_tab_label` to return the cached expanded title

## 5. Title Refresh Logic

- [x] 5.1 Call title update when a tab is created
- [x] 5.2 Call title update when a tab is activated (switched to)
- [x] 5.3 Integrate periodic title refresh with existing process polling timer (or add dedicated timer)
- [x] 5.4 Only refresh title for the currently selected (visible) tab to minimize overhead
- [x] 5.5 Trigger tab bar repaint when title changes

## 6. Validation & Testing

- [ ] 6.1 Manual testing: verify `%p`, `%w`, `%t` work correctly
- [ ] 6.2 Manual testing: verify title updates when switching files in Neovim
- [ ] 6.3 Manual testing: verify stripping of leading/trailing characters
- [x] 6.4 Run `cargo clippy` and `cargo fmt`
- [x] 6.5 Run `cargo test`
