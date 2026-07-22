use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct TimelinePoint {
    pub value: u64,
    pub output_idx: usize,
}

#[derive(Debug)]
pub struct FrameSync {
    pending: BTreeMap<u64, Instant>,
    max_timeline: u64,
}

impl FrameSync {
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            max_timeline: 0,
        }
    }

    pub fn submit(&mut self, timeline: u64) {
        self.pending.insert(timeline, Instant::now());
        self.max_timeline = self.max_timeline.max(timeline);
    }

    pub fn complete(&mut self, timeline: u64) {
        self.pending.remove(&timeline);
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn gc(&mut self) {
        let cutoff = self.max_timeline.saturating_sub(64);
        self.pending.retain(|&t, _| t > cutoff);
    }
}

impl Default for FrameSync {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct MultiOutputSync {
    syncs: Vec<FrameSync>,
}

impl MultiOutputSync {
    pub fn new(count: usize) -> Self {
        Self {
            syncs: (0..count).map(|_| FrameSync::new()).collect(),
        }
    }

    pub fn submit(&mut self, idx: usize, timeline: u64) {
        if let Some(s) = self.syncs.get_mut(idx) {
            s.submit(timeline);
        }
    }

    pub fn complete(&mut self, idx: usize, timeline: u64) {
        if let Some(s) = self.syncs.get_mut(idx) {
            s.complete(timeline);
        }
    }

    pub fn all_complete(&self) -> bool {
        self.syncs.iter().all(|s| s.pending_count() == 0)
    }

    pub fn gc_all(&mut self) {
        for s in &mut self.syncs {
            s.gc();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_sync() {
        let mut s = FrameSync::new();
        s.submit(1);
        assert_eq!(s.pending_count(), 1);
        s.complete(1);
        assert_eq!(s.pending_count(), 0);
    }

    #[test]
    fn test_multi_output() {
        let mut mos = MultiOutputSync::new(2);
        mos.submit(0, 1);
        mos.submit(1, 2);
        assert!(!mos.all_complete());
        mos.complete(0, 1);
        mos.complete(1, 2);
        assert!(mos.all_complete());
    }

    #[test]
    fn test_gc() {
        let mut s = FrameSync::new();
        s.submit(1);
        s.submit(100);
        s.gc();
        assert_eq!(s.pending_count(), 1); // old timeline pruned
    }
}
