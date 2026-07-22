//! Directional edge types for snapping, tiling, and window movement.
//!
//! The `Edge` type represents one or more screen edges using bitflags,
//! matching the semantics of the original C `enum lab_edge`.

use bitflags::bitflags;

bitflags! {
    /// Directional edges used for snapping, tiling, and move-to-edge.
    ///
    /// Edges can be combined: `Edge::Top | Edge::Left` represents the top-left corner.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Edge: u32 {
        /// No edge.
        const NONE = 0;
        /// Top edge of the screen.
        const TOP = 1 << 0;
        /// Bottom edge of the screen.
        const BOTTOM = 1 << 1;
        /// Left edge of the screen.
        const LEFT = 1 << 2;
        /// Right edge of the screen.
        const RIGHT = 1 << 3;
        /// Top-left corner (TOP | LEFT).
        const TOP_LEFT = 1 << 4;
        /// Top-right corner (TOP | RIGHT).
        const TOP_RIGHT = 1 << 5;
        /// Bottom-left corner (BOTTOM | LEFT).
        const BOTTOM_LEFT = 1 << 6;
        /// Bottom-right corner (BOTTOM | RIGHT).
        const BOTTOM_RIGHT = 1 << 7;
        /// Center position (for tiling).
        const CENTER = 1 << 8;
    }
}

impl Edge {
    /// All four cardinal edges.
    pub const CARDINAL: Edge = Edge::TOP
        .union(Edge::BOTTOM)
        .union(Edge::LEFT)
        .union(Edge::RIGHT);

    /// Returns `true` if this is a single cardinal edge.
    #[must_use]
    #[inline]
    pub const fn is_cardinal(self) -> bool {
        matches!(self, Edge::TOP | Edge::BOTTOM | Edge::LEFT | Edge::RIGHT)
    }

    /// Returns `true` if this is a corner (top-left, top-right, etc.).
    #[must_use]
    pub const fn is_corner(self) -> bool {
        matches!(
            self,
            Edge::TOP_LEFT | Edge::TOP_RIGHT | Edge::BOTTOM_LEFT | Edge::BOTTOM_RIGHT
        )
    }

    /// Invert the edge direction.
    ///
    /// TOP ↔ BOTTOM, LEFT ↔ RIGHT, corners invert both axes.
    #[must_use]
    #[inline]
    pub fn invert(self) -> Edge {
        if self.is_empty() || self == Edge::CENTER {
            return self;
        }

        let mut result = Edge::NONE;
        if self.contains(Edge::TOP) {
            result |= Edge::BOTTOM;
        }
        if self.contains(Edge::BOTTOM) {
            result |= Edge::TOP;
        }
        if self.contains(Edge::LEFT) {
            result |= Edge::RIGHT;
        }
        if self.contains(Edge::RIGHT) {
            result |= Edge::LEFT;
        }
        result
    }

    /// Parse an edge from an XML attribute string.
    ///
    /// Accepted values: "top", "bottom", "left", "right",
    /// "topleft", "topright", "bottomleft", "bottomright", "center".
    #[must_use]
    #[inline]
    pub fn parse(s: &str) -> Edge {
        match s.to_lowercase().as_str() {
            "top" => Edge::TOP,
            "bottom" => Edge::BOTTOM,
            "left" => Edge::LEFT,
            "right" => Edge::RIGHT,
            "topleft" | "top_left" => Edge::TOP_LEFT,
            "topright" | "top_right" => Edge::TOP_RIGHT,
            "bottomleft" | "bottom_left" => Edge::BOTTOM_LEFT,
            "bottomright" | "bottom_right" => Edge::BOTTOM_RIGHT,
            "center" => Edge::CENTER,
            _ => Edge::NONE,
        }
    }
}

impl Default for Edge {
    fn default() -> Self {
        Edge::NONE
    }
}
