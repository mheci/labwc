Name:           labwc-rs
Version:        0.20.1
Release:        1%{?dist}
Summary:        Wayland window-stacking compositor (Rust rewrite of labwc)
License:        GPL-2.0-only
URL:            https://github.com/mheci/labwc
Source0:        https://github.com/mheci/labwc/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo rust gcc cmake make
BuildRequires:  pkgconfig(wayland-server) pkgconfig(cairo) pkgconfig(pango)
BuildRequires:  pkgconfig(libinput) pkgconfig(libxml-2.0) pkgconfig(pixman-1)
BuildRequires:  pkgconfig(xkbcommon) pkgconfig(libpng) pkgconfig(librsvg-2.0)

Requires:       cairo glib2 libinput libxcb pango wayland pixman libpng librsvg2 libxml2 seatd
Provides:       labwc = %{version}-%{release}

%description
labwc-rs is a wlroots-based stacking compositor for Wayland rewritten in Rust.
22 crates, 7 Wayland protocol implementations, 1.3 MB stripped binary.

%prep
%autosetup -n labwc-%{version}

%build
cargo build --release --locked

%install
install -Dm755 target/release/labwc-rs %{buildroot}%{_bindir}/labwc-rs
install -Dm644 LICENSE %{buildroot}%{_datadir}/licenses/labwc-rs/LICENSE

%files
%license LICENSE
%{_bindir}/labwc-rs

%changelog
* Mon Jul 21 2026 labwc-rs contributors - 0.20.1-1
- Initial stable release
- 22-crate Cargo workspace
- 7 Wayland protocol implementations
- CI: Arch Linux + Fedora + Ubuntu
- 1.3 MB stripped release binary
