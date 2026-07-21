//! Implementation of `wp_pointer_warp_v1`.
//!
//! Allows clients to request the compositor to warp (teleport) the pointer
//! to a specific position on a surface. This is useful for accessibility
//! and input redirection.

use tracing::{debug, info, warn};

/// Pointer warp request.
#[derive(Debug, Clone)]
pub struct PointerWarpRequest {
    /// Target surface ID.
    pub surface_id: u64,
    /// Target x coordinate in surface-local space (24.8 fixed point).
    pub x: f64,
    /// Target y coordinate in surface-local space (24.8 fixed point).
    pub y: f64,
}

/// Pointer warp state.
#[derive(Debug, Default)]
pub struct PointerWarpState {
    /// Whether warping is currently allowed.
    pub enabled: bool,
    /// Most recent warp request, if any.
    pub last_request: Option<PointerWarpRequest>,
}

impl PointerWarpState {
    /// Create new pointer warp state with warping enabled by default.
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled: true,
            last_request: None,
        }
    }

    /// Handle a warp request from a client.
    pub fn warp_to(&mut self, surface_id: u64, x: f64, y: f64) -> bool {
        if !self.enabled {
            warn!("Pointer warp rejected — warping disabled");
            return false;
        }

        debug!(surface_id, x, y, "Pointer warped to surface");
        self.last_request = Some(PointerWarpRequest { surface_id, x, y });
        true
    }
}

/// Register the `wp_pointer_warp_manager_v1` global.
pub fn register() {
    info!("Registered wp_pointer_warp_manager_v1");
}
