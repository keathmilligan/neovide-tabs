#![cfg(target_os = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::collapsible_if)]

use anyhow::{Context, Result};
use std::cell::Cell;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BITMAP, BeginPaint, BitBlt, ClientToScreen, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateFontIndirectW, CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, EndPaint, FillRect,
    GetObjectW, GetTextExtentPoint32W, GetTextMetricsW, HBITMAP, HBRUSH, HGDIOBJ, InvalidateRect,
    LOGFONTW, LineTo, MoveToEx, PAINTSTRUCT, PS_SOLID, SRCCOPY, STRETCH_HALFTONE, ScreenToClient,
    SelectObject, SetBkMode, SetStretchBltMode, SetTextColor, StretchBlt, TEXTMETRICW, TRANSPARENT,
    TextOutW,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use crate::config::{Config, Profile};
use crate::hotkeys;
use crate::icons::{ICON_SIZE, create_window_icons, get_icon_bitmap};
use crate::tabs::{DragState, TabManager};

const WINDOW_CLASS_NAME: PCWSTR = w!("NeovideTabsWindow");
const DROPDOWN_CLASS_NAME: PCWSTR = w!("NeovideTabsDropdown");
const OVERFLOW_CLASS_NAME: PCWSTR = w!("NeovideTabsOverflow");
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
const TAB_WIDTH: i32 = 200;
/// Size of the close button within a tab
const TAB_CLOSE_SIZE: i32 = 16;
/// Padding around the close button
const TAB_CLOSE_PADDING: i32 = 8;
/// Width of the new tab (+) button
const NEW_TAB_BUTTON_WIDTH: i32 = 32;
/// Width of the profile dropdown button (caret)
const DROPDOWN_BUTTON_WIDTH: i32 = 20;
/// Width of the overflow tabs button (accommodates icon + "+N" text when selected tab is in overflow)
const OVERFLOW_BUTTON_WIDTH: i32 = 48;
/// Left margin before the first tab
const TAB_BAR_LEFT_MARGIN: i32 = 8;
/// Vertical padding for tabs within the titlebar
const TAB_VERTICAL_PADDING: i32 = 4;
/// Height of each item in the dropdown menu
const DROPDOWN_ITEM_HEIGHT: i32 = 28;
/// Padding around dropdown menu
const DROPDOWN_PADDING: i32 = 4;

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
    /// Hit the profile dropdown button (caret)
    ProfileDropdown,
    /// Hit a profile in the dropdown menu (index)
    DropdownItem(usize),
    /// Hit the overflow tabs dropdown button
    OverflowButton,
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
    /// Hovering over profile dropdown button
    ProfileDropdown,
    /// Hovering over dropdown menu item (index) - unused, popup handles hover
    #[allow(dead_code)]
    DropdownItem(usize),
    /// Hovering over overflow tabs button
    OverflowButton,
}

/// State of the profile dropdown menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DropdownState {
    /// Dropdown is closed
    Closed,
    /// Dropdown is open
    Open,
}

/// Application state stored in window user data
struct WindowState {
    tab_manager: TabManager,
    config: Config,
    in_size_move: bool,
    background_color: u32,
    hovered_button: HoveredButton,
    hovered_tab: HoveredTab,
    tracking_mouse: bool,
    dropdown_state: DropdownState,
    dropdown_hwnd: Option<HWND>,
    /// Handle to the overflow tabs popup window (if open)
    overflow_hwnd: Option<HWND>,
    /// IDs of registered global hotkeys (for cleanup on exit)
    registered_hotkeys: Vec<i32>,
}

/// State for the dropdown popup window
struct DropdownPopupState {
    parent_hwnd: HWND,
    profiles: Vec<Profile>,
    hovered_item: Option<usize>,
    background_color: u32,
}

/// Info about an overflow tab for the popup
struct OverflowTabInfo {
    /// Original index of this tab in the tab manager
    index: usize,
    /// Tab label
    label: String,
    /// Icon filename
    icon: String,
    /// Whether this tab is selected
    is_selected: bool,
}

/// State for the overflow tabs popup window
struct OverflowPopupState {
    parent_hwnd: HWND,
    tabs: Vec<OverflowTabInfo>,
    hovered_item: Option<usize>,
    background_color: u32,
}

// Thread-local storage for config during window creation
thread_local! {
    static INITIAL_BG_COLOR: Cell<u32> = const { Cell::new(0x1a1b26) };
    static INITIAL_CONFIG: std::cell::RefCell<Option<Config>> = const { std::cell::RefCell::new(None) };
}

/// Convert RGB color (0x00RRGGBB) to Win32 COLORREF (0x00BBGGRR)
fn rgb_to_colorref(rgb: u32) -> u32 {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;
    (b << 16) | (g << 8) | r
}

