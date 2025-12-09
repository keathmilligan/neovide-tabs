# Design: MSI Installer with WiX Toolset

## Context

The current MSIX build infrastructure includes SignPath.io code signing integration and `.appinstaller` auto-update support. This complexity is unnecessary for the project's current needs. MSI is a well-established Windows installer format that provides a simpler deployment story.

## Goals / Non-Goals

**Goals:**
- Simple MSI installer using WiX Toolset v5
- Start menu shortcut for easy access
- User choice between per-user and per-machine installation
- Preserve user configuration on uninstall
- Automated builds via GitHub Actions on version tags

**Non-Goals:**
- Code signing (can be added later if needed)
- Auto-update functionality
- Desktop shortcut
- PATH modification
- Bundling Neovide (remains a prerequisite)

## Decisions

### WiX Toolset v5

WiX v5 is the latest version with .NET tool distribution (`dotnet tool install wix`). It simplifies CI/CD setup compared to older MSI-based WiX v3 installations.

**Alternatives considered:**
- WiX v3/v4: Requires MSI installation, more complex CI setup
- Inno Setup: Not MSI format, less enterprise-friendly
- NSIS: Not MSI format, script-based rather than declarative

### Directory Structure

```
installer/
  Package.wxs       # Main WiX source file
  neovide-tabs.ico  # Icon for Add/Remove Programs (copy of existing PNG converted)
```

Using a single `Package.wxs` file keeps the installer simple. WiX v5 supports inline UI configuration without separate files.

### Installation Scope

Using `WixUI_Advanced` dialog set which provides:
- Welcome screen
- License agreement (MIT)
- Installation scope selection (per-user vs per-machine)
- Installation directory selection
- Ready to install confirmation

Per-user installs to `%LOCALAPPDATA%\Programs\neovide-tabs\`
Per-machine installs to `%ProgramFiles%\neovide-tabs\`

### Preserving Configuration

The installer will NOT create or manage `~/.config/neovide-tabs/`. This directory is created by the application at runtime. By not including it in the MSI, it automatically survives uninstallation.

### Version Handling

MSI requires 3-part versions (Major.Minor.Build). The GitHub workflow will:
1. Extract version from git tag (e.g., `v1.2.3`)
2. Strip `v` prefix
3. Pass to WiX as `ProductVersion`

### GitHub Actions Workflow

```yaml
on:
  push:
    tags: ['v*.*.*']
```

Steps:
1. Checkout code
2. Install Rust toolchain
3. Build release binary
4. Install WiX v5 via `dotnet tool`
5. Build MSI with `wix build`
6. Create GitHub Release with MSI artifact

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Unsigned MSI shows SmartScreen warning | Users may hesitate to install | Document in README; signing can be added later |
| WiX v5 is relatively new | Potential bugs or documentation gaps | WiX v5 is stable; fallback to v4 if issues arise |
| Per-user install may confuse some users | Support questions | Default to per-user which doesn't require admin |

## Open Questions

None - requirements are clear.
