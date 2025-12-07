#Requires -Version 5.1
<#
.SYNOPSIS
    Build MSIX installer for neovide-tabs locally.

.DESCRIPTION
    This script builds an MSIX package for local testing. It can create either:
    - An unsigned package (for sideloading with developer mode)
    - A self-signed package (for testing installation flow)

.PARAMETER Version
    Version number for the package (default: 0.1.0.0)

.PARAMETER SelfSign
    Create a self-signed certificate and sign the package

.PARAMETER SkipBuild
    Skip cargo build (use existing binary)

.EXAMPLE
    .\build-msix.ps1
    # Builds unsigned MSIX

.EXAMPLE
    .\build-msix.ps1 -SelfSign
    # Builds and signs with self-signed certificate

.EXAMPLE
    .\build-msix.ps1 -Version "1.2.3.0" -SkipBuild
    # Uses existing binary, custom version

.NOTES
    Prerequisites:
    - Windows 10/11 SDK (for makeappx.exe and signtool.exe)
    - Rust toolchain (unless -SkipBuild)
    - For unsigned packages: Enable Developer Mode in Windows Settings
    - For self-signed: Run as Administrator (first time only, to install cert)
#>

param(
    [string]$Version = "0.1.0.0",
    [switch]$SelfSign,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

# ── Configuration ───────────────────────────────────────────────────────────
$AppName = "neovide-tabs"
$IdentityName = "KeathMilligan.NeovideTabs"
$PublisherName = "Keath Milligan"
$PublisherCN = "CN=$PublisherName"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$OutputDir = Join-Path $ProjectRoot "target\msix"
$MsixLayoutDir = Join-Path $OutputDir "layout"
$AssetsDir = Join-Path $MsixLayoutDir "Assets"

# ── Find Windows SDK ────────────────────────────────────────────────────────
function Find-WindowsSDK {
    $sdkRoot = "C:\Program Files (x86)\Windows Kits\10\bin"
    if (-not (Test-Path $sdkRoot)) {
        throw "Windows SDK not found at $sdkRoot. Install Windows 10/11 SDK."
    }
    
    $latestSdk = Get-ChildItem $sdkRoot -Directory | 
        Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
        Sort-Object { [version]$_.Name } -Descending |
        Select-Object -First 1
    
    if (-not $latestSdk) {
        throw "No Windows SDK version found in $sdkRoot"
    }
    
    $sdkBin = Join-Path $latestSdk.FullName "x64"
    Write-Host "Found Windows SDK: $($latestSdk.Name)" -ForegroundColor Green
    return $sdkBin
}

$SdkBin = Find-WindowsSDK
$MakeAppx = Join-Path $SdkBin "makeappx.exe"
$SignTool = Join-Path $SdkBin "signtool.exe"

if (-not (Test-Path $MakeAppx)) {
    throw "makeappx.exe not found at $MakeAppx"
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

$BinaryPath = Join-Path $ProjectRoot "target\release\$AppName.exe"
if (-not (Test-Path $BinaryPath)) {
    throw "Binary not found at $BinaryPath. Run without -SkipBuild."
}

# ── Create MSIX Layout ──────────────────────────────────────────────────────
Write-Host "`n=== Creating MSIX Layout ===" -ForegroundColor Cyan

# Clean and create directories
if (Test-Path $MsixLayoutDir) {
    Remove-Item $MsixLayoutDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $AssetsDir | Out-Null

# Copy binary
Copy-Item $BinaryPath $MsixLayoutDir
Write-Host "  Copied: $AppName.exe"

# Copy icons
# neovide-tabs.png (256x256) - app icon for taskbar, start menu
# neovide-tabs-pix.png (256x256) - display image for installer, tiles, store
$IconApp = Join-Path $ProjectRoot "neovide-tabs.png"
$IconDisplay = Join-Path $ProjectRoot "neovide-tabs-pix.png"

if (Test-Path $IconApp) {
    # App icon (taskbar, start menu small)
    Copy-Item $IconApp (Join-Path $AssetsDir "Square44x44Logo.png")
    Write-Host "  Copied: Square44x44Logo.png (from neovide-tabs.png)"
}
if (Test-Path $IconDisplay) {
    # Display images (installer, tiles, store logo)
    Copy-Item $IconDisplay (Join-Path $AssetsDir "Square150x150Logo.png")
    Copy-Item $IconDisplay (Join-Path $AssetsDir "Square310x310Logo.png")
    Copy-Item $IconDisplay (Join-Path $AssetsDir "Wide310x150Logo.png")
    Copy-Item $IconDisplay (Join-Path $AssetsDir "StoreLogo.png")
    Write-Host "  Copied: Square150x150Logo.png, Square310x310Logo.png, Wide310x150Logo.png, StoreLogo.png (from neovide-tabs-pix.png)"
}

# ── Generate AppxManifest.xml ───────────────────────────────────────────────
Write-Host "`n=== Generating AppxManifest.xml ===" -ForegroundColor Cyan

$ManifestTemplate = Join-Path $ProjectRoot ".github\msix\AppxManifest.xml"
$ManifestContent = Get-Content $ManifestTemplate -Raw

$ManifestContent = $ManifestContent -replace '\$\{VERSION\}', $Version
$ManifestContent = $ManifestContent -replace '\$\{PUBLISHER_DISPLAY_NAME\}', $PublisherName
$ManifestContent = $ManifestContent -replace '\$\{IDENTITY_NAME\}', $IdentityName

$ManifestPath = Join-Path $MsixLayoutDir "AppxManifest.xml"
$ManifestContent | Set-Content $ManifestPath -Encoding UTF8
Write-Host "  Generated: AppxManifest.xml (Version: $Version)"

# ── Pack MSIX ───────────────────────────────────────────────────────────────
Write-Host "`n=== Packing MSIX ===" -ForegroundColor Cyan

$UnsignedMsix = Join-Path $OutputDir "$AppName-$Version-unsigned.msix"
$SignedMsix = Join-Path $OutputDir "$AppName-$Version.msix"

& $MakeAppx pack /d $MsixLayoutDir /p $UnsignedMsix /o
if ($LASTEXITCODE -ne 0) { throw "makeappx pack failed" }
Write-Host "  Created: $UnsignedMsix" -ForegroundColor Green

# ── Self-Sign (Optional) ────────────────────────────────────────────────────
if ($SelfSign) {
    Write-Host "`n=== Self-Signing MSIX ===" -ForegroundColor Cyan
    
    if (-not (Test-Path $SignTool)) {
        throw "signtool.exe not found at $SignTool"
    }
    
    $CertName = "$AppName-dev"
    $CertPath = Join-Path $OutputDir "$CertName.pfx"
    $CertPassword = "msix-dev-password"
    
    # Check if cert exists in store
    $ExistingCert = Get-ChildItem Cert:\CurrentUser\My | 
        Where-Object { $_.Subject -eq $PublisherCN } |
        Select-Object -First 1
    
    if (-not $ExistingCert) {
        Write-Host "  Creating self-signed certificate..." -ForegroundColor Yellow
        
        # Create self-signed certificate
        $Cert = New-SelfSignedCertificate `
            -Type Custom `
            -Subject $PublisherCN `
            -KeyUsage DigitalSignature `
            -FriendlyName "$AppName Development Certificate" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
        
        # Export to PFX
        $SecurePassword = ConvertTo-SecureString -String $CertPassword -Force -AsPlainText
        Export-PfxCertificate -Cert $Cert -FilePath $CertPath -Password $SecurePassword | Out-Null
        Write-Host "  Created certificate: $CertPath"
        
        # Install to LocalMachine Trusted Root (required for MSIX installation)
        # This requires elevation - prompt if not already elevated
        $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        
        if ($isAdmin) {
            Write-Host "  Installing certificate to LocalMachine Trusted Root store..." -ForegroundColor Yellow
            Import-PfxCertificate `
                -FilePath $CertPath `
                -CertStoreLocation "Cert:\LocalMachine\Root" `
                -Password $SecurePassword | Out-Null
            Write-Host "  Certificate installed to LocalMachine\Root. Thumbprint: $($Cert.Thumbprint)"
        }
        else {
            Write-Host "  Installing certificate to LocalMachine Trusted Root store (requires elevation)..." -ForegroundColor Yellow
            $importCmd = "Import-PfxCertificate -FilePath '$CertPath' -CertStoreLocation 'Cert:\LocalMachine\Root' -Password (ConvertTo-SecureString -String '$CertPassword' -Force -AsPlainText)"
            Start-Process powershell -Verb RunAs -ArgumentList "-Command", $importCmd -Wait
            Write-Host "  Certificate installed to LocalMachine\Root. Thumbprint: $($Cert.Thumbprint)"
        }
        
        # Also install to CurrentUser stores as fallback
        Import-PfxCertificate -FilePath $CertPath -CertStoreLocation "Cert:\CurrentUser\Root" -Password $SecurePassword | Out-Null
        Import-PfxCertificate -FilePath $CertPath -CertStoreLocation "Cert:\CurrentUser\TrustedPeople" -Password $SecurePassword | Out-Null
        
        $SigningCert = $Cert
    }
    else {
        Write-Host "  Using existing certificate: $($ExistingCert.Thumbprint)"
        $SigningCert = $ExistingCert
        
        # Export if PFX doesn't exist
        if (-not (Test-Path $CertPath)) {
            $SecurePassword = ConvertTo-SecureString -String $CertPassword -Force -AsPlainText
            Export-PfxCertificate -Cert $SigningCert -FilePath $CertPath -Password $SecurePassword | Out-Null
        }
    }
    
    # Copy unsigned to signed location
    Copy-Item $UnsignedMsix $SignedMsix -Force
    
    # Sign the package
    & $SignTool sign /fd SHA256 /a /f $CertPath /p $CertPassword $SignedMsix
    if ($LASTEXITCODE -ne 0) { throw "signtool sign failed" }
    
    Write-Host "  Signed: $SignedMsix" -ForegroundColor Green
    
    Write-Host "`n=== Installation Instructions ===" -ForegroundColor Cyan
    Write-Host "  The self-signed MSIX can be installed by double-clicking:"
    Write-Host "    $SignedMsix" -ForegroundColor White
    Write-Host ""
    Write-Host "  Note: The certificate has been added to Trusted Root and Trusted People stores."
    Write-Host "  To install on other machines, first install the certificate to Trusted Root:"
    Write-Host "    certutil -user -addstore Root `"$CertPath`"" -ForegroundColor White
    Write-Host "    (password: $CertPassword)" -ForegroundColor Gray
}
else {
    Write-Host "`n=== Installation Instructions ===" -ForegroundColor Cyan
    Write-Host "  Unsigned MSIX requires Developer Mode enabled in Windows Settings."
    Write-Host "  Settings > Privacy & Security > For developers > Developer Mode"
    Write-Host ""
    Write-Host "  Then install with PowerShell:"
    Write-Host "    Add-AppxPackage -Path `"$UnsignedMsix`"" -ForegroundColor White
    Write-Host ""
    Write-Host "  Or run with -SelfSign to create a signed package:"
    Write-Host "    .\build-msix.ps1 -SelfSign" -ForegroundColor White
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Green
Write-Host "  Output directory: $OutputDir"
