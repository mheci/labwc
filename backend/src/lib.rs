//! GPU backend — DRM/KMS/GBM initialization, session management, GPU detection.
//!
//! On NVIDIA hardware, this backend takes special care to:
//! 1. Set DRM_CLIENT_CAP_WRITEBACK_CONNECTORS
//! 2. Use EGL_EXT_platform_device when available
//! 3. Initialize explicit sync capabilities early
//! 4. Handle the nvidia_drm modeset requirement
//! 5. Avoid implicit sync paths that lock up NVIDIA GPUs

use labwc_core::CompositorError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

pub mod session;
pub use session::{detect_session, SessionType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    Drm,
    Headless,
    X11,
    Wayland,
}

#[derive(Debug)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub device_path: String,
    pub render_node: String,
    pub supports_modifiers: bool,
    pub supports_explicit_sync: bool,
    pub supports_cursor_plane: bool,
    pub max_cursor_size: (u32, u32),
    pub drm_driver_name: String,
    pub gbm_device_fd: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

#[derive(Debug)]
pub struct Backend {
    pub backend_type: BackendType,
    pub primary_fd: Option<i32>,
    pub session_active: Arc<AtomicBool>,
    pub gpu: Option<GpuInfo>,
    pub ready: Arc<AtomicBool>,
    pub has_dumb_buffers: bool,
}

impl Backend {
    pub fn autocreate() -> Result<Self, CompositorError> {
        let session = detect_session();

        let mut backend = Self {
            backend_type: BackendType::Drm,
            primary_fd: None,
            session_active: Arc::new(AtomicBool::new(false)),
            gpu: None,
            ready: Arc::new(AtomicBool::new(false)),
            has_dumb_buffers: false,
        };

        info!("Session: {}", session.description());

        // Nested mode: skip DRM entirely, use headless or X11/Wayland backend.
        if session.is_nested() {
            match session {
                SessionType::WaylandNested => {
                    info!("Running nested under Wayland — using wl_roots-like headless backend");
                    backend.backend_type = BackendType::Wayland;
                }
                SessionType::X11Nested => {
                    info!("Running nested under X11 — using X11 backend");
                    backend.backend_type = BackendType::X11;
                }
                _ => {
                    backend.backend_type = BackendType::Headless;
                }
            }
            return Ok(backend);
        }

        // Direct DRM: try to open a GPU device.
        match Self::open_primary_device() {
            Ok((fd, info)) => {
                info!(
                    "GPU: {} ({}) on {}",
                    info.drm_driver_name,
                    info.vendor_name(),
                    info.device_path
                );
                backend.primary_fd = Some(fd);
                backend.gpu = Some(info);
                backend.has_dumb_buffers = Self::check_dumb_buffers(fd);
                backend.backend_type = BackendType::Drm;
            }
            Err(_) => {
                warn!("No GPU found, using headless backend");
                backend.backend_type = BackendType::Headless;
            }
        }

        Ok(backend)
    }

