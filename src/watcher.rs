//! Configuration file watcher for hot-reload functionality.
//!
//! Monitors the config directory for changes to config.jsonc or config.json
//! and posts a Windows message to the main window when changes are detected.

#![cfg(target_os = "windows")]

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

use crate::config::config_dir_path;

/// Custom message ID for config reload events (WM_APP + 10)
pub const WM_CONFIG_RELOAD: u32 = WM_APP + 10;

/// Debounce timeout for file changes (milliseconds)
const DEBOUNCE_TIMEOUT_MS: u64 = 250;

/// Handle to a running config file watcher.
/// The watcher runs in a background thread and will stop when this handle is dropped.
pub struct ConfigWatcher {
    /// Sender to signal the watcher thread to stop
    _stop_tx: mpsc::Sender<()>,
    /// Join handle for the watcher thread (for cleanup)
    _thread_handle: Option<thread::JoinHandle<()>>,
}

impl ConfigWatcher {
    /// Start watching the config directory for changes.
    /// Posts WM_CONFIG_RELOAD to the specified window when the config file changes.
    ///
    /// Returns None if:
    /// - The config directory path cannot be determined
    /// - The watcher fails to initialize
    pub fn start(hwnd: HWND) -> Option<Self> {
        let config_dir = config_dir_path()?;

        if !config_dir.exists() {
            eprintln!(
                "ConfigWatcher: Config directory does not exist: {:?}",
                config_dir
            );
            // Still start watching - directory might be created later
        }

        // Create a channel to receive stop signal
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // Store hwnd as raw pointer for use in thread
        let hwnd_value = hwnd.0 as isize;

        // Start the watcher thread
        let thread_handle = thread::spawn(move || {
            run_watcher(config_dir, hwnd_value, stop_rx);
        });

        Some(ConfigWatcher {
            _stop_tx: stop_tx,
            _thread_handle: Some(thread_handle),
        })
    }
}

/// Run the file watcher (called from the background thread)
fn run_watcher(config_dir: PathBuf, hwnd_value: isize, stop_rx: mpsc::Receiver<()>) {
    eprintln!("ConfigWatcher: Starting to watch {:?}", config_dir);

    // Create a channel for debounced events
    let (tx, rx) = mpsc::channel();

    // Create debounced watcher
    let mut debouncer = match new_debouncer(Duration::from_millis(DEBOUNCE_TIMEOUT_MS), tx) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("ConfigWatcher: Failed to create debouncer: {}", e);
            return;
        }
    };

    // Watch the config directory
    if let Err(e) = debouncer
        .watcher()
        .watch(&config_dir, RecursiveMode::NonRecursive)
    {
        eprintln!("ConfigWatcher: Failed to watch directory: {}", e);
        return;
    }

    eprintln!("ConfigWatcher: Watching {:?} for changes", config_dir);

    // Main event loop
    loop {
        // Check for stop signal (non-blocking)
        if stop_rx.try_recv().is_ok() {
            eprintln!("ConfigWatcher: Received stop signal, shutting down");
            break;
        }

        // Wait for file change events (with timeout to allow stop signal checks)
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(result) => {
                match result {
                    Ok(events) => {
                        // Got debounced events - check if any are config files
                        if should_reload(&events, &config_dir) {
                            eprintln!(
                                "ConfigWatcher: Config file changed, posting reload message"
                            );
                            post_reload_message(hwnd_value);
                        }
                    }
                    Err(error) => {
                        eprintln!("ConfigWatcher: Watch error: {:?}", error);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No events, loop continues
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("ConfigWatcher: Event channel disconnected");
                break;
            }
        }
    }

    eprintln!("ConfigWatcher: Stopped");
}

/// Check if the debounced events include a config file change
fn should_reload(events: &[notify_debouncer_mini::DebouncedEvent], config_dir: &std::path::Path) -> bool {
    let jsonc_path = config_dir.join("config.jsonc");
    let json_path = config_dir.join("config.json");

    for event in events {
        let path = &event.path;
        if path == &jsonc_path || path == &json_path {
            return true;
        }
    }

    false
}

/// Post the config reload message to the window
fn post_reload_message(hwnd_value: isize) {
    unsafe {
        let hwnd = HWND(hwnd_value as *mut std::ffi::c_void);
        if let Err(e) = PostMessageW(
            hwnd,
            WM_CONFIG_RELOAD,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        ) {
            eprintln!("ConfigWatcher: Failed to post reload message: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_reload_jsonc() {
        let config_dir = PathBuf::from("/test/config");
        let events = vec![notify_debouncer_mini::DebouncedEvent {
            path: PathBuf::from("/test/config/config.jsonc"),
            kind: notify_debouncer_mini::DebouncedEventKind::Any,
        }];

        assert!(should_reload(&events, &config_dir));
    }

    #[test]
    fn test_should_reload_json() {
        let config_dir = PathBuf::from("/test/config");
        let events = vec![notify_debouncer_mini::DebouncedEvent {
            path: PathBuf::from("/test/config/config.json"),
            kind: notify_debouncer_mini::DebouncedEventKind::Any,
        }];

        assert!(should_reload(&events, &config_dir));
    }

    #[test]
    fn test_should_not_reload_other_file() {
        let config_dir = PathBuf::from("/test/config");
        let events = vec![notify_debouncer_mini::DebouncedEvent {
            path: PathBuf::from("/test/config/other.txt"),
            kind: notify_debouncer_mini::DebouncedEventKind::Any,
        }];

        assert!(!should_reload(&events, &config_dir));
    }

    #[test]
    fn test_should_not_reload_empty_events() {
        let config_dir = PathBuf::from("/test/config");
        let events: Vec<notify_debouncer_mini::DebouncedEvent> = vec![];

        assert!(!should_reload(&events, &config_dir));
    }
}
