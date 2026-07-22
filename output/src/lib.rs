pub mod edid;
pub mod modes;

use labwc_core::{OutputTransform, Rect};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::info;

pub use modes::{probe_all_connectors, ConnectorType, DrmConnector, DrmMode};

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
    pub preferred_mode: Option<DrmMode>,
    pub gsync_compatible: bool,
    pub freesync_compatible: bool,
    pub adaptive_sync_enabled: bool,
    pub hdr_enabled: bool,
    pub hdr10_supported: bool,
    pub color_depth: u8,
    pub pixel_format: PixelFormat,
    pub scale: f64,
    pub transform: OutputTransform,
    pub connector_type: ConnectorType,
    pub usable_area: Mutex<Rect>,
    pub enabled: Mutex<bool>,
    pub leased: Mutex<bool>,
    pub has_fullscreen_view: Mutex<bool>,
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

/// Probe all connected displays and return fully configured Output objects
/// with the best available resolution and refresh rate for each.
///
/// Priority:
/// 1. EDID preferred mode (manufacturer-recommended native resolution/refresh)
/// 2. EDID highest refresh mode at native resolution
/// 3. Fallback 1920x1080@60
pub fn discover_outputs() -> Vec<Arc<Output>> {
    let connectors = probe_all_connectors();
    let mut outputs = Vec::new();
    let mut next_id_bit: u64 = 1;

    if connectors.is_empty() {
        info!("No displays detected (headless mode)");
        let mut out = Output::new("FALLBACK-1", 1920, 1080, 60);
        out.id_bit = 1;
        return vec![Arc::new(out)];
    }

    for conn in &connectors {
        let pref = conn.preferred_mode.as_ref();
        let edid = conn.edid.as_ref();

        let width = pref.map(|m| m.width as i32).unwrap_or(1920);
        let height = pref.map(|m| m.height as i32).unwrap_or(1080);

        // Use the preferred refresh rate from EDID, or highest from sysfs modes
        let refresh = pref.map(|m| m.refresh_hz).unwrap_or(
            conn.supported_modes
                .iter()
                .filter(|m| m.width as i32 == width && m.height as i32 == height)
                .map(|m| m.refresh_hz)
                .max()
                .unwrap_or(60),
        );

        let max_refresh = conn
            .supported_modes
            .iter()
            .filter(|m| m.width as i32 == width && m.height as i32 == height)
            .map(|m| m.refresh_hz)
            .max()
            .unwrap_or(refresh);

        let min_refresh = conn
            .supported_modes
            .iter()
            .map(|m| m.refresh_hz)
            .min()
            .unwrap_or(if conn.adaptive_sync { 48 } else { refresh });

        let best_mode = pref.cloned().or_else(|| {
            conn.supported_modes
                .iter()
                .filter(|m| m.width as i32 == width && m.height as i32 == height)
                .max_by_key(|m| m.refresh_hz)
                .copied()
        });

        let output = Output {
            id_bit: next_id_bit,
            name: conn.name.clone(),
            make: edid
                .map(|e| e.manufacturer.clone())
                .unwrap_or_else(|| "Unknown".into()),
            model: edid
                .map(|e| e.model.clone())
                .unwrap_or_else(|| conn.connector_type.name().into()),
            serial: edid.as_ref().map(|e| e.serial.clone()).unwrap_or_default(),
            x: 0,
            y: 0,
            width,
            height,
            physical_width_mm: conn.physical_width_mm as i32,
            physical_height_mm: conn.physical_height_mm as i32,
            refresh_rate: refresh,
            max_refresh_rate: max_refresh,
            min_refresh_rate: min_refresh,
            current_refresh_rate: refresh,
            preferred_mode: best_mode,
            gsync_compatible: conn.gsync,
            freesync_compatible: conn.freesync,
            adaptive_sync_enabled: conn.adaptive_sync,
            hdr_enabled: false,
            hdr10_supported: conn.hdr_capable,
            color_depth: if conn.hdr_capable { 10 } else { 8 },
            pixel_format: if conn.hdr_capable {
                PixelFormat::Bgra1010102
            } else {
                PixelFormat::Bgra8888
            },
            scale: 1.0,
            transform: OutputTransform::Normal,
            connector_type: conn.connector_type,
            usable_area: Mutex::new(Rect::new(0, 0, width, height)),
            enabled: Mutex::new(conn.enabled),
            leased: Mutex::new(false),
            has_fullscreen_view: Mutex::new(false),
        };

        output.set_usable_area(0, 0, width, height);

        info!(
            "Output {}: {}x{} @ {}Hz (max {}Hz) [{}] {}{}{}",
            output.name,
            width,
            height,
            refresh,
            max_refresh,
            output.make,
            if output.adaptive_sync_enabled {
                " VRR"
            } else {
                ""
            },
            if output.hdr10_supported { " HDR" } else { "" },
            if output.physical_width_mm > 0 {
                let diag = ((output.physical_width_mm as f32).powi(2)
                    + (output.physical_height_mm as f32).powi(2))
                .sqrt()
                    / 25.4;
                format!(" {:.1}in", diag)
            } else {
                String::new()
            },
        );

        next_id_bit <<= 1;
        outputs.push(Arc::new(output));
    }

    outputs
}

impl Output {
    pub fn new(name: &str, width: i32, height: i32, refresh: u32) -> Self {
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
            refresh_rate: refresh,
            max_refresh_rate: refresh,
            min_refresh_rate: refresh,
            current_refresh_rate: refresh,
            preferred_mode: None,
            gsync_compatible: false,
            freesync_compatible: false,
            adaptive_sync_enabled: false,
            hdr_enabled: false,
            hdr10_supported: false,
            color_depth: 8,
            pixel_format: PixelFormat::Bgra8888,
            scale: 1.0,
            transform: OutputTransform::Normal,
            connector_type: ConnectorType::Virtual,
            usable_area: Mutex::new(Rect::new(0, 0, width, height)),
            enabled: Mutex::new(true),
            leased: Mutex::new(false),
            has_fullscreen_view: Mutex::new(false),
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
    pub fn diagonal_inches(&self) -> f32 {
        if self.physical_width_mm > 0 && self.physical_height_mm > 0 {
            ((self.physical_width_mm as f32).powi(2) + (self.physical_height_mm as f32).powi(2))
                .sqrt()
                / 25.4
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_creation() {
        let out = Output::new("DP-1", 2560, 1440, 165);
        assert_eq!(out.refresh_rate, 165);
        assert_eq!(out.width, 2560);
    }

    #[test]
    fn test_set_refresh_rate() {
        let mut out = Output::new("HDMI-1", 3840, 2160, 144);
        out.min_refresh_rate = 48;
        out.max_refresh_rate = 144;
        out.set_refresh_rate(60);
        assert_eq!(out.current_refresh_rate, 60);
        out.set_refresh_rate(240);
        assert_eq!(out.current_refresh_rate, 144);
    }

    #[test]
    fn test_hdr_detection() {
        let mut out = Output::new("DP-1", 3840, 2160, 144);
        out.hdr10_supported = true;
        out.color_depth = 10;
        assert!(out.supports_hdr());
    }
}