/// Register the window class with Win32
pub fn register_window_class(config: Config) -> Result<()> {
    // Store config for use in WM_CREATE
    let background_color = config.background_color;
    INITIAL_BG_COLOR.with(|c| c.set(background_color));
    INITIAL_CONFIG.with(|c| *c.borrow_mut() = Some(config));

    unsafe {
        let hinstance = GetModuleHandleW(None).context("Failed to get module handle")?;

        // Create a solid brush for the background color
        let colorref = rgb_to_colorref(background_color);
        let brush = CreateSolidBrush(COLORREF(colorref));

        // Create window icon from bundled image
        let window_icon = create_window_icons()
            .map(|(large, _small)| large)
            .unwrap_or_default();

        // Register main window class
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: window_icon,
            hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap_or_default(),
            hbrBackground: HBRUSH(brush.0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: WINDOW_CLASS_NAME,
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            anyhow::bail!("Failed to register window class");
        }

        // Register dropdown popup window class
        let dropdown_brush = CreateSolidBrush(COLORREF(colorref));
        let dropdown_wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
            lpfnWndProc: Some(dropdown_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: Default::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap_or_default(),
            hbrBackground: HBRUSH(dropdown_brush.0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: DROPDOWN_CLASS_NAME,
        };

        let dropdown_atom = RegisterClassW(&dropdown_wc);
        if dropdown_atom == 0 {
            anyhow::bail!("Failed to register dropdown window class");
        }

        // Register overflow tabs popup window class
        let overflow_brush = CreateSolidBrush(COLORREF(colorref));
        let overflow_wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
            lpfnWndProc: Some(overflow_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: Default::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap_or_default(),
            hbrBackground: HBRUSH(overflow_brush.0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: OVERFLOW_CLASS_NAME,
        };

        let overflow_atom = RegisterClassW(&overflow_wc);
        if overflow_atom == 0 {
            anyhow::bail!("Failed to register overflow window class");
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
/// When has_overflow is true, it's positioned after the overflow button
fn get_new_tab_button_rect_ex(
    visible_tab_count: usize,
    has_overflow: bool,
    client_width: i32,
) -> RECT {
    let _ = client_width; // Reserved for future dynamic sizing
    let left = if has_overflow {
        TAB_BAR_LEFT_MARGIN + (visible_tab_count as i32 * TAB_WIDTH) + OVERFLOW_BUTTON_WIDTH
    } else {
        TAB_BAR_LEFT_MARGIN + (visible_tab_count as i32 * TAB_WIDTH)
    };
    RECT {
        left,
        top: TAB_VERTICAL_PADDING,
        right: left + NEW_TAB_BUTTON_WIDTH,
        bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
    }
}

/// Get the rectangle for the new tab (+) button (legacy, assumes no overflow)
#[allow(dead_code)]
fn get_new_tab_button_rect(tab_count: usize, client_width: i32) -> RECT {
    get_new_tab_button_rect_ex(tab_count, false, client_width)
}

/// Get the rectangle for the profile dropdown button (caret)
fn get_dropdown_button_rect_ex(
    visible_tab_count: usize,
    has_overflow: bool,
    client_width: i32,
) -> RECT {
    let new_tab_rect = get_new_tab_button_rect_ex(visible_tab_count, has_overflow, client_width);
    RECT {
        left: new_tab_rect.right,
        top: TAB_VERTICAL_PADDING,
        right: new_tab_rect.right + DROPDOWN_BUTTON_WIDTH,
        bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
    }
}

/// Get the rectangle for the profile dropdown button (legacy, assumes no overflow)
fn get_dropdown_button_rect(tab_count: usize, client_width: i32) -> RECT {
    get_dropdown_button_rect_ex(tab_count, false, client_width)
}

/// Get the rectangle for the dropdown menu (unused - popup handles its own layout)
#[allow(dead_code)]
fn get_dropdown_menu_rect(tab_count: usize, profile_count: usize, client_width: i32) -> RECT {
    let dropdown_btn = get_dropdown_button_rect(tab_count, client_width);
    let menu_width = 180; // Fixed width for dropdown menu
    let menu_height = (profile_count as i32 * DROPDOWN_ITEM_HEIGHT) + (DROPDOWN_PADDING * 2);

    // Position below the dropdown button, aligned to its left edge
    RECT {
        left: dropdown_btn.left,
        top: TITLEBAR_HEIGHT,
        right: dropdown_btn.left + menu_width,
        bottom: TITLEBAR_HEIGHT + menu_height,
    }
}

/// Get the rectangle for a dropdown menu item (unused - popup handles its own layout)
#[allow(dead_code)]
fn get_dropdown_item_rect(
    item_index: usize,
    tab_count: usize,
    profile_count: usize,
    client_width: i32,
) -> RECT {
    let menu_rect = get_dropdown_menu_rect(tab_count, profile_count, client_width);
    let top = menu_rect.top + DROPDOWN_PADDING + (item_index as i32 * DROPDOWN_ITEM_HEIGHT);
    RECT {
        left: menu_rect.left + DROPDOWN_PADDING,
        top,
        right: menu_rect.right - DROPDOWN_PADDING,
        bottom: top + DROPDOWN_ITEM_HEIGHT,
    }
}

/// Get the maximum X position for the tab bar (before window buttons)
fn get_tab_bar_max_x(client_width: i32) -> i32 {
    client_width - (BUTTON_WIDTH * 3) - 8 // Leave some padding before window buttons
}

/// Calculate how many tabs can be displayed before overflow
/// Returns (visible_count, has_overflow)
fn calculate_visible_tabs(tab_count: usize, client_width: i32) -> (usize, bool) {
    if tab_count == 0 {
        return (0, false);
    }

    let max_x = get_tab_bar_max_x(client_width);
    // Reserve space for new tab button, dropdown button, and potentially overflow button
    let reserved_space = NEW_TAB_BUTTON_WIDTH + DROPDOWN_BUTTON_WIDTH + OVERFLOW_BUTTON_WIDTH;
    let available_width = max_x - TAB_BAR_LEFT_MARGIN - reserved_space;

    let max_visible = (available_width / TAB_WIDTH).max(0) as usize;

    if max_visible >= tab_count {
        // All tabs fit (no overflow button needed, so we can reclaim that space)
        let available_without_overflow =
            max_x - TAB_BAR_LEFT_MARGIN - NEW_TAB_BUTTON_WIDTH - DROPDOWN_BUTTON_WIDTH;
        let max_visible_no_overflow = (available_without_overflow / TAB_WIDTH).max(0) as usize;
        if max_visible_no_overflow >= tab_count {
            return (tab_count, false);
        }
    }

    // Need overflow
    (max_visible.min(tab_count), max_visible < tab_count)
}

/// Get the rectangle for the overflow button
fn get_overflow_button_rect(visible_tab_count: usize, client_width: i32) -> RECT {
    let _ = client_width; // Reserved for future dynamic sizing
    let left = TAB_BAR_LEFT_MARGIN + (visible_tab_count as i32 * TAB_WIDTH);
    RECT {
        left,
        top: TAB_VERTICAL_PADDING,
        right: left + OVERFLOW_BUTTON_WIDTH,
        bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
    }
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
    let (visible_count, has_overflow) = calculate_visible_tabs(tab_count, client_width);

    // Check each visible tab
    for i in 0..visible_count {
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

    // Check overflow button if there are overflow tabs
    if has_overflow {
        let overflow_rect = get_overflow_button_rect(visible_count, client_width);
        if x >= overflow_rect.left && x < overflow_rect.right {
            return TabHitResult::OverflowButton;
        }
    }

    // Check new tab button
    let new_tab_rect = get_new_tab_button_rect_ex(visible_count, has_overflow, client_width);
    if new_tab_rect.right <= max_x {
        if x >= new_tab_rect.left && x < new_tab_rect.right {
            return TabHitResult::NewTabButton;
        }
    }

    // Check dropdown button
    let dropdown_rect = get_dropdown_button_rect_ex(visible_count, has_overflow, client_width);
    if dropdown_rect.right <= max_x {
        if x >= dropdown_rect.left && x < dropdown_rect.right {
            return TabHitResult::ProfileDropdown;
        }
    }

    TabHitResult::Caption
}

/// Hit test in the dropdown menu area (unused - popup handles its own hit testing)
#[allow(dead_code)]
fn hit_test_dropdown_menu(
    x: i32,
    y: i32,
    tab_count: usize,
    profile_count: usize,
    client_width: i32,
) -> TabHitResult {
    let menu_rect = get_dropdown_menu_rect(tab_count, profile_count, client_width);

    // Check if in the menu bounds
    if x >= menu_rect.left && x < menu_rect.right && y >= menu_rect.top && y < menu_rect.bottom {
        // Check which item
        for i in 0..profile_count {
            let item_rect = get_dropdown_item_rect(i, tab_count, profile_count, client_width);
            if y >= item_rect.top && y < item_rect.bottom {
                return TabHitResult::DropdownItem(i);
            }
        }
    }

    TabHitResult::None
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
#[allow(unused_must_use, clippy::too_many_arguments)]
fn paint_tab(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    tab_rect: &RECT,
    label: &str,
    icon_filename: Option<&str>,
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

        // Draw outline around tab (top, left, right)
        // For selected tabs, extend sides down to the bottom line (TITLEBAR_HEIGHT - 1)
        // For unselected tabs, stop at the tab rect bottom
        let outline_pen = CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
        let old_pen = SelectObject(hdc, HGDIOBJ(outline_pen.0));

        // Selected tabs extend down to connect with the tab bar bottom line
        let side_bottom = if is_selected {
            TITLEBAR_HEIGHT - 1
        } else {
            tab_rect.bottom
        };

        MoveToEx(hdc, tab_rect.left, side_bottom, None);
        LineTo(hdc, tab_rect.left, tab_rect.top);
        LineTo(hdc, tab_rect.right - 1, tab_rect.top);
        LineTo(hdc, tab_rect.right - 1, side_bottom);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(outline_pen.0));

        // Calculate icon position (centered vertically, with padding from left)
        let icon_x = tab_rect.left + 6;
        let icon_y = (tab_rect.top + tab_rect.bottom - ICON_SIZE) / 2;

        // Draw icon if available
        let label_offset = if let Some(filename) = icon_filename {
            if let Some(hbitmap) = get_icon_bitmap(filename) {
                paint_icon(hdc, hbitmap, icon_x, icon_y, ICON_SIZE, ICON_SIZE);
                ICON_SIZE + 4 // Icon width + padding
            } else {
                0
            }
        } else {
            0
        };

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

        // Get actual text metrics for proper vertical centering
        let mut tm = TEXTMETRICW::default();
        GetTextMetricsW(hdc, &mut tm);
        let text_height = tm.tmHeight;

        // Label position (after icon, leaving room for close button)
        // Center text vertically using actual text height
        let label_x = tab_rect.left + 6 + label_offset;
        let label_y = (tab_rect.top + tab_rect.bottom - text_height) / 2;

        // Calculate available width for text (between icon and close button)
        let close_rect = get_tab_close_rect(tab_rect);
        let max_text_width = close_rect.left - label_x - 4; // 4px padding before close button

        // Measure text width and truncate with ellipsis if needed
        let label_wide: Vec<u16> = label.encode_utf16().collect();
        let mut text_size = SIZE::default();
        GetTextExtentPoint32W(hdc, &label_wide, &mut text_size);

        if text_size.cx <= max_text_width {
            // Text fits - draw normally
            TextOutW(hdc, label_x, label_y, &label_wide);
        } else {
            // Text too wide - truncate with ellipsis
            let ellipsis = "...";
            let ellipsis_wide: Vec<u16> = ellipsis.encode_utf16().collect();
            let mut ellipsis_size = SIZE::default();
            GetTextExtentPoint32W(hdc, &ellipsis_wide, &mut ellipsis_size);

            let available_for_text = max_text_width - ellipsis_size.cx;
            if available_for_text > 0 {
                // Find how many characters fit
                let mut truncated = String::new();
                for ch in label.chars() {
                    let test = format!("{}{}", truncated, ch);
                    let test_wide: Vec<u16> = test.encode_utf16().collect();
                    let mut test_size = SIZE::default();
                    GetTextExtentPoint32W(hdc, &test_wide, &mut test_size);
                    if test_size.cx > available_for_text {
                        break;
                    }
                    truncated.push(ch);
                }
                truncated.push_str(ellipsis);
                let truncated_wide: Vec<u16> = truncated.encode_utf16().collect();
                TextOutW(hdc, label_x, label_y, &truncated_wide);
            } else {
                // Not even ellipsis fits - just draw ellipsis
                TextOutW(hdc, label_x, label_y, &ellipsis_wide);
            }
        }

        SelectObject(hdc, old_font);
        DeleteObject(HGDIOBJ(font.0));

        // Draw close button (close_rect already calculated above for text truncation)
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

/// Paint an icon bitmap to the device context
#[allow(unused_must_use)]
fn paint_icon(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    hbitmap: HBITMAP,
    x: i32,
    y: i32,
    dest_width: i32,
    dest_height: i32,
) {
    unsafe {
        // Get bitmap dimensions
        let mut bm = BITMAP::default();
        let bm_size = std::mem::size_of::<BITMAP>() as i32;
        if GetObjectW(
            HGDIOBJ(hbitmap.0),
            bm_size,
            Some(&mut bm as *mut _ as *mut std::ffi::c_void),
        ) == 0
        {
            return;
        }

        // Create compatible DC for the bitmap
        let mem_dc = CreateCompatibleDC(hdc);
        if mem_dc.is_invalid() {
            return;
        }

        let old_bitmap = SelectObject(mem_dc, HGDIOBJ(hbitmap.0));

        // Set stretch mode for better quality
        SetStretchBltMode(hdc, STRETCH_HALFTONE);

        // Stretch blit the bitmap to the destination
        StretchBlt(
            hdc,
            x,
            y,
            dest_width,
            dest_height,
            mem_dc,
            0,
            0,
            bm.bmWidth,
            bm.bmHeight,
            SRCCOPY,
        );

        // Clean up
        SelectObject(mem_dc, old_bitmap);
        DeleteDC(mem_dc);
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

/// Paint the profile dropdown button (downward caret)
#[allow(unused_must_use)]
fn paint_dropdown_button(hdc: windows::Win32::Graphics::Gdi::HDC, rect: &RECT, is_hovered: bool) {
    unsafe {
        // Background on hover
        if is_hovered {
            let hover_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(TAB_HOVER_COLOR)));
            FillRect(hdc, rect, hover_brush);
            DeleteObject(HGDIOBJ(hover_brush.0));
        }

        // Draw downward caret icon
        let pen = CreatePen(PS_SOLID, 1, COLORREF(0x00FFFFFF));
        let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));

        let cx = (rect.left + rect.right) / 2;
        let cy = (rect.top + rect.bottom) / 2;
        let size = 4;

        // Draw V shape (downward caret)
        MoveToEx(hdc, cx - size, cy - 2, None);
        LineTo(hdc, cx, cy + 2);
        LineTo(hdc, cx + size + 1, cy - 3);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(pen.0));
    }
}

/// Paint the dropdown menu (unused - popup renders itself)
#[allow(unused_must_use, dead_code)]
fn paint_dropdown_menu(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    profiles: &[Profile],
    tab_count: usize,
    hovered_item: Option<usize>,
    client_width: i32,
    background_color: u32,
) {
    unsafe {
        let profile_count = profiles.len();
        let menu_rect = get_dropdown_menu_rect(tab_count, profile_count, client_width);

        // Draw menu background
        let bg_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(background_color)));
        FillRect(hdc, &menu_rect, bg_brush);
        DeleteObject(HGDIOBJ(bg_brush.0));

        // Draw menu border
        let border_pen = CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
        let old_pen = SelectObject(hdc, HGDIOBJ(border_pen.0));

        MoveToEx(hdc, menu_rect.left, menu_rect.top, None);
        LineTo(hdc, menu_rect.right - 1, menu_rect.top);
        LineTo(hdc, menu_rect.right - 1, menu_rect.bottom - 1);
        LineTo(hdc, menu_rect.left, menu_rect.bottom - 1);
        LineTo(hdc, menu_rect.left, menu_rect.top);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(border_pen.0));

        // Draw each menu item
        for (i, profile) in profiles.iter().enumerate() {
            let item_rect = get_dropdown_item_rect(i, tab_count, profile_count, client_width);
            let is_hovered = hovered_item == Some(i);

            // Item background on hover
            if is_hovered {
                let hover_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(TAB_HOVER_COLOR)));
                FillRect(hdc, &item_rect, hover_brush);
                DeleteObject(HGDIOBJ(hover_brush.0));
            }

            // Draw profile name
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(0x00FFFFFF)); // White text

            let mut lf = LOGFONTW::default();
            lf.lfHeight = -12;
            lf.lfWeight = 400;
            let font_name = "Segoe UI";
            for (j, c) in font_name.encode_utf16().enumerate() {
                if j < 32 {
                    lf.lfFaceName[j] = c;
                }
            }
            let font = CreateFontIndirectW(&lf);
            let old_font = SelectObject(hdc, HGDIOBJ(font.0));

            // Text position (with left padding for icon space)
            let text_x = item_rect.left + 24; // Leave space for icon
            let text_y = (item_rect.top + item_rect.bottom - 12) / 2;
            let name_wide: Vec<u16> = profile.name.encode_utf16().collect();
            TextOutW(hdc, text_x, text_y, &name_wide);

            SelectObject(hdc, old_font);
            DeleteObject(HGDIOBJ(font.0));
        }
    }
}

