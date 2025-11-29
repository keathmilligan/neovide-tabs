# Change: Add Windows-only application scaffold with basic window and Neovide instance management

## Why
neovide-tabs currently exists as a skeleton project with only a "Hello, world!" main function. To deliver the core value proposition of a tabbed Neovide wrapper, we need a foundational application window that can spawn, size, and position a single Neovide instance. This iteration focuses on the minimal viable scaffold—proving the window embedding concept on Windows before adding tab management complexity.

## What Changes
- Create main application window using Win32 API (windows-rs crate)
- Implement Neovide process spawning with `--frame none` and `--size` parameters
- Calculate and apply correct positioning for embedded Neovide window within client area
- Handle basic window lifecycle (creation, resize, close)
- Handle Neovide process lifecycle (spawn, monitor, graceful shutdown)
- Add error handling for missing Neovide installation and process failures
- Scope: Windows-only (no cross-platform abstractions yet)
- Scope: Single Neovide instance (no tab bar or multi-instance support yet)

## Impact
- **Affected specs**: Creates new `window-management` capability
- **Affected code**: 
  - `src/main.rs` - Entry point and window initialization
  - New modules for window management and process handling
  - `Cargo.toml` - Add windows-rs dependency
- **User experience**: Users can launch neovide-tabs and see a working Neovide instance embedded within the wrapper window
- **Dependencies**: Requires windows-rs crate for Win32 API access
