Name:           labwc-rs
Version:        0.25.0
Release:        1%{?dist}
Summary:        Wayland window-stacking compositor (Rust rewrite of labwc)
License:        GPL-2.0-only
URL:            https://github.com/mheci/labwc
Source0:        https://github.com/mheci/labwc/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo rust gcc cmake make pkg-config
BuildRequires:  pkgconfig(wayland-server) pkgconfig(cairo) pkgconfig(glib-2.0)
BuildRequires:  pkgconfig(libinput) pkgconfig(libxml-2.0) pkgconfig(pango)
BuildRequires:  pkgconfig(pixman-1) pkgconfig(xkbcommon) pkgconfig(libpng)
BuildRequires:  pkgconfig(librsvg-2.0)

Requires:       cairo glib2 libinput libxcb pango wayland pixman libpng librsvg2 libxml2 seatd
Provides:       labwc = %{version}-%{release}

%description
labwc-rs is a wlroots-based stacking compositor for Wayland rewritten in Rust.
27 crates, 8,100 lines, Vulkan-accelerated, NVIDIA-optimized.

%prep
%autosetup -n labwc-%{version}

%build
cargo build --release --locked -p labwc-rs

%check
cargo test --release --locked

%install
install -Dm755 target/release/labwc-rs %{buildroot}%{_bindir}/labwc-rs
install -Dm644 LICENSE %{buildroot}%{_datadir}/licenses/labwc-rs/LICENSE

%files
%license LICENSE
%{_bindir}/labwc-rs

%changelog
* Tue Jul 22 2026 labwc-rs contributors - 0.24.1-1
- 27 crates, Vulkan multi-head, EDID/HDR, XDG autostart, panel
- NixOS flake support