/// Create the dropdown popup window
fn create_dropdown_popup(
    parent_hwnd: HWND,
    profiles: Vec<Profile>,
    background_color: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Option<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None).ok()?;

        // Create popup state
        let popup_state = Box::new(DropdownPopupState {
            parent_hwnd,
            profiles,
            hovered_item: None,
            background_color,
        });

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            DROPDOWN_CLASS_NAME,
            w!(""),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            width,
            height,
            None, // No parent - independent window
            None,
            hinstance,
            Some(Box::into_raw(popup_state) as *const std::ffi::c_void),
        )
        .ok()?;

        Some(hwnd)
    }
}

/// Show the dropdown popup at the appropriate position
#[allow(unused_must_use)]
fn show_dropdown_popup(parent_hwnd: HWND, state: &mut WindowState) {
    // Close any existing popup first
    if let Some(popup_hwnd) = state.dropdown_hwnd.take() {
        unsafe {
            DestroyWindow(popup_hwnd).ok();
        }
    }

    unsafe {
        let mut client_rect = RECT::default();
        if GetClientRect(parent_hwnd, &mut client_rect).is_err() {
            return;
        }

        let (visible_count, has_overflow) =
            calculate_visible_tabs(state.tab_manager.count(), client_rect.right);
        let dropdown_btn =
            get_dropdown_button_rect_ex(visible_count, has_overflow, client_rect.right);

        // Convert button position to screen coordinates
        let mut screen_pt = POINT {
            x: dropdown_btn.left,
            y: dropdown_btn.bottom,
        };
        ClientToScreen(parent_hwnd, &mut screen_pt);

        let profile_count = state.config.profiles.len();
        let menu_width = 150;
        let menu_height = (profile_count as i32 * DROPDOWN_ITEM_HEIGHT) + (DROPDOWN_PADDING * 2);

        // IMPORTANT: Clicking on our title bar brought our window to the foreground,
        // which covers the Neovide window. We need to bring Neovide back to the
        // foreground BEFORE showing the popup (which is topmost and will appear above it).
        state.tab_manager.bring_selected_to_foreground();

        if let Some(popup_hwnd) = create_dropdown_popup(
            parent_hwnd,
            state.config.profiles.clone(),
            state.background_color,
            screen_pt.x,
            screen_pt.y,
            menu_width,
            menu_height,
        ) {
            state.dropdown_hwnd = Some(popup_hwnd);
            state.dropdown_state = DropdownState::Open;
        }
    }
}

