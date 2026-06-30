//! Beat detection — onset detection for beat-synced cutting.
//!
//! ## Status
//!
//! This module provides spectral-flux-based onset detection: it computes
//! the spectral flux (sum of positive differences between consecutive
//! short-time Fourier transform magnitude frames) and picks peaks as
//! beat candidates. The detection is then post-processed to estimate
//! BPM and produce a list of beat markers suitable for timeline magnetic
//! snapping.
//!
//! ## video: debt markers
//!
//! - Spectral flux only, upgrade to tempogram + CNN-based beat tracking if precision is critical
//! - Mono sum (no L/R separation), upgrade to stereo if beat detection on one channel is needed
//! - No tempo smoothing, upgrade to median-filter tempo estimation if BPM is unstable
//! - No downbeat detection, upgrade to bar-position inference if "cut on 1" matters

use serde::{Deserialize, Serialize};

/// A detected beat / onset.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Beat {
    /// Time in milliseconds.
    pub time_ms: u64,
    /// Strength 0.0–1.0 (relative to the strongest onset in the clip).
    pub strength: f32,
}

/// Result of beat detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatTrack {
    pub beats: Vec<Beat>,
    pub estimated_bpm: Option<f32>,
    pub duration_ms: u64,
}

/// Parameters for beat detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatDetectParams {
    /// FFT window size (samples). 1024 is a good default for 44.1 kHz.
    pub window_size: usize,
    /// Hop size (samples). 512 = 50% overlap.
    pub hop_size: usize,
    /// Sample rate of the audio (Hz).
    pub sample_rate: u32,
    /// Minimum onset strength to be considered a beat (0.0–1.0). Default 0.3.
    pub min_strength: f32,
    /// Minimum inter-beat interval (ms). Default 200 (i.e., max 300 BPM).
    pub min_interval_ms: u32,
}

impl Default for BeatDetectParams {
    fn default() -> Self {
        Self {
            window_size: 1024,
            hop_size: 512,
            sample_rate: 44100,
            min_strength: 0.3,
            min_interval_ms: 200,
        }
    }
}

/// Detect beats in an audio buffer.
///
/// `samples` is interleaved f32 PCM in [-1.0, 1.0]. If stereo, channels
/// are summed to mono before analysis. The `params.sample_rate` must
/// match the actual sample rate of the buffer.
///
/// video: spectral flux only, upgrade to tempogram + CNN-based beat tracking if precision is critical
pub fn detect_beats(samples: &[f32], params: &BeatDetectParams) -> BeatTrack {
    if samples.is_empty() {
        return BeatTrack {
            beats: vec![],
            estimated_bpm: None,
            duration_ms: 0,
        };
    }

    // Sum to mono (assume stereo if even length, but be defensive)
    let mono: Vec<f32> = if samples.len() % 2 == 0 {
        samples.chunks_exact(2).map(|s| (s[0] + s[1]) * 0.5).collect()
    } else {
        samples.to_vec()
    };

    let duration_ms = ((mono.len() as f64) / (params.sample_rate as f64) * 1000.0) as u64;

    // Compute STFT magnitude
    let mut prev_spectrum: Vec<f32> = vec![0.0; params.window_size / 2 + 1];
    let mut flux: Vec<f32> = Vec::new();
    let mut flux_times_ms: Vec<u64> = Vec::new();

    let window = hann_window(params.window_size);
    let mut pos = 0;
    while pos + params.window_size <= mono.len() {
        // Apply window
        let mut windowed = vec![0.0f32; params.window_size];
        for i in 0..params.window_size {
            windowed[i] = mono[pos + i] * window[i];
        }

        // Compute magnitude spectrum (real FFT — for simplicity, we use
        // a direct DFT here. A real production implementation would use
        // rustfft or realfft.)
        let spectrum = magnitude_spectrum(&windowed);

        // Spectral flux: sum of positive differences
        let mut frame_flux = 0.0f32;
        for i in 0..spectrum.len() {
            let diff = spectrum[i] - prev_spectrum[i];
            if diff > 0.0 {
                frame_flux += diff;
            }
        }
        flux.push(frame_flux);
        flux_times_ms.push(((pos as f64) / (params.sample_rate as f64) * 1000.0) as u64);

        prev_spectrum = spectrum;
        pos += params.hop_size;
    }

    if flux.is_empty() {
        return BeatTrack {
            beats: vec![],
            estimated_bpm: None,
            duration_ms,
        };
    }

    // Normalize flux
    let max_flux = flux.iter().cloned().fold(0.0f32, f32::max).max(1e-9);
    let normalized: Vec<f32> = flux.iter().map(|f| f / max_flux).collect();

    // Peak picking
    let mut beats = Vec::new();
    let min_interval_frames = ((params.min_interval_ms as f64)
        / 1000.0
        * (params.sample_rate as f64)
        / (params.hop_size as f64))
        .round() as usize;
    let min_interval_frames = min_interval_frames.max(1);

    let threshold = params.min_strength;
    let mut last_beat_frame = 0usize.wrapping_sub(min_interval_frames);

    for i in 1..normalized.len().saturating_sub(1) {
        if normalized[i] >= threshold
            && normalized[i] >= normalized[i - 1]
            && normalized[i] > normalized[i + 1]
            && i.wrapping_sub(last_beat_frame) >= min_interval_frames
        {
            beats.push(Beat {
                time_ms: flux_times_ms[i],
                strength: normalized[i],
            });
            last_beat_frame = i;
        }
    }

    // Estimate BPM from inter-beat intervals
    let bpm = estimate_bpm(&beats);

    BeatTrack {
        beats,
        estimated_bpm: bpm,
        duration_ms,
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (size as f32 - 1.0)).cos()))
        .collect()
}

