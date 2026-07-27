# MechvibesDX — Deployment & Packaging Guide

## 1. Prerequisites
- **Rust Toolchain**: 1.85+ (2024 Edition)
- **Node.js & pnpm**: Required for building CSS / Tailwind dependencies if modified.
- **Cargo Dependencies**: `cargo-bundle` (optional for standard app packaging).

## 2. Building for Local Development
```bash
# Check compilation
cargo check

# Run tests
cargo test

# Run application locally
cargo run
```

## 3. macOS Packaging (`package_macos.sh`)
The automated script `package_macos.sh` creates a standalone DMG for macOS:
```bash
./package_macos.sh
```
It performs:
1. Production release compilation (`cargo build --release`).
2. Creating `.app` bundle structure (`MechvibesDX.app/Contents/MacOS/`).
3. Embedding `Info.plist`, assets, and soundpack resources.
4. Ad-hoc codesigning with `codesign -s - --force`.
5. Generating signed `MechvibesDX.dmg` distribution disk image.

## 4. Windows Packaging
```bash
cargo build --release
```
Executables are placed in `target/release/mechvibes-dx.exe` alongside embedded ICO taskbar icons.