/// Hide and destroy the dropdown popup
fn hide_dropdown_popup(_parent_hwnd: HWND, state: &mut WindowState) {
    if let Some(popup_hwnd) = state.dropdown_hwnd.take() {
        unsafe {
            ReleaseCapture().ok();
            DestroyWindow(popup_hwnd).ok();
        }
    }
    state.dropdown_state = DropdownState::Closed;
}

/// Window procedure for the dropdown popup
#[allow(unused_must_use)]
unsafe extern "system" fn dropdown_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                if !create_struct.is_null() {
                    let state_ptr = (*create_struct).lpCreateParams as *mut DropdownPopupState;
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                }
                // Capture mouse to detect clicks outside the popup
                SetCapture(hwnd);
                LRESULT(0)
            }

            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DropdownPopupState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;

                    let mut rect = RECT::default();
                    GetClientRect(hwnd, &mut rect).ok();

                    // Fill background
                    let bg_brush =
                        CreateSolidBrush(COLORREF(rgb_to_colorref(state.background_color)));
                    FillRect(hdc, &rect, bg_brush);
                    DeleteObject(HGDIOBJ(bg_brush.0));

                    // Draw border
                    let border_pen =
                        CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
                    let old_pen = SelectObject(hdc, HGDIOBJ(border_pen.0));
                    MoveToEx(hdc, rect.left, rect.top, None);
                    LineTo(hdc, rect.right - 1, rect.top);
                    LineTo(hdc, rect.right - 1, rect.bottom - 1);
                    LineTo(hdc, rect.left, rect.bottom - 1);
                    LineTo(hdc, rect.left, rect.top);
                    SelectObject(hdc, old_pen);
                    DeleteObject(HGDIOBJ(border_pen.0));

                    // Draw each profile item
                    for (i, profile) in state.profiles.iter().enumerate() {
                        let item_top = DROPDOWN_PADDING + (i as i32 * DROPDOWN_ITEM_HEIGHT);
                        let item_rect = RECT {
                            left: DROPDOWN_PADDING,
                            top: item_top,
                            right: rect.right - DROPDOWN_PADDING,
                            bottom: item_top + DROPDOWN_ITEM_HEIGHT,
                        };

                        // Hover background
                        if state.hovered_item == Some(i) {
                            let hover_brush =
                                CreateSolidBrush(COLORREF(rgb_to_colorref(TAB_HOVER_COLOR)));
                            FillRect(hdc, &item_rect, hover_brush);
                            DeleteObject(HGDIOBJ(hover_brush.0));
                        }

                        // Draw icon
                        let icon_x = item_rect.left + 4;
                        let icon_y = (item_rect.top + item_rect.bottom - ICON_SIZE) / 2;
                        if let Some(hbitmap) = get_icon_bitmap(&profile.icon) {
                            paint_icon(hdc, hbitmap, icon_x, icon_y, ICON_SIZE, ICON_SIZE);
                        }

                        // Draw text
                        SetBkMode(hdc, TRANSPARENT);
                        SetTextColor(hdc, COLORREF(0x00FFFFFF));

                        let mut lf = LOGFONTW::default();
                        lf.lfHeight = -12;
                        lf.lfWeight = 400;
                        let font_name = "Segoe UI";
                        for (j, c) in font_name.encode_utf16().enumerate() {
                            if j < 32 {
                                lf.lfFaceName[j] = c;
                            }
                        }
                        let font = CreateFontIndirectW(&lf);
                        let old_font = SelectObject(hdc, HGDIOBJ(font.0));

                        // Get actual text metrics for proper vertical centering
                        let mut tm = TEXTMETRICW::default();
                        GetTextMetricsW(hdc, &mut tm);
                        let text_height = tm.tmHeight;

                        // Text position after icon, vertically centered
                        let text_x = item_rect.left + ICON_SIZE + 8;
                        let text_y = (item_rect.top + item_rect.bottom - text_height) / 2;
                        let name_wide: Vec<u16> = profile.name.encode_utf16().collect();
                        TextOutW(hdc, text_x, text_y, &name_wide);

                        SelectObject(hdc, old_font);
                        DeleteObject(HGDIOBJ(font.0));
                    }
                }

                EndPaint(hwnd, &ps);
                LRESULT(0)
            }

            WM_MOUSEMOVE => {
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DropdownPopupState;
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;

                    // Calculate which item is hovered
                    let item_index = (y - DROPDOWN_PADDING) / DROPDOWN_ITEM_HEIGHT;
                    let new_hovered =
                        if item_index >= 0 && (item_index as usize) < state.profiles.len() {
                            Some(item_index as usize)
                        } else {
                            None
                        };

                    if state.hovered_item != new_hovered {
                        state.hovered_item = new_hovered;
                        InvalidateRect(hwnd, None, false);
                    }
                }
                LRESULT(0)
            }

            WM_LBUTTONDOWN => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DropdownPopupState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;

                    // Check if click is inside the popup
                    let mut rect = RECT::default();
                    GetClientRect(hwnd, &mut rect).ok();

                    if x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom {
                        // Click inside - check which item
                        let item_index = (y - DROPDOWN_PADDING) / DROPDOWN_ITEM_HEIGHT;
                        if item_index >= 0 && (item_index as usize) < state.profiles.len() {
                            // Send custom message to parent with profile index
                            let profile_index = item_index as usize;
                            PostMessageW(
                                state.parent_hwnd,
                                WM_APP,
                                WPARAM(profile_index),
                                LPARAM(0),
                            )
                            .ok();
                        }
                    } else {
                        // Click outside - just notify parent to close
                        PostMessageW(state.parent_hwnd, WM_APP + 1, WPARAM(0), LPARAM(0)).ok();
                    }
                    // Release capture and close popup
                    ReleaseCapture().ok();
                    DestroyWindow(hwnd).ok();
                }
                LRESULT(0)
            }

            WM_CAPTURECHANGED => {
                // We lost capture - close the popup
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DropdownPopupState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    PostMessageW(state.parent_hwnd, WM_APP + 1, WPARAM(0), LPARAM(0)).ok();
                }
                DestroyWindow(hwnd).ok();
                LRESULT(0)
            }

            WM_DESTROY => {
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DropdownPopupState;
                if !state_ptr.is_null() {
                    // Free the state
                    let _ = Box::from_raw(state_ptr);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Create the overflow tabs popup window
fn create_overflow_popup(
    parent_hwnd: HWND,
    tabs: Vec<OverflowTabInfo>,
    background_color: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Option<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None).ok()?;

        // Create popup state
        let popup_state = Box::new(OverflowPopupState {
            parent_hwnd,
            tabs,
            hovered_item: None,
            background_color,
        });

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            OVERFLOW_CLASS_NAME,
            w!(""),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            width,
            height,
            None, // No parent - independent window
            None,
            hinstance,
            Some(Box::into_raw(popup_state) as *const std::ffi::c_void),
        )
        .ok()?;

        Some(hwnd)
    }
}

