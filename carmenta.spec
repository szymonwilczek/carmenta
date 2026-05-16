Name:           carmenta
Version:        0.3.1
Release:        1%{?dist}
Summary:        A GTK4 Emoji Picker for GNOME

License:        MIT
URL:            https://github.com/szymonwilczek/carmenta
Source0:        %{url}/archive/refs/heads/main.tar.gz

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  gtk4-devel
BuildRequires:  libadwaita-devel
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib
BuildRequires:  gdk-pixbuf2-devel

Requires:       gtk4
Requires:       libadwaita

%description
Carmenta is an emoji picker written in Rust using GTK4 and Libadwaita.
It allows searching for Emojis, Kaomojis, and various symbols.

%prep
%autosetup -n %{name}-main

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
install -D -m 755 target/release/carmenta %{buildroot}%{_bindir}/carmenta
install -D -m 644 data/io.github.szymonwilczek.carmenta.desktop %{buildroot}%{_datadir}/applications/io.github.szymonwilczek.carmenta.desktop
install -D -m 644 data/io.github.szymonwilczek.carmenta.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.szymonwilczek.carmenta.svg
install -D -m 644 data/io.github.szymonwilczek.carmenta.metainfo.xml %{buildroot}%{_metainfodir}/io.github.szymonwilczek.carmenta.metainfo.xml

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/*.desktop
appstream-util validate-relax --nonet %{buildroot}%{_metainfodir}/*.metainfo.xml

%files
%{_bindir}/carmenta
%{_datadir}/applications/*.desktop
%{_datadir}/icons/hicolor/scalable/apps/*.svg
%{_metainfodir}/*.metainfo.xml

%changelog
* Sat May 17 2026 Szymon Wilczek <szymonwilczek@github> - 0.3.1-1
- Fix segfault caused by GtkMediaFile race condition in GIF grid
- Fix file descriptor leak by switching to gdk-pixbuf animation
- Use global reqwest::Client to eliminate socket exhaustion
- Add gdk-pixbuf dependency for GIF preview rendering

* Sat Feb 21 2026 Szymon Wilczek <szymonwilczek@github> - 0.3.0-1
- Add CLI runtime config (window width/height bounds, optional GIF tab)
- Improve shutdown stability on Escape and focus loss
- Fix CLI args handling to avoid GTK re-parsing clap flags
- Clean up GIF module compile warnings

* Sun Jan 18 2026 Szymon Wilczek <szymonwilczek@github> - 0.2.0-1
- Added GIF support (powered by Klipy)
- Improved search performance

* Fri Jan 16 2026 Szymon Wilczek <szymonwilczek@github> - 0.1.0-1
- Initial release
