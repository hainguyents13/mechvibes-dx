#!/usr/bin/env bash
#
# Assemble AppDir/ and package it as an AppImage.
#
# Usage: ./scripts/build-linux-appimage.sh <version>
#   e.g. ./scripts/build-linux-appimage.sh 0.8.0
#
# Expects target/release/mechvibes-dx to already exist (the caller builds it,
# so this script never triggers a second 10-minute compile).
#
# The AppDir deliberately mirrors the .deb's filesystem layout - usr/bin,
# usr/share/mechvibes-dx/soundpacks - so that one binary serves both packages
# and src/state/paths.rs needs only a single extra branch rather than a
# separate AppImage code path. Two placements are NOT free choices:
#
#   usr/lib/mechvibes-dx/assets   dioxus-asset-resolver's Linux branch looks
#                                 for <exe>/../../lib/<dir>/assets and takes
#                                 the first entry containing an assets/ dir.
#                                 Put the fonts anywhere else and every
#                                 asset!() font silently 404s at runtime.
#
#   AppDir root .desktop + icon   appimagetool requires both at the top level;
#                                 the copies under usr/share are what a
#                                 desktop-integration helper installs later.

set -euo pipefail

VERSION="${1:?usage: build-linux-appimage.sh <version>}"

APP_NAME="mechvibes-dx"
ARCH="x86_64"
BINARY="target/release/${APP_NAME}"
APPDIR="build/AppDir"
OUTPUT="dist/${APP_NAME}-${VERSION}-${ARCH}.AppImage"

if [ ! -f "$BINARY" ]; then
  echo "::error::$BINARY not found - build it first with 'cargo build --release'"
  exit 1
fi

echo "=== Assembling AppDir for ${APP_NAME} ${VERSION} ==="

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib/${APP_NAME}"
mkdir -p "$APPDIR/usr/share/${APP_NAME}"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/512x512/apps"

# --- binary -----------------------------------------------------------------
install -m 755 "$BINARY" "$APPDIR/usr/bin/${APP_NAME}"

# --- read-only resources ----------------------------------------------------
cp -r soundpacks "$APPDIR/usr/share/${APP_NAME}/soundpacks"
cp -r assets "$APPDIR/usr/lib/${APP_NAME}/assets"

# --- desktop entry ----------------------------------------------------------
# Reuses the same file the .deb ships, so the two packages cannot drift apart.
# It is already appimagetool-safe: `Categories=AudioVideo;` is a single main
# category, which is what a prior attempt (commit 5222274) had to fix. Verified
# rather than assumed - the assertion below fails the build if that regresses.
if ! grep -q '^Categories=AudioVideo;$' mechvibes-dx.desktop; then
  echo "::error::mechvibes-dx.desktop must keep a single main category (Categories=AudioVideo;)"
  echo "  found: $(grep '^Categories=' mechvibes-dx.desktop || echo '<none>')"
  exit 1
fi
cp mechvibes-dx.desktop "$APPDIR/usr/share/applications/${APP_NAME}.desktop"
cp mechvibes-dx.desktop "$APPDIR/${APP_NAME}.desktop"

cp assets/icon.png "$APPDIR/usr/share/icons/hicolor/512x512/apps/${APP_NAME}.png"
cp assets/icon.png "$APPDIR/${APP_NAME}.png"
# .DirIcon is what file managers read for the image's own thumbnail.
cp assets/icon.png "$APPDIR/.DirIcon"

# --- AppRun -----------------------------------------------------------------
# Kept minimal on purpose. It does NOT bundle or preload system libraries:
# webkit2gtk, GTK and libasound come from the host, which is the standard
# trade-off for a webview app (a bundled webkit conflicts with the host's
# GStreamer and portal stack more often than it helps).
#
# APPDIR is exported because the AppImage runtime does so and tooling expects
# it, but the app itself does not read it - paths.rs derives the AppDir from
# the executable's own location, which an inherited stale env var cannot spoof.
cat > "$APPDIR/AppRun" <<'APPRUN'
#!/usr/bin/env bash
set -eu
HERE="$(dirname "$(readlink -f "${0}")")"
export APPDIR="${HERE}"
export PATH="${HERE}/usr/bin:${PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/mechvibes-dx" "$@"
APPRUN
chmod 755 "$APPDIR/AppRun"

# --- guard: resources must match the source tree ----------------------------
# A non-zero check is not enough. The four mouse packs are .mp3 while the
# keyboard packs are .ogg, so an .ogg-only count would pass while silently
# dropping every mouse pack - this exact hole was found during the macOS round.
count_audio() {
  find "$1" -type f \( -name '*.ogg' -o -name '*.mp3' -o -name '*.wav' \) | wc -l
}
count_configs() {
  find "$1" -type f -name 'config.json' | wc -l
}

SRC_AUDIO=$(count_audio soundpacks)
APP_AUDIO=$(count_audio "$APPDIR/usr/share/${APP_NAME}/soundpacks")
SRC_CONFIGS=$(count_configs soundpacks)
APP_CONFIGS=$(count_configs "$APPDIR/usr/share/${APP_NAME}/soundpacks")
SRC_FONTS=$(find assets/fonts -type f -name '*.ttf' | wc -l)
APP_FONTS=$(find "$APPDIR/usr/lib/${APP_NAME}/assets/fonts" -type f -name '*.ttf' | wc -l)

if [ "$SRC_AUDIO" -ne "$APP_AUDIO" ] || [ "$SRC_AUDIO" -eq 0 ]; then
  echo "::error::soundpack audio mismatch: AppDir has $APP_AUDIO, source tree has $SRC_AUDIO"
  exit 1
fi
if [ "$SRC_CONFIGS" -ne "$APP_CONFIGS" ] || [ "$SRC_CONFIGS" -eq 0 ]; then
  echo "::error::soundpack config.json mismatch: AppDir has $APP_CONFIGS, source tree has $SRC_CONFIGS"
  exit 1
fi
if [ "$SRC_FONTS" -ne "$APP_FONTS" ] || [ "$SRC_FONTS" -eq 0 ]; then
  echo "::error::font mismatch: AppDir has $APP_FONTS, source tree has $SRC_FONTS"
  exit 1
fi

echo "Bundled $APP_AUDIO soundpack audio files (matches source tree)"
echo "Bundled $APP_CONFIGS soundpack config.json files (matches source tree)"
echo "Bundled $APP_FONTS fonts (matches source tree)"

# --- appimagetool -----------------------------------------------------------
# GitHub runners have no FUSE, so appimagetool cannot mount *itself* to run.
# APPIMAGE_EXTRACT_AND_RUN=1 makes it unpack to a temp dir and exec from there
# instead, which is the supported workaround and needs no privileges.
TOOL="build/appimagetool-${ARCH}.AppImage"
if [ ! -f "$TOOL" ]; then
  echo "=== Downloading appimagetool ==="
  mkdir -p build
  curl -fsSL -o "$TOOL" \
    "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"
  chmod +x "$TOOL"
fi

mkdir -p dist
echo "=== Packaging ${OUTPUT} ==="
APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$ARCH" "./$TOOL" --no-appstream "$APPDIR" "$OUTPUT"

chmod +x "$OUTPUT"
echo "=== Built $OUTPUT ($(du -h "$OUTPUT" | cut -f1)) ==="
