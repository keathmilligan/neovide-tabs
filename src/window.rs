#![cfg(target_os = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::collapsible_if)]

use anyhow::{Context, Result};
use std::cell::Cell;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontIndirectW, CreatePen, CreateSolidBrush, DeleteObject, EndPaint, FillRect,
    HBRUSH, HGDIOBJ, InvalidateRect, LOGFONTW, LineTo, MoveToEx, PAINTSTRUCT, PS_SOLID,
    ScreenToClient, SelectObject, SetBkMode, SetTextColor, TRANSPARENT, TextOutW,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use crate::process::NeovideProcess;

const WINDOW_CLASS_NAME: PCWSTR = w!("NeovideTabsWindow");
const WINDOW_TITLE: PCWSTR = w!("neovide-tabs");

/// Title bar height in pixels
const TITLEBAR_HEIGHT: i32 = 32;
/// Button width in pixels
const BUTTON_WIDTH: i32 = 46;

/// Timer ID for delayed foreground activation
const FOREGROUND_TIMER_ID: usize = 2;
/// Delay before bringing Neovide to foreground (ms)
const FOREGROUND_DELAY_MS: u32 = 50;

/// Timer ID for deferred position update (for external tools like FancyZones)
const POSITION_UPDATE_TIMER_ID: usize = 3;
/// Delay before updating position after external move/resize (ms)
const POSITION_UPDATE_DELAY_MS: u32 = 100;

/// Which title bar button is being hovered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoveredButton {
    None,
    Minimize,
    Maximize,
    Close,
}

/// Application state stored in window user data
struct WindowState {
    neovide_process: Option<NeovideProcess>,
    in_size_move: bool,
    background_color: u32,
    hovered_button: HoveredButton,
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
        );

        hwnd.context("Failed to create window")
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

/// Get client area dimensions (excluding title bar)
fn get_content_rect(hwnd: HWND) -> Result<RECT> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).context("Failed to get client rect")?;
        // Content area starts below title bar
        rect.top = TITLEBAR_HEIGHT;
        Ok(rect)
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

/// Paint the title bar
#[allow(unused_must_use)]
fn paint_titlebar(
    hwnd: HWND,
    ps: &PAINTSTRUCT,
    background_color: u32,
    hovered_button: HoveredButton,
) {
    unsafe {
        let hdc = ps.hdc;

        // Get client rect
        let mut client_rect = RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_err() {
            return;
        }

        let client_width = client_rect.right;

        // Fill entire client area with background color to prevent white flashing
        let bg_colorref = COLORREF(rgb_to_colorref(background_color));
        let bg_brush = CreateSolidBrush(bg_colorref);
        FillRect(hdc, &client_rect, bg_brush);
        DeleteObject(HGDIOBJ(bg_brush.0));

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

        // Set up text drawing
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00FFFFFF)); // White text

        // Draw window title
        let title = "neovide-tabs";
        let title_wide: Vec<u16> = title.encode_utf16().collect();

        // Create a font for the title
        let mut lf = LOGFONTW::default();
        lf.lfHeight = -14; // 14 pixels
        lf.lfWeight = 400; // Normal weight
        let font_name = "Segoe UI";
        for (i, c) in font_name.encode_utf16().enumerate() {
            if i < 32 {
                lf.lfFaceName[i] = c;
            }
        }
        let font = CreateFontIndirectW(&lf);
        let old_font = SelectObject(hdc, HGDIOBJ(font.0));

        // Draw title (with some padding from left)
        let title_x = 12;
        let title_y = (TITLEBAR_HEIGHT - 14) / 2; // Center vertically
        TextOutW(hdc, title_x, title_y, &title_wide);

        SelectObject(hdc, old_font);
        DeleteObject(HGDIOBJ(font.0));

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

            // Get content area dimensions (below title bar)
            if let Ok(rect) = get_content_rect(hwnd) {
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;

                // Spawn Neovide process with parent window handle
                match NeovideProcess::spawn(width, height, hwnd) {
                    Ok(process) => {
                        let state = Box::new(WindowState {
                            neovide_process: Some(process),
                            in_size_move: false,
                            background_color,
                            hovered_button: HoveredButton::None,
                            tracking_mouse: false,
                        });
                        let state_ptr = Box::into_raw(state);
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to launch Neovide: {}", e);
                        show_error(&error_msg, "Error: Failed to Launch Neovide");
                        // Still create state without neovide process
                        let state = Box::new(WindowState {
                            neovide_process: None,
                            in_size_move: false,
                            background_color,
                            hovered_button: HoveredButton::None,
                            tracking_mouse: false,
                        });
                        let state_ptr = Box::into_raw(state);
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                    }
                }
            }
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
                        // Check buttons
                        let button = hit_test_buttons(pt.x, pt.y, client_width);
                        match button {
                            HoveredButton::Minimize => return LRESULT(HTMINBUTTON as isize),
                            HoveredButton::Maximize => return LRESULT(HTMAXBUTTON as isize),
                            HoveredButton::Close => return LRESULT(HTCLOSE as isize),
                            HoveredButton::None => {
                                // Not on a button, this is the draggable caption area
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
            // Also handle client area mouse leave
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.tracking_mouse = false;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_PAINT => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            let (background_color, hovered_button) = if !state_ptr.is_null() {
                let state = &*state_ptr;
                (state.background_color, state.hovered_button)
            } else {
                (0x1a1b26, HoveredButton::None)
            };

            let mut ps = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut ps);

            // Paint the title bar
            paint_titlebar(hwnd, &ps, background_color, hovered_button);

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
            // User finished dragging or resizing - now reposition Neovide
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                state.in_size_move = false;
                if let Some(ref process) = state.neovide_process {
                    process.update_position(hwnd, TITLEBAR_HEIGHT);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_ACTIVATE => {
            // Bring Neovide to foreground when wrapper is activated
            // Use a short delay to allow WM_ENTERSIZEMOVE to fire first if this is a drag
            let activated = (wparam.0 & 0xFFFF) != 0; // WA_INACTIVE = 0
            if activated {
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    if let Some(ref process) = state.neovide_process {
                        if process.is_ready() {
                            // Schedule delayed foreground activation
                            SetTimer(hwnd, FOREGROUND_TIMER_ID, FOREGROUND_DELAY_MS, None);
                        }
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
                        if let Some(ref process) = state.neovide_process {
                            process.bring_to_foreground();
                        }
                    }
                }
            } else if wparam.0 == POSITION_UPDATE_TIMER_ID {
                KillTimer(hwnd, POSITION_UPDATE_TIMER_ID).ok();

                // Deferred position update for external tools (e.g., FancyZones)
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = &*state_ptr;
                    if !state.in_size_move {
                        if let Some(ref process) = state.neovide_process {
                            process.update_position(hwnd, TITLEBAR_HEIGHT);
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
                if !state.in_size_move && state.neovide_process.is_some() {
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
            InvalidateRect(hwnd, None, true);
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
            // Terminate Neovide process
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;
            if !state_ptr.is_null() {
                let mut state = Box::from_raw(state_ptr);
                if let Some(mut process) = state.neovide_process.take() {
                    let _ = process.terminate();
                }
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            DestroyWindow(hwnd).ok();
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
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
}
