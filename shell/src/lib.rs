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
