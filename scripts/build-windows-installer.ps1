# MechvibesDX Windows Installer Build Script
# This script builds the release binary and creates a Windows installer using Inno Setup

param(
    [switch]$SkipBuild = $false,
    [switch]$UseNSIS = $false,
    [switch]$UseInnoSetup = $true
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "MechvibesDX Windows Installer Builder" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Get project root directory (where this script is located)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "Script directory: $ScriptDir" -ForegroundColor Gray
Write-Host "Project root: $ProjectRoot" -ForegroundColor Gray
Write-Host ""

# Change to project root
Set-Location $ProjectRoot
Write-Host "Working directory: $(Get-Location)" -ForegroundColor Gray
Write-Host ""

# Check if Cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Cargo is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Step 1: Build release binary
if (-not $SkipBuild) {
    Write-Host "[1/4] Building release binary..." -ForegroundColor Yellow
    Write-Host "Running: cargo build --release" -ForegroundColor Gray

    cargo build --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ Build completed successfully" -ForegroundColor Green
    Write-Host ""
} else {
    Write-Host "[1/4] Skipping build (using existing binary)" -ForegroundColor Yellow
    Write-Host ""
}

# Check if executable exists
$ExePath = Join-Path $ProjectRoot "target\release\mechvibes-dx.exe"
if (-not (Test-Path $ExePath)) {
    Write-Host "ERROR: Executable not found at $ExePath" -ForegroundColor Red
    Write-Host "Please run without -SkipBuild flag" -ForegroundColor Red
    exit 1
}

# Get file version
$FileVersion = (Get-Item $ExePath).VersionInfo.FileVersion
Write-Host "Executable version: $FileVersion" -ForegroundColor Cyan
Write-Host ""

# Step 2: Create dist directory
Write-Host "[2/4] Preparing output directory..." -ForegroundColor Yellow
$DistDir = Join-Path $ProjectRoot "dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}
Write-Host "✓ Output directory ready: $DistDir" -ForegroundColor Green
Write-Host ""

# Step 3: Build installer based on selected method
if ($UseInnoSetup) {
    Write-Host "[3/4] Building Inno Setup installer..." -ForegroundColor Yellow
    
    # Check if Inno Setup is installed
    $InnoSetupPaths = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 5\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 5\ISCC.exe"
    )
    
    $ISCC = $null
    foreach ($path in $InnoSetupPaths) {
        if (Test-Path $path) {
            $ISCC = $path
            break
        }
    }
    
    if (-not $ISCC) {
        Write-Host "ERROR: Inno Setup not found" -ForegroundColor Red
        Write-Host "Please install Inno Setup from: https://jrsoftware.org/isinfo.php" -ForegroundColor Yellow
        Write-Host "Or use -UseNSIS flag to build with NSIS instead" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "Found Inno Setup: $ISCC" -ForegroundColor Gray
    
    $InnoScript = Join-Path $ProjectRoot "installer\windows\mechvibes-dx-setup.iss"
    if (-not (Test-Path $InnoScript)) {
        Write-Host "ERROR: Inno Setup script not found at $InnoScript" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Running Inno Setup compiler..." -ForegroundColor Gray
    & $ISCC $InnoScript
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Inno Setup compilation failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ Inno Setup installer created successfully" -ForegroundColor Green
    Write-Host ""
}

if ($UseNSIS) {
    Write-Host "[3/4] Building NSIS installer using Dioxus bundler..." -ForegroundColor Yellow
    
    # Check if dx CLI is installed
    if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: Dioxus CLI (dx) is not installed" -ForegroundColor Red
        Write-Host "Install with: cargo install dioxus-cli" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "Running: dx bundle --release" -ForegroundColor Gray
    dx bundle --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: NSIS bundling failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ NSIS installer created successfully" -ForegroundColor Green
    Write-Host ""
}

# Step 4: Summary
Write-Host "[4/4] Build Summary" -ForegroundColor Yellow
Write-Host "==================" -ForegroundColor Yellow
Write-Host "Executable: $ExePath" -ForegroundColor Gray
Write-Host "Output directory: $DistDir" -ForegroundColor Gray

if ($UseInnoSetup) {
    $InstallerPath = Get-ChildItem -Path $DistDir -Filter "MechvibesDX-*-Setup.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($InstallerPath) {
        Write-Host "Installer: $($InstallerPath.FullName)" -ForegroundColor Gray
        Write-Host ""
        Write-Host "✓ Installer ready!" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

