use super::capture::rms;

/// Which audio source is currently dominant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DominantSource {
    Mic,
    System,
    Neither,
}

/// Tracks segment boundaries based on a timer and dominant-input switches.
pub struct Segmenter {
    /// Segment flush interval in samples (at the stream sample rate).
    interval_samples: u64,
    samples_since_flush: u64,
    current_dominant: DominantSource,
}

impl Segmenter {
    pub fn new(interval_seconds: f32, sample_rate: u32) -> Self {
        Self {
            interval_samples: (interval_seconds * sample_rate as f32) as u64,
            samples_since_flush: 0,
            current_dominant: DominantSource::Neither,
        }
    }

    /// Feed a batch of samples from both streams.
    /// Returns `true` if a segment boundary has been reached and buffers
    /// should be flushed to ASR.
    pub fn advance(&mut self, n_samples: usize, mic: &[f32], sys: &[f32]) -> bool {
        self.samples_since_flush += n_samples as u64;

        // Determine dominant source by RMS
        let mic_rms = rms(mic);
        let sys_rms = rms(sys);
        let new_dominant = if mic_rms == 0.0 && sys_rms == 0.0 {
            DominantSource::Neither
        } else if mic_rms >= sys_rms {
            DominantSource::Mic
        } else {
            DominantSource::System
        };

        let source_switched = new_dominant != DominantSource::Neither
            && self.current_dominant != DominantSource::Neither
            && new_dominant != self.current_dominant;

        if self.current_dominant != DominantSource::Neither || new_dominant != DominantSource::Neither
        {
            self.current_dominant = new_dominant;
        }

        if source_switched || self.samples_since_flush >= self.interval_samples {
            self.samples_since_flush = 0;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.samples_since_flush = 0;
        self.current_dominant = DominantSource::Neither;
    }

    pub fn update_interval(&mut self, interval_seconds: f32, sample_rate: u32) {
        self.interval_samples = (interval_seconds * sample_rate as f32) as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(interval_s: f32) -> Segmenter {
        Segmenter::new(interval_s, 16_000)
    }

    fn samples(val: f32, n: usize) -> Vec<f32> {
        vec![val; n]
    }

    // ── Timer-based flush ────────────────────────────────────────────────────

    #[test]
    fn no_flush_before_interval() {
        let mut s = seg(1.0); // 1 s = 16 000 samples
        // Feed 15 999 samples – should not flush
        let flushed = s.advance(15_999, &samples(0.5, 15_999), &samples(0.1, 15_999));
        assert!(!flushed);
    }

    #[test]
    fn flush_at_interval() {
        let mut s = seg(1.0);
        let flushed = s.advance(16_000, &samples(0.5, 16_000), &samples(0.1, 16_000));
        assert!(flushed);
    }

    #[test]
    fn counter_resets_after_flush() {
        let mut s = seg(1.0);
        s.advance(16_000, &samples(0.5, 16_000), &samples(0.1, 16_000)); // flush
        // Immediately after, counter is reset – 1 sample should not flush
        let flushed = s.advance(1, &samples(0.5, 1), &samples(0.1, 1));
        assert!(!flushed);
    }

    // ── Source-switch flush ──────────────────────────────────────────────────

    #[test]
    fn flush_on_dominant_source_switch() {
        let mut s = seg(60.0); // long interval so timer doesn't trigger
        // First window: mic dominant (mic=0.8, sys=0.1)
        s.advance(100, &samples(0.8, 100), &samples(0.1, 100));
        // Second window: sys dominant (mic=0.1, sys=0.8) → switch → flush
        let flushed = s.advance(100, &samples(0.1, 100), &samples(0.8, 100));
        assert!(flushed);
    }

    #[test]
    fn no_flush_when_source_stays_same() {
        let mut s = seg(60.0);
        s.advance(100, &samples(0.8, 100), &samples(0.1, 100)); // mic dominant
        let flushed = s.advance(100, &samples(0.8, 100), &samples(0.1, 100)); // still mic
        assert!(!flushed);
    }

    #[test]
    fn no_flush_from_neither_to_source() {
        // First window both silent → Neither; second window mic active
        // That's Neither→Mic, not a real switch, so no flush expected.
        let mut s = seg(60.0);
        s.advance(100, &samples(0.0, 100), &samples(0.0, 100)); // Neither
        let flushed = s.advance(100, &samples(0.8, 100), &samples(0.1, 100)); // Mic
        assert!(!flushed);
    }

    // ── reset ────────────────────────────────────────────────────────────────

    #[test]
    fn reset_clears_counter_and_dominant() {
        let mut s = seg(1.0);
        s.advance(15_000, &samples(0.8, 15_000), &samples(0.1, 15_000));
        s.reset();
        // After reset, 15 000 more samples should not flush
        let flushed = s.advance(15_000, &samples(0.8, 15_000), &samples(0.1, 15_000));
        assert!(!flushed);
    }

    // ── update_interval ──────────────────────────────────────────────────────

    #[test]
    fn update_interval_affects_flush_threshold() {
        let mut s = seg(10.0); // 10 s
        s.update_interval(1.0, 16_000); // change to 1 s
        let flushed = s.advance(16_000, &samples(0.5, 16_000), &samples(0.1, 16_000));
        assert!(flushed);
    }

    // ── dominant source logic ────────────────────────────────────────────────

    #[test]
    fn equal_rms_prefers_mic() {
        let mut s = seg(60.0);
        // Both equal → mic wins (mic_rms >= sys_rms)
        s.advance(100, &samples(0.5, 100), &samples(0.5, 100)); // Mic
        // Sys dominant next
        let flushed = s.advance(100, &samples(0.1, 100), &samples(0.8, 100));
        assert!(flushed); // Mic → System switch
    }
}
