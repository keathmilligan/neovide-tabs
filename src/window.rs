#![cfg(target_os = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::collapsible_if)]

use anyhow::{Context, Result};
use std::cell::Cell;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, EndPaint, FillRect, HBRUSH, HGDIOBJ, InvalidateRect,
    LOGFONTW, LineTo, MoveToEx, PAINTSTRUCT, PS_SOLID, SRCCOPY, ScreenToClient, SelectObject,
    SetBkMode, SetTextColor, TRANSPARENT, TextOutW,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use crate::tabs::{DragState, TabManager};

const WINDOW_CLASS_NAME: PCWSTR = w!("NeovideTabsWindow");
const WINDOW_TITLE: PCWSTR = w!("neovide-tabs");

/// Title bar height in pixels
const TITLEBAR_HEIGHT: i32 = 32;
/// Button width in pixels
const BUTTON_WIDTH: i32 = 46;
/// Inset for content area (neovim window) from all edges
pub const CONTENT_INSET: i32 = 12;

/// Timer ID for delayed foreground activation
const FOREGROUND_TIMER_ID: usize = 2;
/// Delay before bringing Neovide to foreground (ms)
const FOREGROUND_DELAY_MS: u32 = 50;

/// Timer ID for deferred position update (for external tools like FancyZones)
const POSITION_UPDATE_TIMER_ID: usize = 3;
/// Delay before updating position after external move/resize (ms)
const POSITION_UPDATE_DELAY_MS: u32 = 100;

/// Timer ID for polling Neovide process status
const PROCESS_POLL_TIMER_ID: usize = 4;
/// Interval for polling Neovide process status (ms) - spec requires detection within 500ms
const PROCESS_POLL_INTERVAL_MS: u32 = 250;

// Tab bar layout constants
/// Width of each tab in pixels
const TAB_WIDTH: i32 = 120;
/// Size of the close button within a tab
const TAB_CLOSE_SIZE: i32 = 16;
/// Padding around the close button
const TAB_CLOSE_PADDING: i32 = 8;
/// Width of the new tab (+) button
const NEW_TAB_BUTTON_WIDTH: i32 = 32;
/// Left margin before the first tab
const TAB_BAR_LEFT_MARGIN: i32 = 8;
/// Vertical padding for tabs within the titlebar
const TAB_VERTICAL_PADDING: i32 = 4;

// Tab bar colors
/// Background color for unselected tabs (slightly darker than titlebar)
const TAB_UNSELECTED_COLOR: u32 = 0x16161e;
/// Outline color for tabs and content area
const TAB_OUTLINE_COLOR: u32 = 0x3d3d3d;
/// Hover color for tabs (same as button hover)
const TAB_HOVER_COLOR: u32 = 0x3d3d3d;
/// Close button hover color (red)
const TAB_CLOSE_HOVER_COLOR: u32 = 0xe81123;

/// Which title bar button is being hovered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoveredButton {
    None,
    Minimize,
    Maximize,
    Close,
}

/// Result of hit testing in the tab bar area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabHitResult {
    /// Hit a tab body (index)
    Tab(usize),
    /// Hit a tab's close button (index)
    TabClose(usize),
    /// Hit the new tab (+) button
    NewTabButton,
    /// Hit the caption/drag area
    Caption,
    /// Hit nothing in the tab bar
    None,
}

/// Which tab bar element is being hovered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoveredTab {
    /// No tab element hovered
    None,
    /// Hovering over tab body (index)
    Tab(usize),
    /// Hovering over tab close button (index)
    TabClose(usize),
    /// Hovering over new tab button
    NewTabButton,
}

/// Application state stored in window user data
struct WindowState {
    tab_manager: TabManager,
    in_size_move: bool,
    background_color: u32,
    hovered_button: HoveredButton,
    hovered_tab: HoveredTab,
    tracking_mouse: bool,
}

// Thread-local storage for background color during window creation
thread_local! {
    static INITIAL_BG_COLOR: Cell<u32> = const { Cell::new(0x1a1b26) };
}

/// Convert RGB color (0x00RRGGBB) to Win32 COLORREF (0x00BBGGRR)
fn rgb_to_colorref(rgb: u32) -> u32 {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;
    (b << 16) | (g << 8) | r
}

/// Register the window class with Win32
pub fn register_window_class(background_color: u32) -> Result<()> {
    // Store background color for use in WM_CREATE
    INITIAL_BG_COLOR.with(|c| c.set(background_color));

    unsafe {
        let hinstance = GetModuleHandleW(None).context("Failed to get module handle")?;

        // Create a solid brush for the background color
        let colorref = rgb_to_colorref(background_color);
        let brush = CreateSolidBrush(COLORREF(colorref));

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: Default::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap_or_default(),
            hbrBackground: HBRUSH(brush.0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: WINDOW_CLASS_NAME,
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            anyhow::bail!("Failed to register window class");
        }
    }

    Ok(())
}

/// Create the main application window with custom title bar
pub fn create_window() -> Result<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None).context("Failed to get module handle")?;

        // Use WS_POPUP with thick frame for resize borders, but no caption
        // WS_SYSMENU ensures the window appears in taskbar and has system menu
        let style =
            WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU | WS_VISIBLE;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1024,
            768,
            None,
            None,
            hinstance,
            None,
        )?;

        // Enable Windows 11 rounded corners
        enable_rounded_corners(hwnd);

        Ok(hwnd)
    }
}

/// Run the message loop
pub fn run_message_loop() -> Result<()> {
    unsafe {
        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

/// Get content area dimensions (excluding title bar and with inset from all edges)
fn get_content_rect(hwnd: HWND) -> Result<RECT> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).context("Failed to get client rect")?;
        // Content area starts below title bar with inset from all edges
        rect.left = CONTENT_INSET;
        rect.top = TITLEBAR_HEIGHT + CONTENT_INSET;
        rect.right -= CONTENT_INSET;
        rect.bottom -= CONTENT_INSET;
        Ok(rect)
    }
}

/// Enable Windows 11 rounded corners for the window
fn enable_rounded_corners(hwnd: HWND) {
    unsafe {
        let preference = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const std::ffi::c_void,
            std::mem::size_of_val(&preference) as u32,
        );
    }
}

/// Get the full client rect (including title bar area)
#[allow(dead_code)]
fn get_full_client_rect(hwnd: HWND) -> Result<RECT> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).context("Failed to get client rect")?;
        Ok(rect)
    }
}