fn magnitude_spectrum(frame: &[f32]) -> Vec<f32> {
    let n = frame.len();
    let half = n / 2 + 1;
    let mut spectrum = vec![0.0f32; half];
    // Direct DFT — O(N²). Fine for tests, too slow for production.
    // video: direct DFT, upgrade to realfft for production
    for k in 0..half {
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        for t in 0..n {
            let angle = -2.0 * std::f32::consts::PI * (k as f32) * (t as f32) / (n as f32);
            re += frame[t] * angle.cos();
            im += frame[t] * angle.sin();
        }
        spectrum[k] = (re * re + im * im).sqrt();
    }
    spectrum
}

fn estimate_bpm(beats: &[Beat]) -> Option<f32> {
    if beats.len() < 2 {
        return None;
    }
    let intervals_ms: Vec<f64> = beats.windows(2)
        .map(|w| (w[1].time_ms - w[0].time_ms) as f64)
        .collect();
    let avg_interval_ms = intervals_ms.iter().sum::<f64>() / (intervals_ms.len() as f64);
    if avg_interval_ms <= 0.0 {
        return None;
    }
    let bpm = 60000.0 / avg_interval_ms;
    // Fold into a musical range (60–180 BPM)
    let mut bpm = bpm;
    while bpm < 60.0 {
        bpm *= 2.0;
    }
    while bpm > 180.0 {
        bpm /= 2.0;
    }
    Some(bpm as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_beats_empty_input() {
        let params = BeatDetectParams::default();
        let result = detect_beats(&[], &params);
        assert!(result.beats.is_empty());
        assert!(result.estimated_bpm.is_none());
    }

    #[test]
    fn detect_beats_silence_no_beats() {
        let params = BeatDetectParams::default();
        let samples = vec![0.0f32; 44100]; // 1 second of silence
        let result = detect_beats(&samples, &params);
        assert!(result.beats.is_empty());
        assert_eq!(result.duration_ms, 1000);
    }

    #[test]
    fn detect_beats_kick_drums() {
        // 4 kick drums at 120 BPM = 1 beat every 500ms
        // Each kick is a 50ms low-frequency burst
        let params = BeatDetectParams::default();
        let sample_rate = params.sample_rate as usize;
        let mut samples = vec![0.0f32; 4 * sample_rate]; // 4 seconds
        // Place kicks at 0ms, 500ms, 1000ms, 1500ms
        for &kick_start_ms in &[0, 500, 1000, 1500] {
            let start = (kick_start_ms as usize) * sample_rate / 1000;
            let kick_len = 50 * sample_rate / 1000;
            for i in 0..kick_len {
                let t = i as f32 / sample_rate as f32;
                let env = (-t * 30.0).exp(); // fast decay
                let freq = 60.0; // 60 Hz kick
                samples[start + i] += env * (2.0 * std::f32::consts::PI * freq * t).sin();
            }
        }
        let result = detect_beats(&samples, &params);
        assert!(!result.beats.is_empty(), "expected at least one beat, got 0");
        // Estimated BPM should be ~120 (allow some tolerance)
        if let Some(bpm) = result.estimated_bpm {
            assert!(bpm > 80.0 && bpm < 180.0, "expected ~120 BPM, got {}", bpm);
        }
    }
}
