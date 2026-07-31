![image](https://github.com/user-attachments/assets/5aa36739-76c8-4a34-9a9b-7e9272927f22)

# MechvibesDX

**A fun and practical way to bring your favorite sounds anywhere!**

MechvibesDX lets you play any sound when you type or click. Use it for education, presentations, gaming, or just for fun.

## Features

-   Play sounds on every keystroke (keydown/keyup) and mouse click (press/release)
-   Works with education, business, gaming, and accessibility needs
-   Global hotkey toggle (`Ctrl+Alt+M`)
-   System tray integration
-   Custom soundpack support
-   Multiple themes available
-   Logo and background customizations

## Installation

### End Users

1. Download from [Releases](https://github.com/hainguyents13/mechvibes-dx/releases)
2. Run installer
3. Select soundpacks
4. Enjoy the sounds or playing with Customizations

### Building from Source

#### Prerequisites

All platforms:
- [Rust](https://rustup.rs/) (1.70 or later)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started): `cargo install dioxus-cli`

#### Platform-Specific Dependencies

**Windows**
- Visual Studio Build Tools (C++ build tools)
- [Inno Setup](https://jrsoftware.org/isinfo.php) (for creating installer)

**Linux (Ubuntu/Debian)**
```bash
sudo apt-get update
sudo apt-get install -y \
    libasound2-dev \
    pkg-config \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libevdev-dev \
    libxdo-dev \
    autoconf \
    automake \
    libtool \
    libfuse2
# Note: libfuse2 is required for building AppImage
```

**Linux (Fedora/RHEL)**
```bash
sudo dnf install -y \
    alsa-lib-devel \
    pkg-config \
    webkit2gtk4.1-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    libevdev-devel \
    xdotool-devel \
    autoconf \
    automake \
    libtool \
    fuse-libs
# Note: fuse-libs is required for building AppImage
```

**macOS**
```bash
xcode-select --install
```

#### Build Instructions

**Development (all platforms)**
```bash
# Clone repository
git clone https://github.com/hainguyents13/mechvibes-dx.git
cd mechvibes-dx

# Run in development mode
dx serve

# Or use cargo directly
cargo run
```

**Release Build**

Windows:
```bash
# Build release binary
cargo build --release

# Create installer (requires Inno Setup)
.\scripts\build-windows-installer.ps1

# Output: dist/MechvibesDX-0.4.0-Setup.exe
```

Linux:
```bash
# Option 1: Build both DEB and AppImage (recommended)
./scripts/build-linux-installer.sh

# Outputs (unified in dist/ directory):
# - dist/mechvibes-dx_0.4.0_amd64.deb (Ubuntu/Debian)
# - dist/mechvibes-dx-0.4.0-x86_64.AppImage (Universal)

# Install DEB (auto-adds user to input group)
sudo dpkg -i dist/mechvibes-dx_0.4.0_amd64.deb

# Or run AppImage (portable, no install needed)
chmod +x dist/mechvibes-dx-0.4.0-x86_64.AppImage
./dist/mechvibes-dx-0.4.0-x86_64.AppImage

# Log out and log back in for group changes to take effect

# Option 2: Build DEB only
cargo install cargo-deb
cargo deb
# Output: target/debian/mechvibes-dx_0.4.0_amd64.deb

# Option 3: Build binary only
# Add user to input group manually (required for keyboard input)
sudo usermod -a -G input $USER
# Log out and log back in for group changes to take effect

# Build release binary
cargo build --release

# Binary location: target/release/mechvibes-dx
```

macOS:
```bash
# Build release binary
cargo build --release

# Bundle app (requires Dioxus CLI)
dx bundle --release

# Output: target/dx/mechvibes-dx/release/macos/MechvibesDX.app
```

## Use cases

**Education** - Musical scales, animal sounds, language learning

**Business** - Professional typewriter sounds, meeting-friendly modes

**Gaming** - Retro arcade sounds, custom sound effects

**Accessibility** - Audio feedback for visually impaired users

## Creating soundpacks

1. Record audio files (OGG, WAV, MP3)
2. Create config.json mapping keys to sounds
3. Drag and drop folder into app

```
Piano pack/
├── config.json
├── piano.ogg
└── icon.png
```

## Troubleshooting

**No sounds?** Check if muted (`Ctrl+Alt+M`), soundpack selected, system volume

**Hotkey not working?** Run as administrator, check for conflicts

**Soundpack won't load?** Verify config.json syntax, supported audio formats

**Linux: No keyboard input detected?**
- Add user to `input` group: `sudo usermod -a -G input $USER`
- Log out and log back in for changes to take effect
- Verify with: `groups $USER` (should include `input`)
- Check device permissions: `ls -la /dev/input/event*` (should show `crw-rw---- root input`)

## License

MIT License - do whatever you want with it.

Report bugs or request features via GitHub Issues.
