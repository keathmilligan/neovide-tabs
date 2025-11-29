#![cfg(target_os = "windows")]

use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, HWND_TOP, IsWindowVisible, SWP_NOZORDER, SetWindowPos,
};

/// Manages the lifecycle of a Neovide process instance
pub struct NeovideProcess {
    child: Arc<Mutex<Option<Child>>>,
    neovide_hwnd: Arc<Mutex<Option<usize>>>,
}

impl NeovideProcess {
    /// Check if neovide is available in the system PATH
    pub fn check_neovide_available() -> Result<()> {
        let output = Command::new("where")
            .arg("neovide")
            .output()
            .context("Failed to execute 'where' command")?;

        if !output.status.success() {
            anyhow::bail!(
                "Neovide not found in PATH. Please install Neovide and ensure it is accessible."
            );
        }

        Ok(())
    }

    /// Spawn a new Neovide process with the specified dimensions and position it
    pub fn spawn(width: u32, height: u32, parent_hwnd: HWND) -> Result<Self> {
        let mut cmd = Command::new("neovide");
        cmd.arg("--frame")
            .arg("none")
            .arg("--size")
            .arg(format!("{}x{}", width, height));

        let child = cmd.spawn().context("Failed to spawn Neovide process")?;

        let child_arc = Arc::new(Mutex::new(Some(child)));
        let child_clone = Arc::clone(&child_arc);
        let neovide_hwnd = Arc::new(Mutex::new(None));
        let neovide_hwnd_clone = Arc::clone(&neovide_hwnd);

        // Convert HWND to raw pointer for thread safety
        let parent_hwnd_raw = parent_hwnd.0 as usize;

        // Find and position the Neovide window
        thread::spawn(move || {
            // Reconstruct HWND from raw pointer
            let parent_hwnd = HWND(parent_hwnd_raw as *mut _);

            // Retry finding the window multiple times
            let mut attempts = 0;
            let max_attempts = 30; // Try for up to 3 seconds

            while attempts < max_attempts {
                thread::sleep(Duration::from_millis(100));

                // Find the Neovide window by exact title and class match
                if let Some(info) = find_neovide_window() {
                    *neovide_hwnd_clone.lock().unwrap() = Some(info.hwnd.0 as usize);

                    // Debug output - show window details
                    eprintln!("Found Neovide window after {} attempts:", attempts + 1);
                    eprintln!(
                        "  HWND: 0x{:X}, PID: {}",
                        info.hwnd.0 as usize, info.process_id
                    );
                    eprintln!("  Title: \"{}\"", info.title);
                    eprintln!("  Class: \"{}\"", info.class_name);
                    eprintln!(
                        "  Rect: ({}, {}) - ({}, {}), Size: {}x{}",
                        info.rect.left,
                        info.rect.top,
                        info.rect.right,
                        info.rect.bottom,
                        info.rect.right - info.rect.left,
                        info.rect.bottom - info.rect.top
                    );
                    eprintln!("  Visible: {}", info.visible);

                    // Position the window
                    match move_window_to_parent_client_area(info.hwnd, parent_hwnd) {
                        Ok(_) => {
                            eprintln!("Successfully positioned Neovide window");
                        }
                        Err(e) => {
                            eprintln!("Failed to position Neovide window: {}", e);
                        }
                    }

                    break;
                }

                attempts += 1;
            }

            if attempts >= max_attempts {
                eprintln!(
                    "Failed to find Neovide window after {} attempts",
                    max_attempts
                );
            }
        });

        // Monitor process in background thread
        thread::spawn(move || {
            if let Some(mut child) = child_clone.lock().unwrap().take() {
                let _ = child.wait();
            }
        });

        Ok(NeovideProcess {
            child: child_arc,
            neovide_hwnd,
        })
    }

