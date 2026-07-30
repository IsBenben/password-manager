#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_VERSION="$(jq -r '.version' < "$ROOT_DIR/package.json")"
DIST_DIR="$ROOT_DIR/dist-package"
EXTENSION_DIR="$ROOT_DIR/extension"

echo "=== Password Manager Build v$APP_VERSION ==="

# Step 1: Build Tauri desktop app
echo ">>> Building desktop app..."
cd "$ROOT_DIR"
npm run build
npx tauri build 2>&1 | tail -5

# Step 2: Locate installer
INSTALLER=""
BUNDLE_DIR="$ROOT_DIR/src-tauri/target/release/bundle"
if [ -d "$BUNDLE_DIR/nsis" ]; then
    INSTALLER=$(ls "$BUNDLE_DIR/nsis/"*.exe 2>/dev/null | head -1)
elif [ -d "$BUNDLE_DIR/msi" ]; then
    INSTALLER=$(ls "$BUNDLE_DIR/msi/"*.msi 2>/dev/null | head -1)
elif [ -d "$BUNDLE_DIR/dmg" ]; then
    INSTALLER=$(ls "$BUNDLE_DIR/dmg/"*.dmg 2>/dev/null | head -1)
elif [ -d "$BUNDLE_DIR/appimage" ]; then
    INSTALLER=$(ls "$BUNDLE_DIR/appimage/"*.AppImage 2>/dev/null | head -1)
elif [ -d "$BUNDLE_DIR/deb" ]; then
    INSTALLER=$(ls "$BUNDLE_DIR/deb/"*.deb 2>/dev/null | head -1)
fi

if [ -z "$INSTALLER" ]; then
    echo "!!! WARNING: no installer found in $BUNDLE_DIR"
    INSTALLER=""
fi

# Step 3: Package browser extension
echo ">>> Packaging browser extension..."
EXT_ZIP="$ROOT_DIR/password-manager-extension-v$APP_VERSION.zip"
rm -f "$EXT_ZIP"
cd "$EXTENSION_DIR"
zip "$EXT_ZIP" manifest.json popup.html popup.js background.js content.js icon-128.png
cd "$ROOT_DIR"

# Step 4: Create combined distribution zip
echo ">>> Creating distribution archive..."
mkdir -p "$DIST_DIR"
rm -f "$DIST_DIR"/*
cp "$EXT_ZIP" "$DIST_DIR/"
if [ -n "$INSTALLER" ]; then
    cp "$INSTALLER" "$DIST_DIR/"
fi

FINAL_ZIP="$ROOT_DIR/password-manager_v$APP_VERSION.zip"
rm -f "$FINAL_ZIP"
cd "$DIST_DIR"
zip "$FINAL_ZIP" ./*
cd "$ROOT_DIR"
rm -rf "$DIST_DIR" "$EXT_ZIP"

echo "=== Done ==="
echo "Output: $FINAL_ZIP"
ls -lh "$FINAL_ZIP"
