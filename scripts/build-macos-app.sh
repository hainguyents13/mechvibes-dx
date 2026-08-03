#!/usr/bin/env bash
#
# Assemble MechvibesDX.app and package it as a compressed DMG.
#
# `dx bundle` is deliberately NOT used: DioxusLabs/dioxus#5723 makes the 0.7.x
# resource copier fail on every directory entry in Dioxus.toml's `resources`,
# and worse, it exits non-zero having already produced a .app with an EMPTY
# Contents/Resources. Hand-assembly is fully specified and verifiable, so it is
# what ships until that fix lands (confirmed present in 0.8.0-alpha.1).
#
# Usage: scripts/build-macos-app.sh <version>
#   Expects target/release/mechvibes-dx to exist already.
#   Writes dist/mechvibes-dx-<version>-macos-<arch>-experimental.dmg
#
# macOS only (uses sips, iconutil, codesign, hdiutil).

set -euo pipefail

VERSION="${1:?usage: build-macos-app.sh <version>}"
ARCH=$(uname -m)

APP_NAME="MechvibesDX"
BUNDLE="dist/${APP_NAME}.app"
BINARY="target/release/mechvibes-dx"
IDENTIFIER="com.hainguyents13.mechvibesdx"
# arm64 macOS starts at 11.0; nothing older can run an Apple Silicon build.
MIN_MACOS="11.0"

if [ ! -f "$BINARY" ]; then
  echo "::error::$BINARY not found - build it before calling this script"
  exit 1
fi

rm -rf dist staging
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"

# ---------------------------------------------------------------------------
# 1. Executable
# ---------------------------------------------------------------------------
cp "$BINARY" "$BUNDLE/Contents/MacOS/mechvibes-dx"
chmod +x "$BUNDLE/Contents/MacOS/mechvibes-dx"

# ---------------------------------------------------------------------------
# 2. Resources
#
# `soundpacks/` is what src/state/paths.rs resolves via ../Resources when it
# detects the bundle layout. `assets/` must ALSO be here because
# dioxus-asset-resolver hardcodes Contents/Resources as the asset root on
# macOS - without it every asset!() font and icon 404s at runtime.
# ---------------------------------------------------------------------------
cp -R soundpacks "$BUNDLE/Contents/Resources/"
cp -R assets "$BUNDLE/Contents/Resources/"
cp README-macos.txt "$BUNDLE/Contents/Resources/"

# The failure mode this job already shipped once: an archive whose Resources
# were silently empty. Assert instead of trusting the copy.
packs=$(find "$BUNDLE/Contents/Resources/soundpacks" -name "*.ogg" | wc -l | tr -d ' ')
if [ "$packs" -eq 0 ]; then
  echo "::error::No soundpacks in the bundle - refusing to ship a silent app"
  exit 1
fi
echo "Bundled $packs soundpack audio files"

if [ ! -d "$BUNDLE/Contents/Resources/assets/fonts" ]; then
  echo "::error::assets/fonts missing from Resources - asset!() lookups would fail"
  exit 1
fi

# ---------------------------------------------------------------------------
# 3. Icon: generate .icns on the runner from the repo's 512x512 PNG.
# ---------------------------------------------------------------------------
ICONSET="staging/${APP_NAME}.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
  sips -z $size $size assets/icon.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z $double $double assets/icon.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$BUNDLE/Contents/Resources/${APP_NAME}.icns"
echo "Generated ${APP_NAME}.icns"

# ---------------------------------------------------------------------------
# 4. Info.plist
#
# LSUIElement is NOT set: the app has a real window, and a tray-only agent
# cannot be granted Accessibility permission through the normal UI flow.
# NSMicrophoneUsageDescription is absent on purpose - the app only plays audio.
# ---------------------------------------------------------------------------
cat > "$BUNDLE/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>mechvibes-dx</string>
    <key>CFBundleIconFile</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${IDENTIFIER}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>${MIN_MACOS}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
PLIST

plutil -lint "$BUNDLE/Contents/Info.plist"

# ---------------------------------------------------------------------------
# 5. Ad-hoc signature.
#
# This is NOT notarization and NOT a Developer ID signature - it only makes the
# bundle internally consistent so macOS will run it after the user clears
# quarantine. `spctl` still rejects it, which is expected and NOT gated on.
# ---------------------------------------------------------------------------
codesign --force --deep -s - "$BUNDLE"
codesign --verify --deep --strict --verbose=2 "$BUNDLE"
echo "Ad-hoc signature verified"

echo "--- Gatekeeper assessment (expected to FAIL: unsigned/not notarized) ---"
spctl --assess --type execute --verbose=4 "$BUNDLE" || true
echo "-----------------------------------------------------------------------"

# ---------------------------------------------------------------------------
# 6. DMG with an /Applications symlink so the user can drag-install.
# ---------------------------------------------------------------------------
DMG_ROOT="staging/dmg"
mkdir -p "$DMG_ROOT"
cp -R "$BUNDLE" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"
cp README-macos.txt "$DMG_ROOT/README.txt"

# "arm64"/"x86_64" contain no "x64" substring and this is not a .exe, so the
# Windows auto-updater filter in src/utils/auto_updater.rs cannot pick it up.
DMG="dist/mechvibes-dx-${VERSION}-macos-${ARCH}-experimental.dmg"
hdiutil create -volname "${APP_NAME}" -srcfolder "$DMG_ROOT" -ov -format UDZO "$DMG"
hdiutil verify "$DMG"

# Ship the bundle only inside the DMG.
rm -rf "$BUNDLE"
cp README-macos.txt "dist/README-macos-${VERSION}.txt"

echo "--- dist/ ---"
ls -la dist/
