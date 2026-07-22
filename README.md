<p align="center">
  <strong>labwc-rs</strong><br>
  a modern Wayland window‑stacking compositor written in Rust<br>
  Vulkan‑accelerated · NVIDIA‑safe · XDG‑native
</p>

<p align="center">
  <a href="https://github.com/mheci/labwc/actions"><img src="https://img.shields.io/github/actions/workflow/status/mheci/labwc/ci.yml?branch=master&style=flat-square" alt="CI"></a>
  <a href="https://github.com/mheci/labwc/releases/latest"><img src="https://img.shields.io/github/v/release/mheci/labwc?style=flat-square" alt="release"></a>
  <img src="https://img.shields.io/badge/rust-stable%201.97+-orange?style=flat-square" alt="rust">
  <img src="https://img.shields.io/badge/license-GPL--2.0-blue?style=flat-square" alt="license">
</p>

---

## Quick Start

```bash
git clone https://github.com/mheci/labwc.git
cd labwc
cargo build --release
./target/release/labwc-rs
```

**System dependencies:** `rust` `cargo` `wayland` `libxkbcommon` `libinput` `cairo` `pango` `pixman` `libpng` `librsvg` `libxml2` `glib2` `libglvnd` `mesa`

## Install

### Arch Linux

```bash
cd packaging/arch && makepkg -si
```

### Fedora

```bash
rpmbuild -bb packaging/fedora/labwc-rs.spec
sudo dnf install ~/rpmbuild/RPMS/x86_64/labwc-rs-*.rpm
```

### NixOS / Nix

```bash
nix run github:mheci/labwc

# or install permanently
nix profile install github:mheci/labwc
```

### Binary download

```bash
curl -LO https://github.com/mheci/labwc/releases/latest/download/labwc-rs-x86_64
chmod +x labwc-rs-x86_64
./labwc-rs-x86_64
```

---

## Configuration

Configuration files live in `~/.config/labwc/`.  Changes are picked up
automatically — no restart needed.

| File | Purpose |
|------|---------|
| `rc.xml` | Compositor settings (placement, focus, snapping, keybinds) |
| `menu.xml` | Menu structure |
| `themerc` | Theme (Openbox‑compatible) |

---

## Features

- **Vulkan multi‑head** — independent refresh rate per display, VRR, HDR10
- **NVIDIA optimized** — explicit sync, G‑Sync detection, no implicit‑sync lockups
- **Session detection** — auto‑detects nested / direct DRM / TTY mode
- **XDG desktop** — `.desktop` parser, autostart, application launcher
- **Control Center** — 12‑tab native GUI for runtime configuration
- **Desktop panel** — launcher, taskbar, system tray, clock, audio, power
- **Live config reload** — inotify watcher, no SIGHUP needed
- **7 Wayland protocol extensions**
- **27‑crate workspace** — 1.4 MB stripped binary, 93 tests, 0 warnings

---

## License

GPL-2.0-only
