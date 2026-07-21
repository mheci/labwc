//! Core enumerated types replacing C integer constants and #defines.

use bitflags::bitflags;

// ─────────────────────────────────────────────
//  Window state enums
// ─────────────────────────────────────────────

/// The maximization state of a view along each axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewAxis {
    /// Not maximized on any axis.
    None,
    /// Maximized horizontally only.
    Horizontal,
    /// Maximized vertically only.
    Vertical,
    /// Maximized on both axes.
    Both,
}

impl ViewAxis {
    /// Returns `true` if horizontally maximized.
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    /// Returns `true` if vertically maximized.
    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
}

/// Server-side decoration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SsdMode {
    /// No server-side decorations.
    None,
    /// Border only (no titlebar).
    Border,
    /// Full decorations: titlebar + border + buttons.
    Full,
}

/// Client decoration preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SsdPreference {
    /// Client wants to draw its own decorations.
    Client,
    /// Client requests server-side decorations.
    Server,
    /// No preference expressed yet.
    Unset,
}

/// View stacking layer for z-ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewLayer {
    /// Always behind normal windows.
    AlwaysOnBottom = 0,
    /// Normal stacking.
    Normal = 1,
    /// Always above normal windows.
    AlwaysOnTop = 2,
}

/// The type of a Wayland surface view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewType {
    /// XDG shell toplevel.
    XdgShell,
    /// XWayland surface.
    XWayland,
    /// Unmanaged XWayland override-redirect surface.
    XWaylandUnmanaged,
    /// Layer shell surface.
    LayerShell,
}

/// Window type classification for window rules matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowType {
    /// Normal application window.
    Normal,
    /// Dialog window (modal or transient).
    Dialog,
    /// Utility window.
    Utility,
    /// Toolbar.
    Toolbar,
    /// Splash screen.
    Splash,
    /// Menu window.
    Menu,
    /// Dropdown menu.
    DropdownMenu,
    /// Popup menu.
    PopupMenu,
    /// Tooltip.
    Tooltip,
    /// Notification.
    Notification,
    /// Combo box dropdown.
    Combo,
    /// Drag-and-drop window.
    Dnd,
}

/// Placement policy for new windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlacementPolicy {
    /// Center on the output.
    Center,
    /// Position near the cursor.
    Cursor,
    /// Automatic placement (first-fit).
    Automatic,
    /// Cascaded placement.
    Cascade,
}

/// Tiling event notification mode for clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilingEventsMode {
    /// Never notify clients of tiling state.
    Never,
    /// Notify only for region-snapped windows.
    Region,
    /// Notify only for edge-snapped windows.
    Edge,
    /// Always notify.
    Always,
}

// ─────────────────────────────────────────────
//  Input state enums
// ─────────────────────────────────────────────

/// Server-side input interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// Normal passthrough to clients.
    Passthrough,
    /// Interactive move in progress.
    Move,
    /// Interactive resize in progress.
    Resize,
    /// Menu navigation.
    Menu,
    /// Window switching (Alt-Tab).
    Cycle,
}

/// Server-side cursor shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorShape {
    /// Default arrow cursor.
    Default,
    /// Move cursor (four-way arrows).
    Move,
    /// Resize cursor with edge specification.
    Resize(ResizeEdge),
    /// Hand pointer.
    Pointer,
    /// Crosshair.
    Crosshair,
    /// Text I-beam.
    Text,
    /// Wait cursor (watch/hourglass).
    Wait,
    /// Help cursor.
    Help,
    /// Not allowed.
    NotAllowed,
    /// Diagonal resize (northwest-southeast).
    Nwse,
    /// Diagonal resize (northeast-southwest).
    Nesw,
    /// Vertical resize (north-south).
    Ns,
    /// Horizontal resize (west-east).
    We,
    /// All-scroll.
    AllScroll,
    /// Zoom in.
    ZoomIn,
    /// Zoom out.
    ZoomOut,
}

/// Input device type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDeviceType {
    /// Physical or virtual keyboard.
    Keyboard,
    /// Mouse, trackball, or touchpad.
    Pointer,
    /// Touchscreen.
    Touch,
    /// Drawing tablet tool (stylus, eraser, etc.).
    TabletTool,
    /// Tablet pad with buttons/rings.
    TabletPad,
    /// Hardware switch (lid, tablet mode).
    Switch,
}

// ─────────────────────────────────────────────
//  Output enums
// ─────────────────────────────────────────────

/// Output transform (rotation + reflection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputTransform {
    /// No transform.
    Normal,
    /// 90 degrees clockwise.
    Rotate90,
    /// 180 degrees.
    Rotate180,
    /// 270 degrees clockwise (90 CCW).
    Rotate270,
    /// Flipped horizontally.
    Flipped,
    /// Flipped + 90 degrees.
    Flipped90,
    /// Flipped + 180 degrees.
    Flipped180,
    /// Flipped + 270 degrees.
    Flipped270,
}

// ─────────────────────────────────────────────
//  Focus enums
// ─────────────────────────────────────────────

/// Whether a view wants focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewWantsFocus {
    /// Always accepts focus.
    Always,
    /// Likely accepts focus (override-redirect windows).
    Likely,
    /// Never accepts focus (desktop, dock, etc.).
    Never,
}

// ─────────────────────────────────────────────
//  Resize edge
// ─────────────────────────────────────────────

/// A resize edge or combination of edges.
///
/// This mirrors the wlroots/XDG shell resize edge values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    /// No edge (invalid).
    None = 0,
    /// Top edge.
    Top = 1,
    /// Bottom edge.
    Bottom = 2,
    /// Left edge.
    Left = 4,
    /// Top-left corner.
    TopLeft = 5,
    /// Bottom-left corner.
    BottomLeft = 6,
    /// Right edge.
    Right = 8,
    /// Top-right corner.
    TopRight = 9,
    /// Bottom-right corner.
    BottomRight = 10,
}

impl ResizeEdge {
    /// Convert from the wlroots/xdg-shell resize edge integer.
    #[must_use]
    pub const fn from_wlr(edge: u32) -> Self {
        match edge {
            1 => Self::Top,
            2 => Self::Bottom,
            4 => Self::Left,
            5 => Self::TopLeft,
            6 => Self::BottomLeft,
            8 => Self::Right,
            9 => Self::TopRight,
            10 => Self::BottomRight,
            _ => Self::None,
        }
    }

    /// Convert to the wlroots/xdg-shell resize edge integer.
    #[must_use]
    pub const fn to_wlr(self) -> u32 {
        self as u32
    }
}

// ─────────────────────────────────────────────
//  Criteria bitflags for view iteration
// ─────────────────────────────────────────────

bitflags! {
    /// Criteria for filtering views during iteration.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ViewCriteria: u32 {
        /// Only views on the current workspace.
        const CURRENT_WORKSPACE = 1 << 0;
        /// Only fullscreen views.
        const FULLSCREEN = 1 << 1;
        /// Only always-on-top views.
        const ALWAYS_ON_TOP = 1 << 2;
        /// Exclude modal dialogs.
        const NO_DIALOG = 1 << 3;
        /// Exclude always-on-top views.
        const NO_ALWAYS_ON_TOP = 1 << 4;
        /// Exclude views that skip the window switcher.
        const NO_SKIP_WINDOW_SWITCHER = 1 << 5;
        /// Exclude omnipresent (sticky) views.
        const NO_OMNIPRESENT = 1 << 6;
    }
}
