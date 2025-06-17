#!/usr/bin/env pwsh
# Cross-Platform Build System for MechvibesDX (Pure PowerShell)
# Reads configuration from Cargo.toml and Dioxus.toml

param(
    [Parameter(Position=0)]
    [ValidateSet("current", "windows", "macos", "linux", "help")]
    [string]$Target = "current",
    
    [switch]$Help
)

if ($Help -or $Target -eq "help") {
    Write-Host "🚀 MechvibesDX Cross-Platform Builder" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\cross-platform-build.ps1 [target]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Targets:" -ForegroundColor Green
    Write-Host "  current   Build installer for current platform (default)" -ForegroundColor White
    Write-Host "  windows   Build Windows NSIS installer (.exe)" -ForegroundColor White
    Write-Host "  macos     Build macOS DMG installer (.dmg)" -ForegroundColor White
    Write-Host "  linux     Build Linux AppImage (.AppImage)" -ForegroundColor White
    Write-Host ""
    Write-Host "Features:" -ForegroundColor Cyan
    Write-Host "  📋 Reads config from Cargo.toml and Dioxus.toml"
    Write-Host "  🎯 Production builds only"
    Write-Host "  📦 Creates proper installers for each platform"
    Write-Host "  🚫 No portable bundles - installers only"
    Write-Host ""
    exit 0
}

function Read-CargoToml {
    $cargoPath = "Cargo.toml"
    if (-not (Test-Path $cargoPath)) {
        Write-Host "❌ Cargo.toml not found!" -ForegroundColor Red
        exit 1
    }
    
    $content = Get-Content $cargoPath -Raw
    
    # Simple TOML parsing for package section
    if ($content -match '(?s)\[package\].*?name\s*=\s*"([^"]*)"') {
        $appName = $matches[1]
    } else {
        $appName = "mechvibes-dx"
    }
    
    if ($content -match '(?s)\[package\].*?version\s*=\s*"([^"]*)"') {
        $appVersion = $matches[1]
    } else {
        $appVersion = "0.1.0"
    }
    
    return @{
        name = $appName
        version = $appVersion
    }
}

function Read-DioxusToml {
    $dioxusPath = "Dioxus.toml"
    if (-not (Test-Path $dioxusPath)) {
        Write-Host "⚠️ Dioxus.toml not found, using defaults" -ForegroundColor Yellow
        return @{
            displayName = "MechvibesDX"
            identifier = "com.mechvibes.dx"
            publisher = "Hai Nguyen"
            description = "Mechanical keyboard soundboard"
        }
    }
    
    $content = Get-Content $dioxusPath -Raw
    
    # Simple TOML parsing for bundle section
    $displayName = if ($content -match '(?s)\[bundle\].*?name\s*=\s*"([^"]*)"') { $matches[1] } else { "MechvibesDX" }
    $identifier = if ($content -match '(?s)\[bundle\].*?identifier\s*=\s*"([^"]*)"') { $matches[1] } else { "com.mechvibes.dx" }
    $publisher = if ($content -match '(?s)\[bundle\].*?publisher\s*=\s*"([^"]*)"') { $matches[1] } else { "Hai Nguyen" }
    $description = if ($content -match '(?s)\[bundle\].*?long_description\s*=\s*"([^"]*)"') { $matches[1] } else { "Mechanical keyboard soundboard" }
    
    return @{
        displayName = $displayName
        identifier = $identifier
        publisher = $publisher
        description = $description
    }
}

function Build-Release {
    Write-Host "🔨 Building release version..." -ForegroundColor Yellow
    cargo build --release --bin mechvibes-dx
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✅ Build completed!" -ForegroundColor Green
}

