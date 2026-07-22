use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Animation {
    pub id: u64,
    pub start_time: Instant,
    pub duration: Duration,
    pub easing: Easing,
    pub from: f64,
    pub to: f64,
    pub current: f64,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f64, f64, f64, f64),
}

impl Animation {
    pub fn new(id: u64, from: f64, to: f64, duration_ms: u64, easing: Easing) -> Self {
        Self {
            id,
            start_time: Instant::now(),
            duration: Duration::from_millis(duration_ms),
            easing,
            from,
            to,
            current: from,
            done: false,
        }
    }

    pub fn tick(&mut self) -> f64 {
        if self.done {
            return self.to;
        }
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let t = (elapsed / self.duration.as_secs_f64()).clamp(0.0, 1.0);
        let eased = match self.easing {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(t, x1, y1, x2, y2),
        };
        self.current = self.from + (self.to - self.from) * eased;
        if t >= 1.0 {
            self.current = self.to;
            self.done = true;
        }
        self.current
    }
}

fn cubic_bezier(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let x = mt3 * 0.0 + 3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3 * 1.0;
    let y = mt3 * 0.0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * 1.0;
    if x.abs() < f64::EPSILON {
        return 0.0;
    }
    y / x
}

pub struct AnimationManager {
    animations: Vec<Animation>,
    next_id: u64,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_id: 0,
        }
    }
    pub fn animate(&mut self, from: f64, to: f64, duration_ms: u64, easing: Easing) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.animations
            .push(Animation::new(id, from, to, duration_ms, easing));
        id
    }
    pub fn tick_all(&mut self) -> Vec<(u64, f64)> {
        let mut results = Vec::new();
        self.animations.retain_mut(|a| {
            let v = a.tick();
            results.push((a.id, v));
            !a.done
        });
        results
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_animation() {
        let mut a = Animation::new(0, 0.0, 100.0, 100, Easing::Linear);
        let v = a.tick();
        assert!(v >= 0.0);
    }

    #[test]
    fn test_animation_completes() {
        let mut a = Animation::new(0, 0.0, 1.0, 1, Easing::Linear);
        std::thread::sleep(std::time::Duration::from_millis(10));
        a.tick();
        assert!(a.done);
        assert!((a.current - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_animation_manager() {
        let mut mgr = AnimationManager::new();
        let id = mgr.animate(0.0, 50.0, 10, Easing::EaseInOut);
        assert_eq!(id, 0);
        let results = mgr.tick_all();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_easing_variants() {
        let easings = [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::CubicBezier(0.42, 0.0, 0.58, 1.0),
        ];
        for e in &easings {
            let mut a = Animation::new(0, 0.0, 1.0, 100, *e);
            let v = a.tick();
            assert!(v.is_finite());
        }
    }
}