/// Calculate button rectangles for the title bar
fn get_button_rects(client_width: i32) -> (RECT, RECT, RECT) {
    let close_rect = RECT {
        left: client_width - BUTTON_WIDTH,
        top: 0,
        right: client_width,
        bottom: TITLEBAR_HEIGHT,
    };

    let maximize_rect = RECT {
        left: client_width - BUTTON_WIDTH * 2,
        top: 0,
        right: client_width - BUTTON_WIDTH,
        bottom: TITLEBAR_HEIGHT,
    };

    let minimize_rect = RECT {
        left: client_width - BUTTON_WIDTH * 3,
        top: 0,
        right: client_width - BUTTON_WIDTH * 2,
        bottom: TITLEBAR_HEIGHT,
    };

    (minimize_rect, maximize_rect, close_rect)
}

/// Check which button (if any) contains the given point
fn hit_test_buttons(x: i32, y: i32, client_width: i32) -> HoveredButton {
    if !(0..TITLEBAR_HEIGHT).contains(&y) {
        return HoveredButton::None;
    }

    let (minimize_rect, maximize_rect, close_rect) = get_button_rects(client_width);

    if x >= close_rect.left && x < close_rect.right {
        HoveredButton::Close
    } else if x >= maximize_rect.left && x < maximize_rect.right {
        HoveredButton::Maximize
    } else if x >= minimize_rect.left && x < minimize_rect.right {
        HoveredButton::Minimize
    } else {
        HoveredButton::None
    }
}

/// Calculate the rectangle for a tab at a given index
fn get_tab_rect(index: usize, client_width: i32) -> RECT {
    let _ = client_width; // Reserved for future dynamic sizing
    let left = TAB_BAR_LEFT_MARGIN + (index as i32 * TAB_WIDTH);
    RECT {
        left,
        top: TAB_VERTICAL_PADDING,
        right: left + TAB_WIDTH,
        bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
    }
}

/// Calculate the rectangle for a tab's close button
fn get_tab_close_rect(tab_rect: &RECT) -> RECT {
    let close_left = tab_rect.right - TAB_CLOSE_PADDING - TAB_CLOSE_SIZE;
    let close_top = (tab_rect.top + tab_rect.bottom - TAB_CLOSE_SIZE) / 2;
    RECT {
        left: close_left,
        top: close_top,
        right: close_left + TAB_CLOSE_SIZE,
        bottom: close_top + TAB_CLOSE_SIZE,
    }
}

/// Get the rectangle for the new tab (+) button
fn get_new_tab_button_rect(tab_count: usize, client_width: i32) -> RECT {
    let _ = client_width; // Reserved for future dynamic sizing
    let left = TAB_BAR_LEFT_MARGIN + (tab_count as i32 * TAB_WIDTH);
    RECT {
        left,
        top: TAB_VERTICAL_PADDING,
        right: left + NEW_TAB_BUTTON_WIDTH,
        bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
    }
}

/// Get the maximum X position for the tab bar (before window buttons)
fn get_tab_bar_max_x(client_width: i32) -> i32 {
    client_width - (BUTTON_WIDTH * 3) - 8 // Leave some padding before window buttons
}

/// Hit test in the tab bar area
fn hit_test_tab_bar(x: i32, y: i32, tab_count: usize, client_width: i32) -> TabHitResult {
    // Must be in the titlebar height range
    if !(TAB_VERTICAL_PADDING..TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING).contains(&y) {
        // Could still be in the caption area if within titlebar
        if (0..TITLEBAR_HEIGHT).contains(&y) {
            return TabHitResult::Caption;
        }
        return TabHitResult::None;
    }

    let max_x = get_tab_bar_max_x(client_width);

    // Check each tab
    for i in 0..tab_count {
        let tab_rect = get_tab_rect(i, client_width);
        if tab_rect.left > max_x {
            break; // Tab bar overflow
        }
        if x >= tab_rect.left && x < tab_rect.right {
            // Check if on the close button
            let close_rect = get_tab_close_rect(&tab_rect);
            if x >= close_rect.left
                && x < close_rect.right
                && y >= close_rect.top
                && y < close_rect.bottom
            {
                return TabHitResult::TabClose(i);
            }
            return TabHitResult::Tab(i);
        }
    }

    // Check new tab button
    let new_tab_rect = get_new_tab_button_rect(tab_count, client_width);
    if new_tab_rect.right <= max_x {
        if x >= new_tab_rect.left && x < new_tab_rect.right {
            return TabHitResult::NewTabButton;
        }
    }

    // In the tab bar area but not on any element - this is caption (draggable)
    if x < TAB_BAR_LEFT_MARGIN + (tab_count as i32 * TAB_WIDTH) + NEW_TAB_BUTTON_WIDTH {
        return TabHitResult::Caption;
    }

    TabHitResult::Caption
}

/// Calculate the target index for dropping a tab at position x
#[cfg(test)]
fn calculate_drop_index(x: i32, tab_count: usize, client_width: i32) -> usize {
    let _ = client_width; // Reserved for future dynamic sizing

    // Calculate which slot the mouse is over
    let relative_x = x - TAB_BAR_LEFT_MARGIN;
    if relative_x < 0 {
        return 0;
    }

    let index = (relative_x / TAB_WIDTH) as usize;
    if index >= tab_count {
        tab_count.saturating_sub(1)
    } else {
        index
    }
}

/// Calculate if a tab swap should occur during drag based on 50% threshold.
/// Returns Some(target_index) if a swap should occur, None otherwise.
///
/// The swap logic:
/// - When dragging right: swap when the dragged tab's center crosses past the center of the next tab
/// - When dragging left: swap when the dragged tab's center crosses past the center of the previous tab
fn calculate_swap_target(
    drag_tab_index: usize,
    drag_visual_x: i32,
    tab_count: usize,
    _client_width: i32,
) -> Option<usize> {
    if tab_count <= 1 {
        return None;
    }

    // Calculate the center of the dragged tab at its visual position
    let drag_center = drag_visual_x + TAB_WIDTH / 2;

    // Check swap with the tab to the right
    if drag_tab_index < tab_count - 1 {
        let right_tab_index = drag_tab_index + 1;
        let right_tab_rect = get_tab_rect(right_tab_index, 0);
        let right_tab_center = (right_tab_rect.left + right_tab_rect.right) / 2;

        // If dragged tab center is past the right tab's center, swap right
        if drag_center > right_tab_center {
            return Some(right_tab_index);
        }
    }

    // Check swap with the tab to the left
    if drag_tab_index > 0 {
        let left_tab_index = drag_tab_index - 1;
        let left_tab_rect = get_tab_rect(left_tab_index, 0);
        let left_tab_center = (left_tab_rect.left + left_tab_rect.right) / 2;

        // If dragged tab center is past the left tab's center (to the left), swap left
        if drag_center < left_tab_center {
            return Some(left_tab_index);
        }
    }

    None
}

