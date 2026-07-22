<p align="center">
  <strong>labwc-rs</strong> — a Wayland window-stacking compositor<br>
  Rust reimplementation of <a href="https://github.com/labwc/labwc">labwc</a> ·
  Openbox-inspired · Vulkan · NVIDIA-safe
</p>

<p align="center">
  <a href="https://github.com/mheci/labwc/actions"><img src="https://img.shields.io/github/actions/workflow/status/mheci/labwc/ci.yml?branch=master&label=CI&style=flat-square" alt="CI"></a>
  <a href="https://github.com/mheci/labwc/releases/latest"><img src="https://img.shields.io/github/v/release/mheci/labwc?style=flat-square" alt="Release"></a>
  <img src="https://img.shields.io/badge/rust-stable%201.97+-orange?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/license-GPL--2.0-blue?style=flat-square" alt="License">
</p>

---

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
# or install permanently:
nix profile install github:mheci/labwc
```

### Binary download

```bash
curl -LO https://github.com/mheci/labwc/releases/latest/download/labwc-rs-x86_64
chmod +x labwc-rs-x86_64
./labwc-rs-x86_64
```

### Build from source

```bash
git clone https://github.com/mheci/labwc.git
cd labwc
cargo build --release
./target/release/labwc-rs
```

**Dependencies:** `rust` `cargo` `wayland` `libxkbcommon` `libinput` `cairo` `pango` `pixman` `libpng` `librsvg` `libxml2` `glib2` `libglvnd` `mesa`

---

## Configure

labwc-rs is compatible with labwc configuration files. Changes are **live-reloaded via inotify** — no restart needed.

| File | Purpose |
|---|---|
| `~/.config/labwc/rc.xml` | Compositor (placement, focus, snaps, keybinds) |
| `~/.config/labwc/menu.xml` | Menu definitions |
| `~/.config/labwc/themerc` | Openbox-compatible theme |

---

## Key features

- **Vulkan multi-head** with independent per-output refresh rates and VRR
- **NVIDIA optimized** — explicit sync, G-Sync detection, no implicit-sync lockups
- **EDID / HDR** — native EDID parser with HDR10 detection
- **XDG native** — `.desktop` entry parser, autostart, app launcher
- **Control Center** — 12-tab egui GUI for runtime configuration management
- **Integrated panel** — launcher, taskbar, system tray, clock, audio, power
- **Config watcher** — inotify-based live reload for all config files
- **7 Wayland protocol extensions**

---

## License

GPL-2.0-only
