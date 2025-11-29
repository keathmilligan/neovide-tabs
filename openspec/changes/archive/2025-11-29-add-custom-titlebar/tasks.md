## 1. Window Style Changes
- [x] 1.1 Modify `create_window()` to use `WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU` instead of `WS_OVERLAPPEDWINDOW`
- [x] 1.2 Add `WM_NCCALCSIZE` handler to eliminate standard title bar while preserving resize borders
- [x] 1.3 Verify window still appears on taskbar and has proper system menu

## 2. Hit Testing Implementation
- [x] 2.1 Add `WM_NCHITTEST` handler to detect clicks in title bar region
- [x] 2.2 Define title bar button rectangles (minimize, maximize, close)
- [x] 2.3 Return `HTCAPTION` for draggable title bar area
- [x] 2.4 Return `HTMINBUTTON`, `HTMAXBUTTON`, `HTCLOSE` for respective button areas
- [x] 2.5 Test window dragging, double-click maximize, and Windows snap gestures

## 3. Title Bar Rendering
- [x] 3.1 Add title bar state to `WindowState` struct (background color, button hover states)
- [x] 3.2 Implement `WM_PAINT` handler for title bar region
- [x] 3.3 Render title bar background with configured color
- [x] 3.4 Load and render application icon (consider embedded resource or drawing)
- [x] 3.5 Render window title text using GDI text functions
- [x] 3.6 Render minimize button (Unicode character or simple glyph)
- [x] 3.7 Render maximize/restore button (changes based on window state)
- [x] 3.8 Render close button

## 4. Button Interactions
- [x] 4.1 Add `WM_NCMOUSEMOVE` handler to track button hover states
- [x] 4.2 Add `WM_NCMOUSELEAVE` handler to clear hover states
- [x] 4.3 Invalidate button regions when hover state changes to trigger repaint
- [x] 4.4 Implement close button red hover background

## 5. Client Area Adjustment
- [x] 5.1 Modify client area calculation to account for 32px title bar height
- [x] 5.2 Adjust Neovide positioning to start below title bar
- [x] 5.3 Test resize behavior with embedded Neovide window

## 6. Testing and Polish
- [x] 6.1 Test window creation and display
- [x] 6.2 Test all button click behaviors (minimize, maximize, restore, close)
- [x] 6.3 Test window dragging and snap gestures
- [x] 6.4 Test with different background colors from config
- [x] 6.5 Test Neovide embedding still works correctly
- [x] 6.6 Run `cargo clippy` and `cargo fmt`
- [x] 6.7 Run `cargo test`