/// Paint a single tab
#[allow(unused_must_use)]
fn paint_tab(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    tab_rect: &RECT,
    label: &str,
    is_selected: bool,
    is_hovered: bool,
    close_hovered: bool,
    background_color: u32,
) {
    unsafe {
        // Determine tab background color
        let tab_bg = if is_selected {
            background_color // Selected tab matches titlebar
        } else if is_hovered {
            TAB_HOVER_COLOR
        } else {
            TAB_UNSELECTED_COLOR
        };

        let tab_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(tab_bg)));
        FillRect(hdc, tab_rect, tab_brush);
        DeleteObject(HGDIOBJ(tab_brush.0));

        // Draw outline around tab (top, left, right - bottom is handled by tab bar line)
        let outline_pen = CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
        let old_pen = SelectObject(hdc, HGDIOBJ(outline_pen.0));

        MoveToEx(hdc, tab_rect.left, tab_rect.bottom, None);
        LineTo(hdc, tab_rect.left, tab_rect.top);
        LineTo(hdc, tab_rect.right - 1, tab_rect.top);
        LineTo(hdc, tab_rect.right - 1, tab_rect.bottom);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(outline_pen.0));

        // Draw tab label
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00FFFFFF)); // White text

        let mut lf = LOGFONTW::default();
        lf.lfHeight = -12;
        lf.lfWeight = 400;
        let font_name = "Segoe UI";
        for (i, c) in font_name.encode_utf16().enumerate() {
            if i < 32 {
                lf.lfFaceName[i] = c;
            }
        }
        let font = CreateFontIndirectW(&lf);
        let old_font = SelectObject(hdc, HGDIOBJ(font.0));

        // Label position (left-aligned with padding, leaving room for close button)
        let label_x = tab_rect.left + 8;
        let label_y = (tab_rect.top + tab_rect.bottom - 12) / 2;
        let label_wide: Vec<u16> = label.encode_utf16().collect();
        TextOutW(hdc, label_x, label_y, &label_wide);

        SelectObject(hdc, old_font);
        DeleteObject(HGDIOBJ(font.0));

        // Draw close button
        let close_rect = get_tab_close_rect(tab_rect);

        // Close button background on hover
        if close_hovered {
            let close_hover_brush =
                CreateSolidBrush(COLORREF(rgb_to_colorref(TAB_CLOSE_HOVER_COLOR)));
            FillRect(hdc, &close_rect, close_hover_brush);
            DeleteObject(HGDIOBJ(close_hover_brush.0));
        }

        // Draw X for close button
        let close_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00FFFFFF));
        let old_pen = SelectObject(hdc, HGDIOBJ(close_pen.0));

        let cx = (close_rect.left + close_rect.right) / 2;
        let cy = (close_rect.top + close_rect.bottom) / 2;
        let size = 4;

        MoveToEx(hdc, cx - size, cy - size, None);
        LineTo(hdc, cx + size + 1, cy + size + 1);
        MoveToEx(hdc, cx + size, cy - size, None);
        LineTo(hdc, cx - size - 1, cy + size + 1);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(close_pen.0));
    }
}

/// Paint the new tab (+) button
#[allow(unused_must_use)]
fn paint_new_tab_button(hdc: windows::Win32::Graphics::Gdi::HDC, rect: &RECT, is_hovered: bool) {
    unsafe {
        // Background on hover
        if is_hovered {
            let hover_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(TAB_HOVER_COLOR)));
            FillRect(hdc, rect, hover_brush);
            DeleteObject(HGDIOBJ(hover_brush.0));
        }

        // Draw + icon
        let pen = CreatePen(PS_SOLID, 1, COLORREF(0x00FFFFFF));
        let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));

        let cx = (rect.left + rect.right) / 2;
        let cy = (rect.top + rect.bottom) / 2;
        let size = 6;

        // Horizontal line
        MoveToEx(hdc, cx - size, cy, None);
        LineTo(hdc, cx + size + 1, cy);
        // Vertical line
        MoveToEx(hdc, cx, cy - size, None);
        LineTo(hdc, cx, cy + size + 1);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(pen.0));
    }
}

/// Paint the tab bar (all tabs and new tab button)
#[allow(unused_must_use)]
fn paint_tab_bar(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    tab_manager: &TabManager,
    hovered_tab: HoveredTab,
    client_width: i32,
    background_color: u32,
) {
    let max_x = get_tab_bar_max_x(client_width);
    let selected_index = tab_manager.selected_index();
    let drag_state = &tab_manager.drag_state;

    // First pass: paint all non-dragged tabs in their normal positions
    for (i, _tab) in tab_manager.iter() {
        // Skip the dragged tab - we'll paint it last so it appears on top
        if let Some(drag) = drag_state {
            if drag.is_active() && i == drag.tab_index {
                continue;
            }
        }

        let tab_rect = get_tab_rect(i, client_width);
        if tab_rect.left > max_x {
            break; // Overflow
        }

        let is_selected = i == selected_index;
        let is_hovered = matches!(hovered_tab, HoveredTab::Tab(idx) if idx == i);
        let close_hovered = matches!(hovered_tab, HoveredTab::TabClose(idx) if idx == i);
        let label = tab_manager.get_tab_label(i);

        paint_tab(
            hdc,
            &tab_rect,
            &label,
            is_selected,
            is_hovered,
            close_hovered,
            background_color,
        );
    }

    // Paint new tab button
    let new_tab_rect = get_new_tab_button_rect(tab_manager.count(), client_width);
    if new_tab_rect.right <= max_x {
        let is_hovered = matches!(hovered_tab, HoveredTab::NewTabButton);
        paint_new_tab_button(hdc, &new_tab_rect, is_hovered);
    }

    // Draw line at the bottom of the tab bar with a gap for the selected tab
    // This creates the illusion of physical tabbed pages
    paint_tab_bar_bottom_line(hdc, tab_manager, client_width);

    // Second pass: paint the dragged tab at its visual position (on top of everything)
    if let Some(drag) = drag_state {
        if drag.is_active() {
            let drag_index = drag.tab_index;
            let visual_x = drag.get_visual_x();

            // Clamp the visual position to stay within the tab bar bounds
            let min_x = TAB_BAR_LEFT_MARGIN;
            let max_tab_x = TAB_BAR_LEFT_MARGIN + ((tab_manager.count() - 1) as i32 * TAB_WIDTH);
            let clamped_x = visual_x.clamp(min_x, max_tab_x.max(min_x));

            let drag_rect = RECT {
                left: clamped_x,
                top: TAB_VERTICAL_PADDING,
                right: clamped_x + TAB_WIDTH,
                bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
            };

            let is_selected = drag_index == selected_index;
            let label = tab_manager.get_tab_label(drag_index);

            // Dragged tab is never hovered (we're dragging it)
            paint_tab(
                hdc,
                &drag_rect,
                &label,
                is_selected,
                false,
                false,
                background_color,
            );
        }
    }
}

