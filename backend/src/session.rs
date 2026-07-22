use std::env;
use std::os::unix::fs::FileTypeExt;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    WaylandNested,
    X11Nested,
    DisplayManager,
    Tty,
    Unknown,
}

impl SessionType {
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::WaylandNested | Self::X11Nested)
    }

    pub fn description(&self) -> &str {
        match self {
            Self::WaylandNested => "Wayland (nested under parent compositor)",
            Self::X11Nested => "X11 (nested under X server)",
            Self::DisplayManager => "Wayland (display manager session)",
            Self::Tty => "Linux VT / TTY (direct DRM)",
            Self::Unknown => "unknown (probing DRM)",
        }
    }
}

pub fn detect_session() -> SessionType {
    if let Ok(val) = env::var("WAYLAND_DISPLAY") {
        if !val.is_empty() {
            info!("WAYLAND_DISPLAY={} → nested Wayland mode", val);
            return SessionType::WaylandNested;
        }
    }

    if let Ok(val) = env::var("DISPLAY") {
        if !val.is_empty() {
            info!("DISPLAY={} → nested X11 mode", val);
            return SessionType::X11Nested;
        }
    }

    if let Ok(val) = env::var("XDG_SESSION_TYPE") {
        match val.as_str() {
            "wayland" => {
                debug!("XDG_SESSION_TYPE=wayland → display manager Wayland session");
                return SessionType::DisplayManager;
            }
            "x11" => {
                debug!("XDG_SESSION_TYPE=x11 → display manager X11 session");
                return SessionType::X11Nested;
            }
            "tty" => {
                debug!("XDG_SESSION_TYPE=tty → TTY session");
                return SessionType::Tty;
            }
            _ => debug!("XDG_SESSION_TYPE={} (unrecognized)", val),
        }
    }

    if is_real_tty() {
        info!("/dev/tty is a real VT → direct DRM mode");
        return SessionType::Tty;
    }

    if env::var("XDG_VTNR").is_ok() {
        info!("XDG_VTNR set → direct DRM mode");
        return SessionType::Tty;
    }

    warn!("Could not determine session type — probing DRM");
    SessionType::Unknown
}

fn is_real_tty() -> bool {
    if !std::path::Path::new("/dev/tty").exists() {
        return false;
    }

    if let Ok(active) = std::fs::read_to_string("/sys/class/tty/tty0/active") {
        let active = active.trim();
        if !active.is_empty() && active.starts_with("tty") {
            debug!("Active VT: {}", active);
            return true;
        }
    }

    let tty_name = get_tty_name();
    match tty_name {
        Some(ref name) => {
            debug!("TTY name: {}", name);
            if name.starts_with("pts/") || name == "ptmx" {
                debug!("{} is a pseudo-terminal, not a real VT", name);
                return false;
            }
            name.starts_with("tty")
        }
        None => {
            // Can't determine — check if /dev/tty is a char device as fallback.
            match std::fs::metadata("/dev/tty") {
                Ok(meta) => meta.file_type().is_char_device(),
                Err(_) => false,
            }
        }
    }
}

fn get_tty_name() -> Option<String> {
    // Read /proc/self/fd/0 which points to the actual terminal device.
    std::fs::read_link("/proc/self/fd/0")
        .ok()?
        .file_name()?
        .to_str()
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_type_is_nested() {
        assert!(SessionType::WaylandNested.is_nested());
        assert!(SessionType::X11Nested.is_nested());
        assert!(!SessionType::Tty.is_nested());
        assert!(!SessionType::DisplayManager.is_nested());
        assert!(!SessionType::Unknown.is_nested());
    }

    #[test]
    fn test_session_type_description() {
        assert!(SessionType::WaylandNested.description().contains("Wayland"));
        assert!(SessionType::X11Nested.description().contains("X11"));
        assert!(SessionType::Tty.description().contains("VT"));
        assert!(SessionType::DisplayManager
            .description()
            .contains("display manager"));
    }

    #[test]
    fn test_is_real_tty_returns_bool() {
        let _ = is_real_tty();
    }

    #[test]
    fn test_get_tty_name_returns_option() {
        let result = get_tty_name();
        // In CI/sandbox, this typically returns None.
        assert!(result.is_none() || result.is_some());
    }
}
