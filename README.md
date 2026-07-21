# labwc-rs

A complete **Rust rewrite** of [labwc](https://github.com/labwc/labwc) — a wlroots-based
Wayland window-stacking compositor inspired by Openbox.

## Architecture

Cargo workspace of 22 crates:

| Crate | Purpose |
|-------|---------|
| `compositor` | Main binary and event loop |
| `core` | Foundational types (geometry, enums, errors) |
| `config` | rc.xml parser with backward compatibility |
| `input` | Keyboard, pointer, touch, tablet input |
| `output` | Multi-output layout management |
| `scene` | Scene graph abstraction |
| `window` | View geometry, state, stacking |
| `workspace` | Virtual desktop workspaces |
| `focus` | Focus management (click-to-focus, focus-follows-mouse) |
| `decorations` | Server-side decorations (SSD) |
| `shell` | XDG shell protocol handler |
| `actions` | 40+ compositor actions |
| `menus` | Menu system |
| `theme` | Theme engine (Openbox compatibility) |
| `wayland` | Wayland protocol glue |
| `backend` | Backend abstraction (DRM/headless) |
| `rendering` | EGL/GLES/Vulkan rendering |
| `xwayland` | XWayland support |
| `ipc` | IPC communication |
| `animation` | Animation system |
| `util` | Common utilities |
| `cli` | CLI argument parsing |

## Building

```bash
# Build everything
cargo build --release

# Target binary
./target/release/labwc-rs
```

### Arch Linux

```bash
cd packaging/arch && makepkg -si
```

### Fedora

```bash
rpmbuild -bb packaging/fedora/labwc-rs.spec
```

## License

GPL-2.0-only
