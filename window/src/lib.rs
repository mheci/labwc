//! Window management — view geometry, state, stacking, and the View trait.

use labwc_core::{
    Edge, IconState, LastPlacement, Rect, SizeHints, SsdMode, SsdPreference, ViewAxis, ViewLayer,
    ViewType,
};
use labwc_output::Output;
use labwc_workspace::Workspace;
use parking_lot::Mutex;
use std::sync::Arc;

pub const MIN_VIEW_WIDTH: i32 = 100;
pub const MIN_VIEW_HEIGHT: i32 = 60;
pub const FALLBACK_WIDTH: i32 = 640;
pub const FALLBACK_HEIGHT: i32 = 480;

pub trait ViewImpl: Send + Sync {
    fn configure(&self, view: &View, geometry: Rect);
    fn close(&self, view: &View);
    fn set_activated(&self, view: &View, activated: bool);
    fn set_fullscreen(&self, view: &View, fullscreen: bool);
    fn maximize(&self, view: &View, maximized: ViewAxis);
    fn notify_tiled(&self, view: &View);
    fn get_size_hints(&self, view: &View) -> SizeHints;
}

pub struct View {
    pub id: u64,
    pub view_type: ViewType,
    pub impl_: Option<Box<dyn ViewImpl>>,

    // Geometry
    pub current: Mutex<Rect>,
    pub pending: Mutex<Rect>,
    pub natural_geometry: Mutex<Rect>,
    pub last_placement: Mutex<LastPlacement>,

    // State
    pub mapped: Mutex<bool>,
    pub been_mapped: bool,
    pub minimized: Mutex<bool>,
    pub maximized: Mutex<ViewAxis>,
    pub fullscreen: Mutex<bool>,
    pub shaded: Mutex<bool>,
    pub tiled: Mutex<Edge>,

    // Ownership
    pub workspace: Mutex<Option<Arc<Workspace>>>,
    pub output: Mutex<Option<Arc<Output>>>,
    pub layer: Mutex<ViewLayer>,

    // Decorations
    pub ssd_mode: Mutex<SsdMode>,
    pub ssd_preference: SsdPreference,

    // Properties
    pub title: Mutex<String>,
    pub app_id: Mutex<String>,
    pub visible_on_all_workspaces: Mutex<bool>,

    // Icon
    pub icon: Mutex<IconState>,
}

impl View {
    pub fn new(id: u64, view_type: ViewType) -> Self {
        Self {
            id,
            view_type,
            impl_: None,
            current: Mutex::new(Rect::default()),
            pending: Mutex::new(Rect::default()),
            natural_geometry: Mutex::new(Rect::default()),
            last_placement: Mutex::new(LastPlacement::default()),
            mapped: Mutex::new(false),
            been_mapped: false,
            minimized: Mutex::new(false),
            maximized: Mutex::new(ViewAxis::None),
            fullscreen: Mutex::new(false),
            shaded: Mutex::new(false),
            tiled: Mutex::new(Edge::NONE),
            workspace: Mutex::new(None),
            output: Mutex::new(None),
            layer: Mutex::new(ViewLayer::Normal),
            ssd_mode: Mutex::new(SsdMode::None),
            ssd_preference: SsdPreference::Unset,
            title: Mutex::new(String::new()),
            app_id: Mutex::new(String::new()),
            visible_on_all_workspaces: Mutex::new(false),
            icon: Mutex::new(IconState::default()),
        }
    }

    pub fn is_floating(&self) -> bool {
        !*self.fullscreen.lock()
            && *self.maximized.lock() == ViewAxis::None
            && *self.tiled.lock() == Edge::NONE
    }

    pub fn is_focusable(&self) -> bool {
        self.surface_exists() && *self.mapped.lock()
    }
    fn surface_exists(&self) -> bool {
        true
    }

    pub fn move_to(&self, x: i32, y: i32) {
        let pending = *self.pending.lock();
        let w = pending.width;
        let h = pending.height;
        self.move_resize(Rect::new(x, y, w, h));
    }