/// Paint the bottom line of the tab bar with a gap for the selected tab
#[allow(unused_must_use)]
fn paint_tab_bar_bottom_line(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    tab_manager: &TabManager,
    client_width: i32,
) {
    unsafe {
        let outline_pen = CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
        let old_pen = SelectObject(hdc, HGDIOBJ(outline_pen.0));

        // The bottom line is at TITLEBAR_HEIGHT - 1 (bottom of tab area)
        let line_y = TITLEBAR_HEIGHT - 1;
        let line_start_x = 0;
        let line_end_x = client_width;

        // Get the selected tab rect to create a gap
        let selected_index = tab_manager.selected_index();
        let selected_rect = get_tab_rect(selected_index, client_width);

        // Draw line from left edge to start of selected tab
        if selected_rect.left > line_start_x {
            MoveToEx(hdc, line_start_x, line_y, None);
            LineTo(hdc, selected_rect.left, line_y);
        }

        // Draw line from end of selected tab to right edge
        if selected_rect.right < line_end_x {
            MoveToEx(hdc, selected_rect.right - 1, line_y, None);
            LineTo(hdc, line_end_x, line_y);
        }

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(outline_pen.0));
    }
}

/// Paint the title bar content to a device context
#[allow(unused_must_use)]
fn paint_titlebar_content(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    client_rect: &RECT,
    background_color: u32,
    hovered_button: HoveredButton,
    hovered_tab: HoveredTab,
    tab_manager: &TabManager,
) {
    unsafe {
        let client_width = client_rect.right;

        // Fill entire client area with background color
        let bg_colorref = COLORREF(rgb_to_colorref(background_color));
        let bg_brush = CreateSolidBrush(bg_colorref);
        FillRect(hdc, client_rect, bg_brush);
        DeleteObject(HGDIOBJ(bg_brush.0));

        // Paint tab bar
        paint_tab_bar(
            hdc,
            tab_manager,
            hovered_tab,
            client_width,
            background_color,
        );

        // Get button rectangles
        let (minimize_rect, maximize_rect, close_rect) = get_button_rects(client_width);

        // Draw button backgrounds for hover states
        let hover_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(0x3d3d3d)));
        let close_hover_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(0xe81123))); // Red for close

        match hovered_button {
            HoveredButton::Minimize => {
                FillRect(hdc, &minimize_rect, hover_brush);
            }
            HoveredButton::Maximize => {
                FillRect(hdc, &maximize_rect, hover_brush);
            }
            HoveredButton::Close => {
                FillRect(hdc, &close_rect, close_hover_brush);
            }
            HoveredButton::None => {}
        }

        DeleteObject(HGDIOBJ(hover_brush.0));
        DeleteObject(HGDIOBJ(close_hover_brush.0));

        // Draw button icons using simple lines (white color)
        let pen = CreatePen(PS_SOLID, 1, COLORREF(0x00FFFFFF));
        let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));

        // Minimize button: horizontal line
        let min_cx = (minimize_rect.left + minimize_rect.right) / 2;
        let min_cy = (minimize_rect.top + minimize_rect.bottom) / 2;
        MoveToEx(hdc, min_cx - 5, min_cy, None);
        LineTo(hdc, min_cx + 6, min_cy);

        // Maximize/Restore button
        let max_cx = (maximize_rect.left + maximize_rect.right) / 2;
        let max_cy = (maximize_rect.top + maximize_rect.bottom) / 2;

        let is_maximized = IsZoomed(hwnd).as_bool();
        if is_maximized {
            // Draw restore icon (two overlapping rectangles)
            // Back rectangle (smaller, offset up-right)
            MoveToEx(hdc, max_cx - 3, max_cy - 5, None);
            LineTo(hdc, max_cx + 5, max_cy - 5);
            MoveToEx(hdc, max_cx + 5, max_cy - 5, None);
            LineTo(hdc, max_cx + 5, max_cy - 2);
            // Front rectangle
            MoveToEx(hdc, max_cx - 5, max_cy - 2, None);
            LineTo(hdc, max_cx + 3, max_cy - 2);
            LineTo(hdc, max_cx + 3, max_cy + 6);
            LineTo(hdc, max_cx - 5, max_cy + 6);
            LineTo(hdc, max_cx - 5, max_cy - 2);
        } else {
            // Draw maximize icon (single rectangle)
            MoveToEx(hdc, max_cx - 5, max_cy - 5, None);
            LineTo(hdc, max_cx + 5, max_cy - 5);
            LineTo(hdc, max_cx + 5, max_cy + 5);
            LineTo(hdc, max_cx - 5, max_cy + 5);
            LineTo(hdc, max_cx - 5, max_cy - 5);
        }

        // Close button: X
        let close_cx = (close_rect.left + close_rect.right) / 2;
        let close_cy = (close_rect.top + close_rect.bottom) / 2;
        let _ = MoveToEx(hdc, close_cx - 5, close_cy - 5, None);
        let _ = LineTo(hdc, close_cx + 6, close_cy + 6);
        let _ = MoveToEx(hdc, close_cx + 5, close_cy - 5, None);
        let _ = LineTo(hdc, close_cx - 6, close_cy + 6);

        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(HGDIOBJ(pen.0));
    }
}

/// Paint the title bar using double-buffering to prevent flicker
#[allow(unused_must_use)]
fn paint_titlebar(
    hwnd: HWND,
    ps: &PAINTSTRUCT,
    background_color: u32,
    hovered_button: HoveredButton,
    hovered_tab: HoveredTab,
    tab_manager: &TabManager,
) {
    unsafe {
        let hdc = ps.hdc;

        // Get client rect
        let mut client_rect = RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_err() {
            return;
        }

        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;

        // Create off-screen buffer for double-buffering
        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bitmap = CreateCompatibleBitmap(hdc, width, height);
        let old_bitmap = SelectObject(mem_dc, HGDIOBJ(mem_bitmap.0));

        // Paint everything to the off-screen buffer
        paint_titlebar_content(
            hwnd,
            mem_dc,
            &client_rect,
            background_color,
            hovered_button,
            hovered_tab,
            tab_manager,
        );

        // Copy the off-screen buffer to the screen in one operation
        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        // Clean up
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(HGDIOBJ(mem_bitmap.0));
        DeleteDC(mem_dc);
    }
}

