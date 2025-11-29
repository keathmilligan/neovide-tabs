# Tasks: Add Tab Profiles

## 1. Profile Data Model and Configuration
- [x] 1.1 Add `Profile` struct to `src/config.rs` with name, icon path, and working directory fields
- [x] 1.2 Add `profiles` field to `ConfigFile` struct for JSON deserialization
- [x] 1.3 Implement profile parsing logic with defaults (icon defaults to "neovide.png", working_directory defaults to home)
- [x] 1.4 Implement Default profile generation when no profiles exist or no "Default" named profile
- [x] 1.5 Add `profiles` field to `Config` struct storing resolved `Vec<Profile>`
- [x] 1.6 Add helper method to get default profile (first profile or generated default)
- [x] 1.7 Write unit tests for profile parsing, defaults, and Default profile generation

## 2. Icon Loading and Caching
- [ ] 2.1 Create icon loading utility that loads PNG/BMP from `~/.config/neovide-tabs/icons/`
- [ ] 2.2 Implement icon caching to avoid repeated disk reads
- [ ] 2.3 Create fallback icon (embedded or runtime-generated) for missing icon files
- [ ] 2.4 Add icon bitmap to Profile struct or create separate icon cache indexed by filename
- [ ] 2.5 Write unit tests for icon path resolution

## 3. Working Directory Support
- [x] 3.1 Modify `NeovideProcess::spawn()` to accept optional working directory parameter
- [x] 3.2 Add `.current_dir()` call to Neovide Command when working directory is specified
- [x] 3.3 Validate working directory exists, fall back to home directory if not
- [x] 3.4 Write unit tests for working directory resolution

## 4. Tab Profile Association
- [x] 4.1 Add profile reference (name and index) to `Tab` struct in `src/tabs.rs`
- [x] 4.2 Modify `TabManager::create_tab()` to accept profile parameter
- [x] 4.3 Update `get_tab_label()` to return profile name instead of "Tab N"
- [ ] 4.4 Add method to get profile icon for a tab

## 5. Tab Display Updates
- [ ] 5.1 Modify `paint_tab()` in `src/window.rs` to render profile icon
- [ ] 5.2 Adjust tab label positioning to account for icon (icon left, name right)
- [ ] 5.3 Implement icon scaling/rendering using Win32 GDI (16x16 in tab)
- [ ] 5.4 Implement tab tooltip showing profile name and working directory

## 6. Profile Dropdown Button
- [x] 6.1 Add dropdown button constants (width, position relative to + button)
- [x] 6.2 Add `get_profile_dropdown_rect()` function
- [x] 6.3 Add `ProfileDropdown` variant to `HoveredTab` enum
- [x] 6.4 Update `hit_test_tab_bar()` to detect dropdown button hits
- [x] 6.5 Implement dropdown button rendering with caret icon
- [x] 6.6 Add hover state rendering for dropdown button

## 7. Profile Dropdown Menu
- [x] 7.1 Add dropdown menu state to `WindowState` (open/closed, position)
- [x] 7.2 Implement dropdown menu rendering (list of profiles with icons and names)
- [x] 7.3 Calculate dropdown menu position (below the dropdown button)
- [x] 7.4 Implement dropdown menu item hit testing
- [x] 7.5 Handle dropdown menu item selection (create tab with selected profile)
- [x] 7.6 Handle dropdown menu dismissal (click outside)
- [ ] 7.7 Add keyboard navigation for dropdown (optional enhancement)

## 8. Integration and Window State
- [x] 8.1 Pass `Config` (with profiles) to `WindowState` during WM_CREATE
- [x] 8.2 Update initial tab creation to use Default profile
- [x] 8.3 Update + button click handler to use Default profile
- [x] 8.4 Add dropdown button click handler to open dropdown menu
- [x] 8.5 Ensure dropdown menu closes on various events (tab creation, window move, etc.)

## 9. Testing and Documentation
- [ ] 9.1 Manual testing: config with multiple profiles
- [ ] 9.2 Manual testing: config with no profiles (default generation)
- [ ] 9.3 Manual testing: invalid icon paths (fallback behavior)
- [ ] 9.4 Manual testing: invalid working directories (fallback behavior)
- [ ] 9.5 Manual testing: dropdown menu interaction
- [x] 9.6 Run `cargo clippy` and `cargo fmt`
- [x] 9.7 Run `cargo test`
