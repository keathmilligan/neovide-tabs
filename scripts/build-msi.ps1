#Requires -Version 5.1
<#
.SYNOPSIS
    Build MSI installer for neovide-tabs locally.

.DESCRIPTION
    This script builds an MSI package using WiX Toolset v5.

.PARAMETER Version
    Version number for the package (default: 0.1.0)

.PARAMETER SkipBuild
    Skip cargo build (use existing binary)

.EXAMPLE
    .\build-msi.ps1
    # Builds MSI with default version

.EXAMPLE
    .\build-msi.ps1 -Version "1.2.3"
    # Builds MSI with specified version

.EXAMPLE
    .\build-msi.ps1 -SkipBuild
    # Uses existing binary, skips cargo build

.NOTES
    Prerequisites:
    - .NET SDK 6.0 or later (for WiX v5)
    - WiX Toolset v5: dotnet tool install --global wix
    - Rust toolchain (unless -SkipBuild)
    - Installer assets (run generate-installer-assets.ps1 first)
#>

param(
    [string]$Version = "0.1.0",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

# ── Configuration ───────────────────────────────────────────────────────────
$AppName = "neovide-tabs"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$InstallerDir = Join-Path $ProjectRoot "installer"
$OutputDir = Join-Path $ProjectRoot "target\installer"
$BinaryPath = Join-Path $ProjectRoot "target\release\$AppName.exe"

# ── Check Prerequisites ─────────────────────────────────────────────────────
Write-Host "=== Checking Prerequisites ===" -ForegroundColor Cyan

# Check for WiX
$wixPath = Get-Command wix -ErrorAction SilentlyContinue
if (-not $wixPath) {
    Write-Host "WiX Toolset not found." -ForegroundColor Red
    Write-Host "Install with: dotnet tool install --global wix" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Prerequisites:" -ForegroundColor Yellow
    Write-Host "  1. Install .NET SDK 6.0+: https://dotnet.microsoft.com/download" -ForegroundColor White
    Write-Host "  2. Install WiX: dotnet tool install --global wix" -ForegroundColor White
    Write-Host "  3. Add WiX UI extension: wix extension add WixToolset.UI.wixext" -ForegroundColor White
    exit 1
}
Write-Host "  WiX Toolset: $($wixPath.Source)" -ForegroundColor Green

# ── Verify Installer Assets ─────────────────────────────────────────────────
Write-Host "`n=== Verifying Installer Assets ===" -ForegroundColor Cyan

$requiredAssets = @(
    @{ Path = "neovide-tabs.ico"; Desc = "Application icon" },
    @{ Path = "WixUIBannerBmp.bmp"; Desc = "Installer banner (493x58)" },
    @{ Path = "WixUIDialogBmp.bmp"; Desc = "Dialog background (493x312)" },
    @{ Path = "License.rtf"; Desc = "License agreement" },
    @{ Path = "Package.wxs"; Desc = "WiX source file" }
)

$missing = @()
foreach ($asset in $requiredAssets) {
    $fullPath = Join-Path $InstallerDir $asset.Path
    if (Test-Path $fullPath) {
        Write-Host "  $($asset.Path): OK" -ForegroundColor Green
    }
    else {
        Write-Host "  $($asset.Path): MISSING - $($asset.Desc)" -ForegroundColor Red
        $missing += $asset.Path
    }
}

if ($missing.Count -gt 0) {
    Write-Host ""
    Write-Host "Missing installer assets. Run the following to generate them:" -ForegroundColor Yellow
    Write-Host "  .\scripts\generate-installer-assets.ps1" -ForegroundColor White
    Write-Host ""
    throw "Missing required installer assets: $($missing -join ', ')"
}

# ── Build Release Binary ────────────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Host "`n=== Building Release Binary ===" -ForegroundColor Cyan
    Push-Location $ProjectRoot
    try {
        cargo build --release
        if ($LASTEXITCODE -ne 0) { throw "Cargo build failed" }
    }
    finally {
        Pop-Location
    }
}

if (-not (Test-Path $BinaryPath)) {
    throw "Binary not found at $BinaryPath. Run without -SkipBuild."
}
Write-Host "  Binary: $BinaryPath" -ForegroundColor Green

# ── Create Output Directory ─────────────────────────────────────────────────
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
}

# ── Build MSI ───────────────────────────────────────────────────────────────
Write-Host "`n=== Building MSI Package ===" -ForegroundColor Cyan

$MsiPath = Join-Path $OutputDir "$AppName-$Version.msi"
$WxsPath = Join-Path $InstallerDir "Package.wxs"

Push-Location $InstallerDir
try {
    # Build with WiX v5
    # -d defines variables: ProductVersion and BinaryPath
    wix build $WxsPath `
        -o $MsiPath `
        -d ProductVersion=$Version `
        -d "BinaryPath=$BinaryPath" `
        -ext WixToolset.UI.wixext
    
    if ($LASTEXITCODE -ne 0) { throw "WiX build failed" }
}
finally {
    Pop-Location
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Green
Write-Host "  MSI: $MsiPath" -ForegroundColor White
Write-Host ""
Write-Host "Installation:" -ForegroundColor Cyan
Write-Host "  Double-click the MSI file, or run:" -ForegroundColor White
Write-Host "    msiexec /i `"$MsiPath`"" -ForegroundColor White
Write-Host ""
Write-Host "Silent installation (per-user):" -ForegroundColor Cyan
Write-Host "    msiexec /i `"$MsiPath`" /qn" -ForegroundColor White
Write-Host ""
Write-Host "Silent installation (per-machine, requires admin):" -ForegroundColor Cyan
Write-Host "    msiexec /i `"$MsiPath`" /qn ALLUSERS=1" -ForegroundColor White
