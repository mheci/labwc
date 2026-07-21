//! Server-Side Decorations — titlebar, borders, buttons, shadows.

use labwc_core::Border;
use labwc_window::View;

pub struct Ssd {
    pub view_id: u64,
    pub titlebar_height: i32,
    pub border_width: i32,
    pub margin: Border,
}

impl Ssd {
    pub fn create(view: &View) -> Self {
        let border = 1;
        let titlebar = 26;
        Self {
            view_id: view.id,
            titlebar_height: titlebar,
            border_width: border,
            margin: Border {
                left: border,
                right: border,
                top: border + titlebar,
                bottom: border,
            },
        }
    }

    pub fn get_margin(&self) -> Border {
        self.margin
    }
    pub fn destroy(&self) {}
    pub fn extents(&self) -> Border {
        self.margin
    }
}