    /// Terminate the Neovide process gracefully
    pub fn terminate(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            child
                .kill()
                .context("Failed to terminate Neovide process")?;
            child.wait().context("Failed to wait for Neovide process")?;
        }
        Ok(())
    }

    /// Check if the Neovide process is still running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            child.try_wait().ok().flatten().is_none()
        } else {
            false
        }
    }

    /// Update the Neovide window position and size to match parent's client area
    pub fn update_position(&self, parent_hwnd: HWND) {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);
            if let Err(e) = move_window_to_parent_client_area(neovide_hwnd, parent_hwnd) {
                eprintln!("Failed to update Neovide position: {}", e);
            }
        }
    }

    /// Check if the Neovide window has been found and positioned
    pub fn is_ready(&self) -> bool {
        self.neovide_hwnd.lock().unwrap().is_some()
    }

    /// Bring the Neovide window to the foreground
    pub fn bring_to_foreground(&self) {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);
            unsafe {
                // Use SetForegroundWindow to bring Neovide to front
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(neovide_hwnd);
                // Also use BringWindowToTop as a backup
                let _ = windows::Win32::UI::WindowsAndMessaging::BringWindowToTop(neovide_hwnd);
            }
        }
    }
}

impl Drop for NeovideProcess {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

/// Information about a window
#[derive(Debug)]
pub struct WindowInfo {
    pub hwnd: HWND,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub rect: RECT,
    pub visible: bool,
}

/// Context for EnumWindows callback - finds Neovide window by exact match
struct NeovideSearchContext {
    result: Option<WindowInfo>,
}

/// Context for listing all matching windows
struct WindowListContext {
    search_name: String,
    windows: Vec<WindowInfo>,
}

/// Callback for EnumWindows to find Neovide window by exact title and class match
unsafe extern "system" fn enum_windows_neovide_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut NeovideSearchContext);

        let visible = IsWindowVisible(hwnd).as_bool();

        // Get window title
        let title = {
            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                return BOOL(1); // Continue - no title
            }
            let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
            let copied = GetWindowTextW(hwnd, &mut buffer);
            if copied == 0 {
                return BOOL(1); // Continue
            }
            String::from_utf16_lossy(&buffer[..copied as usize])
        };

        // Check for exact title match: "Neovide"
        if title != "Neovide" {
            return BOOL(1); // Continue enumeration
        }

        // Get class name
        let class_name = {
            let mut buffer: Vec<u16> = vec![0; 256];
            let len = GetClassNameW(hwnd, &mut buffer);
            if len == 0 {
                return BOOL(1); // Continue
            }
            String::from_utf16_lossy(&buffer[..len as usize])
        };

        // Check for exact class match: "Window Class"
        if class_name != "Window Class" {
            return BOOL(1); // Continue enumeration
        }

        // Get process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Get window rect
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        // Found it!
        context.result = Some(WindowInfo {
            hwnd,
            title,
            class_name,
            process_id,
            rect,
            visible,
        });

        BOOL(0) // Stop enumeration - found it
    }
}

/// Find a Neovide window by exact title "Neovide" and class "Window Class"
fn find_neovide_window() -> Option<WindowInfo> {
    let mut context = NeovideSearchContext { result: None };

    unsafe {
        let context_ptr = &mut context as *mut NeovideSearchContext as isize;
        let _ = EnumWindows(Some(enum_windows_neovide_callback), LPARAM(context_ptr));
    }

    context.result
}

/// Callback for listing all matching windows with details
unsafe extern "system" fn enum_windows_list_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut WindowListContext);

        let visible = IsWindowVisible(hwnd).as_bool();

        // Get window title
        let title = {
            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                String::new()
            } else {
                let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
                let copied = GetWindowTextW(hwnd, &mut buffer);
                if copied == 0 {
                    String::new()
                } else {
                    String::from_utf16_lossy(&buffer[..copied as usize])
                }
            }
        };

        // Get class name
        let class_name = {
            let mut buffer: Vec<u16> = vec![0; 256];
            let len = GetClassNameW(hwnd, &mut buffer);
            if len == 0 {
                String::new()
            } else {
                String::from_utf16_lossy(&buffer[..len as usize])
            }
        };

        // Get process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Get window rect
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        // Check if title or class name matches search (case-insensitive)
        let search_lower = context.search_name.to_lowercase();
        if title.to_lowercase().contains(&search_lower)
            || class_name.to_lowercase().contains(&search_lower)
        {
            context.windows.push(WindowInfo {
                hwnd,
                title,
                class_name,
                process_id,
                rect,
                visible,
            });
        }

        BOOL(1) // Continue enumeration
    }
}

