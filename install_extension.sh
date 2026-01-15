#!/bin/bash
set -e

UUID="carmenta@szymonwilczek.dev"
EXTENSION_DIR="$HOME/.local/share/gnome-shell/extensions/$UUID"
SOURCE_DIR="$(pwd)/extension"

echo "🔨 Kompilowanie schematów GSettings..."
glib-compile-schemas "$SOURCE_DIR/schemas"

echo "📂 Instalowanie rozszerzenia do $EXTENSION_DIR..."
mkdir -p "$EXTENSION_DIR"
rm -rf "$EXTENSION_DIR"/*
cp -r "$SOURCE_DIR"/* "$EXTENSION_DIR/"

echo "✅ Zainstalowano! Teraz musisz:"
echo "1. Wylogować się i zalogować ponownie (lub zrestartować GNOME Shell na X11 przez Alt+F2 -> r)."
echo "2. Włączyć rozszerzenie poleceniem: gnome-extensions enable $UUID"
echo "3. Uruchomić ponownie aplikację Rust: cargo run"
