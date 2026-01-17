#!/bin/bash
set -e

function install_dependencies() {
    echo "🔍 Checking for dependencies..."
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case $ID in
            ubuntu|debian|pop|linuxmint)
                echo "📦 Detected $NAME. Installing dependencies (requires sudo)..."
                sudo apt update
                sudo apt install -y libgtk-4-dev libadwaita-1-dev cargo
                ;;
            fedora|rhel|centos)
                echo "📦 Detected $NAME. Installing dependencies (requires sudo)..."
                sudo dnf install -y gtk4-devel libadwaita-devel cargo
                ;;
            arch|manjaro)
                echo "📦 Detected $NAME. Installing dependencies (requires sudo)..."
                sudo pacman -S --noconfirm gtk4 libadwaita rust
                ;;
            *)
                echo "⚠️  Unknown distribution '$ID'. Please ensure you have the following installed:"
                echo "   - gtk4 (development headers)"
                echo "   - libadwaita (development headers)"
                echo "   - rust / cargo"
                read -p "Press Enter to continue..."
                ;;
        esac
    else
        echo "⚠️  Cannot detect OS. Please ensure dependencies are installed manually."
    fi
}

install_dependencies

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
cp data/io.github.szymonwilczek.carmenta.desktop ~/.local/share/applications/
cp data/io.github.szymonwilczek.carmenta.svg ~/.local/share/icons/hicolor/scalable/apps/

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

echo "=== IF YOU'VE INSTALLED EXTENSION ==="
echo "2. Restart GNOME Shell (Log out and log back in, or Alt+F2 -> r on X11)."
echo "3. Enable 'Carmenta' in the 'Extensions' app."
echo "====================================="
echo "4. Configure the Custom Shortcut using run as \"carmenta\" and you are good to go!"
echo ""
echo "---------------------------------"
