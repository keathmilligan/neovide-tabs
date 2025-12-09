#Requires -Version 5.1
<#
.SYNOPSIS
    Generate installer assets (icons and UI bitmaps) for the MSI installer.

.DESCRIPTION
    This script generates all required assets for the WiX MSI installer:
    - neovide-tabs.ico: Multi-resolution icon for Add/Remove Programs and shortcuts
    - WixUIBannerBmp: 493x58 banner bitmap for installer dialog headers
    - WixUIDialogBmp: 493x312 background bitmap for welcome/finish dialogs

    Source images:
    - neovide-tabs.png: Base icon (256x256) - used for .ico generation
    - neovide-tabs-pix.png: Logo image (256x256) - used for installer bitmaps

.PARAMETER Force
    Overwrite existing assets even if they exist

.EXAMPLE
    .\generate-installer-assets.ps1
    # Generates all installer assets

.EXAMPLE
    .\generate-installer-assets.ps1 -Force
    # Regenerates all assets, overwriting existing files

.NOTES
    Prerequisites:
    - ImageMagick must be installed: https://imagemagick.org/
    - Install via: winget install ImageMagick.ImageMagick
      or: choco install imagemagick
#>

param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# ── Configuration ───────────────────────────────────────────────────────────
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$InstallerDir = Join-Path $ProjectRoot "installer"

# Source images
$IconSource = Join-Path $ProjectRoot "neovide-tabs.png"
$LogoSource = Join-Path $ProjectRoot "neovide-tabs-pix.png"

# Output files
$IconOutput = Join-Path $InstallerDir "neovide-tabs.ico"
$BannerOutput = Join-Path $InstallerDir "WixUIBannerBmp.bmp"
$DialogOutput = Join-Path $InstallerDir "WixUIDialogBmp.bmp"

# WiX UI bitmap specifications
# Banner: 493x58 pixels, shown at top of most dialogs
# Dialog: 493x312 pixels, shown on welcome and completion dialogs
$BannerWidth = 493
$BannerHeight = 58
$DialogWidth = 493
$DialogHeight = 312

# Background color (Tokyo Night theme)
$BgColor = "#1a1b26"

# ── Check Prerequisites ─────────────────────────────────────────────────────
Write-Host "=== Checking Prerequisites ===" -ForegroundColor Cyan

$magick = Get-Command magick -ErrorAction SilentlyContinue
if (-not $magick) {
    Write-Host "ImageMagick not found." -ForegroundColor Red
    Write-Host ""
    Write-Host "Install ImageMagick using one of these methods:" -ForegroundColor Yellow
    Write-Host "  winget install ImageMagick.ImageMagick" -ForegroundColor White
    Write-Host "  choco install imagemagick" -ForegroundColor White
    Write-Host "  https://imagemagick.org/script/download.php" -ForegroundColor White
    exit 1
}
Write-Host "  ImageMagick: $($magick.Source)" -ForegroundColor Green

# Check source images exist
if (-not (Test-Path $IconSource)) {
    throw "Icon source not found: $IconSource"
}
if (-not (Test-Path $LogoSource)) {
    throw "Logo source not found: $LogoSource"
}
Write-Host "  Icon source: $IconSource" -ForegroundColor Green
Write-Host "  Logo source: $LogoSource" -ForegroundColor Green

# ── Generate Icon ───────────────────────────────────────────────────────────
Write-Host "`n=== Generating Icon ===" -ForegroundColor Cyan

if ((Test-Path $IconOutput) -and -not $Force) {
    Write-Host "  Skipping: $IconOutput (already exists, use -Force to overwrite)" -ForegroundColor Yellow
}
else {
    Write-Host "  Creating multi-resolution ICO from $IconSource..." -ForegroundColor White
    
    # Create ICO with multiple sizes for best display at various DPIs
    # 256, 128, 64, 48, 32, 16 are standard Windows icon sizes
    # -type TrueColorAlpha preserves full 32-bit RGBA color
    # -depth 8 ensures 8 bits per channel (32-bit total with alpha)
    & magick $IconSource -type TrueColorAlpha -depth 8 -define icon:auto-resize=256,128,64,48,32,16 $IconOutput
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create icon"
    }
    Write-Host "  Created: $IconOutput" -ForegroundColor Green
}

# ── Generate Banner Bitmap ──────────────────────────────────────────────────
Write-Host "`n=== Generating Banner Bitmap ===" -ForegroundColor Cyan

if ((Test-Path $BannerOutput) -and -not $Force) {
    Write-Host "  Skipping: $BannerOutput (already exists, use -Force to overwrite)" -ForegroundColor Yellow
}
else {
    Write-Host "  Creating ${BannerWidth}x${BannerHeight} banner bitmap..." -ForegroundColor White
    
    # Create banner: white background with logo on the right
    # Scale logo to fit banner height with padding
    $logoHeight = $BannerHeight - 8  # 4px padding top/bottom
    $logoWidth = $logoHeight  # Keep square
    $logoX = $BannerWidth - $logoWidth - 8  # Right padding
    $logoY = 4  # Top padding
    
    & magick -size "${BannerWidth}x${BannerHeight}" "xc:white" `
        "(" $LogoSource -resize "${logoWidth}x${logoHeight}" ")" `
        -gravity NorthWest -geometry "+${logoX}+${logoY}" -composite `
        -type TrueColor -depth 24 "BMP3:$BannerOutput"
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create banner bitmap"
    }
    Write-Host "  Created: $BannerOutput" -ForegroundColor Green
}

# ── Generate Dialog Bitmap ──────────────────────────────────────────────────
Write-Host "`n=== Generating Dialog Bitmap ===" -ForegroundColor Cyan

if ((Test-Path $DialogOutput) -and -not $Force) {
    Write-Host "  Skipping: $DialogOutput (already exists, use -Force to overwrite)" -ForegroundColor Yellow
}
else {
    Write-Host "  Creating ${DialogWidth}x${DialogHeight} dialog bitmap..." -ForegroundColor White
    
    # Create dialog background: white with smaller logo centered in left portion
    # The right side (roughly 2/3) is covered by the dialog text
    # So we place the logo in the left 1/3 area
    $logoSize = 100  # Smaller logo for dialog
    $logoX = 43  # Centered in left portion (~164px wide area)
    $logoY = [math]::Floor(($DialogHeight - $logoSize) / 2)  # Vertically centered
    
    & magick -size "${DialogWidth}x${DialogHeight}" "xc:white" `
        "(" $LogoSource -resize "${logoSize}x${logoSize}" ")" `
        -gravity NorthWest -geometry "+${logoX}+${logoY}" -composite `
        -type TrueColor -depth 24 "BMP3:$DialogOutput"
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create dialog bitmap"
    }
    Write-Host "  Created: $DialogOutput" -ForegroundColor Green
}

# ── Summary ─────────────────────────────────────────────────────────────────
Write-Host "`n=== Asset Generation Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Generated files in $InstallerDir`:" -ForegroundColor White
Write-Host "  - neovide-tabs.ico     (multi-resolution icon)" -ForegroundColor Gray
Write-Host "  - WixUIBannerBmp.bmp   (${BannerWidth}x${BannerHeight} installer banner)" -ForegroundColor Gray
Write-Host "  - WixUIDialogBmp.bmp   (${DialogWidth}x${DialogHeight} dialog background)" -ForegroundColor Gray
Write-Host ""
Write-Host "Remember to commit these files to the repository." -ForegroundColor Yellow