/// Show the overflow tabs popup at the appropriate position
#[allow(unused_must_use)]
fn show_overflow_popup(parent_hwnd: HWND, state: &mut WindowState, client_width: i32) {
    // Close any existing popup first
    if let Some(popup_hwnd) = state.overflow_hwnd.take() {
        unsafe {
            DestroyWindow(popup_hwnd).ok();
        }
    }

    let (visible_count, has_overflow) =
        calculate_visible_tabs(state.tab_manager.count(), client_width);

    if !has_overflow {
        return;
    }

    // Collect overflow tabs info
    let mut overflow_tabs = Vec::new();
    let selected_index = state.tab_manager.selected_index();
    for i in visible_count..state.tab_manager.count() {
        overflow_tabs.push(OverflowTabInfo {
            index: i,
            label: state.tab_manager.get_tab_label(i),
            icon: state
                .tab_manager
                .get_tab_icon(i)
                .unwrap_or_default()
                .to_string(),
            is_selected: i == selected_index,
        });
    }

    if overflow_tabs.is_empty() {
        return;
    }

    unsafe {
        let overflow_btn = get_overflow_button_rect(visible_count, client_width);

        // Convert button position to screen coordinates
        let mut screen_pt = POINT {
            x: overflow_btn.left,
            y: overflow_btn.bottom,
        };
        ClientToScreen(parent_hwnd, &mut screen_pt);

        let tab_count = overflow_tabs.len();
        let menu_width = TAB_WIDTH; // Same width as tabs
        let menu_height = (tab_count as i32 * DROPDOWN_ITEM_HEIGHT) + (DROPDOWN_PADDING * 2);

        // Bring Neovide back to foreground before showing popup
        state.tab_manager.bring_selected_to_foreground();

        if let Some(popup_hwnd) = create_overflow_popup(
            parent_hwnd,
            overflow_tabs,
            state.background_color,
            screen_pt.x,
            screen_pt.y,
            menu_width,
            menu_height,
        ) {
            state.overflow_hwnd = Some(popup_hwnd);
        }
    }
}

/// Hide and destroy the overflow popup
fn hide_overflow_popup(_parent_hwnd: HWND, state: &mut WindowState) {
    if let Some(popup_hwnd) = state.overflow_hwnd.take() {
        unsafe {
            ReleaseCapture().ok();
            DestroyWindow(popup_hwnd).ok();
        }
    }
}

