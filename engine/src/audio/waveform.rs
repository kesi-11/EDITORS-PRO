//! Waveform generation for audio visualization
//!
//! Generates waveform data from audio samples for display on the timeline.

use serde::{Deserialize, Serialize};

/// Waveform data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    /// Peak values for each sample window (0.0 to 1.0)
    pub peaks: Vec<f32>,
    /// RMS values for each sample window (0.0 to 1.0)
    pub rms_values: Vec<f32>,
    /// Number of audio samples per peak value
    pub samples_per_peak: u32,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl WaveformData {
    /// Generate waveform data from raw audio samples
    pub fn from_samples(
        samples: &[f32],
        sample_rate: u32,
        channels: u32,
        target_peaks: u32,
    ) -> Self {
        if samples.is_empty() || sample_rate == 0 {
            return Self {
                peaks: vec![0.0; target_peaks as usize],
                rms_values: vec![0.0; target_peaks as usize],
                samples_per_peak: 1,
                duration_ms: 0,
            };
        }

        let total_samples = samples.len() / channels as usize;
        let duration_ms = (total_samples as u64 * 1000) / sample_rate as u64;
        let samples_per_peak = (total_samples / target_peaks as usize).max(1) as u32;

        let mut peaks = Vec::with_capacity(target_peaks as usize);
        let mut rms_values = Vec::with_capacity(target_peaks as usize);

        let window_size = (samples_per_peak * channels) as usize;

        for window_start in (0..samples.len()).step_by(window_size) {
            let window_end = (window_start + window_size).min(samples.len());
            let window = &samples[window_start..window_end];

            // Calculate peak
            let peak = window.iter()
                .map(|s| s.abs())
                .fold(0.0f32, f32::max);

            // Calculate RMS
            let rms = if window.is_empty() {
                0.0
            } else {
                (window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32).sqrt()
            };

            peaks.push(peak);
            rms_values.push(rms);

            if peaks.len() >= target_peaks as usize {
                break;
            }
        }

        // Ensure we have exactly target_peaks values
        while peaks.len() < target_peaks as usize {
            peaks.push(0.0);
            rms_values.push(0.0);
        }

        Self {
            peaks,
            rms_values,
            samples_per_peak,
            duration_ms,
        }
    }

    /// Generate a simplified waveform with just peak values
    pub fn simple_peaks(samples: &[f32], target_peaks: u32) -> Vec<f32> {
        if samples.is_empty() {
            return vec![0.0; target_peaks as usize];
        }

        let window_size = (samples.len() / target_peaks as usize).max(1);
        let mut peaks = Vec::with_capacity(target_peaks as usize);

        for window_start in (0..samples.len()).step_by(window_size) {
            let window_end = (window_start + window_size).min(samples.len());
            let peak = samples[window_start..window_end]
                .iter()
                .map(|s| s.abs())
                .fold(0.0f32, f32::max);
            peaks.push(peak);

            if peaks.len() >= target_peaks as usize {
                break;
            }
        }

        while peaks.len() < target_peaks as usize {
            peaks.push(0.0);
        }

        peaks
    }

    /// Get the peak at a normalized position (0.0 to 1.0)
    pub fn peak_at_position(&self, position: f32) -> f32 {
        if self.peaks.is_empty() {
            return 0.0;
        }
        let index = (position * (self.peaks.len() - 1) as f32) as usize;
        self.peaks.get(index).copied().unwrap_or(0.0)
    }
}
