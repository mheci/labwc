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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_manager_new() {
        let mgr = InputManager::new();
        assert_eq!(mgr.input_mode, InputMode::Passthrough);
        assert_eq!(mgr.cursor.x, 0.0);
        assert_eq!(mgr.cursor.y, 0.0);
    }

    #[test]
    fn test_cursor_motion() {
        let mut mgr = InputManager::new();
        mgr.handle_pointer_motion(10.0, 20.0);
        assert_eq!(mgr.cursor.x, 10.0);
        assert_eq!(mgr.cursor.y, 20.0);
    }

    #[test]
    fn test_key_press() {
        let mut mgr = InputManager::new();
        assert!(mgr.handle_key_press(65));
        mgr.handle_key_release(65);
        assert!(!mgr.key_state[65]);
    }

    #[test]
    fn test_interactive_mode() {
        let mut mgr = InputManager::new();
        mgr.begin_interactive(InputMode::Move);
        assert_eq!(mgr.input_mode, InputMode::Move);
        mgr.end_interactive();
        assert_eq!(mgr.input_mode, InputMode::Passthrough);
    }
}
