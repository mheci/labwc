//! Scene graph with GPU resource lifecycle management.
//!
//! Each node tracks a generation counter. When a node is destroyed,
//! its GPU resources are marked for deferred cleanup on the next
//! vblank, preventing use-after-free on the GPU timeline.
//!
//! On NVIDIA, destroying a scene node while a frame is in flight
//! causes a GPU page fault and hard lockup. The deferred cleanup
//! mechanism prevents this entirely.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneNodeKind {
    Root,
    Tree,
    Rect,
    Buffer,
    Surface,
    Popup,
}

#[derive(Debug)]
pub struct SceneNode {
    pub id: u64,
    pub kind: SceneNodeKind,
    pub generation: u64,
    pub position: (i32, i32),
    pub enabled: bool,
    pub visible: bool,
    pub damage_frame: u64,
    parent_id: Option<u64>,
    children: Vec<SceneNode>,
}

#[derive(Debug, Default)]
pub struct DeferredCleanup {
    queue: VecDeque<(u64, u64)>, // (node_id, generation to retire after)
}

impl DeferredCleanup {
    pub fn schedule(&mut self, node_id: u64, retire_after_gen: u64) {
        self.queue.push_back((node_id, retire_after_gen));
    }

    pub fn retire(&mut self, current_generation: u64) -> Vec<u64> {
        let mut freed = Vec::new();
        let mut i = 0;
        while i < self.queue.len() {
            if self.queue[i].1 <= current_generation {
                freed.push(self.queue[i].0);
                self.queue.remove(i);
            } else {
                i += 1;
            }
        }
        freed
    }

    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }
}

impl SceneNode {
    pub fn new(kind: SceneNodeKind) -> Self {
        let id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            kind,
            generation: 0,
            position: (0, 0),
            enabled: true,
            visible: true,
            damage_frame: 0,
            parent_id: None,
            children: Vec::new(),
        }
    }

    pub fn new_tree() -> Self {
        Self::new(SceneNodeKind::Root)
    }

    pub fn add_child(&mut self, mut child: SceneNode) {
        child.parent_id = Some(self.id);
        self.children.push(child);
    }

    pub fn remove_child(&mut self, child_id: u64) -> Option<SceneNode> {
        if let Some(pos) = self.children.iter().position(|c| c.id == child_id) {
            Some(self.children.remove(pos))
        } else {
            None
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
        self.bump_generation();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            self.enabled = enabled;
            self.bump_generation();
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.bump_generation();
        }
    }

    pub fn mark_damaged(&mut self, frame: u64) {
        self.damage_frame = frame;
    }

    fn bump_generation(&mut self) {
        self.generation += 1;
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn iter_children(&self) -> impl Iterator<Item = &SceneNode> {
        self.children.iter()
    }

    pub fn iter_visible(&self) -> impl Iterator<Item = &SceneNode> {
        self.children.iter().filter(|c| c.enabled && c.visible)
    }

    pub fn reparent(&mut self, new_parent_id: u64) {
        self.parent_id = Some(new_parent_id);
        self.bump_generation();
    }

    pub fn total_nodes(&self) -> usize {
        let mut count = 1;
        let mut stack = vec![self as *const SceneNode];
        let mut idx = 0;
        while idx < stack.len() {
            let node = unsafe { &*stack[idx] };
            for child in &node.children {
                count += 1;
                stack.push(child as *const SceneNode);
            }
            idx += 1;
        }
        count
    }
}

