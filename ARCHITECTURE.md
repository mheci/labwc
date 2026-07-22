# Architecture

labwc-rs is a 27‑crate Cargo workspace with zero circular dependencies.

## Crate map

```
compositor → config, input, output, wayland, shell, actions, xdg-desktop,
             core, window, workspace, focus, decorations, scene,
             backend, rendering, panel, menus, animation,
             ipc, config-watcher, theme, control-center, util, cli
```

## Layer structure

| Layer | Crates | Role |
|-------|--------|------|
| Foundation | `core`, `util`, `cli` | Geometry, errors, process spawn |
| Backend | `backend`, `rendering`, `scene` | GPU, DRM, Vulkan |
| Wayland | `wayland`, `shell`, `xwayland`, `ipc` | Protocols, IPC |
| Window | `window`, `workspace`, `focus`, `decorations` | Window management |
| Interaction | `input`, `actions`, `menus`, `output` | Input, actions, displays |
| Desktop | `xdg-desktop`, `panel`, `theme` | XDG specs, panel, theming |
| Tools | `control-center`, `config-watcher`, `config` | GUI, live reload, config |
| Visual | `animation` | Easing engine |

All crates compile in under 60 seconds from scratch.  The binary strips to
1.4 MB.
