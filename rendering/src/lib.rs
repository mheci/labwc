pub mod compositor;
pub mod swapchain;
pub mod sync;

use labwc_output::Output;
use std::sync::Arc;
use tracing::info;

pub use compositor::{CompositorPass, LayerDrawCommand, RenderLayer, TextureDrawCommand};
pub use swapchain::{OutputSwapchain, PresentMode, SwapchainConfig, SwapchainStats};
pub use sync::{FrameSync, MultiOutputSync, TimelinePoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
    Vulkan,
    Egl,
    Pixman,
}

#[derive(Debug)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub timeline_semaphore: bool,
    pub nvidia: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

impl GpuVendor {
    pub fn name(&self) -> &str {
        match self {
            Self::Nvidia => "NVIDIA",
            Self::Amd => "AMD",
            Self::Intel => "Intel",
            Self::Other => "Unknown",
        }
    }
}

pub struct Renderer {
    pub renderer_type: RendererType,
    pub gpu: GpuInfo,
    pub outputs: Vec<OutputSwapchain>,
    pub frame_sync: MultiOutputSync,
    frame_counter: u64,
    current_timeline: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("no such output: {0}")]
    NoOutput(usize),
    #[error("gpu: {0}")]
    Gpu(String),
}

impl Renderer {
    pub fn create(outputs: &[Arc<Output>]) -> Result<Self, RenderError> {
        let gpu = GpuInfo {
            vendor: GpuVendor::Other,
            name: "auto-detected".into(),
            timeline_semaphore: true,
            nvidia: false,
        };

        let mut out_scs = Vec::new();
        for o in outputs {
            let mode = PresentMode::best_for_refresh(o.max_refresh_rate, o.gsync_compatible);
            let config = SwapchainConfig {
                output_name: o.name.clone(), // string copy for swapchain identity
                width: o.width.max(1) as u32,
                height: o.height.max(1) as u32,
                refresh_rate: o.max_refresh_rate.max(60),
                present_mode: mode,
                image_count: if mode == PresentMode::Mailbox { 3 } else { 2 },
                gsync_compatible: o.gsync_compatible,
            };
            let sc = OutputSwapchain::create(&config);
            info!(
                "  {}: {}x{} @ {}Hz {:?} (#{})",
                o.name, o.width, o.height, o.max_refresh_rate, mode, config.image_count
            );
            out_scs.push(sc);
        }

        Ok(Self {
            renderer_type: RendererType::Vulkan,
            gpu,
            outputs: out_scs,
            frame_sync: MultiOutputSync::new(outputs.len()),
            frame_counter: 0,
            current_timeline: 0,
        })
    }

    pub fn begin_frame(&mut self, output_idx: usize) -> Result<(), RenderError> {
        self.outputs
            .get_mut(output_idx)
            .ok_or(RenderError::NoOutput(output_idx))?
            .begin_frame();
        Ok(())
    }

    pub fn end_frame(&mut self, output_idx: usize) -> Result<(), RenderError> {
        self.outputs
            .get_mut(output_idx)
            .ok_or(RenderError::NoOutput(output_idx))?
            .end_frame();
        Ok(())
    }

    pub fn present(&mut self, output_idx: usize) -> Result<(), RenderError> {
        self.current_timeline += 1;
        self.outputs
            .get_mut(output_idx)
            .ok_or(RenderError::NoOutput(output_idx))?
            .present();
        self.frame_sync.submit(output_idx, self.current_timeline);
        Ok(())
    }

    pub fn advance_frame(&mut self) -> u64 {
        self.frame_counter = self.frame_counter.wrapping_add(1);
        self.current_timeline += 1;
        self.frame_counter
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
    pub fn needs_repaint(&self, idx: usize) -> bool {
        self.outputs
            .get(idx)
            .map(|s| s.needs_repaint())
            .unwrap_or(false)
    }
    pub fn frame_number(&self) -> u64 {
        self.frame_counter
    }
    pub fn stats(&self, idx: usize) -> Option<SwapchainStats> {
        self.outputs.get(idx).map(|s| s.stats())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        info!(
            "Renderer shutdown: {} frames, {} outputs",
            self.frame_counter,
            self.outputs.len()
        );
    }
}
