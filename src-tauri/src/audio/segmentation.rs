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
