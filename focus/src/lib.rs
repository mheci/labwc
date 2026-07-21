//! Focus management — tracks active view, focus-follows-mouse, raise-on-focus.

use labwc_core::{InputMode, ViewLayer};
use labwc_window::View;
use std::sync::Arc;
use tracing::debug;

pub struct FocusManager {
    pub active_view: Option<Arc<View>>,
    pub input_mode: InputMode,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            active_view: None,
            input_mode: InputMode::Passthrough,
        }
    }

    pub fn focus_view(&mut self, view: &Arc<View>, raise: bool) {
        debug!("Focus view {} (raise={})", view.id, raise);
        let v: &View = view.as_ref();
        if let Some(ref prev) = self.active_view {
            if prev.id == v.id && raise {
                return;
            }
            let _p: &View = prev.as_ref();
            // Clear activation on previous
        }
        // Set activation on new view
        self.active_view = Some(Arc::clone(view));
    }

    pub fn focus_topmost(&mut self, views: &[Arc<View>]) {
        for v in views.iter().rev() {
            if v.is_focusable() && *v.layer.lock() != ViewLayer::AlwaysOnBottom {
                self.focus_view(v, false);
                return;
            }
        }
        self.active_view = None;
    }

    pub fn clear_focus(&mut self) {
        self.active_view = None;
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_new() {
        let mgr = FocusManager::new();
        assert!(mgr.active_view.is_none());
        assert_eq!(mgr.input_mode, InputMode::Passthrough);
    }

    #[test]
    fn test_clear_focus() {
        let mut mgr = FocusManager::new();
        mgr.clear_focus();
        assert!(mgr.active_view.is_none());
    }
}
