#!/bin/bash
set -e

# Build script for Linux installers (DEB + AppImage)
# Usage: ./scripts/build-linux-installer.sh

echo "🚀 Building Mechvibes DX Linux installers..."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}📦 Version: ${VERSION}${NC}"
echo ""

# Step 1: Build release binary
echo -e "${BLUE}🔨 Step 1/3: Building release binary...${NC}"
cargo build --release
echo -e "${GREEN}✓ Release binary built${NC}"
echo ""

# Step 2: Build DEB package
echo -e "${BLUE}📦 Step 2/3: Building DEB package...${NC}"
if ! command -v cargo-deb &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-deb not found, installing...${NC}"
    cargo install cargo-deb
fi

cargo deb
DEB_FILE="target/debian/mechvibes-dx_${VERSION}_amd64.deb"
echo -e "${GREEN}✓ DEB package created: ${DEB_FILE}${NC}"
echo ""

# Step 3: Build AppImage
echo -e "${BLUE}📦 Step 3/3: Building AppImage...${NC}"

# Create AppDir structure
echo "  → Creating AppDir structure..."
rm -rf AppDir
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/icons/hicolor/512x512/apps
mkdir -p AppDir/usr/share/applications

# Copy files to AppDir
cp target/release/mechvibes-dx AppDir/usr/bin/
cp assets/icon.png AppDir/usr/share/icons/hicolor/512x512/apps/mechvibes-dx.png
cp mechvibes-dx.desktop AppDir/usr/share/applications/

# Create AppRun script
cat > AppDir/AppRun << 'APPRUN_EOF'
#!/bin/bash
APPDIR="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="$APPDIR/usr/lib:$LD_LIBRARY_PATH"
export PATH="$APPDIR/usr/bin:$PATH"
exec "$APPDIR/usr/bin/mechvibes-dx" "$@"
APPRUN_EOF
chmod +x AppDir/AppRun

# Download appimagetool if not exists
APPIMAGETOOL="appimagetool-x86_64.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "  → Downloading appimagetool..."
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
fi

# Build AppImage
echo "  → Building AppImage..."
ARCH=x86_64 ./$APPIMAGETOOL AppDir "mechvibes-dx-${VERSION}-x86_64.AppImage"
APPIMAGE_FILE="mechvibes-dx-${VERSION}-x86_64.AppImage"

# Move to dist directory
mkdir -p dist
mv "$APPIMAGE_FILE" "dist/$APPIMAGE_FILE"

echo -e "${GREEN}✓ AppImage created: dist/${APPIMAGE_FILE}${NC}"
echo ""

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ Build completed successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}📦 Packages created:${NC}"
echo -e "  • DEB:      ${DEB_FILE}"
echo -e "  • AppImage: dist/${APPIMAGE_FILE}"
echo ""
echo -e "${BLUE}📝 Installation:${NC}"
echo -e "  • DEB:      sudo dpkg -i ${DEB_FILE}"
echo -e "  • AppImage: chmod +x dist/${APPIMAGE_FILE} && ./dist/${APPIMAGE_FILE}"
echo ""
echo -e "${YELLOW}⚠️  Note: DEB installation requires logout/login for input group access${NC}"
echo ""
