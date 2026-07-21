pub mod compositor;
pub mod swapchain;
pub mod sync;

use labwc_backend::Backend;

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
    pub driver_version: u32,
    pub api_version: u32,
    pub timeline_semaphore: bool,
    pub dynamic_rendering: bool,
    pub synchronization2: bool,
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
    #[error("swapchain needs rebuild")]
    NeedsRebuild,
    #[error("gpu: {0}")]
    Gpu(String),
}

pub struct OutputInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

impl Renderer {
    pub fn create(backend: &Backend, outputs: &[OutputInfo]) -> Result<Self, RenderError> {
        let nvidia = backend.is_nvidia();
        let gpu_name = backend
            .gpu
            .as_ref()
            .map(|g| g.drm_driver_name.clone())
            .unwrap_or_else(|| "unknown".into());

        let vendor = backend
            .gpu
            .as_ref()
            .map(|g| match g.vendor {
                labwc_backend::GpuVendor::Nvidia => GpuVendor::Nvidia,
                labwc_backend::GpuVendor::Amd => GpuVendor::Amd,
                labwc_backend::GpuVendor::Intel => GpuVendor::Intel,
                labwc_backend::GpuVendor::Other => GpuVendor::Other,
            })
            .unwrap_or(GpuVendor::Other);

        info!(
            "Vulkan renderer: {} {} (NVIDIA path: {})",
            vendor.name(),
            gpu_name,
            nvidia
        );

        let gpu = GpuInfo {
            vendor,
            name: gpu_name,
            driver_version: 0,
            api_version: 0x00403000,
            timeline_semaphore: nvidia || true,
            dynamic_rendering: true,
            synchronization2: nvidia || true,
            nvidia,
        };

        let mut out_swapchains = Vec::new();
        for output in outputs {
            let mode = PresentMode::best_for_refresh(output.refresh_rate, false);
            let config = SwapchainConfig {
                output_name: output.name.clone(),
                width: output.width,
                height: output.height,
                refresh_rate: output.refresh_rate,
                present_mode: mode,
                image_count: if mode == PresentMode::Mailbox { 3 } else { 2 },
                gsync_compatible: false,
            };

            let sc = OutputSwapchain::create(&config);
            info!(
                "  {}: {}x{} @ {}Hz {:?} ({} images)",
                output.name,
                output.width,
                output.height,
                output.refresh_rate,
                mode,
                config.image_count
            );
            out_swapchains.push(sc);
        }

        Ok(Self {
            renderer_type: RendererType::Vulkan,
            gpu,
            outputs: out_swapchains,
            frame_sync: MultiOutputSync::new(outputs.len()),
            frame_counter: 0,
            current_timeline: 0,
        })
    }

    pub fn begin_frame(&mut self, output_idx: usize) -> Result<(), RenderError> {
        let sc = self
            .outputs
            .get_mut(output_idx)
            .ok_or(RenderError::NoOutput(output_idx))?;
        sc.begin_frame();
        Ok(())
    }

    pub fn end_frame(&mut self, output_idx: usize) -> Result<(), RenderError> {
        let sc = self
            .outputs
            .get_mut(output_idx)
            .ok_or(RenderError::NoOutput(output_idx))?;
        sc.end_frame();
        Ok(())
    }

    pub fn present(&mut self, output_idx: usize) -> Result<(), RenderError> {
        self.current_timeline += 1;
        let sc = self
            .outputs
            .get_mut(output_idx)
            .ok_or(RenderError::NoOutput(output_idx))?;
        sc.present();
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

    pub fn needs_repaint(&self, output_idx: usize) -> bool {
        self.outputs
            .get(output_idx)
            .map(|sc| sc.needs_repaint())
            .unwrap_or(false)
    }

    pub fn frame_number(&self) -> u64 {
        self.frame_counter
    }

    pub fn stats(&self, output_idx: usize) -> Option<SwapchainStats> {
        self.outputs.get(output_idx).map(|sc| sc.stats())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        info!(
            "Renderer shutdown: {} frames across {} outputs",
            self.frame_counter,
            self.outputs.len()
        );
    }
}