/// Window procedure for the overflow tabs popup
#[allow(unused_must_use)]
unsafe extern "system" fn overflow_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                if !create_struct.is_null() {
                    let state_ptr = (*create_struct).lpCreateParams as *mut OverflowPopupState;
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                }
                // Capture mouse to detect clicks outside the popup
                SetCapture(hwnd);
                LRESULT(0)
            }

            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverflowPopupState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;

                    let mut rect = RECT::default();
                    GetClientRect(hwnd, &mut rect).ok();

                    // Fill background
                    let bg_brush =
                        CreateSolidBrush(COLORREF(rgb_to_colorref(state.background_color)));
                    FillRect(hdc, &rect, bg_brush);
                    DeleteObject(HGDIOBJ(bg_brush.0));

                    // Draw border
                    let border_pen =
                        CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
                    let old_pen = SelectObject(hdc, HGDIOBJ(border_pen.0));
                    MoveToEx(hdc, rect.left, rect.top, None);
                    LineTo(hdc, rect.right - 1, rect.top);
                    LineTo(hdc, rect.right - 1, rect.bottom - 1);
                    LineTo(hdc, rect.left, rect.bottom - 1);
                    LineTo(hdc, rect.left, rect.top);
                    SelectObject(hdc, old_pen);
                    DeleteObject(HGDIOBJ(border_pen.0));

                    // Draw each overflow tab item
                    for (i, tab_info) in state.tabs.iter().enumerate() {
                        let item_top = DROPDOWN_PADDING + (i as i32 * DROPDOWN_ITEM_HEIGHT);
                        let item_rect = RECT {
                            left: DROPDOWN_PADDING,
                            top: item_top,
                            right: rect.right - DROPDOWN_PADDING,
                            bottom: item_top + DROPDOWN_ITEM_HEIGHT,
                        };

                        // Hover or selected background
                        if state.hovered_item == Some(i) || tab_info.is_selected {
                            let bg_color = if state.hovered_item == Some(i) {
                                TAB_HOVER_COLOR
                            } else {
                                TAB_UNSELECTED_COLOR
                            };
                            let item_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(bg_color)));
                            FillRect(hdc, &item_rect, item_brush);
                            DeleteObject(HGDIOBJ(item_brush.0));
                        }

                        // Draw icon
                        let icon_x = item_rect.left + 4;
                        let icon_y = (item_rect.top + item_rect.bottom - ICON_SIZE) / 2;
                        if let Some(hbitmap) = get_icon_bitmap(&tab_info.icon) {
                            paint_icon(hdc, hbitmap, icon_x, icon_y, ICON_SIZE, ICON_SIZE);
                        }

                        // Draw text
                        SetBkMode(hdc, TRANSPARENT);
                        SetTextColor(hdc, COLORREF(0x00FFFFFF));

                        let mut lf = LOGFONTW::default();
                        lf.lfHeight = -12;
                        lf.lfWeight = if tab_info.is_selected { 700 } else { 400 };
                        let font_name = "Segoe UI";
                        for (j, c) in font_name.encode_utf16().enumerate() {
                            if j < 32 {
                                lf.lfFaceName[j] = c;
                            }
                        }
                        let font = CreateFontIndirectW(&lf);
                        let old_font = SelectObject(hdc, HGDIOBJ(font.0));

                        // Get actual text metrics for proper vertical centering
                        let mut tm = TEXTMETRICW::default();
                        GetTextMetricsW(hdc, &mut tm);
                        let text_height = tm.tmHeight;

                        // Text position after icon, vertically centered
                        let text_x = item_rect.left + ICON_SIZE + 8;
                        let text_y = (item_rect.top + item_rect.bottom - text_height) / 2;

                        // Calculate available width for text and truncate if needed
                        let max_text_width = item_rect.right - text_x - 4;
                        let label_wide: Vec<u16> = tab_info.label.encode_utf16().collect();
                        let mut text_size = SIZE::default();
                        GetTextExtentPoint32W(hdc, &label_wide, &mut text_size);

                        if text_size.cx <= max_text_width {
                            TextOutW(hdc, text_x, text_y, &label_wide);
                        } else {
                            // Truncate with ellipsis
                            let ellipsis = "...";
                            let ellipsis_wide: Vec<u16> = ellipsis.encode_utf16().collect();
                            let mut ellipsis_size = SIZE::default();
                            GetTextExtentPoint32W(hdc, &ellipsis_wide, &mut ellipsis_size);

                            let available_for_text = max_text_width - ellipsis_size.cx;
                            if available_for_text > 0 {
                                let mut truncated = String::new();
                                for ch in tab_info.label.chars() {
                                    let test = format!("{}{}", truncated, ch);
                                    let test_wide: Vec<u16> = test.encode_utf16().collect();
                                    let mut test_size = SIZE::default();
                                    GetTextExtentPoint32W(hdc, &test_wide, &mut test_size);
                                    if test_size.cx > available_for_text {
                                        break;
                                    }
                                    truncated.push(ch);
                                }
                                truncated.push_str(ellipsis);
                                let truncated_wide: Vec<u16> = truncated.encode_utf16().collect();
                                TextOutW(hdc, text_x, text_y, &truncated_wide);
                            } else {
                                TextOutW(hdc, text_x, text_y, &ellipsis_wide);
                            }
                        }

                        SelectObject(hdc, old_font);
                        DeleteObject(HGDIOBJ(font.0));
                    }
                }

                EndPaint(hwnd, &ps);
                LRESULT(0)
            }

            WM_MOUSEMOVE => {
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverflowPopupState;
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;

                    // Calculate which item is hovered
                    let item_index = (y - DROPDOWN_PADDING) / DROPDOWN_ITEM_HEIGHT;
                    let new_hovered = if item_index >= 0 && (item_index as usize) < state.tabs.len()
                    {
                        Some(item_index as usize)
                    } else {
                        None
                    };

                    if state.hovered_item != new_hovered {
                        state.hovered_item = new_hovered;
                        InvalidateRect(hwnd, None, false);
                    }
                }
                LRESULT(0)
            }

            WM_LBUTTONDOWN => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverflowPopupState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;

                    // Check if click is inside the popup
                    let mut rect = RECT::default();
                    GetClientRect(hwnd, &mut rect).ok();

                    if x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom {
                        // Click inside - check which item
                        let item_index = (y - DROPDOWN_PADDING) / DROPDOWN_ITEM_HEIGHT;
                        if item_index >= 0 && (item_index as usize) < state.tabs.len() {
                            // Send custom message to parent with original tab index
                            let tab_index = state.tabs[item_index as usize].index;
                            PostMessageW(
                                state.parent_hwnd,
                                WM_APP + 2, // New message for overflow tab selection
                                WPARAM(tab_index),
                                LPARAM(0),
                            )
                            .ok();
                        }
                    } else {
                        // Click outside - just notify parent to close
                        PostMessageW(state.parent_hwnd, WM_APP + 3, WPARAM(0), LPARAM(0)).ok();
                    }
                    // Release capture and close popup
                    ReleaseCapture().ok();
                    DestroyWindow(hwnd).ok();
                }
                LRESULT(0)
            }

            WM_CAPTURECHANGED => {
                // We lost capture - close the popup
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverflowPopupState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    PostMessageW(state.parent_hwnd, WM_APP + 3, WPARAM(0), LPARAM(0)).ok();
                }
                DestroyWindow(hwnd).ok();
                LRESULT(0)
            }

            WM_DESTROY => {
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverflowPopupState;
                if !state_ptr.is_null() {
                    // Free the state
                    let _ = Box::from_raw(state_ptr);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Paint the overflow button (shows "+N" count indicator) styled like a tab
/// When has_selected_overflow is true, also displays the selected tab's icon
#[allow(unused_must_use)]
fn paint_overflow_button(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    overflow_count: usize,
    is_hovered: bool,
    has_selected_overflow: bool,
    selected_icon: Option<&str>,
) {
    unsafe {
        // Determine background color - acts like a "selected" tab if it contains the selected tab
        let bg_color = if is_hovered {
            TAB_HOVER_COLOR
        } else if has_selected_overflow {
            // When selected tab is in overflow, use unselected color (like other non-active tabs)
            TAB_UNSELECTED_COLOR
        } else {
            TAB_UNSELECTED_COLOR
        };

        let bg_brush = CreateSolidBrush(COLORREF(rgb_to_colorref(bg_color)));
        FillRect(hdc, rect, bg_brush);
        DeleteObject(HGDIOBJ(bg_brush.0));

        // Draw outline around overflow button (top, left, right - like a tab)
        let outline_pen = CreatePen(PS_SOLID, 1, COLORREF(rgb_to_colorref(TAB_OUTLINE_COLOR)));
        let old_pen = SelectObject(hdc, HGDIOBJ(outline_pen.0));

        // If selected tab is in overflow, extend sides down to connect with bottom line
        let side_bottom = if has_selected_overflow {
            TITLEBAR_HEIGHT - 1
        } else {
            rect.bottom
        };

        MoveToEx(hdc, rect.left, side_bottom, None);
        LineTo(hdc, rect.left, rect.top);
        LineTo(hdc, rect.right - 1, rect.top);
        LineTo(hdc, rect.right - 1, side_bottom);

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(outline_pen.0));

        // Draw selected tab's icon if selected is in overflow
        let text_offset = if has_selected_overflow {
            if let Some(icon_filename) = selected_icon {
                if let Some(hbitmap) = get_icon_bitmap(icon_filename) {
                    let icon_x = rect.left + 4;
                    let icon_y = (rect.top + rect.bottom - ICON_SIZE) / 2;
                    paint_icon(hdc, hbitmap, icon_x, icon_y, ICON_SIZE, ICON_SIZE);
                    ICON_SIZE + 2 // Icon width + small padding
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        // Draw the count text
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00FFFFFF));

        let mut lf = LOGFONTW::default();
        lf.lfHeight = -11;
        lf.lfWeight = 400;
        let font_name = "Segoe UI";
        for (i, c) in font_name.encode_utf16().enumerate() {
            if i < 32 {
                lf.lfFaceName[i] = c;
            }
        }
        let font = CreateFontIndirectW(&lf);
        let old_font = SelectObject(hdc, HGDIOBJ(font.0));

        // Show count like "+3" for overflow tabs
        let text = format!("+{}", overflow_count);
        let text_wide: Vec<u16> = text.encode_utf16().collect();

        let mut tm = TEXTMETRICW::default();
        GetTextMetricsW(hdc, &mut tm);
        let text_height = tm.tmHeight;

        let mut text_size = SIZE::default();
        GetTextExtentPoint32W(hdc, &text_wide, &mut text_size);

        // Center text in remaining space (after icon if present)
        let text_area_left = rect.left + text_offset;
        let text_x = (text_area_left + rect.right - text_size.cx) / 2;
        let text_y = (rect.top + rect.bottom - text_height) / 2;
        TextOutW(hdc, text_x, text_y, &text_wide);

        SelectObject(hdc, old_font);
        DeleteObject(HGDIOBJ(font.0));
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
    let (visible_count, has_overflow) = calculate_visible_tabs(tab_manager.count(), client_width);

    // First pass: paint all visible non-dragged tabs
    for (i, _tab) in tab_manager.iter() {
        // Skip overflow tabs
        if i >= visible_count {
            break;
        }

        // Skip the dragged tab - we'll paint it last so it appears on top
        if let Some(drag) = drag_state {
            if drag.is_active() && i == drag.tab_index {
                continue;
            }
        }

        let tab_rect = get_tab_rect(i, client_width);
        if tab_rect.left > max_x {
            break;
        }

        let is_selected = i == selected_index;
        let is_hovered = matches!(hovered_tab, HoveredTab::Tab(idx) if idx == i);
        let close_hovered = matches!(hovered_tab, HoveredTab::TabClose(idx) if idx == i);
        let label = tab_manager.get_tab_label(i);
        let icon = tab_manager.get_tab_icon(i);

        paint_tab(
            hdc,
            &tab_rect,
            &label,
            icon,
            is_selected,
            is_hovered,
            close_hovered,
            background_color,
        );
    }

    // Paint overflow button if there are overflow tabs
    if has_overflow {
        let overflow_rect = get_overflow_button_rect(visible_count, client_width);
        let overflow_count = tab_manager.count() - visible_count;
        let is_hovered = matches!(hovered_tab, HoveredTab::OverflowButton);
        let has_selected_overflow = selected_index >= visible_count;
        let selected_icon = if has_selected_overflow {
            tab_manager.get_tab_icon(selected_index)
        } else {
            None
        };
        paint_overflow_button(
            hdc,
            &overflow_rect,
            overflow_count,
            is_hovered,
            has_selected_overflow,
            selected_icon,
        );
    }

    // Paint new tab button
    let new_tab_rect = get_new_tab_button_rect_ex(visible_count, has_overflow, client_width);
    if new_tab_rect.right <= max_x {
        let is_hovered = matches!(hovered_tab, HoveredTab::NewTabButton);
        paint_new_tab_button(hdc, &new_tab_rect, is_hovered);
    }

    // Paint dropdown button
    let dropdown_rect = get_dropdown_button_rect_ex(visible_count, has_overflow, client_width);
    if dropdown_rect.right <= max_x {
        let is_hovered = matches!(hovered_tab, HoveredTab::ProfileDropdown);
        paint_dropdown_button(hdc, &dropdown_rect, is_hovered);
    }

    // Draw line at the bottom of the tab bar with a gap for the selected tab
    // This creates the illusion of physical tabbed pages
    paint_tab_bar_bottom_line(hdc, tab_manager, client_width);

    // Second pass: paint the dragged tab at its visual position (on top of everything)
    if let Some(drag) = drag_state {
        if drag.is_active() {
            let drag_index = drag.tab_index;
            let visual_x = drag.get_visual_x();

            // Clamp the visual position to stay within the visible tab bar bounds
            let min_x = TAB_BAR_LEFT_MARGIN;
            let max_tab_x =
                TAB_BAR_LEFT_MARGIN + ((visible_count.saturating_sub(1)) as i32 * TAB_WIDTH);
            let clamped_x = visual_x.clamp(min_x, max_tab_x.max(min_x));

            let drag_rect = RECT {
                left: clamped_x,
                top: TAB_VERTICAL_PADDING,
                right: clamped_x + TAB_WIDTH,
                bottom: TITLEBAR_HEIGHT - TAB_VERTICAL_PADDING,
            };

            let is_selected = drag_index == selected_index;
            let label = tab_manager.get_tab_label(drag_index);
            let icon = tab_manager.get_tab_icon(drag_index);

            // Dragged tab is never hovered (we're dragging it)
            paint_tab(
                hdc,
                &drag_rect,
                &label,
                icon,
                is_selected,
                false,
                false,
                background_color,
            );
        }
    }
}

/// Paint the bottom line of the tab bar with a gap for the selected tab (or overflow button)
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

        // Determine where the gap should be
        let selected_index = tab_manager.selected_index();
        let (visible_count, has_overflow) =
            calculate_visible_tabs(tab_manager.count(), client_width);

        // If selected tab is in overflow, gap is at the overflow button
        // Otherwise, gap is at the selected tab
        let gap_rect = if has_overflow && selected_index >= visible_count {
            // Selected tab is in overflow - gap at overflow button
            get_overflow_button_rect(visible_count, client_width)
        } else {
            // Selected tab is visible - gap at the selected tab
            get_tab_rect(selected_index, client_width)
        };

        // Draw line from left edge to start of gap (connects with left side)
        if gap_rect.left > line_start_x {
            MoveToEx(hdc, line_start_x, line_y, None);
            LineTo(hdc, gap_rect.left + 1, line_y);
        }

        // Draw line from end of gap to right edge (connects with right side)
        if gap_rect.right < line_end_x {
            MoveToEx(hdc, gap_rect.right - 1, line_y, None);
            LineTo(hdc, line_end_x, line_y);
        }

        SelectObject(hdc, old_pen);
        DeleteObject(HGDIOBJ(outline_pen.0));
    }
}

/// Paint the title bar content to a device context
#[allow(unused_must_use, clippy::too_many_arguments)]
fn paint_titlebar_content(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    client_rect: &RECT,
    background_color: u32,
    hovered_button: HoveredButton,
    hovered_tab: HoveredTab,
    tab_manager: &TabManager,
    dropdown_state: DropdownState,
    profiles: &[Profile],
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

        // Note: Dropdown menu is now rendered as a separate popup window,
        // so we don't paint it here anymore.
        let _ = dropdown_state; // Silence unused warning
        let _ = profiles; // Silence unused warning
    }
}

