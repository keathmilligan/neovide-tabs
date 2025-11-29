# Tasks: Add Basic Tab Support

## 1. Tab Data Structures

- [x] 1.1 Create `Tab` struct with `id: usize` and `process: NeovideProcess` fields
- [x] 1.2 Create `TabManager` struct with `tabs: Vec<Tab>`, `selected_index: usize`, `next_id: usize`
- [x] 1.3 Create `DragState` struct for tab reordering (`tab_index`, `start_x`, `current_x`)
- [x] 1.4 Add `TabManager` to `WindowState` (replacing single `NeovideProcess`)
- [x] 1.5 Add unit tests for `TabManager` basic operations (add, remove, select, reorder)

## 2. Tab Bar Layout and Constants

- [x] 2.1 Define tab bar layout constants (`TAB_WIDTH`, `TAB_CLOSE_SIZE`, `NEW_TAB_BUTTON_WIDTH`)
- [x] 2.2 Define tab bar color constants (selected, unselected, outline, hover)
- [x] 2.3 Create `get_tab_rects()` function to calculate tab positions based on tab count
- [x] 2.4 Create `get_new_tab_button_rect()` function
- [x] 2.5 Add unit tests for layout calculations

## 3. Tab Bar Hit Testing

- [x] 3.1 Create `TabHitResult` enum (`Tab(index)`, `TabClose(index)`, `NewTabButton`, `Caption`, `None`)
- [x] 3.2 Create `hit_test_tab_bar()` function returning `TabHitResult`
- [x] 3.3 Update `WM_NCHITTEST` handler to use tab bar hit testing
- [x] 3.4 Add unit tests for hit testing various positions

## 4. Tab Bar Rendering

- [x] 4.1 Create `paint_tab()` function to render individual tab with label and close button
- [x] 4.2 Create `paint_new_tab_button()` function to render the (+) button
- [x] 4.3 Create `paint_tab_bar()` function orchestrating full tab bar rendering
- [x] 4.4 Add subtle outline rendering around tabs and content area
- [x] 4.5 Update `paint_titlebar()` to call `paint_tab_bar()`
- [x] 4.6 Implement hover states for tabs and close buttons

## 5. Tab Creation

- [x] 5.1 Implement `TabManager::create_tab()` method (spawns Neovide, adds tab, selects it)
- [x] 5.2 Handle new tab button click in window procedure
- [x] 5.3 Update initial window creation to use `TabManager` with one tab
- [x] 5.4 Handle Neovide spawn failure during tab creation (show error, cancel creation)

## 6. Tab Selection

- [x] 6.1 Implement `TabManager::select_tab(index)` method
- [x] 6.2 Implement `show_selected_tab()` to show selected Neovide, hide others
- [x] 6.3 Handle tab click in window procedure (call `select_tab`, `show_selected_tab`)
- [x] 6.4 Update `WM_ACTIVATE` to bring selected tab's Neovide to foreground

## 7. Tab Closing

- [x] 7.1 Implement `TabManager::close_tab(index)` method (terminates process, removes tab, updates selection)
- [x] 7.2 Handle tab close button click in window procedure
- [x] 7.3 Implement "close last tab" behavior (close application)
- [x] 7.4 Update `WM_CLOSE` to terminate all tabs via `TabManager`

## 8. Tab Reordering (Drag and Drop)

- [x] 8.1 Implement drag detection on `WM_LBUTTONDOWN` in tab area
- [x] 8.2 Implement drag tracking on `WM_MOUSEMOVE`
- [x] 8.3 Implement drag visual feedback (highlight drop position or floating tab)
- [x] 8.4 Implement drop on `WM_LBUTTONUP` (reorder tabs vector)
- [x] 8.5 Implement drag cancellation (mouse leaves tab bar)
- [x] 8.6 Implement `TabManager::move_tab(from_index, to_index)` method

## 9. Process Lifecycle Updates

- [x] 9.1 Update `NeovideProcess` to support `show()` and `hide()` methods using `ShowWindow`
- [x] 9.2 Implement process crash detection per-tab (monitor each tab's process)
- [x] 9.3 Handle individual process crash (remove tab, select next, show error if last)
- [x] 9.4 Update position update logic to apply to all tabs' Neovide windows

## 10. Integration and Polish

- [x] 10.1 Verify window resize correctly updates all Neovide instances
- [x] 10.2 Verify focus synchronization works with multiple tabs
- [x] 10.3 Test with 5+ tabs to verify layout and performance
- [x] 10.4 Update debug `list-windows` command if needed
- [x] 10.5 Run `cargo clippy` and `cargo fmt`, fix any issues
- [x] 10.6 Run `cargo test`, ensure all tests pass
- [x] 10.7 Manual testing on Windows 11

## 11. Documentation

- [x] 11.1 Update README.md roadmap to mark "Tab bar UI" and "Tab creation and management" as complete
- [x] 11.2 Update README.md usage section to describe tab functionality
