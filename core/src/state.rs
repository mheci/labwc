//! Top-level compositor state shared across all protocol handlers.
//!
//! This is the user-data type attached to the Wayland display and
//! passed to every protocol dispatch callback.

use crate::cursor::CursorState;
use crate::types::InputMode;

/// Global compositor state.
///
/// This struct is wrapped in an `Arc<Mutex<…>>` by `wayland-server` and
/// passed to every protocol callback. It holds references to all subsystems.
#[derive(Debug)]
pub struct CompositorState {
    /// Current cursor position and shape.
    pub cursor: CursorState,
    /// Current input mode (passthrough, move, resize, menu, cycle).
    pub input_mode: InputMode,
    /// Whether the compositor is shutting down.
    pub shutting_down: bool,
}

impl CompositorState {
    /// Create a new compositor state with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cursor: CursorState::new(),
            input_mode: InputMode::Passthrough,
            shutting_down: false,
        }
    }

    /// Signal that the compositor should shut down.
    pub fn shutdown(&mut self) {
        self.shutting_down = true;
    }
}

impl Default for CompositorState {
    fn default() -> Self {
        Self::new()
    }
}