/// Paint the title bar using double-buffering to prevent flicker
#[allow(unused_must_use, clippy::too_many_arguments)]
fn paint_titlebar(
    hwnd: HWND,
    ps: &PAINTSTRUCT,
    background_color: u32,
    hovered_button: HoveredButton,
    hovered_tab: HoveredTab,
    tab_manager: &TabManager,
    dropdown_state: DropdownState,
    profiles: &[Profile],
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
            dropdown_state,
            profiles,
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
            // Get background color and config from thread-local storage
            let background_color = INITIAL_BG_COLOR.with(|c| c.get());
            let config = INITIAL_CONFIG
                .with(|c| c.borrow_mut().take())
                .unwrap_or_default();

            // Create tab manager and initial tab
            let mut tab_manager = TabManager::new();

            // Get content area dimensions (below title bar)
            if let Ok(rect) = get_content_rect(hwnd) {
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;

                // Create initial tab with Neovide process using default profile
                let default_profile = config.default_profile();
                match tab_manager.create_tab(width, height, hwnd, default_profile, 0) {
                    Ok(_) => {}
                    Err(e) => {
                        let error_msg = format!("Failed to launch Neovide: {}", e);
                        show_error(&error_msg, "Error: Failed to Launch Neovide");
                    }
                }
            }

            // Register global hotkeys
            let mut registered_hotkeys = Vec::new();

            // Register tab hotkeys
            let tab_hotkey_ids = hotkeys::register_tab_hotkeys(hwnd, &config.hotkeys.tab);
            registered_hotkeys.extend(tab_hotkey_ids);

            // Register profile hotkeys
            let profile_hotkey_ids = hotkeys::register_profile_hotkeys(hwnd, &config.profiles);
            registered_hotkeys.extend(profile_hotkey_ids);

            let state = Box::new(WindowState {
                tab_manager,
                config,
                in_size_move: false,
                background_color,
                hovered_button: HoveredButton::None,
                hovered_tab: HoveredTab::None,
                tracking_mouse: false,
                dropdown_state: DropdownState::Closed,
                dropdown_hwnd: None,
                overflow_hwnd: None,
                registered_hotkeys,
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
                                        | TabHitResult::NewTabButton
                                        | TabHitResult::ProfileDropdown
                                        | TabHitResult::DropdownItem(_)
                                        | TabHitResult::OverflowButton => {
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
                    state.dropdown_state,
                    &state.config.profiles,
                );
            } else {
                // Fallback with empty tab manager
                let empty_manager = TabManager::new();
                let empty_profiles: Vec<Profile> = vec![];
                paint_titlebar(
                    hwnd,
                    &ps,
                    0x1a1b26,
                    HoveredButton::None,
                    HoveredTab::None,
                    &empty_manager,
                    DropdownState::Closed,
                    &empty_profiles,
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
                // Poll for exited Neovide processes and refresh tab title
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;
                    let mut should_close = false;
                    let mut needs_repaint = false;

                    // Find all tabs whose processes have exited
                    let exited_indices = state.tab_manager.find_exited_tabs();

                    if !exited_indices.is_empty() {
                        // Remove exited tabs (indices are in reverse order for safe removal)
                        for index in exited_indices {
                            if state.tab_manager.remove_exited_tab(index) {
                                // This was the last tab
                                should_close = true;
                                break;
                            }
                            needs_repaint = true;
                        }

                        // If there are more tabs pending close, continue the sequence
                        // This activates the next tab and sends WM_CLOSE to it
                        if !should_close && state.tab_manager.has_pending_close() {
                            state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                            state.tab_manager.continue_close_sequence();
                        }
                    }

                    // Periodically refresh the selected tab's title (for %t token updates)
                    if !should_close && state.tab_manager.update_selected_tab_title() {
                        needs_repaint = true;
                    }

                    if should_close {
                        // Last tab's process exited - close the application
                        KillTimer(hwnd, PROCESS_POLL_TIMER_ID).ok();
                        PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).ok();
                    } else if needs_repaint {
                        // Activate the newly selected tab and repaint
                        state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                        InvalidateRect(hwnd, None, false);
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
            // Request graceful close for all Neovide windows
            // Process polling will detect exits and close app when last tab is removed
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.tab_manager.request_close_all();

                // If all tabs were forcefully closed (none had ready windows),
                // the tab manager is now empty and we should close immediately
                if state.tab_manager.is_empty() {
                    KillTimer(hwnd, PROCESS_POLL_TIMER_ID).ok();
                    let state = Box::from_raw(state_ptr);
                    drop(state);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    DestroyWindow(hwnd).ok();
                }
                // Otherwise, process polling will handle closing when all processes exit
            } else {
                // No state - just destroy the window
                DestroyWindow(hwnd).ok();
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            // Unregister all global hotkeys
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                hotkeys::unregister_all_hotkeys(hwnd, &state.registered_hotkeys);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }

        WM_HOTKEY => {
            let hotkey_id = wparam.0 as i32;

            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;

                // Bring window to foreground first (restore if minimized)
                bring_window_to_foreground(hwnd);

                if hotkeys::is_tab_hotkey(hotkey_id) {
                    // Tab activation hotkey (1-10)
                    if let Some(tab_index) = hotkeys::tab_index_from_hotkey_id(hotkey_id) {
                        if tab_index < state.tab_manager.count() {
                            // Tab exists - select it
                            if state.tab_manager.select_tab(tab_index) {
                                state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                                InvalidateRect(hwnd, None, false);
                            } else {
                                // Already selected - just ensure foreground
                                state
                                    .tab_manager
                                    .activate_and_foreground_selected(hwnd, TITLEBAR_HEIGHT);
                            }
                        }
                        // If tab doesn't exist, do nothing (no error)
                    }
                } else if hotkeys::is_profile_hotkey(hotkey_id) {
                    // Profile activation hotkey (101+)
                    if let Some(profile_index) = hotkeys::profile_index_from_hotkey_id(hotkey_id) {
                        // Check if we already have a tab with this profile
                        if let Some(existing_tab) =
                            state.tab_manager.find_tab_by_profile_index(profile_index)
                        {
                            // Activate existing tab
                            if state.tab_manager.select_tab(existing_tab) {
                                state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                                InvalidateRect(hwnd, None, false);
                            } else {
                                state
                                    .tab_manager
                                    .activate_and_foreground_selected(hwnd, TITLEBAR_HEIGHT);
                            }
                        } else if let Some(profile) = state.config.get_profile(profile_index) {
                            // Create new tab with this profile
                            if let Ok(rect) = get_content_rect(hwnd) {
                                let width = (rect.right - rect.left) as u32;
                                let height = (rect.bottom - rect.top) as u32;
                                let profile = profile.clone();

                                match state.tab_manager.create_tab(
                                    width,
                                    height,
                                    hwnd,
                                    &profile,
                                    profile_index,
                                ) {
                                    Ok(_) => {
                                        // Hide other tabs
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
                        // If profile doesn't exist, do nothing (no error)
                    }
                }
            }
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
                            // Close popups if open
                            hide_dropdown_popup(hwnd, state);
                            hide_overflow_popup(hwnd, state);
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
                            // Close popups if open
                            hide_dropdown_popup(hwnd, state);
                            hide_overflow_popup(hwnd, state);
                            // Request graceful close - sends WM_CLOSE to Neovide window
                            // Process polling will detect when process exits and remove the tab
                            // If window not ready, falls back to forceful close
                            let graceful = state.tab_manager.request_close_tab(index);
                            if !graceful {
                                // Forceful close occurred - tab already removed
                                // Check if that was the last tab
                                if state.tab_manager.is_empty() {
                                    PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).ok();
                                } else {
                                    // Activate the newly selected tab
                                    state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                                    InvalidateRect(hwnd, None, false);
                                }
                            }
                            // If graceful, do nothing - process polling handles tab removal
                        }
                        TabHitResult::NewTabButton => {
                            // Close popups if open
                            hide_dropdown_popup(hwnd, state);
                            hide_overflow_popup(hwnd, state);
                            // Create new tab with default profile
                            if let Ok(rect) = get_content_rect(hwnd) {
                                let width = (rect.right - rect.left) as u32;
                                let height = (rect.bottom - rect.top) as u32;

                                let default_profile = state.config.default_profile().clone();
                                match state.tab_manager.create_tab(
                                    width,
                                    height,
                                    hwnd,
                                    &default_profile,
                                    0,
                                ) {
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
                        TabHitResult::ProfileDropdown => {
                            // Close overflow popup if open
                            hide_overflow_popup(hwnd, state);
                            // Toggle dropdown popup
                            if state.dropdown_state == DropdownState::Open {
                                hide_dropdown_popup(hwnd, state);
                            } else {
                                show_dropdown_popup(hwnd, state);
                            }
                            InvalidateRect(hwnd, None, false);
                        }
                        TabHitResult::OverflowButton => {
                            // Close dropdown popup if open
                            hide_dropdown_popup(hwnd, state);
                            // Toggle overflow popup
                            if state.overflow_hwnd.is_some() {
                                hide_overflow_popup(hwnd, state);
                            } else {
                                show_overflow_popup(hwnd, state, client_width);
                            }
                            InvalidateRect(hwnd, None, false);
                        }
                        _ => {
                            // Close popups if open and clicking elsewhere
                            if state.dropdown_state == DropdownState::Open {
                                hide_dropdown_popup(hwnd, state);
                                InvalidateRect(hwnd, None, false);
                            }
                            if state.overflow_hwnd.is_some() {
                                hide_overflow_popup(hwnd, state);
                                InvalidateRect(hwnd, None, false);
                            }
                        }
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

                        // Hit test the tab bar (dropdown popup handles its own mouse tracking)
                        let tab_hit =
                            hit_test_tab_bar(x, y, state.tab_manager.count(), client_width);
                        let new_hover = match tab_hit {
                            TabHitResult::Tab(i) => HoveredTab::Tab(i),
                            TabHitResult::TabClose(i) => HoveredTab::TabClose(i),
                            TabHitResult::NewTabButton => HoveredTab::NewTabButton,
                            TabHitResult::ProfileDropdown => HoveredTab::ProfileDropdown,
                            TabHitResult::OverflowButton => HoveredTab::OverflowButton,
                            _ => HoveredTab::None,
                        };

                        if new_hover != state.hovered_tab {
                            state.hovered_tab = new_hover;
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
            LRESULT(0)
        }

        // WM_APP: Profile selected from dropdown popup (wparam = profile index)
        WM_APP => {
            let profile_index = wparam.0;
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.dropdown_hwnd = None; // Popup already destroyed itself
                state.dropdown_state = DropdownState::Closed;

                if let Ok(rect) = get_content_rect(hwnd) {
                    let width = (rect.right - rect.left) as u32;
                    let height = (rect.bottom - rect.top) as u32;

                    if let Some(profile) = state.config.get_profile(profile_index) {
                        let profile = profile.clone();
                        match state.tab_manager.create_tab(
                            width,
                            height,
                            hwnd,
                            &profile,
                            profile_index,
                        ) {
                            Ok(_) => {
                                for (i, tab) in state.tab_manager.iter() {
                                    if i != state.tab_manager.selected_index() {
                                        tab.process.hide();
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to create new tab: {}", e);
                                show_error(&error_msg, "Error: Failed to Create Tab");
                            }
                        }
                    }
                }
                InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        // WM_APP + 1: Dropdown popup closed (lost focus or click outside)
        msg if msg == WM_APP + 1 => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.dropdown_hwnd = None; // Popup already destroyed itself
                state.dropdown_state = DropdownState::Closed;
                InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        // WM_APP + 2: Overflow tab selected (wparam = tab index)
        msg if msg == WM_APP + 2 => {
            let tab_index = wparam.0;
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.overflow_hwnd = None; // Popup already destroyed itself

                // Select the tab
                if state.tab_manager.select_tab(tab_index) {
                    // Hide all other tabs and activate the selected one
                    state.tab_manager.activate_selected(hwnd, TITLEBAR_HEIGHT);
                }
                InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        // WM_APP + 3: Overflow popup closed (lost focus or click outside)
        msg if msg == WM_APP + 3 => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.overflow_hwnd = None; // Popup already destroyed itself
                InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Bring the wrapper window to the foreground, restoring it if minimized
fn bring_window_to_foreground(hwnd: HWND) {
    unsafe {
        // Restore if minimized
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        // Bring to foreground
        let _ = SetForegroundWindow(hwnd);
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
