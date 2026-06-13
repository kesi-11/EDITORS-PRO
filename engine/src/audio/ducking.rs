//! Audio ducking - Automatic volume reduction when voiceover is active
//!
//! Implements a ducking system that reduces background audio volume
//! when a "trigger" audio track (e.g., voiceover) is active.
//! The ducking has configurable attack/release times for smooth
//! volume transitions.

use serde::{Deserialize, Serialize};

use super::mixer::AudioBuffer;

/// Configuration for audio ducking behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckingConfig {
    /// Whether ducking is enabled for this track
    pub enabled: bool,
    /// Volume level when ducking is active (0.0 to 1.0, typically 0.2-0.4)
    pub duck_level: f32,
    /// Attack time in milliseconds (how fast volume drops when trigger activates)
    pub attack_ms: u64,
    /// Release time in milliseconds (how fast volume recovers when trigger deactivates)
    pub release_ms: u64,
    /// RMS energy threshold for detecting speech (0.0 to 1.0, typically 0.02-0.1)
    pub threshold: f32,
    /// Window size for RMS calculation in milliseconds
    pub window_ms: u64,
}

impl Default for DuckingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            duck_level: 0.3,
            attack_ms: 50,
            release_ms: 300,
            threshold: 0.05,
            window_ms: 50,
        }
    }
}

impl DuckingConfig {
    /// Create a new ducking config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a ducking config optimized for voiceover ducking
    pub fn voiceover() -> Self {
        Self {
            enabled: true,
            duck_level: 0.25,
            attack_ms: 30,
            release_ms: 200,
            threshold: 0.04,
            window_ms: 30,
        }
    }

    /// Create a ducking config optimized for music ducking
    pub fn music() -> Self {
        Self {
            enabled: true,
            duck_level: 0.4,
            attack_ms: 100,
            release_ms: 500,
            threshold: 0.06,
            window_ms: 50,
        }
    }
}

/// Apply ducking to an audio buffer based on a trigger signal
///
/// The `main_buffer` is the audio that will have its volume reduced
/// when the `trigger_buffer` is active (e.g., voiceover).
///
/// # Arguments
/// * `main_buffer` - The audio buffer to duck (e.g., background music)
/// * `trigger_buffer` - The audio buffer that triggers ducking (e.g., voiceover)
/// * `config` - Ducking configuration
pub fn apply_ducking(
    main_buffer: &mut AudioBuffer,
    trigger_buffer: &AudioBuffer,
    config: &DuckingConfig,
) {
    if !config.enabled {
        return;
    }

    let channels = main_buffer.channels as usize;
    let sample_rate = main_buffer.sample_rate as f64;

    // Calculate sample-level timing
    let samples_per_ms = (sample_rate * channels as f64 / 1000.0) as usize;
    let attack_samples = (config.attack_ms as usize * samples_per_ms).max(1);
    let release_samples = (config.release_ms as usize * samples_per_ms).max(1);

    // Window size for RMS calculation
    let window_size = (config.window_ms as usize * samples_per_ms).max(1);

    let min_len = main_buffer.samples.len().min(trigger_buffer.samples.len());

    let mut is_ducking = false;
    let mut duck_progress = 0.0f32; // 0.0 = no ducking, 1.0 = fully ducked

    for window_start in (0..min_len).step_by(window_size) {
        let window_end = (window_start + window_size).min(trigger_buffer.samples.len());

        // Calculate RMS energy of the trigger signal in this window
        let window = &trigger_buffer.samples[window_start..window_end];
        let rms = if window.is_empty() {
            0.0
        } else {
            (window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32).sqrt()
        };

        let should_duck = rms > config.threshold;

        // Update ducking state with hysteresis
        if should_duck && !is_ducking {
            is_ducking = true;
        } else if !should_duck && is_ducking {
            is_ducking = false;
        }

        // Apply smooth volume transition within this window
        for j in window_start..window_end.min(min_len) {
            // Calculate rate of transition based on direction
            if is_ducking {
                // Attack: volume drops (duck_progress increases toward 1.0)
                duck_progress = (duck_progress + 1.0 / attack_samples as f32).min(1.0);
            } else {
                // Release: volume recovers (duck_progress decreases toward 0.0)
                duck_progress = (duck_progress - 1.0 / release_samples as f32).max(0.0);
            }

            // Calculate volume: 1.0 at no ducking, config.duck_level at full ducking
            let volume = 1.0 - (1.0 - config.duck_level) * duck_progress;
            main_buffer.samples[j] *= volume;
        }
    }
}

