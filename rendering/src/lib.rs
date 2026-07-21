use labwc_core::{CompositorError, Rect};
use tracing::debug;

pub struct Renderer {
    pub renderer_type: RendererType,
    pub gpu_name: String,
    pub nvidia_optimized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
    Auto,
    Egl,
    Vulkan,
    Pixman,
}

pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub transform: OutputTransform,
    pub scale: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTransform {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl Renderer {
    pub fn autocreate() -> Result<Self, CompositorError> {
        let renderer_type = if std::env::var("WLR_RENDERER").as_deref() == Ok("pixman") {
            RendererType::Pixman
        } else {
            RendererType::Auto
        };
        Ok(Self {
            renderer_type,
            gpu_name: Self::detect_gpu(),
            nvidia_optimized: Self::detect_nvidia(),
        })
    }

    fn detect_gpu() -> String {
        std::fs::read_to_string("/sys/class/drm/card0/device/vendor")
            .ok()
            .map(|v| match v.trim() {
                "0x10de" => "NVIDIA".into(),
                "0x1002" => "AMD".into(),
                "0x8086" => "Intel".into(),
                vendor => format!("vendor:{vendor}"),
            })
            .unwrap_or_else(|| "unknown".into())
    }

    fn detect_nvidia() -> bool {
        std::path::Path::new("/sys/module/nvidia").exists()
            || std::path::Path::new("/sys/module/nvidia_drm").exists()
    }

    pub fn begin_frame(&mut self, ctx: &RenderContext) -> Result<(), CompositorError> {
        debug!(
            "Begin frame {}x{} (scale={})",
            ctx.width, ctx.height, ctx.scale
        );
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), CompositorError> {
        Ok(())
    }

    pub fn clear(&mut self, color: [f32; 4]) {
        debug!("Clear {:?}", color);
    }

    pub fn render_rect(&mut self, rect: Rect, color: [f32; 4]) {
        debug!("Rect {:?} color={:?}", rect, color);
    }

    pub fn render_texture(&mut self, rect: Rect, texture_id: u64) {
        debug!("Texture {} at {:?}", texture_id, rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_type_values() {
        assert_ne!(RendererType::Auto as u32, RendererType::Vulkan as u32);
    }

    #[test]
    fn test_output_transform_is_copy() {
        let t = OutputTransform::Normal;
        let t2 = t;
        assert_eq!(t, t2);
    }
}
