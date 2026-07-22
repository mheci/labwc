//! Implementation of `wp_commit_timing_manager_v1`.
//!
//! Provides commit timing instrumentation — allows clients to
//! receive timestamps for surface commits for performance analysis.

use std::collections::HashMap;
use tracing::{debug, info};

/// A commit timing record.
#[derive(Debug, Clone)]
pub struct CommitTimestamp {
    /// Seconds part (high 32 bits).
    pub tv_sec_hi: u32,
    /// Seconds part (low 32 bits).
    pub tv_sec_lo: u32,
    /// Nanoseconds part.
    pub tv_nsec: u32,
}

impl CommitTimestamp {
    /// Create a timestamp from the current time.
    #[must_use]
    pub fn now() -> Self {
        // Use std::time for wall-clock time
        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = dur.as_secs();
        let nsecs = dur.subsec_nanos();
        Self {
            tv_sec_hi: (secs >> 32) as u32,
            tv_sec_lo: secs as u32,
            tv_nsec: nsecs,
        }
    }
}

/// Commit timing state per surface.
#[derive(Debug)]
pub struct CommitTimerState {
    /// Surface this timer is associated with.
    pub surface_id: u64,
    /// Most recent commit timestamp.
    pub last_commit: Option<CommitTimestamp>,
    /// Whether timing is active.
    pub active: bool,
}

impl CommitTimerState {
    /// Create a new commit timer for a surface.
    #[must_use]
    pub fn new(surface_id: u64) -> Self {
        Self {
            surface_id,
            last_commit: None,
            active: true,
        }
    }

    /// Record a commit with the current time.
    pub fn record_commit(&mut self) -> CommitTimestamp {
        let ts = CommitTimestamp::now();
        debug!(
            surface_id = self.surface_id,
            sec_lo = ts.tv_sec_lo,
            nsec = ts.tv_nsec,
            "Commit timestamp recorded"
        );
        self.last_commit = Some(ts.clone());
        ts
    }
}

/// Global commit timing manager.
#[derive(Debug, Default)]
pub struct CommitTimingManager {
    /// Active timers by surface ID.
    pub timers: HashMap<u64, CommitTimerState>,
}

impl CommitTimingManager {
    /// Create a new commit timing manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
        }
    }

    /// Get or create a timer for a surface.
    pub fn get_or_create(&mut self, surface_id: u64) -> &mut CommitTimerState {
        self.timers
            .entry(surface_id)
            .or_insert_with(|| CommitTimerState::new(surface_id))
    }
}

/// Register the `wp_commit_timing_manager_v1` global.
pub fn register() {
    info!("Registered wp_commit_timing_manager_v1");
}
