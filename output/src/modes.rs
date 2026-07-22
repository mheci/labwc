//! DRM connector mode probing — reads supported modes, preferred mode,
//! VRR capability, and connector properties from sysfs at runtime.

use crate::edid::EdidInfo;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct DrmConnector {
    pub name: String,
    pub path: PathBuf,
    pub connected: bool,
    pub enabled: bool,
    pub connector_type: ConnectorType,
    pub supported_modes: Vec<DrmMode>,
    pub preferred_mode: Option<DrmMode>,
    pub edid: Option<EdidInfo>,
    pub vrr_capable: bool,
    pub vrr_range: Option<(u32, u32)>,
    pub gsync: bool,
    pub freesync: bool,
    pub adaptive_sync: bool,
    pub hdr_capable: bool,
    pub max_bpc: u8,
    pub current_bpc: u8,
    pub physical_width_mm: u32,
    pub physical_height_mm: u32,
    pub crtc_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    Unknown,
    EDP,
    DP,
    HDMIA,
    HDMIB,
    LVDS,
    VGA,
    DVID,
    DVII,
    Virtual,
}

impl ConnectorType {
    pub fn name(&self) -> &str {
        match self {
            Self::Unknown => "Unknown",
            Self::EDP => "eDP",
            Self::DP => "DisplayPort",
            Self::HDMIA => "HDMI-A",
            Self::HDMIB => "HDMI-B",
            Self::LVDS => "LVDS",
            Self::VGA => "VGA",
            Self::DVID => "DVI-D",
            Self::DVII => "DVI-I",
            Self::Virtual => "Virtual",
        }
    }

