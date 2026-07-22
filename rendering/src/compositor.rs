use labwc_core::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    Background = 0,
    Bottom = 1,
    Normal = 2,
    Top = 3,
    Overlay = 4,
    Popup = 5,
    LockScreen = 6,
    Debug = 7,
}

impl RenderLayer {
    pub fn all() -> [RenderLayer; 8] {
        [
            RenderLayer::Background,
            RenderLayer::Bottom,
            RenderLayer::Normal,
            RenderLayer::Top,
            RenderLayer::Overlay,
            RenderLayer::Popup,
            RenderLayer::LockScreen,
            RenderLayer::Debug,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct TextureDrawCommand {
    pub texture_id: u64,
    pub src_rect: Rect,
    pub dst_rect: Rect,
    pub alpha: f32,
    pub color_tint: Option<[f32; 4]>,
    pub blur_radius: f32,
}

impl TextureDrawCommand {
    pub fn new(texture_id: u64, dst: Rect, src: Rect) -> Self {
        Self {
            texture_id,
            src_rect: src,
            dst_rect: dst,
            alpha: 1.0,
            color_tint: None,
            blur_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerDrawCommand {
    pub layer: RenderLayer,
    pub rects: Vec<(Rect, [f32; 4])>,
    pub textures: Vec<TextureDrawCommand>,
}

#[derive(Debug, Clone)]
pub struct CompositorPass {
    pub output_idx: usize,
    pub output_width: u32,
    pub output_height: u32,
    pub clear_color: [f32; 4],
    pub layers: Vec<LayerDrawCommand>,
    pub needs_render: bool,
}

impl CompositorPass {
    pub fn new(output_idx: usize, width: u32, height: u32) -> Self {
        Self {
            output_idx,
            output_width: width,
            output_height: height,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            layers: Vec::new(),
            needs_render: false,
        }
    }

    pub fn begin(&mut self) {
        self.layers.clear();
        self.needs_render = false;
    }

    pub fn add_rect(&mut self, layer: RenderLayer, rect: Rect, color: [f32; 4]) {
        if let Some(lc) = self.layers.iter_mut().find(|l| l.layer == layer) {
            lc.rects.push((rect, color));
        } else {
            self.layers.push(LayerDrawCommand {
                layer,
                rects: vec![(rect, color)],
                textures: vec![],
            });
        }
        self.needs_render = true;
    }

    pub fn add_texture(&mut self, layer: RenderLayer, cmd: TextureDrawCommand) {
        if let Some(lc) = self.layers.iter_mut().find(|l| l.layer == layer) {
            lc.textures.push(cmd);
        } else {
            self.layers.push(LayerDrawCommand {
                layer,
                rects: vec![],
                textures: vec![cmd],
            });
        }
        self.needs_render = true;
    }

    pub fn sort_layers(&mut self) {
        self.layers.sort_by_key(|l| l.layer);
    }

    pub fn total_draws(&self) -> usize {
        self.layers
            .iter()
            .map(|l| l.rects.len() + l.textures.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ordering() {
        assert!(RenderLayer::Background < RenderLayer::Normal);
        assert!(RenderLayer::Top < RenderLayer::Overlay);
    }

    #[test]
    fn test_compositor_pass() {
        let mut pass = CompositorPass::new(0, 1920, 1080);
        pass.begin();
        pass.add_rect(RenderLayer::Normal, Rect::new(100, 100, 200, 200), [0.5; 4]);
        assert!(pass.needs_render);
        assert_eq!(pass.total_draws(), 1);
    }

    #[test]
    fn test_texture_command() {
        let cmd = TextureDrawCommand::new(42, Rect::new(0, 0, 100, 100), Rect::new(0, 0, 200, 200));
        assert_eq!(cmd.texture_id, 42);
        assert!((cmd.alpha - 1.0).abs() < f32::EPSILON);
    }
}
