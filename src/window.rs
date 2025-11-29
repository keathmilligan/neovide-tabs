#![cfg(target_os = "windows")]

use anyhow::{Context, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use crate::process::NeovideProcess;

const WINDOW_CLASS_NAME: PCWSTR = w!("NeovideTabsWindow");
const WINDOW_TITLE: PCWSTR = w!("neovide-tabs");

/// Timer ID for delayed foreground activation
const FOREGROUND_TIMER_ID: usize = 2;
/// Delay before bringing Neovide to foreground (ms)
const FOREGROUND_DELAY_MS: u32 = 50;

/// Timer ID for deferred position update (for external tools like FancyZones)
const POSITION_UPDATE_TIMER_ID: usize = 3;
/// Delay before updating position after external move/resize (ms)
const POSITION_UPDATE_DELAY_MS: u32 = 100;

/// Application state stored in window user data
struct WindowState {
    neovide_process: Option<NeovideProcess>,
    in_size_move: bool,
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
    unsafe {
        let hinstance = GetModuleHandleW(None).context("Failed to get module handle")?;

        // Create a solid brush for the background color
        let colorref = rgb_to_colorref(background_color);
        let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(colorref));

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

/// Create the main application window
pub fn create_window() -> Result<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None).context("Failed to get module handle")?;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            WINDOW_CLASS_NAME,
            WINDOW_TITLE,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
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

/// Get client area dimensions
fn get_client_rect(hwnd: HWND) -> Result<RECT> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).context("Failed to get client rect")?;
        Ok(rect)
    }
}

/// Window procedure callback
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Get client area dimensions
            if let Ok(rect) = get_client_rect(hwnd) {
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;

                // Spawn Neovide process with parent window handle
                match NeovideProcess::spawn(width, height, hwnd) {
                    Ok(process) => {
                        let state = Box::new(WindowState {
                            neovide_process: Some(process),
                            in_size_move: false,
                        });
                        let state_ptr = Box::into_raw(state);
                        unsafe {
                            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to launch Neovide: {}", e);
                        show_error(&error_msg, "Error: Failed to Launch Neovide");
                    }
                }
            }
            LRESULT(0)
        }

        WM_ENTERSIZEMOVE => {
            // User started dragging or resizing - set flag and cancel any pending timers
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
            if !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                state.in_size_move = true;
                unsafe {
                    KillTimer(hwnd, FOREGROUND_TIMER_ID).ok();
                    KillTimer(hwnd, POSITION_UPDATE_TIMER_ID).ok();
                }
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        WM_EXITSIZEMOVE => {
            // User finished dragging or resizing - now reposition Neovide
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
            if !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                state.in_size_move = false;
                if let Some(ref process) = state.neovide_process {
                    process.update_position(hwnd);
                }
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        WM_ACTIVATE => {
            // Bring Neovide to foreground when wrapper is activated
            // Use a short delay to allow WM_ENTERSIZEMOVE to fire first if this is a drag
            let activated = (wparam.0 & 0xFFFF) != 0; // WA_INACTIVE = 0
            if activated {
                let state_ptr =
                    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = unsafe { &*state_ptr };
                    if let Some(ref process) = state.neovide_process
                        && process.is_ready()
                    {
                        // Schedule delayed foreground activation
                        unsafe {
                            SetTimer(hwnd, FOREGROUND_TIMER_ID, FOREGROUND_DELAY_MS, None);
                        }
                    }
                }
            } else {
                // Deactivating - cancel any pending foreground timer
                unsafe {
                    KillTimer(hwnd, FOREGROUND_TIMER_ID).ok();
                }
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        WM_TIMER => {
            if wparam.0 == FOREGROUND_TIMER_ID {
                unsafe {
                    KillTimer(hwnd, FOREGROUND_TIMER_ID).ok();
                }

                // Now check if we're in a size/move operation
                let state_ptr =
                    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = unsafe { &*state_ptr };
                    if !state.in_size_move
                        && let Some(ref process) = state.neovide_process
                    {
                        process.bring_to_foreground();
                    }
                }
            } else if wparam.0 == POSITION_UPDATE_TIMER_ID {
                unsafe {
                    KillTimer(hwnd, POSITION_UPDATE_TIMER_ID).ok();
                }

                // Deferred position update for external tools (e.g., FancyZones)
                let state_ptr =
                    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
                if !state_ptr.is_null() {
                    let state = unsafe { &*state_ptr };
                    if !state.in_size_move
                        && let Some(ref process) = state.neovide_process
                    {
                        process.update_position(hwnd);
                    }
                }
            }
            LRESULT(0)
        }

        WM_WINDOWPOSCHANGED => {
            // Handle programmatic window position/size changes (e.g., from FancyZones)
            // Only schedule update if we're not in a manual size/move operation
            // Use a timer to debounce and avoid interfering with resize operations
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                if !state.in_size_move && state.neovide_process.is_some() {
                    // Schedule a deferred position update - this will be cancelled
                    // if more WM_WINDOWPOSCHANGED messages arrive, effectively debouncing
                    unsafe {
                        SetTimer(hwnd, POSITION_UPDATE_TIMER_ID, POSITION_UPDATE_DELAY_MS, None);
                    }
                }
            }
            // Must call DefWindowProcW to get WM_SIZE and WM_MOVE messages
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        WM_GETMINMAXINFO => {
            // Set minimum window size to 800x600
            let info = lparam.0 as *mut MINMAXINFO;
            if !info.is_null() {
                unsafe {
                    (*info).ptMinTrackSize.x = 800;
                    (*info).ptMinTrackSize.y = 600;
                }
            }
            LRESULT(0)
        }

        WM_CLOSE => {
            // Terminate Neovide process
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
            if !state_ptr.is_null() {
                let mut state = unsafe { Box::from_raw(state_ptr) };
                if let Some(mut process) = state.neovide_process.take() {
                    let _ = process.terminate();
                }
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
            }
            unsafe {
                DestroyWindow(hwnd).ok();
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_area_calculation() {
        // Test that client area dimensions are calculated correctly
        let rect = RECT {
            left: 0,
            top: 0,
            right: 1024,
            bottom: 768,
        };

        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        assert_eq!(width, 1024);
        assert_eq!(height, 768);
    }

    #[test]
    fn test_client_area_with_offset() {
        // Test with non-zero origin
        let rect = RECT {
            left: 100,
            top: 50,
            right: 1124,
            bottom: 818,
        };

        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        assert_eq!(width, 1024);
        assert_eq!(height, 768);
    }
}