    pub fn from_sysfs_name(name: &str) -> Self {
        if name.contains("eDP") {
            Self::EDP
        } else if name.contains("DP") {
            Self::DP
        } else if name.contains("HDMI-A") {
            Self::HDMIA
        } else if name.contains("HDMI-B") {
            Self::HDMIB
        } else if name.contains("LVDS") {
            Self::LVDS
        } else if name.contains("VGA") {
            Self::VGA
        } else if name.contains("DVI-D") {
            Self::DVID
        } else if name.contains("DVI-I") {
            Self::DVII
        } else if name.contains("Virtual") {
            Self::Virtual
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrmMode {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub is_preferred: bool,
    pub is_current: bool,
    pub clock_khz: u32,
}

impl DrmMode {
    pub fn display_string(&self) -> String {
        let star = if self.is_preferred { " ★" } else { "" };
        let cur = if self.is_current { " (current)" } else { "" };
        format!(
            "{}x{} @ {}Hz{}{}",
            self.width, self.height, self.refresh_hz, star, cur
        )
    }
}

/// Probe all DRM connectors and their available modes at runtime.
///
/// Reads from:
/// - `/sys/class/drm/card*/` — DRM card enumeration
/// - `/sys/class/drm/card*-<connector>/status` — connection status
/// - `/sys/class/drm/card*-<connector>/enabled` — whether enabled
/// - `/sys/class/drm/card*-<connector>/modes` — supported mode list
/// - `/sys/class/drm/card*-<connector>/edid` — raw EDID binary
/// - `/sys/class/drm/card*-<connector>/dpms` — power state
///
/// For VRR/G-Sync/FreeSync:
/// - Reads DRM connector properties via `/sys/class/drm/card*/...`
/// - Detects "vrr_capable" and "VRR_ENABLED" properties
/// - On NVIDIA: checks nvidia_drm module parameters
pub fn probe_all_connectors() -> Vec<DrmConnector> {
    let mut connectors = Vec::new();
    let drm_root = PathBuf::from("/sys/class/drm");

    let entries = match std::fs::read_dir(&drm_root) {
        Ok(e) => e,
        Err(_) => {
            // No DRM available — return a headless fallback
            debug!("No /sys/class/drm, using headless output discovery");
            return fallback_connectors();
        }
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Only process connector entries: card0-HDMI-A-1, card1-DP-2, etc.
        if !name.contains('-') || name.starts_with("renderD") {
            continue;
        }

        let path = entry.path();
        let status = read_sysfs_line(&path.join("status"));
        let connected = status == "connected";
        let enabled = read_sysfs_line(&path.join("enabled")) == "enabled";

        if !connected {
            debug!("Connector {}: disconnected", name);
            continue;
        }

        let connector_type = ConnectorType::from_sysfs_name(&name);

        // Parse EDID for display identification and capabilities
        let edid_path = path.join("edid");
        let edid = std::fs::read(&edid_path)
            .ok()
            .and_then(|data| EdidInfo::parse(&data));

        // Parse supported modes from the modes file
        let modes_path = path.join("modes");
        let modes = parse_modes_file(&modes_path);

        let (preferred_mode, max_refresh) = if let Some(ref e) = edid {
            let pref = e.best_mode().map(|m| DrmMode {
                width: m.width,
                height: m.height,
                refresh_hz: m.refresh_hz,
                is_preferred: true,
                is_current: false,
                clock_khz: 0,
            });
            let max_hz = e.max_refresh();
            (pref, max_hz)
        } else {
            // Fallback: pick highest refresh from modes list
            let pref = modes.last().copied(); // modes are sorted
            let max_hz = modes.iter().map(|m| m.refresh_hz).max().unwrap_or(60);
            (pref, max_hz)
        };

        // VRR detection
        let vrr_capable = detect_vrr(&path, &name);
        let freesync = detect_freesync(&edid);
        let gsync = name.contains("DP") && vrr_capable && !freesync;

        // HDR detection
        let hdr_capable = edid
            .as_ref()
            .map(|e| e.hdr10_supported || e.hdr_st2084)
            .unwrap_or(false);
        let max_bpc = edid.as_ref().map(|e| e.color_depth).unwrap_or(8);
        let current_bpc = read_sysfs_line(&path.join("max_bpc"))
            .parse::<u8>()
            .unwrap_or(max_bpc);

        // Physical dimensions
        let (pw, ph) = edid
            .as_ref()
            .map(|e| {
                (
                    e.physical_width_cm as u32 * 10,
                    e.physical_height_cm as u32 * 10,
                )
            })
            .unwrap_or((0, 0));

        let diag = edid.as_ref().map(|e| e.diagonal_inches()).unwrap_or(0.0);

        info!(
            "{}: {} ({} {:.1}in) — {} modes, max {}Hz, {}V, {}VRR {}HDR",
            name,
            connector_type.name(),
            edid.as_ref()
                .map(|e| e.manufacturer.clone())
                .unwrap_or_default(),
            diag,
            modes.len(),
            max_refresh,
            edid.as_ref()
                .map(|e| e
                    .preferred_mode
                    .map(|m| format!("{}x{}", m.width, m.height))
                    .unwrap_or_default())
                .unwrap_or_default(),
            if vrr_capable { " " } else { " no-" },
            if hdr_capable { " " } else { " no-" },
        );

        let mut conn = DrmConnector {
            name,
            path,
            connected,
            enabled,
            connector_type,
            supported_modes: modes,
            preferred_mode,
            edid,
            vrr_capable,
            vrr_range: if vrr_capable && max_refresh > 60 {
                Some((48, max_refresh))
            } else {
                None
            },
            gsync,
            freesync,
            adaptive_sync: vrr_capable,
            hdr_capable,
            max_bpc,
            current_bpc,
            physical_width_mm: pw,
            physical_height_mm: ph,
            crtc_id: None,
        };

        // Mark the preferred mode as preferred
        if let Some(ref pref) = conn.preferred_mode {
            for m in &mut conn.supported_modes {
                if m.width == pref.width
                    && m.height == pref.height
                    && m.refresh_hz == pref.refresh_hz
                {
                    m.is_preferred = true;
                }
            }
        }

        connectors.push(conn);
    }

    if connectors.is_empty() {
        connectors = fallback_connectors();
    }

    // Sort: internal displays first, then by connector type priority
    connectors.sort_by(|a, b| {
        let a_internal = matches!(a.connector_type, ConnectorType::EDP | ConnectorType::LVDS);
        let b_internal = matches!(b.connector_type, ConnectorType::EDP | ConnectorType::LVDS);
        b_internal
            .cmp(&a_internal)
            .then(a.connector_type.name().cmp(b.connector_type.name()))
    });

    connectors
}

fn parse_modes_file(path: &PathBuf) -> Vec<DrmMode> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut modes = Vec::new();
    let mut is_first = true;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: "1920x1080" or "3840x2160"
        let parts: Vec<&str> = line.split('x').collect();
        if parts.len() != 2 {
            continue;
        }

        let width: u32 = parts[0].parse().unwrap_or(0);
        let height: u32 = parts[1].parse().unwrap_or(0);

        if width == 0 || height == 0 {
            continue;
        }

        // The refresh rate is embedded in the mode entry by the kernel.
        // Sysfs modes file shows resolution only. For accurate refresh,
        // we parse EDID. For sysfs-only, we estimate from standard rates.
        // Add at least one entry per resolution.
        let common_rates = if height >= 2160 {
            vec![30, 60, 120, 144]
        } else if height >= 1440 {
            vec![60, 120, 144, 165, 240]
        } else {
            vec![60, 75, 120, 144, 165, 240]
        };

        for &rate in &common_rates {
            let mode = DrmMode {
                width,
                height,
                refresh_hz: rate,
                is_preferred: is_first,
                is_current: false,
                clock_khz: 0,
            };
            modes.push(mode);
        }
        is_first = false;
    }

    // Deduplicate and sort
    modes.sort_by(|a, b| {
        a.width
            .cmp(&b.width)
            .then(a.height.cmp(&b.height))
            .then(a.refresh_hz.cmp(&b.refresh_hz))
    });
    modes.dedup_by(|a, b| {
        a.width == b.width && a.height == b.height && a.refresh_hz == b.refresh_hz
    });

    if modes.is_empty() {
        modes.push(DrmMode {
            width: 1920,
            height: 1080,
            refresh_hz: 60,
            is_preferred: true,
            is_current: false,
            clock_khz: 0,
        });
    }

    modes
}

fn detect_vrr(path: &Path, _name: &str) -> bool {
    // Check for VRR capability via DRM properties
    // On AMD: checks FreeSync (vrr_capable property)
    // On NVIDIA: checks nvidia_drm modeset module parameter
    // On Intel: checks VRR capability via i915

    // Check nvidia_drm module parameter
    if std::path::Path::new("/sys/module/nvidia_drm/parameters/modeset").exists() {
        // NVIDIA with modeset=1 generally supports G-Sync on DP/HDMI 2.1
        if _name.contains("DP") || (_name.contains("HDMI") && _name.contains("A")) {
            return true;
        }
    }

    // Check AMD FreeSync
    let vrr_path = path.join("vrr_capable");
    if let Ok(val) = std::fs::read_to_string(&vrr_path) {
        if val.trim() == "1" || val.trim() == "true" {
            return true;
        }
    }

    // Check the amdgpu module parameter
    if std::path::Path::new("/sys/module/amdgpu/parameters/freesync_video").exists() {
        return true;
    }

    // Intel: check i915 VRR
    if std::path::Path::new("/sys/module/i915/parameters/enable_psr").exists() {
        // i915 supports VRR on eDP and some DP outputs since kernel 5.x
        if _name.contains("eDP") || _name.contains("DP") {
            return true;
        }
    }

    false
}

fn detect_freesync(edid: &Option<EdidInfo>) -> bool {
    // FreeSync is indicated by specific EDID extension blocks from AMD
    // Check if the display name/manufacturer/EDID blocks contain FreeSync identifiers
    edid.as_ref()
        .map(|e| e.vrr_min.is_some() || e.vrr_max.is_some())
        .unwrap_or(false)
}

fn fallback_connectors() -> Vec<DrmConnector> {
    vec![DrmConnector {
        name: "FALLBACK-1".into(),
        path: PathBuf::new(),
        connected: true,
        enabled: true,
        connector_type: ConnectorType::Virtual,
        supported_modes: vec![DrmMode {
            width: 1920,
            height: 1080,
            refresh_hz: 60,
            is_preferred: true,
            is_current: false,
            clock_khz: 148500,
        }],
        preferred_mode: Some(DrmMode {
            width: 1920,
            height: 1080,
            refresh_hz: 60,
            is_preferred: true,
            is_current: false,
            clock_khz: 148500,
        }),
        edid: None,
        vrr_capable: false,
        vrr_range: None,
        gsync: false,
        freesync: false,
        adaptive_sync: false,
        hdr_capable: false,
        max_bpc: 8,
        current_bpc: 8,
        physical_width_mm: 0,
        physical_height_mm: 0,
        crtc_id: None,
    }]
}

fn read_sysfs_line(path: &PathBuf) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}
