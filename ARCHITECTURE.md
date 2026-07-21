# labwc-rs Architecture

## Crate Dependency Graph

```
                         compositor (binary)
                        /    |    |    |    \
              cli  config  input  output  wayland  actions  menus
               |     |       |      |       |        |       |
               +-----+-------+------+-------+--------+-------+
                         |           |           |
                      window ---- focus --- workspace
                        |           |
                  decorations   shell
                        |        |
                      scene   xwayland
                        |
                     backend --- rendering
                        |
                   [wlroots FFI]
                        |
                      core
                        |
                      util
```

## Layer Architecture

1. **Foundation Layer** — `core`, `util`
   - Geometry primitives, enums, error types
   - String helpers, spawn, path resolution
   - Zero or minimal external dependencies

2. **Infrastructure Layer** — `config`, `cli`
   - rc.xml parsing with backward compatibility
   - CLI argument definitions

3. **Platform Abstraction Layer** — `backend`, `rendering`, `scene`
   - DRM/headless backend
   - EGL/GLES/Vulkan rendering
   - Scene graph tree

4. **Protocol Layer** — `wayland`, `shell`, `xwayland`, `ipc`
   - Wayland protocol implementations
   - XDG shell, layer shell, popups
   - XWayland support

5. **Window Management Layer** — `window`, `workspace`, `focus`, `decorations`
   - View lifecycle, geometry, stacking
   - Virtual desktops
   - Input focus tracking
   - Server-side decorations

6. **Interaction Layer** — `input`, `actions`, `menus`, `output`
   - Input device handling
   - Action execution engine
   - Menu system
   - Multi-output management

7. **Presentation Layer** — `theme`, `animation`
   - Theme engine (Openbox compatibility)
   - Animation system

8. **Application Layer** — `compositor`
   - Main binary: CLI → config → init → run

## Key Design Decisions

### Ownership Model
- `Arc<View>` for shared access across modules (scene graph, focus, workspace)
- `parking_lot::Mutex` for interior mutability (better performance than std::sync::Mutex)
- Views are never cloned — always shared via Arc

### Type Safety
- All C enums replaced with Rust enums with exhaustive matching
- `Edge` and `ViewCriteria` use bitflags for composable options
- `Rect`/`Border` are `Copy + Clone` plain-old-data
- No raw pointer exposure outside FFI boundaries

### Error Handling
- `thiserror` for custom error types
- `Result<T, E>` for all fallible operations
- No panics in library code (only `expect` for programmer errors)

### Memory Safety
- `#![deny(unsafe_code)]` on `core`, `config`, `window`, and all pure-Rust crates
- `unsafe` blocks allowed only in `backend`, `rendering`, `wayland`, `xwayland`
- Every `unsafe` block must include a safety comment
