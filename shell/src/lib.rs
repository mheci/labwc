//! Shell protocol handler — XDG shell, layer shell, popups.

use labwc_core::{SizeHints, ViewAxis};
use labwc_window::{View, ViewImpl, ViewManager};
use tracing::debug;

pub struct XdgViewImpl;
impl ViewImpl for XdgViewImpl {
    fn configure(&self, view: &View, geometry: labwc_core::Rect) {
        debug!("XDG configure view {} -> {:?}", view.id, geometry);
        *view.pending.lock() = geometry;
    }
    fn close(&self, _view: &View) {}
    fn set_activated(&self, _view: &View, _activated: bool) {}
    fn set_fullscreen(&self, _view: &View, _fullscreen: bool) {}
    fn maximize(&self, _view: &View, _maximized: ViewAxis) {}
    fn notify_tiled(&self, _view: &View) {}
    fn get_size_hints(&self, _view: &View) -> SizeHints {
        SizeHints::default()
    }
}

pub fn xdg_shell_init(_views: &mut ViewManager) {
    debug!("Initializing XDG shell handler");
}

#[cfg(test)]
mod tests {
    use super::*;
    use labwc_core::ViewType;
    use labwc_window::{View, ViewManager};

    #[test]
    fn test_xdg_view_impl() {
        let imp = XdgViewImpl;
        let v = View::new(1, ViewType::XdgShell);
        imp.configure(&v, labwc_core::Rect::new(10, 20, 200, 300));
        let p = *v.pending.lock();
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
        assert_eq!(p.width, 200);
        assert_eq!(p.height, 300);
    }

    #[test]
    fn test_xdg_shell_init() {
        let mut vm = ViewManager::new();
        xdg_shell_init(&mut vm);
        assert!(vm.views.is_empty());
    }
}
