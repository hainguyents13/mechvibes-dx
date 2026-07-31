# MechvibesDX Installer

This directory contains installer configurations for different platforms.

## Windows Installer

### Prerequisites

**Inno Setup**
- Download from: https://jrsoftware.org/isinfo.php
- Version 6.0 or later required
- Free and open source

### Building the Installer

#### Option 1: Using PowerShell Script (Recommended)

```powershell
# Build installer
.\scripts\build-windows-installer.ps1

# Skip rebuild (use existing binary)
.\scripts\build-windows-installer.ps1 -SkipBuild
```

#### Option 2: Using Batch Script

```cmd
.\scripts\build-windows-installer.bat
```

#### Option 3: Manual Build

1. Build release binary:
   ```cmd
   cargo build --release
   ```

2. Build installer:
   ```cmd
   "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\windows\mechvibes-dx-setup.iss
   ```

### Installer Features

The Windows installer includes:

- ✅ Installation to Program Files
- ✅ Desktop shortcut (optional)
- ✅ Start Menu shortcuts
- ✅ Run at Windows startup (optional)
- ✅ Automatic creation of custom soundpacks directory
- ✅ Clean uninstallation with option to keep user data
- ✅ Built-in soundpacks bundled with installer
- ✅ Modern wizard-style UI

### Output

Installer is created in the `dist` folder: `dist/MechvibesDX-{version}-Setup.exe`

### Installer Configuration

Configuration file: `installer/windows/mechvibes-dx-setup.iss`

Key settings:
- App ID: Unique identifier for the application
- App Version: Should match `Cargo.toml` version
- Install location: `%ProgramFiles%\MechvibesDX`
- User data: `%APPDATA%\Mechvibes`
- Compression: LZMA2 (maximum)
- Privileges: User-level (no admin required)

## Testing the Installer

1. Build the installer using one of the methods above
2. Run the installer on a clean Windows machine or VM
3. Test installation options:
   - Default installation
   - Custom installation path
   - Desktop shortcut creation
   - Startup option
4. Test the installed application
5. Test uninstallation:
   - Keep user data
   - Remove all data

## Troubleshooting

### Inno Setup not found

Install Inno Setup from https://jrsoftware.org/isinfo.php

### Build fails

1. Ensure you have the latest Rust toolchain: `rustup update`
2. Clean build: `cargo clean && cargo build --release`
3. Check that all dependencies are installed

### Installer doesn't include all files

Check the `[Files]` section in `mechvibes-dx-setup.iss` and ensure all necessary files are listed.

## Future Enhancements

- [ ] Code signing certificate
- [ ] Auto-update functionality
- [ ] Silent installation mode
- [ ] Custom installation components
- [ ] Multi-language support
- [ ] macOS DMG installer
- [ ] Linux AppImage/Flatpak

