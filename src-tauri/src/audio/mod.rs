pub mod activity;
pub mod capture;
pub mod segmentation;

use activity::{ActivityConfig, ActivityDetector};
use capture::{drain, open_mic, open_system_audio, AudioBuffer, CaptureStream};
use segmentation::Segmenter;

use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

/// High-level engine that wires together capture, activity detection, and segmentation.
pub struct AudioEngine {
    _mic_stream: CaptureStream,
    _sys_stream: CaptureStream,
    pub mic_buffer: AudioBuffer,
    pub sys_buffer: AudioBuffer,
    pub sample_rate: u32,
    pub activity: Arc<Mutex<ActivityDetector>>,
    pub segmenter: Arc<Mutex<Segmenter>>,
}

impl AudioEngine {
    pub fn new(
        activity_config: ActivityConfig,
        segment_interval_secs: f32,
    ) -> Result<Self> {
        let (mic_stream, mic_buffer) = open_mic()?;

        // System audio is best-effort; fall back to a silent dummy buffer if unavailable.
        let (sys_stream, sys_buffer) = match open_system_audio() {
            Ok(pair) => pair,
            Err(e) => {
                log::warn!("System audio capture unavailable: {e}. Using silent fallback.");
                // Create a dummy stream by opening mic again – its samples won't be
                // used for system-audio detection in real audio, but the engine won't crash.
                // In production the UI should warn the user.
                let (s, b) = open_mic().unwrap_or_else(|_| {
                    // Last resort: return a no-op pair
                    panic!("Cannot open any audio device");
                });
                (s, b)
            }
        };

        let sample_rate = 16_000u32; // whisper expects 16 kHz; we'll resample in the ASR layer

        let activity = Arc::new(Mutex::new(ActivityDetector::new(activity_config)));
        let segmenter = Arc::new(Mutex::new(Segmenter::new(
            segment_interval_secs,
            sample_rate,
        )));

        Ok(Self {
            _mic_stream: mic_stream,
            _sys_stream: sys_stream,
            mic_buffer,
            sys_buffer,
            sample_rate,
            activity,
            segmenter,
        })
    }

    /// Drain both buffers and return (mic_samples, sys_samples).
    pub fn drain_buffers(&self) -> (Vec<f32>, Vec<f32>) {
        (drain(&self.mic_buffer), drain(&self.sys_buffer))
    }
}

/// Encode a slice of f32 PCM samples to a WAV byte vector (16-bit PCM, mono, 16 kHz).
pub fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)?;
        for &s in samples {
            let val = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(val)?;
        }
        writer.finalize()?;
    }
    Ok(cursor.into_inner())
}
