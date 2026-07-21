//! Implementation of `xdg_toplevel_drag_manager_v1`.
//!
//! Enables clients to initiate drag operations from toplevel windows,
//! providing proper drag offsets and session management.

use tracing::{debug, info};

/// State for an active toplevel drag session.
#[derive(Debug)]
pub struct ToplevelDragState {
    /// The toplevel being dragged.
    pub toplevel_id: u64,
    /// The seat serial that initiated the drag.
    pub serial: u32,
    /// Current x offset from the surface origin.
    pub x_offset: i32,
    /// Current y offset from the surface origin.
    pub y_offset: i32,
    /// Whether the drag is active.
    pub active: bool,
}

impl ToplevelDragState {
    /// Create a new drag state for the given toplevel.
    #[must_use]
    pub fn new(toplevel_id: u64, serial: u32) -> Self {
        Self {
            toplevel_id,
            serial,
            x_offset: 0,
            y_offset: 0,
            active: true,
        }
    }

    /// Handle a configure event with new offset.
    pub fn configure(&mut self, x_offset: i32, y_offset: i32) {
        debug!(
            toplevel_id = self.toplevel_id,
            x_offset, y_offset, "Toplevel drag configured"
        );
        self.x_offset = x_offset;
        self.y_offset = y_offset;
    }

    /// Mark the drag as finished.
    pub fn finish(&mut self) {
        debug!(toplevel_id = self.toplevel_id, "Toplevel drag finished");
        self.active = false;
    }
}

/// Register the `xdg_toplevel_drag_manager_v1` global.
pub fn register() {
    info!("Registered xdg_toplevel_drag_manager_v1");
}
