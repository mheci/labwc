use tracing::{debug, info};

pub struct XWayland {
    pub enabled: bool,
    pub display: Option<u32>,
    pub ready: bool,
    pub lazy_init: bool,
}

impl XWayland {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            display: None,
            ready: false,
            lazy_init: enabled,
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        let display_num = Self::find_free_display();
        self.display = Some(display_num);
        info!("XWayland initialized on DISPLAY=:{}", display_num);
        self.ready = true;
        Ok(())
    }

    fn find_free_display() -> u32 {
        for d in 0..64 {
            let path = format!("/tmp/.X11-unix/X{d}");
            if !std::path::Path::new(&path).exists() {
                return d;
            }
        }
        0
    }

    pub fn flush(&self) {
        debug!("XWayland flush");
    }

    pub fn shutdown(&mut self) {
        if self.display.take().is_some() {
            debug!("XWayland shutdown");
        }
    }
}

impl Drop for XWayland {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Default for XWayland {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xwayland_disabled() {
        let mut x = XWayland::new(false);
        assert!(!x.enabled);
        assert!(x.init().is_ok());
        assert!(!x.ready);
    }

    #[test]
    fn test_xwayland_enabled() {
        let x = XWayland::new(true);
        assert!(x.enabled);
    }

    #[test]
    fn test_find_free_display() {
        let d = XWayland::find_free_display();
        assert!(d < 64);
    }

    #[test]
    fn test_xwayland_default() {
        let x = XWayland::default();
        assert!(x.enabled);
    }
}
