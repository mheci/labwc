//! Cursor state and context tracking.

use super::types::CursorShape;

/// Saved cursor context used during button-press tracking and drag operations.
///
/// Captures the state when a mouse button is pressed so that cursor motion
/// can be forwarded to the correct surface even after the cursor leaves its bounds.
#[derive(Debug, Clone, Default)]
pub struct CursorContext {
    /// ID of the view under the cursor, if any.
    pub view_id: Option<u64>,
    /// ID of the surface under the cursor, if any.
    pub surface_id: Option<u64>,
    /// Surface-relative x coordinate at press time.
    pub sx: f64,
    /// Surface-relative y coordinate at press time.
    pub sy: f64,
    /// Active resize edges during a resize operation.
    pub resize_edges: u32,
}

/// Accumulated scroll state for high-resolution scroll wheels.
///
/// libinput provides both pixel-precise and discrete ("click") scroll values.
/// This struct accumulates pixel deltas and dispatches discrete events when
/// enough has accumulated.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccumulatedScroll {
    /// Accumulated pixel delta.
    pub delta: f64,
    /// Accumulated discrete delta in clicks.
    pub delta_discrete: f64,
}

/// Current cursor state — position, shape, visibility, and accumulated input.
#[derive(Debug, Clone)]
pub struct CursorState {
    /// Absolute x position in layout coordinates.
    pub x: f64,
    /// Absolute y position in layout coordinates.
    pub y: f64,
    /// Active server-side cursor shape.
    pub shape: CursorShape,
    /// Whether the cursor image is visible.
    pub visible: bool,
    /// Whether scroll-wheel emulation (via trackpoint) is active.
    pub scroll_wheel_emulation: bool,
    /// Context saved on button press.
    pub pressed_context: CursorContext,
    /// Context from the most recent motion event.
    pub last_motion_context: CursorContext,
    /// Accumulated scroll deltas per axis.
    pub scroll_accumulators: [AccumulatedScroll; 2],
}

impl CursorState {
    /// Create a new cursor state at the origin with default pointer shape.
    #[must_use]
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            shape: CursorShape::Default,
            visible: true,
            scroll_wheel_emulation: false,
            pressed_context: CursorContext::default(),
            last_motion_context: CursorContext::default(),
            scroll_accumulators: [AccumulatedScroll::default(); 2],
        }
    }

    /// Move the cursor to an absolute position in layout coordinates.
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    /// Set the cursor shape (pointer, resize, move, etc.).
    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}
