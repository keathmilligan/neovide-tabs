## Context
The application currently uses `WS_OVERLAPPEDWINDOW` which provides standard Windows decorations. To achieve a minimalist aesthetic matching Neovide's frameless mode, we need to implement a custom title bar using Win32 APIs.

**Constraints:**
- Must maintain window functionality (move, resize, minimize, maximize, close)
- Must support Windows 11 snap layouts and gestures
- Must use existing configuration infrastructure
- Windows-only implementation (matches current platform constraint)

## Goals / Non-Goals

**Goals:**
- Replace standard title bar with custom-rendered minimalist title bar
- Display app icon, window title, and window control buttons
- Use theme background color for visual consistency
- Maintain all standard window behaviors (drag, resize, snap)

**Non-Goals:**
- Custom resize borders (will use standard thin borders)
- Animated button effects (keep it simple/minimalist)
- Configurable title bar height or button styles (may be added later)

## Decisions

### Decision 1: Use WS_POPUP with custom non-client area handling
**Choice:** Remove `WS_OVERLAPPEDWINDOW`, use `WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU` combined with `WM_NCCALCSIZE` to eliminate the standard title bar while keeping resize borders.

**Alternatives considered:**
- `WS_CAPTION` removal only: Loses resize functionality
- DWM extend frame: More complex, less control over appearance
- Completely custom frame (no WS_THICKFRAME): Requires manual resize handling

**Rationale:** This approach removes the title bar while preserving standard resize handles and Windows snap behavior. It's the standard pattern for custom title bars in Win32.

### Decision 2: Handle WM_NCHITTEST for title bar interactions
**Choice:** Override `WM_NCHITTEST` to return appropriate hit-test codes:
- `HTCAPTION` for the draggable title bar region
- `HTMINBUTTON`, `HTMAXBUTTON`, `HTCLOSE` for window control buttons

**Rationale:** This leverages Windows' built-in window movement and snap gesture support rather than implementing custom drag logic.

### Decision 3: Custom paint in WM_PAINT for title bar content
**Choice:** Render title bar elements (icon, title text, buttons) in `WM_PAINT` using GDI.

**Alternatives considered:**
- Direct2D: More complex setup, overkill for simple rendering
- WM_NCPAINT: More complex coordinate handling

**Rationale:** GDI is simple, already available in the Windows crate, and sufficient for static UI elements.

### Decision 4: Fixed title bar height of 32 pixels
**Choice:** Use 32px height matching Windows 11 default title bar height.

**Rationale:** Matches user expectations and Windows 11 design guidelines. Can be made configurable later if needed.

### Decision 5: Minimal button design
**Choice:** Simple icon-based buttons (Unicode symbols or custom glyphs) with hover state indicated by subtle background color change.

**Alternatives considered:**
- Bitmap icons: Requires resource embedding
- SVG rendering: Requires additional dependencies

**Rationale:** Keeps implementation simple and maintains minimalist aesthetic.

## Risks / Trade-offs

- **Risk:** Windows accessibility features may not work correctly with custom title bar
  - **Mitigation:** Ensure proper hit-test codes are returned; test with accessibility tools
  
- **Risk:** Future Windows updates may change expected behavior
  - **Mitigation:** Use documented Win32 APIs; avoid undocumented behaviors

- **Trade-off:** Custom buttons won't match system theme button colors
  - **Acceptance:** This is intentional for the minimalist design goal

## Open Questions

1. Should the close button have a red hover background (Windows convention) or maintain theme color consistency?
   - **Proposed:** Use red hover for close button for discoverability
2. Should the title bar buttons support keyboard focus/navigation?
   - **Proposed:** Not in initial implementation; can be added for accessibility later