function Create-DistDir($platform) {
    $distDir = "dist\$platform"
    if (Test-Path $distDir) {
        Remove-Item $distDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
    return $distDir
}

function Build-WindowsInstaller($config) {
    Write-Host "🪟 Building Windows NSIS installer..." -ForegroundColor Cyan
    
    # Check NSIS installation
    $nsisFound = $false
    
    # Try common NSIS locations
    $nsisLocations = @(
        "makensis",  # In PATH
        "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
        "${env:ProgramFiles}\NSIS\makensis.exe",
        "C:\Program Files (x86)\NSIS\makensis.exe",
        "C:\Program Files\NSIS\makensis.exe"
    )
    
    foreach ($location in $nsisLocations) {
        if (Get-Command $location -ErrorAction SilentlyContinue) {
            $nsisPath = $location
            $nsisFound = $true
            Write-Host "✅ Found NSIS: $location" -ForegroundColor Green
            break
        }
    }
    
    if (-not $nsisFound) {
        Write-Host "❌ NSIS not found!" -ForegroundColor Red
        Write-Host "" 
        Write-Host "💡 Install options:" -ForegroundColor Yellow
        Write-Host "   1. .\install-nsis.ps1                    # Normal install" -ForegroundColor Gray
        Write-Host "   2. .\install-nsis-admin.bat              # With admin rights" -ForegroundColor Gray
        Write-Host "   3. .\install-nsis.ps1 -RunAsAdmin        # Auto-elevate" -ForegroundColor Gray
        Write-Host "   4. Manual: https://nsis.sourceforge.io/Download" -ForegroundColor Gray
        Write-Host ""
        return $false
    }
    
    $distDir = Create-DistDir "windows"
    $exePath = "target\release\$($config.cargo.name).exe"
    
    if (-not (Test-Path $exePath)) {
        Write-Host "❌ Executable not found: $exePath" -ForegroundColor Red
        return $false
    }
      # Create NSIS script
    $nsisScript = "$distDir\installer.nsi"
    $installerName = "$($config.cargo.name)-setup-v$($config.cargo.version).exe"
    
    # Check if directories exist and have content
    $assetsExist = (Test-Path "assets") -and (Get-ChildItem "assets" -Recurse -File).Count -gt 0
    $soundpacksExist = (Test-Path "soundpacks") -and (Get-ChildItem "soundpacks" -Recurse -File).Count -gt 0
    
    # Build the file inclusion section
    $fileInclusions = @()
    if ($assetsExist) {
        $fileInclusions += "    File /r `"$((Get-Location).Path)\assets`""
    }
    if ($soundpacksExist) {
        $fileInclusions += "    File /r `"$((Get-Location).Path)\soundpacks`""
    }
    $fileInclusionSection = $fileInclusions -join "`n"
    
    # Build the uninstall section
    $uninstallSections = @()
    if ($assetsExist) {
        $uninstallSections += "    RMDir /r `"`$INSTDIR\assets`""
    }
    if ($soundpacksExist) {
        $uninstallSections += "    RMDir /r `"`$INSTDIR\soundpacks`""
    }
    $uninstallSection = $uninstallSections -join "`n"
    
    $scriptContent = @"
!define APP_NAME "$($config.dioxus.displayName)"
!define APP_VERSION "$($config.cargo.version)"
!define APP_PUBLISHER "$($config.dioxus.publisher)"
!define APP_IDENTIFIER "$($config.dioxus.identifier)"
!define APP_DESCRIPTION "$($config.dioxus.description)"
!define APP_EXE "$($config.cargo.name).exe"

# Interface Configuration - MUST be defined BEFORE including MUI2.nsh
!define MUI_ABORTWARNING
!define MUI_ICON "$((Get-Location).Path)\assets\icon.ico"
!define MUI_UNICON "$((Get-Location).Path)\assets\icon.ico"
!define MUI_HEADERIMAGE
# !define MUI_HEADERIMAGE_BITMAP_NOSTRETCH
# !define MUI_WELCOMEFINISHPAGE_BITMAP "$((Get-Location).Path)\assets\sidebar.bmp"
# !define MUI_UNWELCOMEFINISHPAGE_BITMAP "$((Get-Location).Path)\assets\sidebar.bmp"
# !define MUI_WELCOMEFINISHPAGE_BITMAP_NOSTRETCH
# !define MUI_UNWELCOMEFINISHPAGE_BITMAP_NOSTRETCH

# Modern UI includes
!include "MUI2.nsh"

# General settings
Name "`${APP_NAME}"
OutFile "$installerName"
InstallDir "`$PROGRAMFILES64\\`${APP_NAME}"
InstallDirRegKey HKLM "Software\\`${APP_NAME}" ""
RequestExecutionLevel admin

# Best compression
SetCompressor /SOLID lzma

# Variables
Var StartMenuFolder

# Welcome page
!define MUI_WELCOMEPAGE_TITLE "Welcome to `${APP_NAME} Setup"
!define MUI_WELCOMEPAGE_TEXT "This wizard will guide you through the installation of `${APP_NAME}. Click Next to continue."
!insertmacro MUI_PAGE_WELCOME

# License page (optional - uncomment if you have a license file)
# !insertmacro MUI_PAGE_LICENSE "LICENSE.txt"

# Components page
!insertmacro MUI_PAGE_COMPONENTS

# Directory page
!insertmacro MUI_PAGE_DIRECTORY

# Start Menu page
!define MUI_STARTMENUPAGE_REGISTRY_ROOT "HKLM"
!define MUI_STARTMENUPAGE_REGISTRY_KEY "Software\\`${APP_NAME}"
!define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "Start Menu Folder"
!insertmacro MUI_PAGE_STARTMENU Application `$StartMenuFolder

# Installation page
!insertmacro MUI_PAGE_INSTFILES

# Finish page
!define MUI_FINISHPAGE_RUN "`$INSTDIR\\`${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch `${APP_NAME}"
!define MUI_FINISHPAGE_LINK "Visit our website"
!define MUI_FINISHPAGE_LINK_LOCATION "https://github.com/hainguyents13/mechvibes"
!insertmacro MUI_PAGE_FINISH

# Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

# Languages
!insertmacro MUI_LANGUAGE "English"

# Version Information
VIProductVersion "`${APP_VERSION}.0"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "ProductName" "`${APP_NAME}"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "Comments" "`${APP_DESCRIPTION}"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "CompanyName" "`${APP_PUBLISHER}"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "LegalCopyright" "© `${APP_PUBLISHER}"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "FileDescription" "`${APP_NAME} Setup"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "FileVersion" "`${APP_VERSION}"
VIAddVersionKey /LANG=`${LANG_ENGLISH} "ProductVersion" "`${APP_VERSION}"

# Installer sections
Section "!`${APP_NAME}" SecMain
    SectionIn RO
    SetOutPath "`$INSTDIR"
    
    # Main executable
    File "$((Get-Location).Path)\$exePath"
    
    # Additional files
$fileInclusionSection
    
    # Create data directory
    CreateDirectory "`$INSTDIR\data"
    
    # Store installation folder
    WriteRegStr HKLM "Software\\`${APP_NAME}" "" `$INSTDIR
    
    # Create uninstaller
    WriteUninstaller "`$INSTDIR\Uninstall.exe"
    
    # Add to Add/Remove Programs
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "DisplayName" "`${APP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "UninstallString" "`$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "DisplayVersion" "`${APP_VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "Publisher" "`${APP_PUBLISHER}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "DisplayIcon" "`$INSTDIR\\`${APP_EXE}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "URLInfoAbout" "https://github.com/hainguyents13/mechvibes"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "NoRepair" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}" "EstimatedSize" 50000
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortcut "`$DESKTOP\\`${APP_NAME}.lnk" "`$INSTDIR\\`${APP_EXE}" "" "`$INSTDIR\\`${APP_EXE}" 0
SectionEnd

Section "Start Menu Shortcuts" SecStartMenu
    !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    CreateDirectory "`$SMPROGRAMS\`$StartMenuFolder"
    CreateShortcut "`$SMPROGRAMS\`$StartMenuFolder\\`${APP_NAME}.lnk" "`$INSTDIR\\`${APP_EXE}" "" "`$INSTDIR\\`${APP_EXE}" 0
    CreateShortcut "`$SMPROGRAMS\`$StartMenuFolder\Uninstall `${APP_NAME}.lnk" "`$INSTDIR\Uninstall.exe"
    !insertmacro MUI_STARTMENU_WRITE_END
SectionEnd

# Section descriptions
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT `${SecMain} "Main application files (required)"
  !insertmacro MUI_DESCRIPTION_TEXT `${SecDesktop} "Create a desktop shortcut"
  !insertmacro MUI_DESCRIPTION_TEXT `${SecStartMenu} "Create Start Menu shortcuts"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

# Uninstaller section
Section "Uninstall"
    # Remove files
    Delete "`$INSTDIR\\`${APP_EXE}"
    Delete "`$INSTDIR\Uninstall.exe"
$uninstallSection
    RMDir /r "`$INSTDIR\data"
    
    # Remove shortcuts
    !insertmacro MUI_STARTMENU_GETFOLDER Application `$StartMenuFolder
    Delete "`$SMPROGRAMS\`$StartMenuFolder\\`${APP_NAME}.lnk"
    Delete "`$SMPROGRAMS\`$StartMenuFolder\Uninstall `${APP_NAME}.lnk"
    RMDir "`$SMPROGRAMS\`$StartMenuFolder"
    Delete "`$DESKTOP\\`${APP_NAME}.lnk"
    
    # Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\\`${APP_NAME}"
    DeleteRegKey HKLM "Software\\`${APP_NAME}"
    
    # Remove installation directory if empty
    RMDir "`$INSTDIR"
SectionEnd
"@
    
    Set-Content $nsisScript $scriptContent
      # Run NSIS
    Write-Host "📦 Creating installer with NSIS..." -ForegroundColor Yellow
    Write-Host "   Using: $nsisPath" -ForegroundColor Gray
    
    $nsisResult = if ($nsisPath -eq "makensis") {
        & makensis $nsisScript
    } else {
        & $nsisPath $nsisScript
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Windows installer created: $distDir\$installerName" -ForegroundColor Green
        return $true
    } else {
        Write-Host "❌ NSIS installer creation failed!" -ForegroundColor Red
        Write-Host "💡 Check NSIS script and dependencies" -ForegroundColor Yellow
        return $false
    }
}

function Build-MacOSInstaller($config) {
    Write-Host "🍎 Building macOS DMG..." -ForegroundColor Cyan
    
    if ($env:OS -eq "Windows_NT") {
        Write-Host "❌ Cannot build macOS DMG on Windows" -ForegroundColor Red
        Write-Host "💡 Use macOS or cross-compilation setup" -ForegroundColor Yellow
        return $false
    }
    
    Write-Host "⚠️ macOS DMG creation requires macOS environment" -ForegroundColor Yellow
    Write-Host "💡 This would create .app bundle and DMG on macOS" -ForegroundColor Yellow
    return $false
}

function Build-LinuxInstaller($config) {
    Write-Host "🐧 Building Linux AppImage..." -ForegroundColor Cyan
    
    if ($env:OS -eq "Windows_NT") {
        Write-Host "❌ Cannot build Linux AppImage on Windows" -ForegroundColor Red
        Write-Host "💡 Use Linux or cross-compilation setup" -ForegroundColor Yellow
        return $false
    }
    
    Write-Host "⚠️ Linux AppImage creation requires Linux environment" -ForegroundColor Yellow
    Write-Host "💡 This would create AppImage on Linux" -ForegroundColor Yellow
    return $false
}

# Main execution
Write-Host "🚀 MechvibesDX Cross-Platform Builder" -ForegroundColor Cyan
Write-Host ""

# Read configuration
$cargoConfig = Read-CargoToml
$dioxusConfig = Read-DioxusToml

$config = @{
    cargo = $cargoConfig
    dioxus = $dioxusConfig
}

Write-Host "📋 Configuration:" -ForegroundColor Yellow
Write-Host "   App: $($config.dioxus.displayName) v$($config.cargo.version)" -ForegroundColor Gray
Write-Host "   Publisher: $($config.dioxus.publisher)" -ForegroundColor Gray
Write-Host "   Identifier: $($config.dioxus.identifier)" -ForegroundColor Gray
Write-Host ""

# Determine target platform
if ($Target -eq "current") {
    $Target = if ($env:OS -eq "Windows_NT") { "windows" } else { "linux" }
    Write-Host "🎯 Auto-detected platform: $Target" -ForegroundColor Cyan
}

# Build release first
Build-Release

# Build installer for target platform
$success = switch ($Target) {
    "windows" { Build-WindowsInstaller $config }
    "macos" { Build-MacOSInstaller $config }
    "linux" { Build-LinuxInstaller $config }
    default { 
        Write-Host "❌ Unknown target: $Target" -ForegroundColor Red
        $false
    }
}

if ($success) {
    Write-Host ""
    Write-Host "🎉 Build completed successfully!" -ForegroundColor Green
    Write-Host "📦 Check dist\$Target\ folder for installer" -ForegroundColor Cyan
    
    if (Test-Path "dist\$Target") {
        Write-Host ""
        Write-Host "📁 Distribution files:" -ForegroundColor Yellow
        Get-ChildItem "dist\$Target" -File | ForEach-Object {
            $size = [math]::Round($_.Length / 1MB, 2)
            Write-Host "  📄 $($_.Name) ($size MB)" -ForegroundColor Gray
        }
    }
} else {
    Write-Host ""
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
