use labwc_core::{OutputTransform, Rect};
use parking_lot::Mutex;

pub struct Output {
    pub id_bit: u64,
    pub name: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub physical_width_mm: i32,
    pub physical_height_mm: i32,
    pub refresh_rate: u32,
    pub max_refresh_rate: u32,
    pub min_refresh_rate: u32,
    pub current_refresh_rate: u32,
    pub gsync_compatible: bool,
    pub freesync_compatible: bool,
    pub adaptive_sync_enabled: bool,
    pub hdr_enabled: bool,
    pub hdr10_supported: bool,
    pub color_depth: u8,
    pub pixel_format: PixelFormat,
    pub scale: f64,
    pub transform: OutputTransform,
    pub usable_area: Mutex<Rect>,
    pub enabled: Mutex<bool>,
    pub leased: Mutex<bool>,
    pub has_fullscreen_view: Mutex<bool>,
    pub wlr_output: Option<*mut libc::c_void>,
    pub scene_output: Option<*mut libc::c_void>,
    pub session_lock_tree: Option<*mut libc::c_void>,
    pub cycle_osd_tree: Option<*mut libc::c_void>,
    pub layer_popup_tree: Option<*mut libc::c_void>,
    pub layer_trees: Vec<Option<*mut libc::c_void>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Bgra8888,
    Rgba8888,
    Xrgb8888,
    Xbgr8888,
    Rgb565,
    Bgra1010102,
    Rgba1010102,
}

impl Output {
    pub fn new(name: &str, width: i32, height: i32) -> Self {
        Self {
            id_bit: 1,
            name: name.to_string(),
            make: String::new(),
            model: String::new(),
            serial: String::new(),
            x: 0,
            y: 0,
            width,
            height,
            physical_width_mm: 0,
            physical_height_mm: 0,
            refresh_rate: 60,
            max_refresh_rate: 60,
            min_refresh_rate: 60,
            current_refresh_rate: 60,
            gsync_compatible: false,
            freesync_compatible: false,
            adaptive_sync_enabled: false,
            hdr_enabled: false,
            hdr10_supported: false,
            color_depth: 8,
            pixel_format: PixelFormat::Bgra8888,
            scale: 1.0,
            transform: OutputTransform::Normal,
            usable_area: Mutex::new(Rect::new(0, 0, width, height)),
            enabled: Mutex::new(true),
            leased: Mutex::new(false),
            has_fullscreen_view: Mutex::new(false),
            wlr_output: None,
            scene_output: None,
            session_lock_tree: None,
            cycle_osd_tree: None,
            layer_popup_tree: None,
            layer_trees: vec![None; 4],
        }
    }

    pub fn is_usable(&self) -> bool {
        *self.enabled.lock() && !*self.leased.lock()
    }

    pub fn usable_area(&self) -> Rect {
        *self.usable_area.lock()
    }

    pub fn set_usable_area(&self, x: i32, y: i32, w: i32, h: i32) {
        *self.usable_area.lock() = Rect::new(x, y, w, h);
    }

    pub fn set_has_fullscreen_view(&self, has: bool) {
        *self.has_fullscreen_view.lock() = has;
    }

    pub fn has_fullscreen_view(&self) -> bool {
        *self.has_fullscreen_view.lock()
    }

    pub fn layout_geometry(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn set_refresh_rate(&mut self, hz: u32) {
        self.current_refresh_rate = hz.clamp(self.min_refresh_rate, self.max_refresh_rate);
    }

    pub fn detect_adaptive_sync(&mut self) {
        if self.gsync_compatible || self.freesync_compatible {
            self.adaptive_sync_enabled = true;
        }
    }

    pub fn supports_hdr(&self) -> bool {
        self.hdr10_supported && self.color_depth >= 10
    }
}

pub fn output_init() {}

pub fn output_finish() {}

pub fn output_nearest_to(_x: i32, _y: i32) -> Option<&'static Output> {
    None
}

pub fn output_nearest_to_cursor() -> Option<&'static Output> {
    None
}

pub fn output_from_name(_name: &str) -> Option<&'static Output> {
    None
}

pub fn output_get_adjacent(
    _output: &Output,
    _direction: labwc_core::Edge,
    _wrap: bool,
) -> Option<&'static Output> {
    None
}

pub fn output_from_wlr_output(_wlr_output: *mut libc::c_void) -> Option<&'static Output> {
    None
}

pub fn output_update_all_usable_areas(_layout_changed: bool) {}
