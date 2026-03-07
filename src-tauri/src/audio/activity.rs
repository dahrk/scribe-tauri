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