/// List all windows matching a search string
pub fn list_matching_windows(search: &str) -> Vec<WindowInfo> {
    let mut context = WindowListContext {
        search_name: search.to_string(),
        windows: Vec::new(),
    };

    unsafe {
        let context_ptr = &mut context as *mut WindowListContext as isize;
        let _ = EnumWindows(Some(enum_windows_list_callback), LPARAM(context_ptr));
    }

    context.windows
}

/// Print detailed information about all windows matching a search string
pub fn debug_list_windows(search: &str) {
    let windows = list_matching_windows(search);

    if windows.is_empty() {
        eprintln!("No windows found matching \"{}\"", search);
        return;
    }

    eprintln!("Windows matching \"{}\":", search);
    eprintln!("{}", "=".repeat(80));

    for (i, info) in windows.iter().enumerate() {
        eprintln!(
            "{}. HWND: 0x{:X}, PID: {}",
            i + 1,
            info.hwnd.0 as usize,
            info.process_id
        );
        eprintln!("   Title: \"{}\"", info.title);
        eprintln!("   Class: \"{}\"", info.class_name);
        eprintln!(
            "   Rect: ({}, {}) - ({}, {}), Size: {}x{}",
            info.rect.left,
            info.rect.top,
            info.rect.right,
            info.rect.bottom,
            info.rect.right - info.rect.left,
            info.rect.bottom - info.rect.top
        );
        eprintln!("   Visible: {}", info.visible);
        eprintln!("{}", "-".repeat(80));
    }
}

/// Move and resize a window to fill the parent's client area
fn move_window_to_parent_client_area(neovide_hwnd: HWND, parent_hwnd: HWND) -> Result<()> {
    unsafe {
        // Get parent window's client area
        let mut client_rect = RECT::default();
        windows::Win32::UI::WindowsAndMessaging::GetClientRect(parent_hwnd, &mut client_rect)
            .context("Failed to get parent client rect")?;

        // Convert top-left of client area to screen coordinates
        let mut top_left = windows::Win32::Foundation::POINT {
            x: client_rect.left,
            y: client_rect.top,
        };

        let result = windows::Win32::Graphics::Gdi::ClientToScreen(parent_hwnd, &mut top_left);
        if !result.as_bool() {
            anyhow::bail!("Failed to convert client to screen coordinates");
        }

        // Target size is the parent's client area size
        let target_width = client_rect.right - client_rect.left;
        let target_height = client_rect.bottom - client_rect.top;

        // Get Neovide's current rect for debug output
        let mut neovide_rect = RECT::default();
        GetWindowRect(neovide_hwnd, &mut neovide_rect)
            .context("Failed to get Neovide window rect")?;

        eprintln!(
            "Moving Neovide: from ({}, {}) size {}x{} to ({}, {}) size {}x{}",
            neovide_rect.left,
            neovide_rect.top,
            neovide_rect.right - neovide_rect.left,
            neovide_rect.bottom - neovide_rect.top,
            top_left.x,
            top_left.y,
            target_width,
            target_height
        );

        // SetWindowPos with SWP_NOZORDER to move AND resize
        SetWindowPos(
            neovide_hwnd,
            HWND_TOP,
            top_left.x,
            top_left.y,
            target_width,
            target_height,
            SWP_NOZORDER,
        )
        .context("SetWindowPos failed")?;

        // Verify the move and resize
        GetWindowRect(neovide_hwnd, &mut neovide_rect)
            .context("Failed to get Neovide window rect after move")?;

        eprintln!(
            "After move: pos=({}, {}), size={}x{}",
            neovide_rect.left,
            neovide_rect.top,
            neovide_rect.right - neovide_rect.left,
            neovide_rect.bottom - neovide_rect.top
        );
    }

    Ok(())
}
