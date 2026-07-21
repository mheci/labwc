//! Implementation of `wp_content_type_manager_v1`.
//!
//! Allows clients to hint the content type of their surfaces
//! (e.g., game, video) so the compositor can optimize presentation.

use tracing::{debug, info};

/// Content type values as defined in the protocol.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContentType {
    /// No specific content type hint.
    #[default]
    None = 0,
    /// Game content — compositor may optimize for low latency.
    Game = 1,
    /// Video content — compositor may optimize for smooth playback.
    Video = 2,
}

/// State for a content type surface.
#[derive(Debug, Default)]
pub struct ContentTypeState {
    /// Current content type for the surface.
    pub content_type: ContentType,
}

impl ContentTypeState {
    /// Create a new content type state with default (None) type.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content type for the associated surface.
    pub fn set_content_type(&mut self, ct: ContentType) {
        debug!(content_type = ?ct, "Surface content type updated");
        self.content_type = ct;
    }
}

/// Register the `wp_content_type_manager_v1` global.
///
/// In the full compositor, this would call
/// `display.handle().create_global()` with the proper dispatch handlers.
pub fn register() {
    info!("Registered wp_content_type_manager_v1");
}
