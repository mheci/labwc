//! Output/monitor management — multi-output layout, usable areas, transforms.

use labwc_core::{OutputTransform, Rect};
use parking_lot::Mutex;

pub struct Output {
    pub id_bit: u64,
    pub name: String,
    pub make: String,
    pub model: String,
    pub scale: f64,
    pub transform: OutputTransform,
    pub geometry: Mutex<Rect>,
    pub usable_area: Mutex<Rect>,
    pub enabled: Mutex<bool>,
    pub has_fullscreen_view: Mutex<bool>,
}

impl Output {
    pub fn new(name: &str, geometry: Rect) -> Self {
        Self {
            id_bit: 1,
            name: name.to_string(),
            make: String::new(),
            model: String::new(),
            scale: 1.0,
            transform: OutputTransform::Normal,
            geometry: Mutex::new(geometry),
            usable_area: Mutex::new(geometry),
            enabled: Mutex::new(true),
            has_fullscreen_view: Mutex::new(false),
        }
    }

    pub fn is_usable(&self) -> bool {
        *self.enabled.lock()
    }
    pub fn set_usable_area(&self, area: Rect) {
        *self.usable_area.lock() = area;
    }
    pub fn usable_area(&self) -> Rect {
        *self.usable_area.lock()
    }
}
