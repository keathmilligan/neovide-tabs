# Change: Add Custom Title Bar

## Why
The standard Windows title bar creates visual inconsistency with Neovide's frameless appearance and occupies space that could be used for content. A custom title bar using the theme background color provides a unified, minimalist aesthetic that better integrates with the embedded Neovide instance.

## What Changes
- **BREAKING**: Remove standard Windows title bar decorations (WS_OVERLAPPEDWINDOW)
- Add custom-rendered title bar region at the top of the window
- Render app icon, window title text, and custom window control buttons (minimize, maximize, close)
- Use configured background color for title bar background
- Implement hit-testing for custom title bar elements (drag, button clicks)
- Support standard window behaviors: dragging, double-click maximize, snap gestures

## Impact
- Affected specs: `window-management`, `app-config`
- Affected code: `src/window.rs`, `src/config.rs`
- Visual appearance change: window will have custom minimalist chrome instead of Windows default
- User interaction: window controls remain functional but with custom appearance
