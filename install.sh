#!/bin/bash
set -e

echo "🚀 Building Carmenta (Release)..."
cargo build --release

echo "📂 Creating directories..."
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
EXT_UUID="carmenta@szymonwilczek.dev"
EXT_DIR="$HOME/.local/share/gnome-shell/extensions/$EXT_UUID"
mkdir -p "$EXT_DIR"

echo "📦 Installing Binary..."
cp target/release/carmenta ~/.local/bin/
chmod +x ~/.local/bin/carmenta

echo "🖥️ Installing Desktop File & Icon..."
cp data/org.carmenta.App.desktop ~/.local/share/applications/
cp data/org.carmenta.App.svg ~/.local/share/icons/hicolor/scalable/apps/

echo ""
echo "🧩 GNOME Shell Extension"
echo "The Carmenta extension allows:"
echo " - Inserting emojis directly into other windows (focus switching)"
echo " - Pinning the window 'Always on Top'"
echo "Without it, emojis will only be copied to the clipboard."
read -p "Do you want to install the GNOME Shell Extension? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Installing extension..."
    cp -r extension/* "$EXT_DIR/"
    glib-compile-schemas "$EXT_DIR/schemas"
    echo "Extension installed."
else
    echo "Skipping extension installation."
fi

echo "✅ Installation Complete!"
echo ""
echo "---------------------------------"
echo "👉 Next Steps:"
echo "1. Ensure '$HOME/.local/bin' is in your PATH."
echo "2. Restart GNOME Shell (Log out and log back in, or Alt+F2 -> r on X11)."
echo "3. Enable 'Carmenta' in the 'Extensions' app."
echo "4. Press Super+. (Meta+Period) to launch!"
echo ""
echo "---------------------------------"
