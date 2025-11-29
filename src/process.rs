#![cfg(target_os = "windows")]

use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, HWND_TOP, IsWindowVisible, MB_ICONERROR, MB_OK, MessageBoxW,
    PostMessageW, SW_HIDE, SW_SHOW, SWP_NOZORDER, SetWindowPos, ShowWindow, WM_CLOSE,
};
use windows::core::PCWSTR;

use crate::window::CONTENT_INSET;

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

    /// Spawn a new Neovide process with the specified dimensions, working directory, and position it
    pub fn spawn(
        width: u32,
        height: u32,
        parent_hwnd: HWND,
        working_directory: Option<&Path>,
    ) -> Result<Self> {
        let mut cmd = Command::new("neovide");
        cmd.arg("--frame")
            .arg("none")
            .arg("--size")
            .arg(format!("{}x{}", width, height));

        // Set working directory if specified
        if let Some(dir) = working_directory {
            if dir.is_dir() {
                cmd.current_dir(dir);
                eprintln!("Spawning Neovide in directory: {:?}", dir);
            } else {
                eprintln!(
                    "Warning: Working directory {:?} does not exist, using default",
                    dir
                );
            }
        }

        let child = cmd.spawn().context("Failed to spawn Neovide process")?;

        // Get the process ID to find the correct window later
        let child_pid = child.id();

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
            let max_attempts = 600; // Try for up to 60 seconds
            let mut found = false;

            while attempts < max_attempts {
                thread::sleep(Duration::from_millis(100));

                // Find the Neovide window by process ID
                if let Some(info) = find_neovide_window_by_pid(child_pid) {
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

                    // Position the window (32 is the title bar height)
                    match move_window_to_parent_content_area(info.hwnd, parent_hwnd, 32) {
                        Ok(_) => {
                            eprintln!("Successfully positioned Neovide window");
                        }
                        Err(e) => {
                            eprintln!("Failed to position Neovide window: {}", e);
                        }
                    }

                    found = true;
                    break;
                }

                attempts += 1;
            }

            if !found {
                eprintln!(
                    "Failed to find Neovide window (PID: {}) after {} seconds",
                    child_pid,
                    max_attempts / 10
                );
                show_neovide_window_timeout_error();
                std::process::exit(1);
            }
        });

        // Note: We no longer use a background thread to wait on the child process.
        // Instead, we poll the process status via is_running() and try_wait().
        // The child_clone is no longer needed since we keep the child in the Arc<Mutex>.
        drop(child_clone);

        Ok(NeovideProcess {
            child: child_arc,
            neovide_hwnd,
        })
    }

    /// Terminate the Neovide process forcefully using kill()
    pub fn terminate(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            child
                .kill()
                .context("Failed to terminate Neovide process")?;
            child.wait().context("Failed to wait for Neovide process")?;
        }
        Ok(())
    }

    /// Request graceful close by sending WM_CLOSE to the Neovide window.
    /// Returns true if the message was sent successfully, false if the window
    /// handle is not yet available (caller should fall back to terminate()).
    /// Note: This does not immediately close the window - Neovide may prompt
    /// the user to save unsaved files. The process polling will detect when
    /// the process actually exits.
    pub fn request_close(&self) -> bool {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);
            unsafe {
                // PostMessageW returns Ok(()) on success
                PostMessageW(neovide_hwnd, WM_CLOSE, None, None).is_ok()
            }
        } else {
            false
        }
    }

    /// Check if the Neovide process is still running.
    /// Returns true if the process is still running, false if it has exited or was never started.
    pub fn is_running(&self) -> bool {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            // try_wait() returns Ok(Some(status)) if exited, Ok(None) if still running
            match child.try_wait() {
                Ok(Some(_status)) => false, // Process has exited
                Ok(None) => true,           // Process is still running
                Err(_) => false,            // Error checking status, assume not running
            }
        } else {
            false // No child process (already terminated or never started)
        }
    }

    /// Update the Neovide window position and size to match parent's content area
    /// (client area minus title bar)
    /// Returns true if the window was actually moved, false if already in position or not ready
    pub fn update_position(&self, parent_hwnd: HWND, titlebar_height: i32) -> bool {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);
            match move_window_to_parent_content_area(neovide_hwnd, parent_hwnd, titlebar_height) {
                Ok(moved) => moved,
                Err(e) => {
                    eprintln!("Failed to update Neovide position: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Update position only if needed, then show and bring to foreground
    /// This is the proper sequence for activating a tab
    pub fn activate(&self, parent_hwnd: HWND, titlebar_height: i32) {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);

            // First ensure position is correct (only moves if needed)
            let _ = move_window_to_parent_content_area(neovide_hwnd, parent_hwnd, titlebar_height);

            unsafe {
                // Show the window
                let _ = ShowWindow(neovide_hwnd, SW_SHOW);
                // Bring to foreground
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(neovide_hwnd);
                let _ = windows::Win32::UI::WindowsAndMessaging::BringWindowToTop(neovide_hwnd);
            }
        }
    }

    /// Check if the Neovide window has been found and positioned
    pub fn is_ready(&self) -> bool {
        self.neovide_hwnd.lock().unwrap().is_some()
    }

    /// Bring the Neovide window to the foreground
    #[allow(dead_code)]
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

    /// Show the Neovide window
    #[allow(dead_code)]
    pub fn show(&self) {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);
            unsafe {
                let _ = ShowWindow(neovide_hwnd, SW_SHOW);
            }
        }
    }

    /// Hide the Neovide window
    pub fn hide(&self) {
        if let Some(hwnd_raw) = *self.neovide_hwnd.lock().unwrap() {
            let neovide_hwnd = HWND(hwnd_raw as *mut _);
            unsafe {
                let _ = ShowWindow(neovide_hwnd, SW_HIDE);
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
    target_pid: Option<u32>,
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

        // If we're looking for a specific PID, check it
        if context
            .target_pid
            .is_some_and(|target_pid| process_id != target_pid)
        {
            return BOOL(1); // Continue enumeration - wrong process
        }

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
#[allow(dead_code)]
fn find_neovide_window() -> Option<WindowInfo> {
    let mut context = NeovideSearchContext {
        result: None,
        target_pid: None,
    };

    unsafe {
        let context_ptr = &mut context as *mut NeovideSearchContext as isize;
        let _ = EnumWindows(Some(enum_windows_neovide_callback), LPARAM(context_ptr));
    }

    context.result
}

/// Find a Neovide window by process ID
fn find_neovide_window_by_pid(pid: u32) -> Option<WindowInfo> {
    let mut context = NeovideSearchContext {
        result: None,
        target_pid: Some(pid),
    };

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

/// Calculate the target position and size for a Neovide window within the parent's content area
fn calculate_target_rect(parent_hwnd: HWND, titlebar_height: i32) -> Result<(i32, i32, i32, i32)> {
    unsafe {
        // Get parent window's client area
        let mut client_rect = RECT::default();
        windows::Win32::UI::WindowsAndMessaging::GetClientRect(parent_hwnd, &mut client_rect)
            .context("Failed to get parent client rect")?;

        // Convert top-left of content area (below title bar, with inset) to screen coordinates
        let mut top_left = windows::Win32::Foundation::POINT {
            x: client_rect.left + CONTENT_INSET,
            y: client_rect.top + titlebar_height + CONTENT_INSET,
        };

        let result = windows::Win32::Graphics::Gdi::ClientToScreen(parent_hwnd, &mut top_left);
        if !result.as_bool() {
            anyhow::bail!("Failed to convert client to screen coordinates");
        }

        // Target size is the parent's client area size minus title bar height and insets
        let target_width = client_rect.right - client_rect.left - (CONTENT_INSET * 2);
        let target_height =
            client_rect.bottom - client_rect.top - titlebar_height - (CONTENT_INSET * 2);

        Ok((top_left.x, top_left.y, target_width, target_height))
    }
}

/// Move and resize a window to fill the parent's content area (below title bar, with inset)
/// Returns true if the window was actually moved, false if it was already in position
fn move_window_to_parent_content_area(
    neovide_hwnd: HWND,
    parent_hwnd: HWND,
    titlebar_height: i32,
) -> Result<bool> {
    unsafe {
        let (target_x, target_y, target_width, target_height) =
            calculate_target_rect(parent_hwnd, titlebar_height)?;

        // Get Neovide's current rect
        let mut neovide_rect = RECT::default();
        GetWindowRect(neovide_hwnd, &mut neovide_rect)
            .context("Failed to get Neovide window rect")?;

        let current_x = neovide_rect.left;
        let current_y = neovide_rect.top;
        let current_width = neovide_rect.right - neovide_rect.left;
        let current_height = neovide_rect.bottom - neovide_rect.top;

        // Check if already in the correct position and size
        if current_x == target_x
            && current_y == target_y
            && current_width == target_width
            && current_height == target_height
        {
            // Already in position, no need to move
            return Ok(false);
        }

        eprintln!(
            "Moving Neovide: from ({}, {}) size {}x{} to ({}, {}) size {}x{}",
            current_x,
            current_y,
            current_width,
            current_height,
            target_x,
            target_y,
            target_width,
            target_height
        );

        // SetWindowPos with SWP_NOZORDER to move AND resize
        SetWindowPos(
            neovide_hwnd,
            HWND_TOP,
            target_x,
            target_y,
            target_width,
            target_height,
            SWP_NOZORDER,
        )
        .context("SetWindowPos failed")?;

        Ok(true)
    }
}

/// Display an error message when Neovide window is not found after timeout
fn show_neovide_window_timeout_error() {
    let message = "Failed to find Neovide window after 60 seconds.\n\n\
        The Neovide process was started but its window could not be detected.\n\n\
        This may be caused by:\n\
        - Neovide taking too long to initialize\n\
        - A configuration issue with Neovide\n\
        - Neovide crashing during startup\n\n\
        Please check your Neovide installation and try again.";
    let title = "Error: Neovide Window Not Found";

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
