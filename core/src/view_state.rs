//! View state tracking: size hints, placement info, icon state.

use super::geometry::Rect;

/// Size constraints reported by a client (ICCCM §4.1.2.3).
///
/// These are negotiated via the XDG shell or X11 WM_NORMAL_HINTS protocol.
#[derive(Debug, Clone, Copy, Default)]
pub struct SizeHints {
    /// Minimum width the client can accept.
    pub min_width: i32,
    /// Minimum height the client can accept.
    pub min_height: i32,
    /// Base width for size-increment calculations.
    pub base_width: i32,
    /// Base height for size-increment calculations.
    pub base_height: i32,
    /// Width increment (e.g. terminal cell width).
    pub width_inc: i32,
    /// Height increment (e.g. terminal cell height).
    pub height_inc: i32,
    /// Maximum width the client can accept.
    pub max_width: i32,
    /// Maximum height the client can accept.
    pub max_height: i32,
}

impl SizeHints {
    /// Compute effective minimum dimensions, falling back to defaults.
    #[must_use]
    pub fn effective_min_size(&self) -> (i32, i32) {
        let w = if self.min_width < 1 {
            100
        } else {
            self.min_width
        };
        let h = if self.min_height < 1 {
            60
        } else {
            self.min_height
        };
        (w, h)
    }

    /// Snap width/height to the client's size increments.
    ///
    /// This ensures terminal windows resize to whole character cells.
    #[must_use]
    pub fn snap_to_increments(&self, w: i32, h: i32) -> (i32, i32) {
        let sw = if self.width_inc > 0 {
            self.base_width
                + ((w - self.base_width + self.width_inc / 2) / self.width_inc) * self.width_inc
        } else {
            w
        };
        let sh = if self.height_inc > 0 {
            self.base_height
                + ((h - self.base_height + self.height_inc / 2) / self.height_inc) * self.height_inc
        } else {
            h
        };
        (sw, sh)
    }
}

/// Saved placement information for restoring view positions after output layout changes.
#[derive(Debug, Clone, Default)]
pub struct LastPlacement {
    /// Name of the output the view was on.
    pub output_name: Option<String>,
    /// Geometry in absolute layout coordinates.
    pub layout_geometry: Rect,
    /// Geometry relative to the output origin.
    pub relative_geometry: Rect,
}

/// Icon state for a view — name and optional cached surfaces.
#[derive(Debug, Clone, Default)]
pub struct IconState {
    /// Freedesktop icon name.
    pub name: Option<String>,
}
