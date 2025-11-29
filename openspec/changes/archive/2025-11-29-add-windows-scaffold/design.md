# Design: Windows Application Scaffold

## Context
This change implements the foundational window management and process lifecycle for neovide-tabs on Windows. The goal is to prove the core concept (embedding a frameless Neovide instance) without the complexity of tab management. This is the first iteration toward the full tabbed interface described in the project vision.

**Constraints:**
- Windows-only (Win32 API via windows-rs crate)
- Single Neovide instance (no tabs yet)
- Neovide must be installed and available in system PATH
- Neovide must support `--frame none` and `--size WxH` flags

**Stakeholders:**
- End users: Want a working wrapper that launches Neovide seamlessly
- Future development: Need clean separation of concerns for adding tab support later

## Goals / Non-Goals

**Goals:**
- Create a native Windows application window using Win32 API
- Spawn a single Neovide instance with frameless configuration
- Position and size the Neovide window to fill the wrapper's client area
- Handle window resize events and update Neovide accordingly
- Gracefully terminate Neovide when the wrapper closes
- Provide clear error messages if Neovide is not available

**Non-Goals:**
- Tab bar or multi-instance management (deferred to future changes)
- Cross-platform support (Linux/macOS deferred)
- Configuration file or command-line options (deferred)
- Window embedding using parent-child window relationships (current approach uses positioning only)
- Saving window state or session persistence (deferred)

## Decisions

### Decision: Use windows-rs for Win32 API access
**Rationale:** The windows-rs crate provides safe, idiomatic Rust bindings for Win32 APIs. It's the official Microsoft-supported approach and aligns with Rust best practices.

**Alternatives considered:**
- `winapi` crate: Older, less ergonomic, not actively maintained by Microsoft
- `native-windows-gui`: Higher-level abstraction, but adds unnecessary complexity for our minimal needs
- Raw FFI: Unsafe and error-prone

### Decision: Position Neovide window via coordinates, not parent-child embedding
**Rationale:** Neovide runs as a separate process with its own window. Win32 parent-child relationships require HWND handles, which we cannot obtain before Neovide creates its window. For this iteration, we'll position the Neovide window at coordinates that align with our client area. This is sufficient for a single instance and avoids complex window handle discovery.

**Alternatives considered:**
- Parent-child window relationship: Requires finding Neovide's HWND after spawn (via EnumWindows or process ID matching), which adds complexity and timing issues
- OLE/COM embedding: Overkill for this use case
- Wait for future iteration: When tabs are added, we may revisit true embedding

**Trade-offs:**
- Pro: Simple, predictable behavior
- Con: Neovide window appears as a separate window in taskbar and Alt+Tab (acceptable for MVP)

### Decision: Use std::process::Command for process spawning
**Rationale:** Rust's standard library provides cross-platform process management. We'll spawn Neovide as a detached child process and track its handle for lifecycle management.

**Alternatives considered:**
- Win32 CreateProcess directly: More control, but std::process::Command is sufficient
- Embedded subprocess crate: Adds dependency overhead

### Decision: No automatic Neovide installation or bundling
**Rationale:** Neovide is a separate project with its own release cycle. Requiring users to install Neovide separately keeps neovide-tabs lightweight and avoids versioning conflicts.

**Alternatives considered:**
- Bundle Neovide binary: Increases distribution size, complicates updates
- Auto-download on first run: Security and trust concerns

### Decision: Calculate `--size` based on client area, not window size
**Rationale:** The wrapper's client area (excluding title bar and borders) is the drawable region. Neovide should fill this area precisely. We'll use GetClientRect to retrieve dimensions and pass them via `--size WxH`.

**Formula:**
```rust
let (width, height) = get_client_area_size();
let neovide_args = vec!["--frame", "none", "--size", &format!("{}x{}", width, height)];
```

## Architecture

### Module Structure
```
src/
├── main.rs           # Entry point, window initialization, message loop
├── window.rs         # Win32 window creation, message handling
└── process.rs        # Neovide process spawning, lifecycle management
```

### Window Message Flow
1. **WM_CREATE**: Spawn Neovide process with initial client area size
2. **WM_SIZE**: Recalculate client area, reposition/resize Neovide window
3. **WM_CLOSE**: Terminate Neovide process, destroy window
4. **WM_DESTROY**: Post quit message, exit message loop

### Process Lifecycle
1. **Discovery**: Check if `neovide` is in PATH using `which` or `where` command
2. **Spawn**: Execute `neovide --frame none --size WxH` with current working directory
3. **Monitor**: Keep process handle to detect exit (poll or wait in background thread)
4. **Shutdown**: On wrapper close, send SIGTERM (or Windows equivalent) to Neovide process

### Error Handling Strategy
- **Missing Neovide**: Display MessageBox error on startup and exit gracefully
- **Spawn failure**: Log error and display MessageBox with details
- **Neovide crash**: Detect process exit and display error message (future: offer to restart)

## Risks / Trade-offs

**Risk: Neovide window not positioned correctly**
- **Mitigation**: Use precise client area calculations and test on multiple DPI settings
- **Validation**: Manual testing with various window sizes and screen resolutions

**Risk: Zombie processes if wrapper crashes**
- **Mitigation**: Ensure Neovide process is terminated in drop handler or panic handler
- **Trade-off**: May result in abrupt Neovide termination; future improvements can add graceful shutdown signals

**Risk: Race condition between window creation and process spawn**
- **Mitigation**: Spawn Neovide in WM_CREATE handler after window is fully initialized
- **Timing**: Small delay is acceptable; users will see wrapper window first, then Neovide appears

**Risk: DPI scaling issues**
- **Mitigation**: Use DPI-aware window creation flags; test on high-DPI displays
- **Future work**: May need explicit DPI handling for precise sizing

## Migration Plan
N/A - This is the initial implementation with no existing users or state to migrate.

## Open Questions
1. **Should we poll for Neovide process exit or use a background thread?**
   - Proposal: Background thread with `process.wait()` is cleaner than polling
   - Decision: Use background thread to avoid blocking message loop

2. **How should we handle window Z-order (Neovide appearing behind wrapper)?**
   - Proposal: Use `SetWindowPos` with `HWND_TOP` or `HWND_NOTOPMOST` to ensure Neovide appears above wrapper
   - Decision: Test and adjust based on observed behavior

3. **Should we set a minimum window size for the wrapper?**
   - Proposal: Yes, enforce minimum 800x600 to ensure usable Neovide area
   - Decision: Implement WM_GETMINMAXINFO handler to set minimum window size