/// Detect speech segments in an audio buffer
///
/// Returns a list of (start_ms, end_ms) tuples indicating when
/// speech is detected based on the RMS energy threshold.
pub fn detect_speech_segments(
    buffer: &AudioBuffer,
    threshold: f32,
    window_ms: u64,
    min_silence_ms: u64,
) -> Vec<(u64, u64)> {
    let channels = buffer.channels as usize;
    let sample_rate = buffer.sample_rate as f64;
    let samples_per_ms = (sample_rate * channels as f64 / 1000.0) as usize;
    let window_size = (window_ms as usize * samples_per_ms).max(1);
    let min_silence_samples = (min_silence_ms as usize * samples_per_ms).max(1);

    let mut segments: Vec<(u64, u64)> = Vec::new();
    let mut segment_start: Option<usize> = None;
    let mut silence_count = 0;

    for window_start in (0..buffer.samples.len()).step_by(window_size) {
        let window_end = (window_start + window_size).min(buffer.samples.len());

        let rms = {
            let window = &buffer.samples[window_start..window_end];
            if window.is_empty() {
                0.0
            } else {
                (window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32).sqrt()
            }
        };

        let current_ms = (window_start as f64 * 1000.0 / (sample_rate * channels as f64)) as u64;

        if rms > threshold {
            if segment_start.is_none() {
                segment_start = Some(window_start);
            }
            silence_count = 0;
        } else if segment_start.is_some() {
            silence_count += window_size;
            if silence_count >= min_silence_samples {
                let start_ms = (segment_start.unwrap() as f64 * 1000.0
                    / (sample_rate * channels as f64)) as u64;
                segments.push((start_ms, current_ms));
                segment_start = None;
                silence_count = 0;
            }
        }
    }

    // Close any open segment
    if let Some(start) = segment_start {
        let start_ms = (start as f64 * 1000.0 / (sample_rate * channels as f64)) as u64;
        segments.push((start_ms, buffer.duration_ms));
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::mixer::AudioMixer;

    #[test]
    fn test_ducking_config_defaults() {
        let config = DuckingConfig::default();
        assert!(!config.enabled);
        assert!((config.duck_level - 0.3).abs() < 0.001);
        assert_eq!(config.attack_ms, 50);
        assert_eq!(config.release_ms, 300);
    }

    #[test]
    fn test_ducking_with_silent_trigger() {
        let mut main = AudioBuffer::new(44100, 2, 1000);
        main.samples.fill(0.5); // Constant audio

        let trigger = AudioBuffer::new(44100, 2, 1000); // Silent trigger

        let config = DuckingConfig {
            enabled: true,
            duck_level: 0.3,
            ..Default::default()
        };

        apply_ducking(&mut main, &trigger, &config);

        // With silent trigger, main should remain at full volume
        let avg = main.samples.iter().map(|s| s.abs()).sum::<f32>() / main.samples.len() as f32;
        assert!(avg > 0.4, "Expected near-full volume, got {}", avg);
    }

    #[test]
    fn test_ducking_with_active_trigger() {
        let mut main = AudioBuffer::new(44100, 2, 1000);
        main.samples.fill(0.5);

        let mut trigger = AudioBuffer::new(44100, 2, 1000);
        trigger.samples.fill(0.8); // Loud trigger

        let config = DuckingConfig {
            enabled: true,
            duck_level: 0.3,
            attack_ms: 10,
            release_ms: 100,
            threshold: 0.05,
            window_ms: 10,
        };

        apply_ducking(&mut main, &trigger, &config);

        // With active trigger, main should be ducked
        // Check the last portion where ducking should be fully active
        let tail_start = main.samples.len() * 3 / 4;
        let tail_avg: f32 = main.samples[tail_start..]
            .iter()
            .map(|s| s.abs())
            .sum::<f32>()
            / (main.samples.len() - tail_start) as f32;

        assert!(
            tail_avg < 0.5 * config.duck_level * 1.5,
            "Expected ducked volume (~{}), got {}",
            0.5 * config.duck_level,
            tail_avg
        );
    }

    #[test]
    fn test_speech_detection() {
        let mut buffer = AudioBuffer::new(44100, 2, 2000);
        // First half: silent, second half: "speech"
        let half = buffer.samples.len() / 2;
        buffer.samples[half..].fill(0.5);

        let segments = detect_speech_segments(&buffer, 0.05, 50, 200);
        assert!(
            !segments.is_empty(),
            "Should detect at least one speech segment"
        );
    }
}
