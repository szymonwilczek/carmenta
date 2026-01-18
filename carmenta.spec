Name:           carmenta
Version:        0.2.0
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
* Sun Jan 18 2026 Szymon Wilczek <szymonwilczek@github> - 0.2.0-1
- Added GIF support (powered by Klipy)
- Improved search performance

* Fri Jan 16 2026 Szymon Wilczek <szymonwilczek@github> - 0.1.0-1
- Initial release
