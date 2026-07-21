//! Scene graph abstraction — tree of renderable nodes in z-order.

pub enum SceneNodeType {
    Root,
    Tree,
    Rect { color: [f32; 4] },
    Buffer,
}

pub struct SceneNode {
    pub id: u64,
    pub node_type: SceneNodeType,
    pub position: (i32, i32),
    pub enabled: bool,
    pub children: Vec<SceneNode>,
}

impl SceneNode {
    pub fn new_tree() -> Self {
        Self {
            id: 0,
            node_type: SceneNodeType::Tree,
            position: (0, 0),
            enabled: true,
            children: vec![],
        }
    }

    pub fn add_child(&mut self, child: SceneNode) {
        self.children.push(child);
    }
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
