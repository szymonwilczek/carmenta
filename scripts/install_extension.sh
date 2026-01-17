#!/bin/bash
set -e

SOURCE_DIR="$(pwd)/extension"

UUID=$(grep -oP '"uuid":\s*"\K[^"]+' "$SOURCE_DIR/metadata.json")
if [ -z "$UUID" ]; then
  echo "❌ Could NOT find UUID inside of metadata.json - report that Issue immediately!"
  exit 1
fi
EXTENSION_DIR="$HOME/.local/share/gnome-shell/extensions/$UUID"

echo "🔨 Compiling GSettings schemas..."
glib-compile-schemas "$SOURCE_DIR/schemas"

echo "📂 Installing extension to $EXTENSION_DIR..."
mkdir -p "$EXTENSION_DIR"
rm -rf "$EXTENSION_DIR"/*
cp -r "$SOURCE_DIR"/* "$EXTENSION_DIR/"

echo "✅ Installed successfully! Now you need to:"
echo "1. Log out and log in."
echo "2. Enable the extension via gnome-extension app or turn it on using: gnome-extensions enable $UUID"
echo "And you are good to go! Thanks for using my project!"
