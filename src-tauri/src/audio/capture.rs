use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};
use parking_lot::Mutex;
use std::sync::Arc;

/// Shared audio buffer: raw f32 samples appended from the capture callback.
pub type AudioBuffer = Arc<Mutex<Vec<f32>>>;

pub struct CaptureStream {
    _stream: cpal::Stream,
}

// cpal streams are not Send by default on macOS (they hold raw pointers).
// We wrap them and assert Send since the stream is only dropped on our background thread.
unsafe impl Send for CaptureStream {}

/// Open the default **microphone** input and return (stream, shared buffer).
pub fn open_mic() -> Result<(CaptureStream, AudioBuffer)> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No microphone input device found"))?;
    let config = device.default_input_config()?;
    open_input_device(device, config.into())
}

/// On Linux: open the **monitor** of the default output sink (system audio).
/// On macOS 13+: placeholder – system audio capture via ScreenCaptureKit is
/// implemented in the platform-specific module. This path is used when that
/// isn't available (e.g. older macOS with BlackHole configured as a virtual device).
pub fn open_system_audio() -> Result<(CaptureStream, AudioBuffer)> {
    let host = cpal::default_host();

    // Try to find a monitor / loopback device by name heuristic.
    let devices: Vec<Device> = host.input_devices()?.collect();
    let monitor = devices.into_iter().find(|d| {
        d.name()
            .map(|n| {
                let n = n.to_lowercase();
                n.contains("monitor")
                    || n.contains("loopback")
                    || n.contains("blackhole")
                    || n.contains("soundflower")
            })
            .unwrap_or(false)
    });

    let device = monitor.ok_or_else(|| {
        anyhow::anyhow!(
            "No system-audio loopback device found. \
             On Linux ensure PulseAudio/PipeWire monitor is available. \
             On macOS install BlackHole and configure an aggregate device."
        )
    })?;
    let config = device.default_input_config()?;
    open_input_device(device, config.into())
}

fn open_input_device(device: Device, config: StreamConfig) -> Result<(CaptureStream, AudioBuffer)> {
    let buffer: AudioBuffer = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = buffer.clone();

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _info: &cpal::InputCallbackInfo| {
            buf_clone.lock().extend_from_slice(data);
        },
        |err| {
            log::error!("Audio capture error: {err}");
        },
        None,
    )?;
    stream.play()?;
    Ok((CaptureStream { _stream: stream }, buffer))
}

/// Drain all samples from the buffer and return them.
pub fn drain(buffer: &AudioBuffer) -> Vec<f32> {
    let mut lock = buffer.lock();
    std::mem::take(&mut *lock)
}

/// Compute RMS energy (0.0–1.0) of a slice of f32 samples.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Convert RMS to approximate dBFS.
pub fn to_dbfs(rms_linear: f32) -> f32 {
    if rms_linear <= 0.0 {
        return f32::NEG_INFINITY;
    }
    20.0 * rms_linear.log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── rms ──────────────────────────────────────────────────────────────────

    #[test]
    fn rms_empty_is_zero() {
        assert_eq!(rms(&[]), 0.0);
    }

    #[test]
    fn rms_silence_is_zero() {
        assert_eq!(rms(&[0.0, 0.0, 0.0]), 0.0);
    }

    #[test]
    fn rms_constant_signal() {
        // All samples = 0.5 → RMS = 0.5
        let samples = vec![0.5f32; 1000];
        let r = rms(&samples);
        assert!((r - 0.5).abs() < 1e-5, "expected ~0.5, got {r}");
    }

    #[test]
    fn rms_mixed_signal() {
        // [1.0, -1.0] → RMS = 1.0
        let r = rms(&[1.0, -1.0]);
        assert!((r - 1.0).abs() < 1e-5, "expected 1.0, got {r}");
    }

    #[test]
    fn rms_single_sample() {
        let r = rms(&[0.707_107]);
        assert!((r - 0.707_107).abs() < 1e-5);
    }

    // ── to_dbfs ───────────────────────────────────────────────────────────────

    #[test]
    fn dbfs_of_zero_is_neg_infinity() {
        assert_eq!(to_dbfs(0.0), f32::NEG_INFINITY);
    }

    #[test]
    fn dbfs_of_negative_is_neg_infinity() {
        assert_eq!(to_dbfs(-1.0), f32::NEG_INFINITY);
    }

    #[test]
    fn dbfs_of_one_is_zero() {
        let db = to_dbfs(1.0);
        assert!(db.abs() < 1e-5, "expected ~0 dBFS, got {db}");
    }

    #[test]
    fn dbfs_of_half_is_minus_six() {
        // 20*log10(0.5) ≈ -6.02 dB
        let db = to_dbfs(0.5);
        assert!((db - (-6.020_599)).abs() < 0.01, "expected ~-6 dBFS, got {db}");
    }

    #[test]
    fn dbfs_of_0_01_is_minus_40() {
        // 20*log10(0.01) = -40 dB
        let db = to_dbfs(0.01);
        assert!((db - (-40.0)).abs() < 0.01, "expected -40 dBFS, got {db}");
    }

    // ── drain ─────────────────────────────────────────────────────────────────

    #[test]
    fn drain_returns_and_clears() {
        let buf: AudioBuffer = std::sync::Arc::new(parking_lot::Mutex::new(vec![1.0, 2.0, 3.0]));
        let out = drain(&buf);
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
        assert!(buf.lock().is_empty());
    }

    #[test]
    fn drain_empty_buffer() {
        let buf: AudioBuffer = std::sync::Arc::new(parking_lot::Mutex::new(vec![]));
        let out = drain(&buf);
        assert!(out.is_empty());
    }
}
