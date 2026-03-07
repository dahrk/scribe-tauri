use super::capture::{rms, to_dbfs};

/// Parameters controlling activity detection.
#[derive(Debug, Clone)]
pub struct ActivityConfig {
    /// dBFS threshold above which a stream is considered "active".
    /// Default: -40 dB.
    pub threshold_dbfs: f32,
    /// How many consecutive analysis windows both streams must be above threshold
    /// before recording starts. Each window is ~`window_samples / sample_rate` s.
    pub min_active_windows: u32,
    /// How many consecutive analysis windows both streams must be below threshold
    /// before recording stops.
    pub min_idle_windows: u32,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            threshold_dbfs: -40.0,
            min_active_windows: 3,   // ~3 s with 1-s windows
            min_idle_windows: 180,   // 3 min
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityState {
    Idle,
    Starting { count: u32 },
    Active,
    Stopping { count: u32 },
}

pub struct ActivityDetector {
    config: ActivityConfig,
    state: ActivityState,
}

impl ActivityDetector {
    pub fn new(config: ActivityConfig) -> Self {
        Self {
            config,
            state: ActivityState::Idle,
        }
    }

    /// Feed a window of samples from both streams.
    /// Returns `Some(true)` when recording should START,
    ///         `Some(false)` when recording should STOP,
    ///         `None` for no state change.
    pub fn feed(&mut self, mic_samples: &[f32], sys_samples: &[f32]) -> Option<bool> {
        let mic_db = to_dbfs(rms(mic_samples));
        let sys_db = to_dbfs(rms(sys_samples));
        let both_active =
            mic_db >= self.config.threshold_dbfs && sys_db >= self.config.threshold_dbfs;
        let both_idle = !both_active;

        match &self.state {
            ActivityState::Idle => {
                if both_active {
                    self.state = ActivityState::Starting { count: 1 };
                }
                None
            }
            ActivityState::Starting { count } => {
                if both_active {
                    let next = count + 1;
                    if next >= self.config.min_active_windows {
                        self.state = ActivityState::Active;
                        Some(true)
                    } else {
                        self.state = ActivityState::Starting { count: next };
                        None
                    }
                } else {
                    self.state = ActivityState::Idle;
                    None
                }
            }
            ActivityState::Active => {
                if both_idle {
                    self.state = ActivityState::Stopping { count: 1 };
                }
                None
            }
            ActivityState::Stopping { count } => {
                if both_idle {
                    let next = count + 1;
                    if next >= self.config.min_idle_windows {
                        self.state = ActivityState::Idle;
                        Some(false)
                    } else {
                        self.state = ActivityState::Stopping { count: next };
                        None
                    }
                } else {
                    // Activity resumed
                    self.state = ActivityState::Active;
                    None
                }
            }
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            ActivityState::Active | ActivityState::Stopping { .. }
        )
    }

    pub fn update_config(&mut self, config: ActivityConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_det(threshold: f32, min_active: u32, min_idle: u32) -> ActivityDetector {
        ActivityDetector::new(ActivityConfig {
            threshold_dbfs: threshold,
            min_active_windows: min_active,
            min_idle_windows: min_idle,
        })
    }

    /// Samples loud enough to pass -40 dB threshold.
    fn loud() -> Vec<f32> {
        vec![0.1f32; 160]
    }

    /// Samples quiet enough to be below -40 dB.
    fn silent() -> Vec<f32> {
        vec![0.0f32; 160]
    }

    // ── Idle → Starting → Active ─────────────────────────────────────────────

    #[test]
    fn starts_idle() {
        let det = make_det(-40.0, 3, 5);
        assert!(!det.is_active());
    }

    #[test]
    fn single_loud_window_does_not_start() {
        let mut det = make_det(-40.0, 3, 5);
        let r = det.feed(&loud(), &loud());
        assert!(r.is_none());
        assert!(!det.is_active());
    }

    #[test]
    fn reaches_active_after_min_windows() {
        let mut det = make_det(-40.0, 3, 5);
        // First two windows → Starting
        assert!(det.feed(&loud(), &loud()).is_none());
        assert!(det.feed(&loud(), &loud()).is_none());
        // Third window → Active (returns Some(true))
        let r = det.feed(&loud(), &loud());
        assert_eq!(r, Some(true));
        assert!(det.is_active());
    }

    #[test]
    fn starting_resets_to_idle_on_quiet() {
        let mut det = make_det(-40.0, 3, 5);
        det.feed(&loud(), &loud()); // Starting { count: 1 }
        let r = det.feed(&silent(), &silent()); // → Idle
        assert!(r.is_none());
        assert!(!det.is_active());
    }

    #[test]
    fn only_mic_active_does_not_start() {
        // spec: BOTH streams must be active
        let mut det = make_det(-40.0, 3, 5);
        for _ in 0..5 {
            let r = det.feed(&loud(), &silent());
            assert!(r.is_none());
        }
        assert!(!det.is_active());
    }

    // ── Active → Stopping → Idle ─────────────────────────────────────────────

    #[test]
    fn stops_after_min_idle_windows() {
        let mut det = make_det(-40.0, 3, 3);
        // Activate
        det.feed(&loud(), &loud());
        det.feed(&loud(), &loud());
        det.feed(&loud(), &loud()); // → Active

        // Two quiet windows → Stopping
        det.feed(&silent(), &silent());
        det.feed(&silent(), &silent());
        // Third quiet window → Idle (returns Some(false))
        let r = det.feed(&silent(), &silent());
        assert_eq!(r, Some(false));
        assert!(!det.is_active());
    }

    #[test]
    fn stopping_returns_to_active_on_loud() {
        let mut det = make_det(-40.0, 2, 3);
        // Activate
        det.feed(&loud(), &loud());
        det.feed(&loud(), &loud()); // → Active
        // Start stopping
        det.feed(&silent(), &silent()); // Stopping { 1 }
        // Loud again → back to Active, no stop event
        let r = det.feed(&loud(), &loud());
        assert!(r.is_none());
        assert!(det.is_active());
    }

    // ── is_active covers both Active and Stopping ────────────────────────────

    #[test]
    fn is_active_true_during_stopping() {
        let mut det = make_det(-40.0, 2, 5);
        det.feed(&loud(), &loud());
        det.feed(&loud(), &loud()); // → Active
        det.feed(&silent(), &silent()); // → Stopping { 1 }
        assert!(det.is_active()); // still "active" while counting down
    }

    // ── update_config ────────────────────────────────────────────────────────

    #[test]
    fn update_config_changes_threshold() {
        let mut det = make_det(-40.0, 1, 1);
        // With very high threshold nothing should trigger
        det.update_config(ActivityConfig {
            threshold_dbfs: 0.0, // 0 dBFS – nothing ever reaches this
            min_active_windows: 1,
            min_idle_windows: 1,
        });
        let r = det.feed(&loud(), &loud());
        assert!(r.is_none());
    }
}