/// Window procedure callback
#[allow(unused_must_use)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Get background color from thread-local storage
            let background_color = INITIAL_BG_COLOR.with(|c| c.get());

            // Create tab manager and initial tab
            let mut tab_manager = TabManager::new();

            // Get content area dimensions (below title bar)
            if let Ok(rect) = get_content_rect(hwnd) {
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;

                // Create initial tab with Neovide process
                match tab_manager.create_tab(width, height, hwnd) {
                    Ok(_) => {}
                    Err(e) => {
                        let error_msg = format!("Failed to launch Neovide: {}", e);
                        show_error(&error_msg, "Error: Failed to Launch Neovide");
                    }
                }
            }

            let state = Box::new(WindowState {
                tab_manager,
                in_size_move: false,
                background_color,
                hovered_button: HoveredButton::None,
                hovered_tab: HoveredTab::None,
                tracking_mouse: false,
            });
            let state_ptr = Box::into_raw(state);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

            // Start the process polling timer to detect when Neovide processes exit
            SetTimer(hwnd, PROCESS_POLL_TIMER_ID, PROCESS_POLL_INTERVAL_MS, None);

            LRESULT(0)
        }

        WM_NCCALCSIZE => {
            // When wparam is TRUE, we need to handle the calculation
            if wparam.0 != 0 {
                let params = lparam.0 as *mut NCCALCSIZE_PARAMS;
                if !params.is_null() {
                    // When maximized, we need to account for the invisible borders
                    // that Windows adds, otherwise the window extends beyond the screen
                    if IsZoomed(hwnd).as_bool() {
                        let frame_x = GetSystemMetrics(SM_CXFRAME);
                        let frame_y = GetSystemMetrics(SM_CYFRAME);
                        let padding = GetSystemMetrics(SM_CXPADDEDBORDER);

                        (*params).rgrc[0].left += frame_x + padding;
                        (*params).rgrc[0].top += frame_y + padding;
                        (*params).rgrc[0].right -= frame_x + padding;
                        (*params).rgrc[0].bottom -= frame_y + padding;
                    }
                    // When not maximized, don't adjust - let client area extend to
                    // the full window bounds. We handle resize hit-testing ourselves.
                }
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_NCPAINT => {
            // Don't paint the non-client area - we handle everything in WM_PAINT
            LRESULT(0)
        }

        WM_NCACTIVATE => {
            // Return TRUE to prevent Windows from painting the non-client area
            // The lparam check prevents the default frame from being drawn
            LRESULT(1)
        }

        WM_NCHITTEST => {
            // Get mouse position in screen coordinates
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            // Convert to client coordinates
            let mut pt = POINT { x, y };
            if ScreenToClient(hwnd, &mut pt).as_bool() {
                let mut client_rect = RECT::default();
                if GetClientRect(hwnd, &mut client_rect).is_ok() {
                    let client_width = client_rect.right;
                    let client_height = client_rect.bottom;

                    // Check resize borders first (when not maximized)
                    if !IsZoomed(hwnd).as_bool() {
                        let border_width = 8;

                        // Top edge
                        if pt.y <= border_width {
                            if pt.x <= border_width {
                                return LRESULT(HTTOPLEFT as isize);
                            } else if pt.x >= client_width - border_width {
                                return LRESULT(HTTOPRIGHT as isize);
                            }
                            return LRESULT(HTTOP as isize);
                        }

                        // Bottom edge
                        if pt.y >= client_height - border_width {
                            if pt.x <= border_width {
                                return LRESULT(HTBOTTOMLEFT as isize);
                            } else if pt.x >= client_width - border_width {
                                return LRESULT(HTBOTTOMRIGHT as isize);
                            }
                            return LRESULT(HTBOTTOM as isize);
                        }

                        // Left edge
                        if pt.x <= border_width {
                            return LRESULT(HTLEFT as isize);
                        }

                        // Right edge
                        if pt.x >= client_width - border_width {
                            return LRESULT(HTRIGHT as isize);
                        }
                    }

                    // Check if in title bar area
                    if pt.y >= 0 && pt.y < TITLEBAR_HEIGHT {
                        // Check window control buttons first
                        let button = hit_test_buttons(pt.x, pt.y, client_width);
                        match button {
                            HoveredButton::Minimize => return LRESULT(HTMINBUTTON as isize),
                            HoveredButton::Maximize => return LRESULT(HTMAXBUTTON as isize),
                            HoveredButton::Close => return LRESULT(HTCLOSE as isize),
                            HoveredButton::None => {
                                // Check tab bar area
                                let state_ptr =
                                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                                if !state_ptr.is_null() {
                                    let state = &*state_ptr;
                                    let tab_hit = hit_test_tab_bar(
                                        pt.x,
                                        pt.y,
                                        state.tab_manager.count(),
                                        client_width,
                                    );
                                    match tab_hit {
                                        TabHitResult::Tab(_)
                                        | TabHitResult::TabClose(_)
                                        | TabHitResult::NewTabButton => {
                                            // These are handled as client area clicks
                                            return LRESULT(HTCLIENT as isize);
                                        }
                                        TabHitResult::Caption | TabHitResult::None => {
                                            return LRESULT(HTCAPTION as isize);
                                        }
                                    }
                                }
                                // Default to caption if no state
                                return LRESULT(HTCAPTION as isize);
                            }
                        }
                    }

                    // In client area (below title bar)
                    return LRESULT(HTCLIENT as isize);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_NCMOUSEMOVE => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;

                // Track mouse to get WM_NCMOUSELEAVE
                if !state.tracking_mouse {
                    let mut tme = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE | TME_NONCLIENT,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    TrackMouseEvent(&mut tme);
                    state.tracking_mouse = true;
                }

                // Determine which button is hovered based on wparam (hit test result)
                let new_hover = match wparam.0 as u32 {
                    x if x == HTMINBUTTON => HoveredButton::Minimize,
                    x if x == HTMAXBUTTON => HoveredButton::Maximize,
                    x if x == HTCLOSE => HoveredButton::Close,
                    _ => HoveredButton::None,
                };

                if new_hover != state.hovered_button {
                    state.hovered_button = new_hover;
                    // Invalidate title bar to repaint
                    let mut client_rect = RECT::default();
                    if GetClientRect(hwnd, &mut client_rect).is_ok() {
                        let titlebar_rect = RECT {
                            left: 0,
                            top: 0,
                            right: client_rect.right,
                            bottom: TITLEBAR_HEIGHT,
                        };
                        InvalidateRect(hwnd, Some(&titlebar_rect), false);
                    }
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_NCMOUSELEAVE => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.tracking_mouse = false;
                if state.hovered_button != HoveredButton::None {
                    state.hovered_button = HoveredButton::None;
                    // Invalidate title bar to repaint
                    let mut client_rect = RECT::default();
                    if GetClientRect(hwnd, &mut client_rect).is_ok() {
                        let titlebar_rect = RECT {
                            left: 0,
                            top: 0,
                            right: client_rect.right,
                            bottom: TITLEBAR_HEIGHT,
                        };
                        InvalidateRect(hwnd, Some(&titlebar_rect), false);
                    }
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_MOUSELEAVE => {
            // Handle client area mouse leave
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.tracking_mouse = false;

                // Clear tab hover state
                if state.hovered_tab != HoveredTab::None {
                    state.hovered_tab = HoveredTab::None;
                    let mut client_rect = RECT::default();
                    if GetClientRect(hwnd, &mut client_rect).is_ok() {
                        let titlebar_rect = RECT {
                            left: 0,
                            top: 0,
                            right: client_rect.right,
                            bottom: TITLEBAR_HEIGHT,
                        };
                        InvalidateRect(hwnd, Some(&titlebar_rect), false);
                    }
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_PAINT => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;

            let mut ps = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut ps);

            if !state_ptr.is_null() {
                let state = &*state_ptr;
                paint_titlebar(
                    hwnd,
                    &ps,
                    state.background_color,
                    state.hovered_button,
                    state.hovered_tab,
                    &state.tab_manager,
                );
            } else {
                // Fallback with empty tab manager
                let empty_manager = TabManager::new();
                paint_titlebar(
                    hwnd,
                    &ps,
                    0x1a1b26,
                    HoveredButton::None,
                    HoveredTab::None,
                    &empty_manager,
                );
            }

            EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_ERASEBKGND => {
            // We handle painting ourselves
            LRESULT(1)
        }

        WM_ENTERSIZEMOVE => {
            // User started dragging or resizing - set flag and cancel any pending timers
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.in_size_move = true;
                KillTimer(hwnd, FOREGROUND_TIMER_ID).ok();
                KillTimer(hwnd, POSITION_UPDATE_TIMER_ID).ok();
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_EXITSIZEMOVE => {
            // User finished dragging or resizing - now reposition all Neovide windows
            // and bring the selected one to the foreground
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.in_size_move = false;
                // Update positions for all tabs (so switching tabs later works correctly)
                state
                    .tab_manager
                    .update_all_positions(hwnd, TITLEBAR_HEIGHT);
                // Activate the selected tab (show + bring to foreground)
                state
                    .tab_manager
                    .activate_and_foreground_selected(hwnd, TITLEBAR_HEIGHT);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_ACTIVATE => {
            // Bring selected tab's Neovide to foreground when wrapper is activated
            // Use a short delay to allow WM_ENTERSIZEMOVE to fire first if this is a drag
            let activated = (wparam.0 & 0xFFFF) != 0; // WA_INACTIVE = 0
            if activated {
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    if state.tab_manager.is_selected_ready() {
                        // Schedule delayed foreground activation
                        SetTimer(hwnd, FOREGROUND_TIMER_ID, FOREGROUND_DELAY_MS, None);
                    }
                }
            } else {
                // Deactivating - cancel any pending foreground timer
                KillTimer(hwnd, FOREGROUND_TIMER_ID).ok();
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_TIMER => {
            if wparam.0 == FOREGROUND_TIMER_ID {
                KillTimer(hwnd, FOREGROUND_TIMER_ID).ok();

                // Now check if we're in a size/move operation
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    if !state.in_size_move {
                        // Use activate which checks position first, then brings to foreground
                        state
                            .tab_manager
                            .activate_and_foreground_selected(hwnd, TITLEBAR_HEIGHT);
                    }
                }
            } else if wparam.0 == POSITION_UPDATE_TIMER_ID {
                KillTimer(hwnd, POSITION_UPDATE_TIMER_ID).ok();

                // Deferred position update for external tools (e.g., FancyZones)
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    if !state.in_size_move {
                        state
                            .tab_manager
                            .update_all_positions(hwnd, TITLEBAR_HEIGHT);
                    }
                }
            } else if wparam.0 == PROCESS_POLL_TIMER_ID {
                // Poll for exited Neovide processes
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;

                    // Find all tabs whose processes have exited
                    let exited_indices = state.tab_manager.find_exited_tabs();

                    if !exited_indices.is_empty() {
                        let mut should_close = false;

                        // Remove exited tabs (indices are in reverse order for safe removal)
                        for index in exited_indices {
                            if state.tab_manager.remove_exited_tab(index) {
                                // This was the last tab
                                should_close = true;
                                break;
                            }
                        }

                        if should_close {
                            // Last tab's process exited - close the application
                            KillTimer(hwnd, PROCESS_POLL_TIMER_ID).ok();
                            PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).ok();
                        } else {
                            // Activate the newly selected tab and repaint
                            state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                            InvalidateRect(hwnd, None, false);
                        }
                    }
                }
            }
            LRESULT(0)
        }

        WM_WINDOWPOSCHANGED => {
            // Handle programmatic window position/size changes (e.g., from FancyZones)
            // Only schedule update if we're not in a manual size/move operation
            // Use a timer to debounce and avoid interfering with resize operations
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                if !state.in_size_move && !state.tab_manager.is_empty() {
                    // Schedule a deferred position update - this will be cancelled
                    // if more WM_WINDOWPOSCHANGED messages arrive, effectively debouncing
                    SetTimer(
                        hwnd,
                        POSITION_UPDATE_TIMER_ID,
                        POSITION_UPDATE_DELAY_MS,
                        None,
                    );
                }
            }
            // Must call DefWindowProcW to get WM_SIZE and WM_MOVE messages
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_SIZE => {
            // Invalidate the window to repaint title bar (maximize/restore button may change)
            InvalidateRect(hwnd, None, false);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_GETMINMAXINFO => {
            // Set minimum window size to 800x600
            let info = lparam.0 as *mut MINMAXINFO;
            if !info.is_null() {
                (*info).ptMinTrackSize.x = 800;
                (*info).ptMinTrackSize.y = 600;
            }
            LRESULT(0)
        }

        WM_CLOSE => {
            // Stop the process polling timer
            KillTimer(hwnd, PROCESS_POLL_TIMER_ID).ok();

            // Terminate all Neovide processes spawned by this application
            // (only processes tracked via Child handles are terminated)
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let mut state = Box::from_raw(state_ptr);
                state.tab_manager.terminate_all();
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            DestroyWindow(hwnd).ok();
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                let mut client_rect = RECT::default();
                if GetClientRect(hwnd, &mut client_rect).is_ok() {
                    let client_width = client_rect.right;
                    let tab_hit = hit_test_tab_bar(x, y, state.tab_manager.count(), client_width);

                    match tab_hit {
                        TabHitResult::Tab(index) => {
                            // Start potential drag - get the tab's initial position
                            let tab_rect = get_tab_rect(index, client_width);
                            state.tab_manager.drag_state = Some(DragState {
                                tab_index: index,
                                start_x: x,
                                current_x: x,
                                tab_start_left: tab_rect.left,
                            });
                            // Capture mouse for drag tracking
                            SetCapture(hwnd);
                        }
                        TabHitResult::TabClose(index) => {
                            // Close the tab
                            let should_close_window = state.tab_manager.close_tab(index);
                            if should_close_window {
                                // Last tab closed - close the window
                                PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).ok();
                            } else {
                                // Activate the newly selected tab (with proper position check)
                                state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                                InvalidateRect(hwnd, None, false);
                            }
                        }
                        TabHitResult::NewTabButton => {
                            // Create new tab
                            if let Ok(rect) = get_content_rect(hwnd) {
                                let width = (rect.right - rect.left) as u32;
                                let height = (rect.bottom - rect.top) as u32;

                                match state.tab_manager.create_tab(width, height, hwnd) {
                                    Ok(_) => {
                                        // Hide other tabs immediately
                                        // The new tab will be activated by the spawner thread
                                        // once the window is ready
                                        for (i, tab) in state.tab_manager.iter() {
                                            if i != state.tab_manager.selected_index() {
                                                tab.process.hide();
                                            }
                                        }
                                        InvalidateRect(hwnd, None, false);
                                    }
                                    Err(e) => {
                                        let error_msg = format!("Failed to create new tab: {}", e);
                                        show_error(&error_msg, "Error: Failed to Create Tab");
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            LRESULT(0)
        }

        WM_LBUTTONUP => {
            let _x = (lparam.0 & 0xFFFF) as i16 as i32;
            let _y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;

                if let Some(drag) = state.tab_manager.drag_state.take() {
                    ReleaseCapture().ok();

                    if drag.is_active() {
                        // Drag completed - tabs have already been swapped during drag
                        // Just repaint to show final positions
                        InvalidateRect(hwnd, None, false);
                    } else {
                        // This was a click, not a drag - select the tab
                        if state.tab_manager.select_tab(drag.tab_index) {
                            // Selection changed - activate with proper position check
                            state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                            InvalidateRect(hwnd, None, false);
                        } else {
                            // Already selected - just ensure it's in foreground (no reposition)
                            state
                                .tab_manager
                                .activate_and_foreground_selected(hwnd, TITLEBAR_HEIGHT);
                        }
                    }
                }
            }
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;

                // Check if we're dragging and extract needed info
                let drag_info = if let Some(ref mut drag) = state.tab_manager.drag_state {
                    drag.current_x = x;
                    if drag.is_active() {
                        Some((drag.tab_index, drag.get_visual_x()))
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Handle active drag - check for swaps
                if let Some((current_tab_index, visual_x)) = drag_info {
                    let tab_count = state.tab_manager.count();
                    let mut client_rect = RECT::default();
                    if GetClientRect(hwnd, &mut client_rect).is_ok() {
                        let client_width = client_rect.right;

                        if let Some(target_index) = calculate_swap_target(
                            current_tab_index,
                            visual_x,
                            tab_count,
                            client_width,
                        ) {
                            // Perform the swap
                            state.tab_manager.move_tab(current_tab_index, target_index);

                            // Update drag state to track the new position
                            if let Some(ref mut drag) = state.tab_manager.drag_state {
                                drag.tab_index = target_index;
                                // Update tab_start_left to the new slot position
                                let new_slot_rect = get_tab_rect(target_index, client_width);
                                drag.tab_start_left = new_slot_rect.left;
                                // Recalculate start_x to maintain visual continuity
                                // The tab should stay where it visually is
                                drag.start_x = x - (visual_x - new_slot_rect.left);
                            }
                        }
                    }

                    // Repaint for drag feedback (false = don't erase, we use double-buffering)
                    InvalidateRect(hwnd, None, false);
                } else if state.tab_manager.drag_state.is_none() {
                    // Not dragging - update hover state
                    let mut client_rect = RECT::default();
                    if GetClientRect(hwnd, &mut client_rect).is_ok() {
                        let client_width = client_rect.right;
                        let tab_hit =
                            hit_test_tab_bar(x, y, state.tab_manager.count(), client_width);

                        let new_hover = match tab_hit {
                            TabHitResult::Tab(i) => HoveredTab::Tab(i),
                            TabHitResult::TabClose(i) => HoveredTab::TabClose(i),
                            TabHitResult::NewTabButton => HoveredTab::NewTabButton,
                            _ => HoveredTab::None,
                        };

                        if new_hover != state.hovered_tab {
                            state.hovered_tab = new_hover;
                            // Invalidate titlebar for hover effect
                            let titlebar_rect = RECT {
                                left: 0,
                                top: 0,
                                right: client_rect.right,
                                bottom: TITLEBAR_HEIGHT,
                            };
                            InvalidateRect(hwnd, Some(&titlebar_rect), false);
                        }
                    }

                    // Track mouse to get WM_MOUSELEAVE
                    if !state.tracking_mouse {
                        let mut tme = TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        TrackMouseEvent(&mut tme);
                        state.tracking_mouse = true;
                    }
                }
            }
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Display an error message box
fn show_error(message: &str, title: &str) {
    unsafe {
        let wide_message: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

        MessageBoxW(
            None,
            PCWSTR(wide_message.as_ptr()),
            PCWSTR(wide_title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

/// Display an error message for missing Neovide
pub fn show_neovide_not_found_error() {
    show_error(
        "Neovide not found in PATH. Please install Neovide and ensure it is accessible.\n\n\
        Installation options:\n\
        - winget install Neovide.Neovide\n\
        - scoop install neovide\n\
        - Download from: https://github.com/neovide/neovide/releases",
        "Error: Neovide Not Found",
    );
}

/// Get the title bar height (for use by other modules)
#[allow(dead_code)]
pub fn get_titlebar_height() -> i32 {
    TITLEBAR_HEIGHT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_colorref() {
        // Test conversion from RGB to COLORREF
        assert_eq!(rgb_to_colorref(0xFF0000), 0x0000FF); // Red
        assert_eq!(rgb_to_colorref(0x00FF00), 0x00FF00); // Green (no change)
        assert_eq!(rgb_to_colorref(0x0000FF), 0xFF0000); // Blue
        assert_eq!(rgb_to_colorref(0x1a1b26), 0x261b1a); // Tokyo Night dark
    }

    #[test]
    fn test_button_rects() {
        let (min, max, close) = get_button_rects(1024);

        // Close button should be rightmost
        assert_eq!(close.right, 1024);
        assert_eq!(close.left, 1024 - BUTTON_WIDTH);

        // Maximize button should be next
        assert_eq!(max.right, close.left);
        assert_eq!(max.left, 1024 - BUTTON_WIDTH * 2);

        // Minimize button should be leftmost
        assert_eq!(min.right, max.left);
        assert_eq!(min.left, 1024 - BUTTON_WIDTH * 3);

        // All buttons should span the title bar height
        assert_eq!(min.top, 0);
        assert_eq!(min.bottom, TITLEBAR_HEIGHT);
    }

    #[test]
    fn test_hit_test_buttons() {
        let width = 1024;
        let (min, max, close) = get_button_rects(width);

        // Test close button area
        assert_eq!(
            hit_test_buttons(close.left + 5, TITLEBAR_HEIGHT / 2, width),
            HoveredButton::Close
        );

        // Test maximize button area
        assert_eq!(
            hit_test_buttons(max.left + 5, TITLEBAR_HEIGHT / 2, width),
            HoveredButton::Maximize
        );

        // Test minimize button area
        assert_eq!(
            hit_test_buttons(min.left + 5, TITLEBAR_HEIGHT / 2, width),
            HoveredButton::Minimize
        );

        // Test caption area (before buttons)
        assert_eq!(
            hit_test_buttons(100, TITLEBAR_HEIGHT / 2, width),
            HoveredButton::None
        );

        // Test below title bar
        assert_eq!(
            hit_test_buttons(100, TITLEBAR_HEIGHT + 10, width),
            HoveredButton::None
        );
    }

    #[test]
    fn test_get_tab_rect() {
        let tab0 = get_tab_rect(0, 1024);
        assert_eq!(tab0.left, TAB_BAR_LEFT_MARGIN);
        assert_eq!(tab0.right, TAB_BAR_LEFT_MARGIN + TAB_WIDTH);
        assert_eq!(tab0.top, TAB_VERTICAL_PADDING);
        assert_eq!(tab0.bottom, TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING);

        let tab1 = get_tab_rect(1, 1024);
        assert_eq!(tab1.left, TAB_BAR_LEFT_MARGIN + TAB_WIDTH);
        assert_eq!(tab1.right, TAB_BAR_LEFT_MARGIN + TAB_WIDTH * 2);
    }

    #[test]
    fn test_get_new_tab_button_rect() {
        let btn = get_new_tab_button_rect(0, 1024);
        assert_eq!(btn.left, TAB_BAR_LEFT_MARGIN);
        assert_eq!(btn.right, TAB_BAR_LEFT_MARGIN + NEW_TAB_BUTTON_WIDTH);

        let btn = get_new_tab_button_rect(2, 1024);
        assert_eq!(btn.left, TAB_BAR_LEFT_MARGIN + TAB_WIDTH * 2);
    }

    #[test]
    fn test_hit_test_tab_bar() {
        let width = 1024;
        let tab_count = 2;
        let y = (TAB_VERTICAL_PADDING + TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING) / 2;

        // First tab area
        let x = TAB_BAR_LEFT_MARGIN + 20;
        assert_eq!(
            hit_test_tab_bar(x, y, tab_count, width),
            TabHitResult::Tab(0)
        );

        // Second tab area
        let x = TAB_BAR_LEFT_MARGIN + TAB_WIDTH + 20;
        assert_eq!(
            hit_test_tab_bar(x, y, tab_count, width),
            TabHitResult::Tab(1)
        );

        // New tab button area
        let x = TAB_BAR_LEFT_MARGIN + TAB_WIDTH * 2 + 10;
        assert_eq!(
            hit_test_tab_bar(x, y, tab_count, width),
            TabHitResult::NewTabButton
        );

        // Caption area (between new tab button and window buttons)
        let x = TAB_BAR_LEFT_MARGIN + TAB_WIDTH * 2 + NEW_TAB_BUTTON_WIDTH + 50;
        assert_eq!(
            hit_test_tab_bar(x, y, tab_count, width),
            TabHitResult::Caption
        );
    }

    #[test]
    fn test_calculate_drop_index() {
        let width = 1024;
        let tab_count = 3;

        // Position at first tab
        assert_eq!(
            calculate_drop_index(TAB_BAR_LEFT_MARGIN + 10, tab_count, width),
            0
        );

        // Position at second tab
        assert_eq!(
            calculate_drop_index(TAB_BAR_LEFT_MARGIN + TAB_WIDTH + 10, tab_count, width),
            1
        );

        // Position at third tab
        assert_eq!(
            calculate_drop_index(TAB_BAR_LEFT_MARGIN + TAB_WIDTH * 2 + 10, tab_count, width),
            2
        );

        // Position beyond last tab
        assert_eq!(
            calculate_drop_index(TAB_BAR_LEFT_MARGIN + TAB_WIDTH * 10, tab_count, width),
            2
        );

        // Position before first tab
        assert_eq!(calculate_drop_index(0, tab_count, width), 0);
    }

    #[test]
    fn test_calculate_swap_target() {
        let width = 1024;
        let tab_count = 3;

        // Tab at index 0, visual position at its normal spot - no swap
        let tab0_rect = get_tab_rect(0, width);
        assert_eq!(
            calculate_swap_target(0, tab0_rect.left, tab_count, width),
            None
        );

        // Tab at index 0, dragged right past center of tab 1 - should swap to index 1
        let tab1_rect = get_tab_rect(1, width);
        let tab1_center = (tab1_rect.left + tab1_rect.right) / 2;
        // Position where tab 0's center is past tab 1's center
        let visual_x = tab1_center - TAB_WIDTH / 2 + 1;
        assert_eq!(
            calculate_swap_target(0, visual_x, tab_count, width),
            Some(1)
        );

        // Tab at index 1, dragged left past center of tab 0 - should swap to index 0
        let tab0_center = (tab0_rect.left + tab0_rect.right) / 2;
        // Position where tab 1's center is past tab 0's center (to the left)
        let visual_x = tab0_center - TAB_WIDTH / 2 - 1;
        assert_eq!(
            calculate_swap_target(1, visual_x, tab_count, width),
            Some(0)
        );

        // Tab at index 0 (leftmost) - can't swap left
        let visual_x = -50; // Far left
        assert_eq!(calculate_swap_target(0, visual_x, tab_count, width), None);

        // Tab at index 2 (rightmost with 3 tabs) - can't swap right
        let tab2_rect = get_tab_rect(2, width);
        let visual_x = tab2_rect.left + TAB_WIDTH * 2; // Far right
        assert_eq!(calculate_swap_target(2, visual_x, tab_count, width), None);

        // Single tab - no swaps possible
        assert_eq!(calculate_swap_target(0, 0, 1, width), None);
    }
}
