# Implementation Tasks

## 1. Project Setup
- [x] 1.1 Add windows-rs crate dependency to Cargo.toml with required features (Win32_Foundation, Win32_UI_WindowsAndMessaging, Win32_System_Threading)
- [x] 1.2 Add anyhow crate for error handling
- [x] 1.3 Create module structure: src/window.rs, src/process.rs

## 2. Window Creation
- [x] 2.1 Implement window class registration with Win32 API
- [x] 2.2 Create main application window with standard decorations (title bar, close button, resize borders)
- [x] 2.3 Set up window message loop (WM_CREATE, WM_SIZE, WM_DESTROY, WM_CLOSE)
- [x] 2.4 Calculate client area dimensions for embedded content
- [x] 2.5 Add window title: "neovide-tabs"

## 3. Neovide Process Management
- [x] 3.1 Implement Neovide discovery (check if 'neovide' exists in PATH)
- [x] 3.2 Build command with `--frame none` and calculated `--size WxH` parameters
- [x] 3.3 Spawn Neovide process using std::process::Command
- [x] 3.4 Capture process handle and monitor for exit
- [x] 3.5 Implement graceful shutdown: terminate Neovide when wrapper closes

## 4. Window Positioning
- [x] 4.1 Calculate initial Neovide window size based on wrapper client area
- [x] 4.2 Handle WM_SIZE events to reposition/resize embedded Neovide window
- [x] 4.3 Use Win32 SetWindowPos to update Neovide window coordinates

## 5. Error Handling
- [x] 5.1 Detect and report when Neovide is not installed or not in PATH
- [x] 5.2 Handle Neovide process spawn failures
- [x] 5.3 Handle Neovide process crashes (detect exit and display error message)
- [x] 5.4 Validate command-line arguments and environment setup

## 6. Testing
- [x] 6.1 Manual test: Launch wrapper and verify Neovide appears
- [x] 6.2 Manual test: Resize wrapper window and verify Neovide resizes
- [x] 6.3 Manual test: Close wrapper and verify Neovide terminates
- [x] 6.4 Manual test: Launch without Neovide installed and verify error message
- [x] 6.5 Add unit tests for window dimension calculations
- [x] 6.6 Run cargo clippy and cargo fmt to ensure code quality

## Notes
- All work is Windows-only; use `#[cfg(target_os = "windows")]` where appropriate
- Focus on correctness over performance in this iteration
- Defer tab management, configuration, and cross-platform support to future changes
