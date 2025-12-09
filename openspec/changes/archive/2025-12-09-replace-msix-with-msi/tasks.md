# Tasks

## 1. Remove MSIX Infrastructure

- [x] 1.1 Delete `.github/workflows/release-msix.yml`
- [x] 1.2 Delete `.github/msix/` directory (AppxManifest.xml)
- [x] 1.3 Delete `scripts/build-msix.ps1`

## 2. Create WiX MSI Package Definition

- [x] 2.1 Create `installer/` directory structure
- [x] 2.2 Create WiX source file (`installer/Package.wxs`) with:
  - Product/Package metadata
  - Start menu shortcut
  - Per-user/per-machine installation choice (WixUI_Advanced)
  - Preserve config files on uninstall
- [x] 2.3 Create License.rtf for installer dialog

## 3. Create Local Build Script

- [x] 3.1 Create `scripts/build-msi.ps1` with:
  - WiX Toolset detection/installation guidance
  - Cargo release build integration
  - MSI compilation using `wix build`
  - Version parameter support
  - PNG to ICO conversion

## 4. Create GitHub Actions Workflow

- [x] 4.1 Create `.github/workflows/release-msi.yml` with:
  - Trigger on version tags (`v*.*.*`)
  - Rust toolchain setup
  - WiX Toolset installation
  - Release binary build
  - PNG to ICO conversion (ImageMagick)
  - MSI package creation (unsigned)
  - GitHub Release creation with MSI artifact

## 5. Update Documentation

- [x] 5.1 Update README.md installation section to reference MSI installer

## 6. Validation

- [x] 6.1 Validate Package.wxs XML syntax
- [ ] 6.2 Test local MSI build with `scripts/build-msi.ps1` (requires WiX install)
- [ ] 6.3 Verify MSI installs correctly (per-user and per-machine)
- [ ] 6.4 Verify Start menu shortcut created
- [ ] 6.5 Verify uninstall preserves `~/.config/neovide-tabs/`
- [ ] 6.6 Verify GitHub Actions workflow produces valid MSI