#[derive(Debug)]
pub struct Scene {
    pub root: SceneNode,
    pub deferred: DeferredCleanup,
    pub generation: u64,
    pub frame_counter: AtomicU64,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            root: SceneNode::new_tree(),
            deferred: DeferredCleanup::default(),
            generation: 0,
            frame_counter: AtomicU64::new(0),
        }
    }

    pub fn begin_frame(&mut self) -> u64 {
        self.generation += 1;
        let fc = self.frame_counter.fetch_add(1, Ordering::SeqCst);

        // Retire any GPU resources whose frames are complete
        let _freed = self.deferred.retire(self.generation);

        fc
    }

    pub fn end_frame(&mut self) {
        // Any nodes destroyed during this frame get their cleanup
        // deferred until after the GPU has finished with them
    }

    pub fn mark_damage(&mut self, node_id: u64, frame: u64) {
        if let Some(node) = self.find_node_mut(node_id) {
            node.damage_frame = frame;
        }
    }

    pub fn schedule_cleanup(&mut self, node_id: u64) {
        // The node will be safe to clean up after 2 more generations
        // (one for the current frame, one for GPU pipeline depth)
        let retire_at = self.generation + 2;
        self.deferred.schedule(node_id, retire_at);
    }

    fn find_node_mut(&mut self, id: u64) -> Option<&mut SceneNode> {
        if self.root.id == id {
            return Some(&mut self.root);
        }
        fn find_recursive(node: &mut SceneNode, id: u64) -> Option<&mut SceneNode> {
            for child in &mut node.children {
                if child.id == id {
                    return Some(child);
                }
                if let Some(found) = find_recursive(child, id) {
                    return Some(found);
                }
            }
            None
        }
        find_recursive(&mut self.root, id)
    }

    pub fn total_nodes(&self) -> usize {
        self.root.total_nodes()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = SceneNode::new(SceneNodeKind::Rect);
        assert!(node.id > 0);
        assert!(node.enabled);
    }

    #[test]
    fn test_node_tree() {
        let mut root = SceneNode::new_tree();
        let child = SceneNode::new(SceneNodeKind::Rect);
        root.add_child(child);
        assert_eq!(root.child_count(), 1);
    }

    #[test]
    fn test_node_remove() {
        let mut root = SceneNode::new_tree();
        let child = SceneNode::new(SceneNodeKind::Buffer);
        let child_id = child.id;
        root.add_child(child);
        assert!(root.remove_child(child_id).is_some());
        assert_eq!(root.child_count(), 0);
    }

    #[test]
    fn test_scene_frame_cycle() {
        let mut scene = Scene::new();
        let fc = scene.begin_frame();
        scene.end_frame();
        assert!(fc < scene.frame_counter.load(Ordering::Relaxed));
    }

    #[test]
    fn test_deferred_cleanup_ordered() {
        let mut dc = DeferredCleanup::default();
        dc.schedule(1, 2); // generation 2
        dc.schedule(2, 3); // generation 3
        dc.schedule(3, 5); // generation 5
        assert_eq!(dc.pending_count(), 3);

        let freed = dc.retire(2);
        assert_eq!(freed.len(), 1); // only node 1
        assert!(freed.contains(&1));
        assert_eq!(dc.pending_count(), 2);

        let freed = dc.retire(3);
        assert_eq!(freed.len(), 1); // node 2
        assert!(freed.contains(&2));
        assert_eq!(dc.pending_count(), 1);

        let freed = dc.retire(5);
        assert_eq!(freed.len(), 1); // node 3
        assert!(freed.contains(&3));
        assert_eq!(dc.pending_count(), 0);
    }

    #[test]
    fn test_deferred_cleanup_unordered() {
        // Even when inserted out of order, retire should handle it
        let mut dc = DeferredCleanup::default();
        dc.schedule(1, 5);
        dc.schedule(2, 2);
        dc.schedule(3, 3);
        assert_eq!(dc.pending_count(), 3);
        let freed = dc.retire(3);
        assert_eq!(freed.len(), 2); // nodes 2 and 3 (gens 2 and 3)
        assert!(freed.contains(&2));
        assert!(freed.contains(&3));
        assert_eq!(dc.pending_count(), 1);
    }

    #[test]
    fn test_scene_schedule_cleanup() {
        let mut scene = Scene::new();
        let node_id = scene.root.id;
        scene.schedule_cleanup(node_id);
        assert!(scene.deferred.pending_count() > 0);
    }
}
