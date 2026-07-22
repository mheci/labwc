use labwc_core::{Edge, SizeHints, ViewAxis};
use labwc_window::View;
use tracing::debug;

pub fn xdg_shell_init() {
    debug!("XDG shell handler initialized");
}

pub fn configure(view: &View, geometry: labwc_core::Rect) {
    *view.pending.lock() = geometry;
    *view.current.lock() = geometry;
}

pub fn close(view: &View) {
    *view.mapped.lock() = false;
    *view.minimized.lock() = false;
}

pub fn set_activated(_view: &View, _activated: bool) {}

pub fn set_fullscreen(view: &View, fullscreen: bool) {
    *view.fullscreen.lock() = fullscreen;
    if fullscreen {
        *view.maximized.lock() = ViewAxis::None;
        *view.tiled.lock() = Edge::NONE;
    }
}

pub fn maximize(view: &View, maximized: ViewAxis) {
    *view.maximized.lock() = maximized;
    if maximized != ViewAxis::None {
        *view.tiled.lock() = Edge::NONE;
    }
}

pub fn notify_tiled(view: &View) {
    let _ = *view.tiled.lock();
}

pub fn get_size_hints(_view: &View) -> SizeHints {
    SizeHints::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use labwc_core::ViewType;
    use labwc_window::View;

    #[test]
    fn test_close() {
        let v = View::new(1, ViewType::XdgShell);
        *v.mapped.lock() = true;
        close(&v);
        assert!(!*v.mapped.lock());
    }

    #[test]
    fn test_fullscreen() {
        let v = View::new(1, ViewType::XdgShell);
        *v.maximized.lock() = ViewAxis::Both;
        set_fullscreen(&v, true);
        assert!(*v.fullscreen.lock());
        assert_eq!(*v.maximized.lock(), ViewAxis::None);
    }

    #[test]
    fn test_maximize() {
        let v = View::new(1, ViewType::XdgShell);
        *v.tiled.lock() = Edge::LEFT;
        maximize(&v, ViewAxis::Both);
        assert_eq!(*v.maximized.lock(), ViewAxis::Both);
        assert_eq!(*v.tiled.lock(), Edge::NONE);
    }

    #[test]
    fn test_configure() {
        let v = View::new(1, ViewType::XdgShell);
        configure(&v, labwc_core::Rect::new(10, 20, 200, 300));
        assert_eq!(v.pending.lock().x, 10);
        assert_eq!(v.current.lock().x, 10);
    }
}
