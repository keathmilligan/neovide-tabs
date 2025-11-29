# Design: Add Basic Tab Support

## Context

The neovide-tabs application currently embeds a single frameless Neovide window within a custom wrapper window. The wrapper has a custom-painted titlebar with minimize/maximize/close buttons. This change extends the titlebar to include a tab bar for managing multiple Neovide instances.

**Stakeholders**: End users who want to manage multiple Neovide instances in a single window

**Constraints**:
- Must integrate with existing custom titlebar painting
- Each tab corresponds to exactly one Neovide subprocess
- Neovide windows are positioned/shown/hidden based on tab selection
- Win32 API-based implementation (Windows only)

## Goals / Non-Goals

**Goals**:
- Enable users to create, close, and switch between multiple Neovide tabs
- Allow users to reorder tabs via drag-and-drop
- Maintain visual consistency with existing titlebar styling
- Ensure clean process lifecycle (each tab's Neovide is properly terminated)

**Non-Goals**:
- Tab persistence across application restarts
- Custom working directories per tab (future enhancement)
- Keyboard shortcuts for tab navigation (future enhancement)
- Tab renaming or custom labels

## Decisions

### Decision 1: Tab State Management

**What**: Create a `TabManager` struct to manage tab state, stored in `WindowState`.

**Why**: Centralizes tab logic (creation, deletion, selection, reordering) and keeps window procedure focused on message handling.

**Structure**:
```rust
struct Tab {
    id: usize,                    // Unique identifier
    process: NeovideProcess,      // Owning the Neovide instance
}

struct TabManager {
    tabs: Vec<Tab>,
    selected_index: usize,
    next_id: usize,               // Counter for unique IDs
    drag_state: Option<DragState>, // For reordering
}
```

**Alternatives Considered**:
- Store tabs directly in `WindowState`: Rejected; violates single responsibility
- Use `HashMap<TabId, Tab>`: Rejected; order matters for display, `Vec` is simpler

### Decision 2: Tab Bar Layout

**What**: Tab bar occupies the left portion of the titlebar, with the new tab (+) button to the right of the last tab, and window buttons remain on the far right.

**Layout**:
```
[Tab1][Tab2][Tab3][+]     [−][□][×]
|--- Tab area ---|       |--Buttons--|
```

**Why**: Follows familiar browser/IDE patterns. Window controls stay in expected position.

**Sizing**:
- Each tab: ~120px width (or dynamic based on available space)
- Close (x) button within each tab: ~16x16px, right-aligned within tab
- New tab (+) button: ~32px wide

### Decision 3: Tab Rendering

**What**: Each tab is rendered with:
- Background fill (darker when not selected, matches titlebar when selected)
- Subtle 1px outline around the tab and content area to distinguish from titlebar
- Tab label (initially "Tab 1", "Tab 2", etc., or "Neovide")
- Small "x" close button on hover or always visible

**Why**: Provides clear visual hierarchy and familiar interaction patterns.

**Colors** (matching existing theme):
- Titlebar background: `0x1a1b26` (from config)
- Selected tab: Same as titlebar or slightly lighter
- Unselected tab: Slightly darker, e.g., `0x16161e`
- Outline color: Subtle gray, e.g., `0x3d3d3d`
- Tab hover: `0x3d3d3d` (reuse existing button hover color)

### Decision 4: Tab Switching Mechanism

**What**: When a tab is selected, show its Neovide window and hide all others.

**Implementation**:
- Use `ShowWindow(hwnd, SW_SHOW)` for selected tab's Neovide
- Use `ShowWindow(hwnd, SW_HIDE)` for all other tabs' Neovide windows
- Call `SetForegroundWindow` on the selected Neovide to ensure focus

**Why**: Simpler than z-order manipulation. Hidden windows don't consume rendering resources.

**Alternatives Considered**:
- Z-order stacking with `SetWindowPos`: More complex, potential visual artifacts
- Moving windows off-screen: Clunky, could cause issues with multi-monitor setups

### Decision 5: Tab Drag-and-Drop Reordering

**What**: Implement mouse-based drag reordering within the tab bar.

**Implementation**:
- On `WM_LBUTTONDOWN` in tab area: Start potential drag, record initial position
- On `WM_MOUSEMOVE` with button held: If moved beyond threshold (~5px), enter drag mode
- During drag: Render visual feedback (highlight drop position or "floating" tab)
- On `WM_LBUTTONUP`: Complete drop, reorder `tabs` vector, repaint

**State**:
```rust
struct DragState {
    tab_index: usize,      // Which tab is being dragged
    start_x: i32,          // Initial mouse X position
    current_x: i32,        // Current mouse X position
}
```

### Decision 6: New Tab Button Behavior

**What**: Clicking the (+) button creates a new tab with a new Neovide instance.

**Behavior**:
1. Calculate content area dimensions (same as current tab)
2. Spawn new `NeovideProcess` with those dimensions
3. Create new `Tab` with unique ID
4. Add to end of `tabs` vector
5. Set as selected tab
6. Hide previous tab's Neovide, show new one
7. Repaint tab bar

### Decision 7: Tab Close Behavior

**What**: Clicking a tab's (x) button closes that tab.

**Behavior**:
1. Terminate the `NeovideProcess` (call `terminate()`)
2. Remove tab from `tabs` vector
3. If closing the selected tab:
   - Select the next tab if available
   - Otherwise select the previous tab
   - If no tabs remain, close the application
4. Update selected tab's Neovide visibility
5. Repaint tab bar

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Complex hit-testing with tabs, close buttons, and drag | Structured hit-test function returning enum variant |
| Memory usage with many tabs | Each Neovide is a separate process; limit is system resources (document in README) |
| Race conditions in tab/process lifecycle | Use synchronous operations in message handlers; Neovide processes managed by TabManager |
| Visual glitches during tab switching | Use `WS_CLIPCHILDREN` if needed; test with smooth transitions |

## Migration Plan

No migration needed. This is a new feature extending existing functionality.

**Rollback**: Remove tab-related code; revert to single-tab behavior.

## Open Questions

1. **Tab labels**: Should tabs show "Neovide" (consistent) or "Tab 1", "Tab 2" (distinguishable)? 
   - Recommendation: Use "Tab 1", "Tab 2", etc. for now. Future enhancement could show working directory.

2. **Maximum tabs**: Should we enforce a limit?
   - Recommendation: No artificial limit; system resources are the natural constraint.

3. **Close last tab behavior**: Should closing the last tab close the app or show an empty state?
   - Recommendation: Close the app (simpler, matches browser behavior when closing last tab).
