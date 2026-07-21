//! Input handling — keyboard, pointer, touch, tablet, gestures.

use labwc_core::{CursorState, InputMode};
use tracing::debug;

pub struct InputManager {
    pub cursor: CursorState,
    pub input_mode: InputMode,
    pub key_state: Vec<bool>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            cursor: CursorState::new(),
            input_mode: InputMode::Passthrough,
            key_state: vec![false; 256],
        }
    }

    pub fn handle_key_press(&mut self, keycode: u32) -> bool {
        self.key_state[keycode as usize % 256] = true;
        debug!("Key press: {}", keycode);
        true
    }

    pub fn handle_key_release(&mut self, keycode: u32) {
        self.key_state[keycode as usize % 256] = false;
    }

    pub fn handle_pointer_motion(&mut self, dx: f64, dy: f64) {
        self.cursor.move_to(self.cursor.x + dx, self.cursor.y + dy);
    }

    pub fn begin_interactive(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }
    pub fn end_interactive(&mut self) {
        self.input_mode = InputMode::Passthrough;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
