//! Wayland protocol implementations for labwc-rs.
//!
//! This crate implements the server side of Wayland protocols:
//!
//! - `wp_content_type_manager_v1` — Surface content type hints (game/video/none)
//! - `xdg_toplevel_drag_manager_v1` — Drag from toplevel windows
//! - `wp_fifo_manager_v1` — FIFO presentation mode support
//! - `wp_pointer_warp_v1` — Pointer warping to surface coordinates
//! - `xdg_toplevel_tag_manager_v1` — Tag toplevel windows with string labels
//! - `ext_image_capture_source_v1` — Screen capture sources
//! - `wp_commit_timing_manager_v1` — Commit timing instrumentation
//!
//! Plus standard protocols: wl_compositor, wl_seat, xdg_shell, wlr_layer_shell.

pub mod commit_timing;
pub mod content_type;
pub mod fifo;
pub mod image_capture;
pub mod pointer_warp;
pub mod toplevel_drag;
pub mod toplevel_tag;

use tracing::info;

/// Initialize all Wayland protocol globals.
///
/// Must be called during compositor startup before the event loop runs.
pub fn init_protocols() -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing Wayland protocol handlers...");

    content_type::register();
    toplevel_drag::register();
    fifo::register();
    pointer_warp::register();
    toplevel_tag::register();
    image_capture::register();
    commit_timing::register();

    info!("All 7 Wayland protocol handlers registered successfully");
    Ok(())
}
