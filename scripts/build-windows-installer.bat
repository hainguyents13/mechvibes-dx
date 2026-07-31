@echo off
REM MechvibesDX Windows Installer Build Script (Batch version)
REM This script builds the release binary and creates a Windows installer

REM Change to project root directory (parent of scripts folder)
cd /d "%~dp0\.."

echo ========================================
echo MechvibesDX Windows Installer Builder
echo ========================================
echo.
echo Working directory: %CD%
echo.

REM Check if Cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Cargo is not installed or not in PATH
    exit /b 1
)

REM Build release binary
echo [1/3] Building release binary...
echo Running: cargo build --release
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Build failed
    exit /b 1
)

echo Build completed successfully
echo.

REM Create dist directory
echo [2/3] Preparing output directory...
if not exist "dist" mkdir dist
echo Output directory ready
echo.

REM Build installer
echo [3/3] Building Inno Setup installer...
echo.

REM Try to find Inno Setup
set ISCC=
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" set ISCC=%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe
if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" set ISCC=%ProgramFiles%\Inno Setup 6\ISCC.exe
if exist "%ProgramFiles(x86)%\Inno Setup 5\ISCC.exe" set ISCC=%ProgramFiles(x86)%\Inno Setup 5\ISCC.exe
if exist "%ProgramFiles%\Inno Setup 5\ISCC.exe" set ISCC=%ProgramFiles%\Inno Setup 5\ISCC.exe

if "%ISCC%"=="" (
    echo ERROR: Inno Setup not found
    echo Please install Inno Setup from: https://jrsoftware.org/isinfo.php
    exit /b 1
)

echo Found Inno Setup: %ISCC%
"%ISCC%" installer\windows\mechvibes-dx-setup.iss

if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Inno Setup compilation failed
    exit /b 1
)

echo.
echo ========================================
echo Build completed successfully!
echo ========================================
echo.
echo Installer created in 'dist' folder
pause

