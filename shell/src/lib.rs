use labwc_core::{SizeHints, ViewAxis};
use labwc_window::{View, ViewImpl, ViewManager};
use tracing::debug;

pub struct XdgViewImpl;

impl ViewImpl for XdgViewImpl {
    fn configure(&self, view: &View, geometry: labwc_core::Rect) {
        debug!("XDG configure view {} -> {:?}", view.id, geometry);
        *view.pending.lock() = geometry;
        *view.current.lock() = geometry;
    }

    fn close(&self, view: &View) {
        debug!("XDG close view {}", view.id);
        *view.mapped.lock() = false;
        *view.minimized.lock() = false;
    }

    fn set_activated(&self, view: &View, activated: bool) {
        debug!("XDG set_activated {} = {}", view.id, activated);
    }

    fn set_fullscreen(&self, view: &View, fullscreen: bool) {
        debug!("XDG set_fullscreen {} = {}", view.id, fullscreen);
        *view.fullscreen.lock() = fullscreen;
        if fullscreen {
            *view.maximized.lock() = ViewAxis::None;
            *view.tiled.lock() = labwc_core::Edge::NONE;
        }
    }

    fn maximize(&self, view: &View, maximized: ViewAxis) {
        debug!("XDG maximize {} = {:?}", view.id, maximized);
        *view.maximized.lock() = maximized;
        if maximized != ViewAxis::None {
            *view.tiled.lock() = labwc_core::Edge::NONE;
        }
    }

    fn notify_tiled(&self, view: &View) {
        let tiled = *view.tiled.lock();
        debug!("XDG notify_tiled {} = {:?}", view.id, tiled);
    }

    fn get_size_hints(&self, _view: &View) -> SizeHints {
        SizeHints::default()
    }
}

pub fn xdg_shell_init(_views: &mut ViewManager) {
    debug!("XDG shell handler initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use labwc_core::ViewType;
    use labwc_window::View;

    #[test]
    fn test_close_sets_unmapped() {
        let imp = XdgViewImpl;
        let v = View::new(1, ViewType::XdgShell);
        *v.mapped.lock() = true;
        imp.close(&v);
        assert!(!*v.mapped.lock());
    }

    #[test]
    fn test_fullscreen_clears_maximize() {
        let imp = XdgViewImpl;
        let v = View::new(1, ViewType::XdgShell);
        *v.maximized.lock() = ViewAxis::Both;
        imp.set_fullscreen(&v, true);
        assert!(*v.fullscreen.lock());
        assert_eq!(*v.maximized.lock(), ViewAxis::None);
    }

    #[test]
    fn test_maximize_clears_tiled() {
        let imp = XdgViewImpl;
        let v = View::new(1, ViewType::XdgShell);
        *v.tiled.lock() = labwc_core::Edge::LEFT;
        imp.maximize(&v, ViewAxis::Both);
        assert_eq!(*v.maximized.lock(), ViewAxis::Both);
        assert_eq!(*v.tiled.lock(), labwc_core::Edge::NONE);
    }

    #[test]
    fn test_configure_updates_both() {
        let imp = XdgViewImpl;
        let v = View::new(1, ViewType::XdgShell);
        imp.configure(&v, labwc_core::Rect::new(10, 20, 200, 300));
        let pending = *v.pending.lock();
        let current = *v.current.lock();
        assert_eq!(pending.x, 10);
        assert_eq!(current.x, 10);
    }
}
