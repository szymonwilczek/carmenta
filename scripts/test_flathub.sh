#!/bin/bash
set -e

APP_ID="io.github.szymonwilczek.carmenta"
MANIFEST="${APP_ID}.json"

echo "🔍 Checking Environment..."
if ! command -v flatpak &> /dev/null; then
    echo "❌ Flatpak is not installed."
    exit 1
fi

# if JSON manifest exists
if [ ! -f "$MANIFEST" ]; then
    echo "⚠️  Manifest '$MANIFEST' not found."
    echo "   Regenerating it now via scripts/merge_manifest.py..."
    python3 scripts/merge_manifest.py
fi

echo "📦 Ensuring org.flatpak.Builder is installed..."

# flathub remote exists
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo || true
flatpak install -y flathub org.flatpak.Builder

echo "🏗️ Building ${APP_ID} using Flatpak Builder..."

# --force-clean to ensure fresh build
flatpak run --command=flathub-build org.flatpak.Builder --install --user build-dir "$MANIFEST" --force-clean

echo "✅ Build Complete."
echo ""
echo "🧐 Running Flatpak Linter..."
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest "$MANIFEST"

echo ""
echo "🚀 You can now run the app with:"
echo "flatpak run $APP_ID"
