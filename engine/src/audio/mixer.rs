//! Audio mixer - Combines multiple audio tracks with volume control
//!
//! Mixes audio from multiple tracks, applying volume, fade in/out,
//! and ducking effects to produce a final audio output.

use serde::{Deserialize, Serialize};

/// Audio sample data in interleaved f32 format
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u32,
    pub start_ms: u64,
    pub duration_ms: u64,
}

impl AudioBuffer {
    pub fn new(sample_rate: u32, channels: u32, duration_ms: u64) -> Self {
        let sample_count = (sample_rate as f64 * channels as f64 * duration_ms as f64 / 1000.0) as usize;
        Self {
            samples: vec![0.0; sample_count],
            sample_rate,
            channels,
            start_ms: 0,
            duration_ms,
        }
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }
}

/// Volume envelope for fade in/out effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeEnvelope {
    pub volume: f32,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}

impl Default for VolumeEnvelope {
    fn default() -> Self {
        Self {
            volume: 1.0,
            fade_in_ms: 0,
            fade_out_ms: 0,
        }
    }
}

/// The audio mixer combines multiple audio sources
pub struct AudioMixer {
    sample_rate: u32,
    channels: u32,
}

impl AudioMixer {
    pub fn new(sample_rate: u32, channels: u32) -> Self {
        Self { sample_rate, channels }
    }

    /// Mix multiple audio buffers into a single output
    pub fn mix(&self, sources: &[(&AudioBuffer, f32)]) -> AudioBuffer {
        if sources.is_empty() {
            return AudioBuffer::new(self.sample_rate, self.channels, 0);
        }

        // Find the longest buffer
        let max_duration = sources.iter().map(|(buf, _)| buf.duration_ms).max().unwrap_or(0);
        let max_samples = (self.sample_rate as f64 * self.channels as f64 * max_duration as f64 / 1000.0) as usize;

        let mut output = vec![0.0f32; max_samples];

        for (buffer, volume) in sources {
            let samples_to_mix = buffer.samples.len().min(max_samples);
            for i in 0..samples_to_mix {
                output[i] += buffer.samples[i] * volume;
            }
        }

        // Clamp output to prevent clipping
        for sample in output.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }

        AudioBuffer {
            samples: output,
            sample_rate: self.sample_rate,
            channels: self.channels,
            start_ms: 0,
            duration_ms: max_duration,
        }
    }

    /// Apply a volume envelope (fade in/out) to an audio buffer
    pub fn apply_envelope(&self, buffer: &mut AudioBuffer, envelope: &VolumeEnvelope) {
        let total_samples = buffer.samples.len();
        let channels = buffer.channels as usize;
        let samples_per_ms = (buffer.sample_rate as f64 * channels as f64 / 1000.0) as usize;

        // Apply fade in
        let fade_in_samples = (envelope.fade_in_ms as usize * samples_per_ms).min(total_samples);
        for i in 0..fade_in_samples {
            let progress = i as f32 / fade_in_samples as f32;
            buffer.samples[i] *= progress * envelope.volume;
        }

        // Apply constant volume to middle section
        let fade_in_end = fade_in_samples;
        let fade_out_start = total_samples.saturating_sub(
            (envelope.fade_out_ms as usize * samples_per_ms).min(total_samples)
        );

        for i in fade_in_end..fade_out_start {
            buffer.samples[i] *= envelope.volume;
        }

        // Apply fade out
        for i in fade_out_start..total_samples {
            let remaining = total_samples - i;
            let fade_out_total = total_samples - fade_out_start;
            let progress = remaining as f32 / fade_out_total as f32;
            buffer.samples[i] *= progress * envelope.volume;
        }
    }

    /// Apply audio ducking (reduce volume when another track is active)
    pub fn apply_ducking(
        &self,
        main_buffer: &mut AudioBuffer,
        duck_trigger: &AudioBuffer,
        duck_level: f32,
        attack_ms: u64,
        release_ms: u64,
    ) {
        let channels = main_buffer.channels as usize;
        let samples_per_ms = (main_buffer.sample_rate as f64 * channels as f64 / 1000.0) as usize;

        let attack_samples = (attack_ms as usize * samples_per_ms).min(main_buffer.samples.len());
        let release_samples = (release_ms as usize * samples_per_ms).min(main_buffer.samples.len());

        let min_len = main_buffer.samples.len().min(duck_trigger.samples.len());

        // Detect when the duck trigger is active (simplified: RMS energy threshold)
        let window_size = (50.0 * main_buffer.sample_rate as f64 * channels as f64 / 1000.0) as usize;

        let mut is_ducking = false;
        let mut duck_progress = 0.0f32;

        for i in (0..min_len).step_by(window_size.max(1)) {
            // Calculate RMS energy of the duck trigger in this window
            let end = (i + window_size).min(duck_trigger.samples.len());
            let window = &duck_trigger.samples[i..end];
            let rms = if window.is_empty() {
                0.0
            } else {
                (window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32).sqrt()
            };

            let should_duck = rms > 0.05; // Threshold for ducking

            // Smooth transition between ducking and not ducking
            if should_duck && !is_ducking {
                is_ducking = true;
                duck_progress = 0.0;
            } else if !should_duck && is_ducking {
                is_ducking = false;
                duck_progress = 1.0;
            }

            // Apply ducking with smooth attack/release
            for j in i..end.min(min_len) {
                if is_ducking {
                    duck_progress = (duck_progress + 1.0 / attack_samples as f32).min(1.0);
                } else {
                    duck_progress = (duck_progress - 1.0 / release_samples as f32).max(0.0);
                }

                let volume = 1.0 - (1.0 - duck_level) * duck_progress;
                main_buffer.samples[j] *= volume;
            }
        }
    }

    /// Normalize audio levels
    pub fn normalize(&self, buffer: &mut AudioBuffer, target_peak: f32) {
        let peak = buffer.samples.iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        if peak > 0.0 {
            let gain = target_peak / peak;
            for sample in buffer.samples.iter_mut() {
                *sample *= gain;
            }
        }
    }
}
