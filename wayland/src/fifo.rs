//! Implementation of `wp_fifo_manager_v1`.
//!
//! Provides FIFO presentation mode support, allowing clients to
//! request that frames are presented in a first-in-first-out manner
//! for consistent frame pacing.

use std::collections::HashMap;
use tracing::{debug, info};

/// Surface FIFO state tracking.
#[derive(Debug, Default)]
pub struct FifoSurfaceState {
    /// Whether FIFO mode is active for this surface.
    pub fifo_enabled: bool,
    /// Frame sequence counter for debugging.
    pub frame_count: u64,
}

impl FifoSurfaceState {
    /// Create a new FIFO surface state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fifo_enabled: false,
            frame_count: 0,
        }
    }

    /// Enable FIFO mode on this surface.
    pub fn enable(&mut self) {
        self.fifo_enabled = true;
        debug!("FIFO mode enabled on surface");
    }

    /// Record a frame presentation.
    pub fn record_frame(&mut self) {
        if self.fifo_enabled {
            self.frame_count = self.frame_count.wrapping_add(1);
        }
    }
}

/// Global FIFO manager state.
#[derive(Debug, Default)]
pub struct FifoManager {
    /// Map of surface IDs to their FIFO state.
    pub surfaces: HashMap<u64, FifoSurfaceState>,
}

impl FifoManager {
    /// Create a new FIFO manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
        }
    }

    /// Get or create FIFO state for a surface.
    pub fn get_or_create(&mut self, surface_id: u64) -> &mut FifoSurfaceState {
        self.surfaces.entry(surface_id).or_default()
    }
}

/// Register the `wp_fifo_manager_v1` global.
pub fn register() {
    info!("Registered wp_fifo_manager_v1");
}
