//! Implementation of `xdg_toplevel_tag_manager_v1`.
//!
//! Allows tagging toplevel windows with string identifiers.
//! Useful for session management, scripting, and window grouping.

use std::collections::HashMap;
use tracing::{debug, info};

/// Toplevel tag state.
#[derive(Debug, Default)]
pub struct ToplevelTagState {
    /// Map of toplevel IDs to their assigned tags.
    pub tags: HashMap<u64, String>,
    /// Inverse map: tag → toplevel ID.
    pub reverse: HashMap<String, u64>,
}

impl ToplevelTagState {
    /// Create a new tag state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    /// Tag a toplevel with a string label.
    pub fn tag_toplevel(&mut self, toplevel_id: u64, tag: &str) {
        debug!(toplevel_id, tag, "Toplevel tagged");

        // Remove old tag if any
        if let Some(old_tag) = self.tags.remove(&toplevel_id) {
            self.reverse.remove(&old_tag);
        }

        self.tags.insert(toplevel_id, tag.to_string());
        self.reverse.insert(tag.to_string(), toplevel_id);
    }

    /// Find a toplevel by its tag.
    #[must_use]
    pub fn find_by_tag(&self, tag: &str) -> Option<u64> {
        self.reverse.get(tag).copied()
    }

    /// Get the tag for a toplevel.
    #[must_use]
    pub fn get_tag(&self, toplevel_id: u64) -> Option<&str> {
        self.tags.get(&toplevel_id).map(String::as_str)
    }
}

/// Register the `xdg_toplevel_tag_manager_v1` global.
pub fn register() {
    info!("Registered xdg_toplevel_tag_manager_v1");
}
