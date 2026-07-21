use labwc_core::CompositorError;
use tracing::{info, warn};

pub struct Backend {
    pub backend_type: BackendType,
    pub drm_fd: Option<i32>,
    pub session_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    Drm,
    Headless,
    X11,
    Wayland,
}

impl Backend {
    pub fn autocreate() -> Result<Self, CompositorError> {
        if let Ok(fd) = Self::open_drm_device() {
            info!("DRM backend initialized");
            return Ok(Self {
                backend_type: BackendType::Drm,
                drm_fd: Some(fd),
                session_active: true,
            });
        }
        warn!("No DRM device found, using headless backend");
        Ok(Self {
            backend_type: BackendType::Headless,
            drm_fd: None,
            session_active: true,
        })
    }

    fn open_drm_device() -> Result<i32, CompositorError> {
        for i in 0..16 {
            let path = format!("/dev/dri/card{i}");
            let fd =
                unsafe { libc::open(path.as_ptr() as *const i8, libc::O_RDWR | libc::O_CLOEXEC) };
            if fd >= 0 {
                return Ok(fd);
            }
        }
        Err(CompositorError::Backend("no DRM device found".into()))
    }

    pub fn start(&mut self) -> Result<(), CompositorError> {
        info!("Backend started");
        Ok(())
    }

    pub fn shutdown(&mut self) {
        if let Some(fd) = self.drm_fd.take() {
            unsafe {
                libc::close(fd);
            }
        }
    }
}

impl Drop for Backend {
    // SAFETY: shutdown() already called take() so fd is None if we've been shut down.
    // Only close if still open (normal drop without explicit shutdown).
    fn drop(&mut self) {
        if let Some(fd) = self.drm_fd {
            unsafe {
                libc::close(fd);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_clone() {
        let bt = BackendType::Drm;
        assert_eq!(bt, BackendType::Drm);
        assert_ne!(bt, BackendType::Headless);
    }

    #[test]
    fn test_backend_creation() {
        let backend = Backend {
            backend_type: BackendType::Headless,
            drm_fd: None,
            session_active: true,
        };
        assert!(backend.session_active);
    }
}
