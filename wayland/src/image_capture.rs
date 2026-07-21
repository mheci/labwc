//! Implementation of `ext_image_capture_source_v1`.
//!
//! Provides screen capture functionality — clients can request
//! image captures of outputs at specific resolutions and formats.

use tracing::{debug, error, info, warn};

/// Image capture format negotiation.
#[derive(Debug, Clone)]
pub struct CaptureRequest {
    /// Requested capture width.
    pub width: i32,
    /// Requested capture height.
    pub height: i32,
    /// DRM fourcc format code.
    pub format: u32,
    /// DRM format modifier.
    pub modifier: u64,
    /// Whether the capture is still pending.
    pub pending: bool,
}

impl CaptureRequest {
    /// Create a new capture request.
    #[must_use]
    pub fn new(width: i32, height: i32, format: u32, modifier: u64) -> Self {
        if width <= 0 || height <= 0 {
            warn!(width, height, "Invalid capture dimensions requested");
        }
        Self {
            width,
            height,
            format,
            modifier,
            pending: true,
        }
    }
}

/// Image capture source state.
#[derive(Debug)]
pub struct CaptureSource {
    /// The output being captured, if source is output-specific.
    pub output_id: Option<u64>,
    /// Active capture request.
    pub current_request: Option<CaptureRequest>,
}

impl CaptureSource {
    /// Create a new capture source.
    #[must_use]
    pub fn new(output_id: Option<u64>) -> Self {
        Self {
            output_id,
            current_request: None,
        }
    }

    /// Start a new capture request.
    pub fn create_capture(&mut self, req: CaptureRequest) -> bool {
        if req.width <= 0 || req.height <= 0 {
            error!(?req, "Rejecting capture with invalid dimensions");
            return false;
        }
        debug!(?req, "Capture request created");
        self.current_request = Some(req);
        true
    }

    /// Mark the current capture as completed.
    pub fn complete(&mut self) {
        if let Some(ref mut req) = self.current_request {
            req.pending = false;
        }
    }
}

/// Register the `ext_image_capture_source_manager_v1` global.
pub fn register() {
    info!("Registered ext_image_capture_source_manager_v1");
}
