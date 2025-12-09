# Change: Replace MSIX installer with MSI installer

## Why

MSIX packaging adds complexity with code signing requirements (SignPath.io integration) and auto-update infrastructure that isn't needed for this project. A simpler MSI installer built with WiX Toolset provides a familiar installation experience for Windows users without the overhead.

## What Changes

- **REMOVED**: MSIX build workflow (`.github/workflows/release-msix.yml`)
- **REMOVED**: MSIX manifest and assets (`.github/msix/AppxManifest.xml`)
- **REMOVED**: Local MSIX build script (`scripts/build-msix.ps1`)
- **ADDED**: WiX Toolset MSI build workflow for GitHub Actions
- **ADDED**: WiX source files for MSI package definition
- **ADDED**: Local MSI build script for development
- **ADDED**: Start menu shortcut in installer
- **ADDED**: Per-user or per-machine installation choice

## Impact

- Affected specs: None (this is a tooling-only change; use `openspec archive replace-msix-with-msi --skip-specs` when archiving)
- Affected code:
  - `.github/workflows/release-msix.yml` - removed
  - `.github/msix/` - removed
  - `scripts/build-msix.ps1` - removed
  - `.github/workflows/release-msi.yml` - new
  - `installer/` - new WiX source files
  - `scripts/build-msi.ps1` - new