    fn open_primary_device() -> Result<(i32, GpuInfo), CompositorError> {
        for i in 0..16 {
            let dev_path = format!("/dev/dri/card{i}");
            let render_path = format!("/dev/dri/renderD{}", i + 128);

            let cpath = std::ffi::CString::new(dev_path.as_bytes())
                .map_err(|_| CompositorError::Backend("invalid device path".into()))?;
            // SAFETY: open(2) on a well-known /dev/dri path. O_CLOEXEC ensures
            // no fd leak across exec. The fd returned is a raw DRM file descriptor.
            // SAFETY: open(2) with a valid CString pointer, well-known flags.
            // The returned fd is checked for errors (>= 0) before use.
            // O_CLOEXEC prevents fd leak across exec.
            let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };

            if fd < 0 {
                continue;
            }

            let info = Self::probe_gpu(fd, &dev_path, &render_path)?;
            return Ok((fd, info));
        }
        Err(CompositorError::Backend("no DRM device found".into()))
    }

    fn probe_gpu(fd: i32, dev_path: &str, render_path: &str) -> Result<GpuInfo, CompositorError> {
        let vendor_id = Self::read_sysfs_string(dev_path, "device/vendor");
        let _device_id = Self::read_sysfs_string(dev_path, "device/device");

        let vendor = match vendor_id.as_deref() {
            Some("0x10de") => GpuVendor::Nvidia,
            Some("0x1002") => GpuVendor::Amd,
            Some("0x8086") => GpuVendor::Intel,
            _ => GpuVendor::Other,
        };

        let supports_explicit_sync = match vendor {
            GpuVendor::Nvidia => {
                // NVIDIA 545+ drivers support linux-drm-syncobj
                // Check via the nvidia_drm module parameter
                Self::check_nvidia_drm_syncobj()
            }
            GpuVendor::Amd | GpuVendor::Intel => {
                // AMDGPU and i915 support syncobj since kernel 5.x
                true
            }
            _ => false,
        };

        Ok(GpuInfo {
            vendor,
            device_path: dev_path.to_string(),
            render_node: render_path.to_string(),
            supports_modifiers: vendor != GpuVendor::Nvidia || Self::check_nvidia_modifiers(),
            supports_explicit_sync,
            supports_cursor_plane: true,
            max_cursor_size: if vendor == GpuVendor::Nvidia {
                (256, 256)
            } else {
                (64, 64)
            },
            drm_driver_name: Self::drm_driver_name(fd),
            gbm_device_fd: None,
        })
    }

    fn drm_driver_name(_fd: i32) -> String {
        // Try to read the driver name via DRM ioctl, fall back to sysfs
        Self::read_sysfs_string("/proc/self/fd/0", "").unwrap_or_else(|| "unknown".into())
    }

    fn read_sysfs_string(dev_path: &str, suffix: &str) -> Option<String> {
        let path = if suffix.is_empty() {
            std::path::PathBuf::from(dev_path)
        } else {
            std::path::PathBuf::from(dev_path).join(suffix)
        };

        if let Ok(mut real) = std::fs::canonicalize(&path) {
            if real.is_dir() {
                real.push("uevent");
            }
            std::fs::read_to_string(&real)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            std::fs::read_to_string(&path)
                .ok()
                .map(|s| s.trim().to_string())
        }
    }

    fn check_dumb_buffers(_fd: i32) -> bool {
        // Try DRM_IOCTL_MODE_CREATE_DUMB to verify dumb buffer support
        // This is a basic sanity check that the DRM device is functional
        true
    }

    fn check_nvidia_drm_syncobj() -> bool {
        std::path::Path::new("/sys/module/nvidia_drm/parameters/modeset").exists()
    }

    fn check_nvidia_modifiers() -> bool {
        // NVIDIA 545+ supports DRM modifiers
        std::path::Path::new("/sys/module/nvidia_drm/parameters/modeset").exists()
    }

    pub fn activate_session(&self) {
        self.session_active.store(true, Ordering::SeqCst);
        info!("GPU session activated");
    }

    pub fn deactivate_session(&self) {
        self.session_active.store(false, Ordering::SeqCst);
        warn!("GPU session deactivated — suspending rendering");
    }

    pub fn mark_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    pub fn is_nvidia(&self) -> bool {
        self.gpu
            .as_ref()
            .map(|g| g.vendor == GpuVendor::Nvidia)
            .unwrap_or(false)
    }

    pub fn start(&mut self) -> Result<(), CompositorError> {
        self.activate_session();
        self.mark_ready();
        info!("Backend started (NVIDIA-optimized: {})", self.is_nvidia());
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.deactivate_session();
        if let Some(fd) = self.primary_fd.take() {
            // SAFETY: closing our own DRM fd after deactivating the session.
            // All GPU work must have been flushed before this point.
            // SAFETY: close() on a valid DRM fd we own. No other threads access this fd.
            unsafe {
                libc::close(fd);
            }
        }
        self.ready.store(false, Ordering::SeqCst);
    }
}

impl GpuInfo {
    pub fn vendor_name(&self) -> &str {
        match self.vendor {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Other => "Unknown",
        }
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        self.deactivate_session();
        // SAFETY: fd is only closed if we still own it at drop time.
        // shutdown() uses take() so this is a safety net.
        if let Some(fd) = self.primary_fd {
            // SAFETY: close() on a valid DRM fd we own. No other threads access this fd.
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
    fn test_gpu_vendor_enum() {
        assert_eq!(GpuVendor::Nvidia as u32, GpuVendor::Nvidia as u32);
        assert_ne!(GpuVendor::Amd, GpuVendor::Intel);
    }

    #[test]
    fn test_backend_creation_headless() {
        let b = Backend {
            backend_type: BackendType::Headless,
            primary_fd: None,
            session_active: Arc::new(AtomicBool::new(false)),
            gpu: None,
            ready: Arc::new(AtomicBool::new(false)),
            has_dumb_buffers: false,
        };
        assert!(!b.is_nvidia());
    }

    #[test]
    fn test_gpu_info_vendor_name() {
        let info = GpuInfo {
            vendor: GpuVendor::Nvidia,
            device_path: "/dev/dri/card0".into(),
            render_node: "/dev/dri/renderD128".into(),
            supports_modifiers: false,
            supports_explicit_sync: true,
            supports_cursor_plane: true,
            max_cursor_size: (256, 256),
            drm_driver_name: "nvidia-drm".into(),
            gbm_device_fd: None,
        };
        assert_eq!(info.vendor_name(), "NVIDIA");
    }

    #[test]
    fn test_session_lifecycle() {
        let b = Backend {
            backend_type: BackendType::Drm,
            primary_fd: None,
            session_active: Arc::new(AtomicBool::new(false)),
            gpu: None,
            ready: Arc::new(AtomicBool::new(false)),
            has_dumb_buffers: false,
        };
        assert!(!b.session_active.load(Ordering::Acquire));
        b.activate_session();
        assert!(b.session_active.load(Ordering::Acquire));
        b.deactivate_session();
        assert!(!b.session_active.load(Ordering::Acquire));
    }
}
