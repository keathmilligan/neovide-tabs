## Context
The application currently has no configuration system. Users cannot customize behavior or appearance. During window resize, the default system background color (often white) causes visual flash before Neovide redraws.

## Goals / Non-Goals
- Goals:
  - Establish cross-platform configuration file loading
  - Support background color customization
  - Eliminate white flash during window resize
- Non-Goals:
  - GUI-based configuration editor
  - Hot-reload of configuration changes
  - Configuration file creation/migration tooling

## Decisions

### Config File Location
- **Decision**: Use `~/.config/neovide-tabs/config.json` on all platforms
- **Rationale**: Cross-platform consistency. On Windows, `~` expands to `C:\Users\<username>`. The `dirs` crate provides reliable cross-platform path resolution.
- **Alternatives considered**:
  - Platform-specific paths (`%APPDATA%` on Windows, `~/.config` on Linux): More idiomatic per-platform but adds complexity
  - XDG base directories: More complex, overkill for a simple config file

### Config Format
- **Decision**: JSON with `serde_json`
- **Rationale**: Human-readable, widely understood, excellent Rust ecosystem support
- **Alternatives considered**:
  - TOML: More Rust-idiomatic but less familiar to general users
  - YAML: More complex parsing, potential security concerns

### Background Color Format
- **Decision**: Accept hex color strings with or without `#` prefix (e.g., `"1a1b26"` or `"#1a1b26"`)
- **Rationale**: Familiar format for developers, easy to copy from themes

### Missing Config Handling
- **Decision**: Use defaults silently when config file is missing or invalid
- **Rationale**: Zero-friction first run experience. Errors should not block the application.
- **Alternatives considered**:
  - Create default config on first run: Adds complexity, may clutter user's config directory
  - Error on invalid config: Poor UX for simple typos

### Resize Flash Prevention
- **Decision**: Set `hbrBackground` in `WNDCLASSW` to a solid brush created from the configured color
- **Rationale**: This is the Win32-idiomatic way to set window background color. The brush is used automatically by `DefWindowProc` during `WM_ERASEBKGND`.

## Risks / Trade-offs
- **Risk**: Color parsing edge cases (invalid hex, wrong length)
  - Mitigation: Fall back to default color on parse failure
- **Risk**: Config directory doesn't exist
  - Mitigation: Handle gracefully, don't attempt to create it

## Open Questions
None - the scope is intentionally minimal for the first iteration.
