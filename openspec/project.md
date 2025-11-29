# Project Context

## Purpose
neovide-tabs is a native window wrapper application for Neovide that provides tabbed interface functionality. The application manages multiple instances of Neovide, each running in a separate tab with a frameless window configuration.

### Goals
- Create a lightweight tabbed interface for Neovide instances
- Manage multiple Neovide windows seamlessly within a single wrapper application
- Provide a clean, integrated experience by removing Neovide's default window chrome
- Automatically configure each Neovide instance with appropriate sizing and positioning

## Tech Stack
- **Language**: Rust (2024 edition)
- **GUI Framework**: TBD (consider: egui, iced, gtk-rs, or native Windows API)
- **Platform**: Windows 11 (primary), with future cross-platform support for Linux and macOS
- **External Dependency**: Neovide (launched as subprocess)

## Project Conventions

### Code Style
- Follow Rust standard formatting (rustfmt with default settings)
- Use `snake_case` for functions and variables
- Use `PascalCase` for types, structs, enums, and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Explicit error handling with `Result<T, E>` and `?` operator
- Avoid `unwrap()` in production code; prefer proper error propagation
- Document all public APIs with `///` doc comments
- Use `//` for inline implementation comments
- Group imports: std → external crates → local modules

### Architecture Patterns
- **Window Management**: Each tab represents a separate Neovide process instance
- **Process Lifecycle**: Parent process (neovide-tabs) spawns and manages child processes (Neovide instances)
- **Window Embedding**: Neovide windows are embedded within the wrapper's client area using `--frame none`
- **Coordinate System**: Parent window manages absolute positioning and sizing for each embedded child window

#### Initial Architecture (MVP)
- Single-window application with tab bar
- Each tab creates a new Neovide instance with:
  - `--frame none` flag (removes window chrome)
  - `--size WxH` parameter (fills wrapper's client area)
  - Explicit window position coordinates
  - Working directory set to wrapper's current directory

#### Future Enhancements
- Configurable tab behavior (open in current directory vs. custom paths)
- Persistent tab sessions
- Tab reordering and window management
- Configuration file for default settings
- Keyboard shortcuts for tab navigation

### Testing Strategy
- Unit tests for core window management logic
- Integration tests for Neovide process spawning and lifecycle
- Platform-specific tests using `#[cfg(target_os = "windows")]`
- Manual testing on target platforms (Windows, Linux, macOS)
- Test error conditions: Neovide not found, invalid parameters, process crashes

### Git Workflow
- Feature branches for new functionality
- Descriptive commit messages following conventional commits
- No direct commits to main branch
- Run `cargo fmt` and `cargo clippy` before committing

## Domain Context

### Neovide
Neovide is a graphical UI for Neovim written in Rust. Key characteristics:
- Cross-platform (Windows, Linux, macOS)
- Supports command-line arguments for window configuration
- `--frame none`: Removes window decorations (title bar, borders)
- `--size WxH`: Sets initial window dimensions
- Can accept working directory and file arguments

### Window Embedding Concepts
- **Frameless Windows**: Windows without standard OS chrome (title bar, resize borders)
- **Client Area**: The drawable region within a window, excluding decorations
- **Window Positioning**: Absolute screen coordinates or parent-relative coordinates
- **Process Ownership**: Parent process must track child process lifecycle to prevent orphaned processes

### Platform Considerations
- **Windows**: Use Win32 API for window manipulation and embedding
- **Linux**: X11 or Wayland window management
- **macOS**: Cocoa/AppKit for window management
- Cross-platform abstractions needed for window operations

## Important Constraints

### Technical Constraints
- Must correctly calculate and maintain window positions when resizing wrapper
- Must handle Neovide process lifecycle (graceful shutdown, crash recovery)
- Must respect platform-specific window management behaviors
- Initial version launches Neovide in current working directory only
- Requires Neovide to be installed and accessible in system PATH

### Performance Constraints
- Minimize overhead when managing multiple Neovide instances
- Efficient window message handling to avoid UI lag
- Each Neovide instance consumes its own memory; tab count limited by system resources

### User Experience Constraints
- Seamless integration - users should not notice the wrapper overhead
- Consistent behavior across platforms (when multi-platform support added)
- Clean visual integration without visible seams between wrapper and embedded windows

## External Dependencies

### Runtime Dependencies
- **Neovide**: Must be installed and available in system PATH
  - Version: Latest stable (check compatibility with `--frame none` flag)
  - Installation: User-managed (neovide-tabs does not bundle Neovide)

### Development Dependencies
- **Rust toolchain**: 2024 edition
- **cargo**: Build system and package manager
- **rustfmt**: Code formatting
- **clippy**: Linting

### Platform-Specific Dependencies
- **Windows**: Win32 API bindings (e.g., `windows-rs` crate)
- **Linux**: X11/Wayland bindings (future)
- **macOS**: Cocoa bindings (future)

## Current Status
- **Phase**: Initial development / MVP
- **Version**: 0.1.0
- **Implementation Status**: Skeleton project with basic Cargo configuration
