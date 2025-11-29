# Change: Add Basic Tab Support

## Why

The current implementation only supports a single Neovide instance. Users need the ability to manage multiple Neovide instances within the same wrapper window, similar to how browser tabs work, to improve workflow efficiency and reduce desktop clutter.

## What Changes

- **Tab Bar UI**: Add a tab bar to the existing title bar area, positioned to the left of the window control buttons (minimize, maximize, close)
- **Tab Creation**: Add a "+" button to the right of existing tabs that spawns a new Neovide instance and creates a corresponding tab
- **Tab Display**: Each tab shows a label and a small "x" close button
- **Tab Selection**: Clicking a tab brings its associated Neovide instance to the foreground, occluding the previous
- **Tab Closing**: Clicking the "x" on a tab terminates the associated Neovide process and removes the tab
- **Tab Reordering**: Tabs can be dragged to rearrange their order
- **Visual Distinction**: A subtle outline distinguishes tabs and the client area from the titlebar background
- **Default Behavior**: Application opens with a single tab containing one Neovide instance (maintains current behavior)

## Impact

- Affected specs: `window-management`
- Affected code:
  - `src/window.rs` - Tab bar rendering, hit testing, mouse handling for tabs
  - `src/process.rs` - Multiple NeovideProcess instances management
  - `src/main.rs` - State management for multiple tabs
- New state management: `TabManager` or similar structure to track tabs, their associated processes, and selection state