    pub fn move_resize(&self, geo: Rect) {
        if let Some(ref imp) = self.impl_ {
            imp.configure(self, geo);
        }
        *self.pending.lock() = geo;
    }

    pub fn move_relative(&self, dx: i32, dy: i32) {
        let p = *self.pending.lock();
        self.move_to(p.x + dx, p.y + dy);
    }

    pub fn maximize(&self, axis: ViewAxis) {
        if let Some(ref imp) = self.impl_ {
            imp.maximize(self, axis);
        }
        *self.maximized.lock() = axis;
    }

    pub fn set_fullscreen(&self, fs: bool) {
        if let Some(ref imp) = self.impl_ {
            imp.set_fullscreen(self, fs);
        }
        *self.fullscreen.lock() = fs;
    }

    pub fn toggle_maximize(&self) {
        let new = if *self.maximized.lock() == ViewAxis::Both {
            ViewAxis::None
        } else {
            ViewAxis::Both
        };
        self.maximize(new);
    }

    pub fn toggle_fullscreen(&self) {
        let cur = *self.fullscreen.lock();
        self.set_fullscreen(!cur);
    }

    pub fn snap_to_edge(&self, edge: Edge, output: &Output) {
        if *self.fullscreen.lock() {
            return;
        }
        *self.tiled.lock() = edge;
        let usable = output.usable_area();
        let gap = 0;
        let (x1, y1, x2, y2) = (
            usable.x + gap,
            usable.y + gap,
            usable.x + usable.width - gap,
            usable.y + usable.height - gap,
        );
        let (nx, ny, nw, nh) = if edge.contains(Edge::RIGHT) {
            ((x1 + x2) / 2, y1, (x2 - x1) / 2, y2 - y1)
        } else if edge.contains(Edge::LEFT) {
            (x1, y1, (x2 - x1) / 2, y2 - y1)
        } else {
            (x1, y1, x2 - x1, (y2 - y1) / 2)
        };
        self.move_resize(Rect::new(nx, ny, nw, nh));
    }
}

pub struct ViewManager {
    pub views: Vec<Arc<View>>,
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewManager {
    pub fn new() -> Self {
        Self { views: vec![] }
    }
    pub fn add(&mut self, view: Arc<View>) {
        self.views.push(view);
    }
    pub fn remove(&mut self, id: u64) {
        self.views.retain(|v| v.id != id);
    }
    pub fn topmost(&self) -> Option<&Arc<View>> {
        self.views.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use labwc_core::ViewAxis;

    #[test]
    fn test_view_new() {
        let v = View::new(1, ViewType::XdgShell);
        assert_eq!(v.id, 1);
        assert!(!*v.mapped.lock());
        assert_eq!(*v.maximized.lock(), ViewAxis::None);
    }

    #[test]
    fn test_view_move() {
        let v = View::new(1, ViewType::XdgShell);
        *v.pending.lock() = Rect::new(100, 100, 400, 300);
        v.move_to(200, 150);
        let p = *v.pending.lock();
        assert_eq!(p.x, 200);
        assert_eq!(p.y, 150);
    }

    #[test]
    fn test_view_maximize() {
        let v = View::new(1, ViewType::XdgShell);
        v.maximize(ViewAxis::Both);
        assert_eq!(*v.maximized.lock(), ViewAxis::Both);
        v.maximize(ViewAxis::None);
        assert_eq!(*v.maximized.lock(), ViewAxis::None);
    }

    #[test]
    fn test_view_fullscreen() {
        let v = View::new(1, ViewType::XdgShell);
        v.set_fullscreen(true);
        assert!(*v.fullscreen.lock());
        v.set_fullscreen(false);
        assert!(!*v.fullscreen.lock());
    }

    #[test]
    fn test_view_is_floating() {
        let v = View::new(1, ViewType::XdgShell);
        assert!(v.is_floating());
        v.maximize(ViewAxis::Both);
        assert!(!v.is_floating());
    }

    #[test]
    fn test_view_min_max() {
        assert_eq!(MIN_VIEW_WIDTH, 100);
        assert_eq!(MIN_VIEW_HEIGHT, 60);
    }
}
