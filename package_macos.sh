#!/bin/bash
set -e

echo "🚀 Starting MechvibesDX macOS Packaging..."

# 1. Clean previous build artifacts
echo "🧹 Cleaning previous staging and output files..."
rm -rf MechvibesDX.app MechvibesDX.dmg dmg_stage assets/icon.png

# 2. Build release binary if not already compiled
echo "⚙️ Building release binary..."
source $HOME/.cargo/env
cargo build --release

# 3. Create app bundle structure
echo "📁 Creating MechvibesDX.app bundle directory structure..."
mkdir -p MechvibesDX.app/Contents/MacOS
mkdir -p MechvibesDX.app/Contents/Resources

# 4. Generate app icon (.icns)
echo "🎨 Generating app icon..."
if [ -f assets/favicon.ico ]; then
    sips -s format png assets/favicon.ico --out assets/icon.png
    mkdir -p icon.iconset
    sips -z 16 16     assets/icon.png --out icon.iconset/icon_16x16.png
    sips -z 32 32     assets/icon.png --out icon.iconset/icon_16x16@2x.png
    sips -z 32 32     assets/icon.png --out icon.iconset/icon_32x32.png
    sips -z 64 64     assets/icon.png --out icon.iconset/icon_32x32@2x.png
    sips -z 128 128   assets/icon.png --out icon.iconset/icon_128x128.png
    sips -z 256 256   assets/icon.png --out icon.iconset/icon_128x128@2x.png
    sips -z 256 256   assets/icon.png --out icon.iconset/icon_256x256.png
    iconutil -c icns icon.iconset -o MechvibesDX.app/Contents/Resources/icon.icns
    rm -rf icon.iconset assets/icon.png
    echo "✅ App icon generated successfully."
else
    echo "⚠️ favicon.ico not found, skipping icon generation."
fi

# 5. Copy executable and resource files
echo "📦 Copying files to bundle..."
cp target/release/mechvibes-dx MechvibesDX.app/Contents/MacOS/
chmod +x MechvibesDX.app/Contents/MacOS/mechvibes-dx

# Copy soundpacks and assets inside Contents/MacOS/ since app root resolves to that parent folder
cp -R soundpacks MechvibesDX.app/Contents/MacOS/
cp -R assets MechvibesDX.app/Contents/MacOS/

# 6. Write Info.plist
echo "📝 Writing Info.plist..."
cat <<EOF > MechvibesDX.app/Contents/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleDisplayName</key>
    <string>MechvibesDX</string>
    <key>CFBundleExecutable</key>
    <string>mechvibes-dx</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>com.hainguyents13.mechvibesdx</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>MechvibesDX</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.4.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>0.4.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.10</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 7. Ad-hoc codesign the app bundle recursively (critical for macOS/Apple Silicon execution)
echo "🔒 Codesigning MechvibesDX.app bundle..."
codesign --force --deep --sign - MechvibesDX.app

# 8. Create DMG installer
echo "💿 Creating DMG installer..."
mkdir -p dmg_stage
cp -R MechvibesDX.app dmg_stage/
ln -s /Applications dmg_stage/Applications

hdiutil create -fs HFS+ -volname "MechvibesDX" -srcfolder dmg_stage MechvibesDX.dmg
rm -rf dmg_stage

# 9. Reset macOS TCC cache for the app bundle identifier to clear cached permission mismatches
echo "🧹 Resetting macOS Accessibility permission cache for MechvibesDX..."
tccutil reset Accessibility com.hainguyents13.mechvibesdx || true

echo "🎉 DMG packaging completed! MechvibesDX.dmg is ready."
