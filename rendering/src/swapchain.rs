use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    Fifo,
    Mailbox,
    Immediate,
    FifoRelaxed,
}

impl PresentMode {
    pub fn best_for_refresh(hz: u32, gsync: bool) -> Self {
        match (hz, gsync) {
            (r, true) if r >= 120 => PresentMode::Immediate,
            (r, _) if r >= 120 => PresentMode::Mailbox,
            _ => PresentMode::Fifo,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapchainConfig {
    pub output_name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub present_mode: PresentMode,
    pub image_count: u32,
    pub gsync_compatible: bool,
}

#[derive(Debug)]
pub struct SwapchainImage {
    pub index: u32,
    pub width: u32,
    pub height: u32,
    pub in_flight: bool,
    pub acquired_at: Option<Instant>,
}

pub struct OutputSwapchain {
    pub config: SwapchainConfig,
    pub present_mode: PresentMode,
    pub images: Vec<SwapchainImage>,
    pub current_image: u32,
    pub frame_period: Duration,
    pub last_present: Instant,
    pub frames_presented: u64,
    pub frames_dropped: u64,
    pub needs_rebuild: bool,
    pub frame_in_flight: bool,
}

impl OutputSwapchain {
    pub fn create(config: &SwapchainConfig) -> Self {
        let frame_ns = 1_000_000_000u64 / config.refresh_rate.max(1) as u64;
        let image_count = config.image_count.clamp(2, 8);

        let mut images = Vec::with_capacity(image_count as usize);
        for i in 0..image_count {
            images.push(SwapchainImage {
                index: i,
                width: config.width,
                height: config.height,
                in_flight: false,
                acquired_at: None,
            });
        }

        Self {
            config: config.clone(),
            present_mode: config.present_mode,
            images,
            current_image: 0,
            frame_period: Duration::from_nanos(frame_ns),
            last_present: Instant::now(),
            frames_presented: 0,
            frames_dropped: 0,
            needs_rebuild: false,
            frame_in_flight: false,
        }
    }

    pub fn begin_frame(&mut self) {
        if self.frame_in_flight {
            self.frames_dropped += 1;
            debug!(
                "{}: frame dropped (previous still in flight)",
                self.config.output_name
            );
            return;
        }

        self.current_image = (self.current_image + 1) % self.images.len() as u32;

        if let Some(img) = self.images.get_mut(self.current_image as usize) {
            img.in_flight = true;
            img.acquired_at = Some(Instant::now());
        }

        self.frame_in_flight = true;
    }

    pub fn end_frame(&mut self) {
        if let Some(img) = self.images.get(self.current_image as usize) {
            if let Some(t) = img.acquired_at {
                if t.elapsed() > self.frame_period * 2 {
                    debug!(
                        "{}: frame late ({:?} > {:?})",
                        self.config.output_name,
                        t.elapsed(),
                        self.frame_period
                    );
                }
            }
        }
    }

    pub fn present(&mut self) {
        if let Some(img) = self.images.get_mut(self.current_image as usize) {
            img.in_flight = false;
        }
        self.frame_in_flight = false;
        self.frames_presented = self.frames_presented.wrapping_add(1);
        self.last_present = Instant::now();
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == self.config.width && h == self.config.height {
            return;
        }
        debug!(
            "{}: resize {}x{} -> {}x{}",
            self.config.output_name, self.config.width, self.config.height, w, h
        );
        self.config.width = w;
        self.config.height = h;
        for img in &mut self.images {
            img.width = w;
            img.height = h;
        }
        self.needs_rebuild = true;
    }

    pub fn rebuild_complete(&mut self) {
        self.needs_rebuild = false;
    }

    pub fn needs_repaint(&self) -> bool {
        !self.frame_in_flight || self.needs_rebuild
    }

    pub fn stats(&self) -> SwapchainStats {
        SwapchainStats {
            output_name: self.config.output_name.clone(),
            refresh_rate: self.config.refresh_rate,
            present_mode: self.present_mode,
            frames_presented: self.frames_presented,
            frames_dropped: self.frames_dropped,
            image_count: self.images.len() as u32,
            frame_period_us: self.frame_period.as_micros() as u64,
            in_flight: self.frame_in_flight,
            needs_rebuild: self.needs_rebuild,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapchainStats {
    pub output_name: String,
    pub refresh_rate: u32,
    pub present_mode: PresentMode,
    pub frames_presented: u64,
    pub frames_dropped: u64,
    pub image_count: u32,
    pub frame_period_us: u64,
    pub in_flight: bool,
    pub needs_rebuild: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swapchain_creation() {
        let config = SwapchainConfig {
            output_name: "test".into(),
            width: 1920,
            height: 1080,
            refresh_rate: 144,
            present_mode: PresentMode::Mailbox,
            image_count: 3,
            gsync_compatible: true,
        };
        let sc = OutputSwapchain::create(&config);
        assert_eq!(sc.images.len(), 3);
        assert_eq!(sc.present_mode, PresentMode::Mailbox);
    }

    #[test]
    fn test_frame_cycle() {
        let config = SwapchainConfig {
            output_name: "test".into(),
            width: 800,
            height: 600,
            refresh_rate: 60,
            present_mode: PresentMode::Fifo,
            image_count: 2,
            gsync_compatible: false,
        };
        let mut sc = OutputSwapchain::create(&config);
        assert!(!sc.frame_in_flight);
        sc.begin_frame();
        assert!(sc.frame_in_flight);
        sc.end_frame();
        sc.present();
        assert!(!sc.frame_in_flight);
        assert_eq!(sc.frames_presented, 1);
    }

    #[test]
    fn test_drop_frame() {
        let config = SwapchainConfig {
            output_name: "test".into(),
            width: 800,
            height: 600,
            refresh_rate: 60,
            present_mode: PresentMode::Fifo,
            image_count: 2,
            gsync_compatible: false,
        };
        let mut sc = OutputSwapchain::create(&config);
        sc.begin_frame();
        sc.begin_frame(); // second begin while frame in flight = drop
        assert_eq!(sc.frames_dropped, 1);
    }

    #[test]
    fn test_resize() {
        let config = SwapchainConfig {
            output_name: "test".into(),
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            present_mode: PresentMode::Fifo,
            image_count: 2,
            gsync_compatible: false,
        };
        let mut sc = OutputSwapchain::create(&config);
        sc.resize(3840, 2160);
        assert!(sc.needs_rebuild);
        assert_eq!(sc.config.width, 3840);
        assert_eq!(sc.config.height, 2160);
    }
}
