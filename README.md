# Carmenta

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Rust](https://img.shields.io/badge/language-Rust-orange.svg) ![GTK4](https://img.shields.io/badge/toolkit-GTK4%20%2B%20Adwaita-green.svg)

**Carmenta** is blazingly fast emoji picker for Linux desktops, built with Rust and GTK4. It integrates seamlessly with GNOME Shell to provide instant access to Emojis, Kaomojis, and mathematical symbols.

## 🚀 Performance
Carmenta is engineered for speed and efficiency.

| Metric | Result |
| :--- | :--- |
| **Startup Time** | **< 200ms** (Internal init: ~135ms) |
| **Insertion Latency** | **~1.2ms** |
| **Memory Usage** | **~105MB** (RSS) |

*Measured on standard hardware.*

## ✨ Features
- **Instant Search**: Localized, debounce-optimized search for thousands of items.
- **Three Modes**:
  - 😃 **Emoji**: Full Unicode support with categories and skin tones.
  - (◕‿◕) **Kaomoji**: Extensive library of Japanese emoticons.
  - ∑ **Symbols**: Math, currency, arrows, and more.
- **Smart History**: Remembers your most used items.
- **"Always on Top"**: Stays visible while you work, but gets out of the way when you don't need it.
- **Seamless Integration**: Uses an optional, companion GNOME Shell extension for reliable text insertion into any application.

## 📦 Installation

### Fedora (Recommended)
You can install Carmenta directly from the [COPR repository](https://copr.fedorainfracloud.org/coprs/szymonwilczek/carmenta/):

```bash
sudo dnf copr enable szymonwilczek/carmenta
sudo dnf install carmenta
```

### Manual Build
If you are not using Fedora or prefer to build from source:

1.  Clone the repository:
    ```bash
    git clone https://github.com/szymonwilczek/carmenta.git
    cd carmenta
    ```
2.  Run the installation script:
    ```bash
    ./install.sh
    ```

### Install Extension (Optional)
Carmenta does not require a companion extension to function correctly, but it makes the work much easier. Currently, Wayland prohibits inserting anything from other applications into other windows. A workaround for this is a Companion extension that communicates with the application, allowing emoticons to be inserted.

1. Copy the `extension` folder to your GNOME Shell extensions directory:
```bash
mkdir -p ~/.local/share/gnome-shell/extensions/carmenta@szymonwilczek.dev
cp -r extension/* ~/.local/share/gnome-shell/extensions/carmenta@szymonwilczek.dev/
```
2. Restart GNOME Shell (logout and login back).
3. Enable the extension using the **Extensions** app.

## ⌨️ Usage
- Launch Carmenta.
- Type to search (or use Arrows and/or Tab/Ctrl-Tab to navigate around the app).
- Click to copy & insert.
- **Esc** to quit instantly.
