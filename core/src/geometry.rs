//! Geometry primitives — axis-aligned rectangles, borders, and regions.

use std::cmp;

/// An axis-aligned integer rectangle.
///
/// This is the primary geometry type used throughout the compositor.
/// All coordinates are in layout space (global compositor coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rect {
    /// Left edge x-coordinate
    pub x: i32,
    /// Top edge y-coordinate
    pub y: i32,
    /// Width in pixels
    pub width: i32,
    /// Height in pixels
    pub height: i32,
}

impl Rect {
    /// Create a new rectangle.
    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns `true` if either dimension is ≤ 0.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    /// The right edge (x + width).
    #[inline]
    #[must_use]
    pub const fn right(&self) -> i32 {
        self.x + self.width
    }

    /// The bottom edge (y + height).
    #[inline]
    #[must_use]
    pub const fn bottom(&self) -> i32 {
        self.y + self.height
    }

    /// Center point of the rectangle.
    #[inline]
    #[must_use]
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Returns `true` if `other` is fully inside `self`.
    #[inline]
    #[must_use]
    pub fn contains(&self, other: &Rect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }

    /// Returns `true` if a point is inside the rectangle.
    #[inline]
    #[must_use]
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    /// Returns `true` if the rectangles overlap.
    #[inline]
    #[must_use]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Compute the intersection of two rectangles.
    #[inline]
    #[must_use]
    pub fn intersection(&self, other: &Rect) -> Rect {
        let x1 = cmp::max(self.x, other.x);
        let y1 = cmp::max(self.y, other.y);
        let x2 = cmp::min(self.right(), other.right());
        let y2 = cmp::min(self.bottom(), other.bottom());
        Rect::new(x1, y1, cmp::max(0, x2 - x1), cmp::max(0, y2 - y1))
    }

    /// Compute the bounding box of two rectangles.
    #[inline]
    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        let x = cmp::min(self.x, other.x);
        let y = cmp::min(self.y, other.y);
        let r = cmp::max(self.right(), other.right());
        let b = cmp::max(self.bottom(), other.bottom());
        Rect::new(x, y, r - x, b - y)
    }

    /// Center a rectangle of size `(w, h)` within `self`,
    /// writing the position into `(ox, oy)`.
    #[inline]
    pub fn center_within(&self, w: i32, h: i32, ox: &mut i32, oy: &mut i32) {
        *ox = self.x + (self.width - w) / 2;
        *oy = self.y + (self.height - h) / 2;
    }
}

impl From<(i32, i32, i32, i32)> for Rect {
    #[inline]
    fn from((x, y, w, h): (i32, i32, i32, i32)) -> Self {
        Self::new(x, y, w, h)
    }
}

/// A border with independent thickness on each side.
///
/// Used for SSD margins, window borders, and gap calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Border {
    /// Left side thickness in pixels
    pub left: i32,
    /// Right side thickness in pixels
    pub right: i32,
    /// Top side thickness in pixels
    pub top: i32,
    /// Bottom side thickness in pixels
    pub bottom: i32,
}

impl Border {
    /// Uniform border of `n` pixels on all sides.
    #[inline]
    #[must_use]
    pub const fn uniform(n: i32) -> Self {
        Self {
            left: n,
            right: n,
            top: n,
            bottom: n,
        }
    }

    /// Horizontal total (left + right).
    #[inline]
    #[must_use]
    pub const fn horizontal(&self) -> i32 {
        self.left + self.right
    }

    /// Vertical total (top + bottom).
    #[inline]
    #[must_use]
    pub const fn vertical(&self) -> i32 {
        self.top + self.bottom
    }

    /// Shrink a rectangle by this border (inset).
    #[inline]
    #[must_use]
    pub fn inset_rect(&self, r: &Rect) -> Rect {
        Rect::new(
            r.x + self.left,
            r.y + self.top,
            cmp::max(0, r.width - self.horizontal()),
            cmp::max(0, r.height - self.vertical()),
        )
    }

    /// Expand a rectangle by this border (outset).
    #[inline]
    #[must_use]
    pub fn outset_rect(&self, r: &Rect) -> Rect {
        Rect::new(
            r.x - self.left,
            r.y - self.top,
            r.width + self.horizontal(),
            r.height + self.vertical(),
        )
    }
}

/// A floating-point rectangle used for sub-pixel calculations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FRect {
    /// Left edge
    pub x: f64,
    /// Top edge
    pub y: f64,
    /// Width
    pub width: f64,
    /// Height
    pub height: f64,
}

impl FRect {
    /// Create a new float rectangle.
    #[inline]
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Round to an integer rectangle.
    #[inline]
    #[must_use]
    pub fn round(&self) -> Rect {
        Rect::new(
            self.x.round() as i32,
            self.y.round() as i32,
            self.width.round() as i32,
            self.height.round() as i32,
        )
    }
}
