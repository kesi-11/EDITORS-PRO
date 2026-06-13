//! Test utilities and helpers for the EDITORS-PRO engine test suite
//!
//! Provides shared fixtures, mock data generators, and assertion helpers
//! used across multiple test modules.

pub mod perf_tests;

/// Create a sample VideoInfo for testing (1920x1080, 30fps, 60s)
pub fn sample_video_info() -> crate::decoder::VideoInfo {
    crate::decoder::VideoInfo {
        width: 1920,
        height: 1080,
        fps: 30.0,
        duration_ms: 60000,
        codec_name: "h264".to_string(),
        bitrate: 5000000,
        has_audio: true,
        audio_codec: Some("aac".to_string()),
        audio_sample_rate: Some(44100),
        audio_channels: Some(2),
    }
}

/// Create a sample 4K VideoInfo for testing
pub fn sample_4k_video_info() -> crate::decoder::VideoInfo {
    crate::decoder::VideoInfo {
        width: 3840,
        height: 2160,
        fps: 60.0,
        duration_ms: 120000,
        codec_name: "h265".to_string(),
        bitrate: 20000000,
        has_audio: true,
        audio_codec: Some("aac".to_string()),
        audio_sample_rate: Some(48000),
        audio_channels: Some(2),
    }
}

/// Generate a sine wave audio buffer for testing
///
/// Creates `duration_ms` milliseconds of a sine wave at the given
/// frequency and sample rate. Returns interleaved stereo samples.
pub fn generate_sine_wave(
    frequency_hz: f32,
    sample_rate: u32,
    duration_ms: u64,
    amplitude: f32,
) -> Vec<f32> {
    let num_samples = (sample_rate as u64 * duration_ms / 1000) as usize;
    let mut samples = Vec::with_capacity(num_samples * 2); // stereo
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let value = amplitude * (2.0 * std::f32::consts::PI * frequency_hz * t).sin();
        samples.push(value); // left channel
        samples.push(value); // right channel
    }
    samples
}

/// Generate white noise audio buffer for testing
///
/// Uses a simple linear congruential generator for reproducibility.
pub fn generate_white_noise(
    sample_rate: u32,
    duration_ms: u64,
    amplitude: f32,
    seed: u64,
) -> Vec<f32> {
    let num_samples = (sample_rate as u64 * duration_ms / 1000) as usize;
    let mut samples = Vec::with_capacity(num_samples * 2);
    let mut state = seed;
    for _ in 0..num_samples {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let raw = ((state >> 33) as i32) as f32 / (i32::MAX as f32);
        let value = amplitude * raw;
        samples.push(value);
        samples.push(value);
    }
    samples
}

/// Assert that two floating-point values are approximately equal
pub fn assert_approx_eq(a: f32, b: f32, tolerance: f32) {
    assert!(
        (a - b).abs() <= tolerance,
        "Values are not approximately equal: {} vs {} (tolerance: {})",
        a, b, tolerance
    );
}

/// Assert that a floating-point value is within a range
pub fn assert_in_range(value: f32, min: f32, max: f32) {
    assert!(
        value >= min && value <= max,
        "Value {} is not in range [{}, {}]",
        value, min, max
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_video_info_has_expected_values() {
        let info = sample_video_info();
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
        assert_eq!(info.fps, 30.0);
    }

    #[test]
    fn sample_4k_video_info_has_expected_values() {
        let info = sample_4k_video_info();
        assert_eq!(info.width, 3840);
        assert_eq!(info.height, 2160);
        assert_eq!(info.fps, 60.0);
    }

    #[test]
    fn generate_sine_wave_produces_correct_length() {
        let samples = generate_sine_wave(440.0, 44100, 1000, 0.5);
        // 1 second of stereo audio at 44100 Hz = 88200 samples
        assert_eq!(samples.len(), 88200);
    }

    #[test]
    fn generate_sine_wave_amplitude_is_bounded() {
        let samples = generate_sine_wave(440.0, 44100, 100, 0.8);
        let max_val = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_val <= 0.81); // some tolerance
    }

    #[test]
    fn generate_white_noise_produces_correct_length() {
        let samples = generate_white_noise(44100, 1000, 0.5, 42);
        assert_eq!(samples.len(), 88200);
    }

    #[test]
    fn generate_white_noise_is_reproducible() {
        let a = generate_white_noise(44100, 100, 0.5, 42);
        let b = generate_white_noise(44100, 100, 0.5, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn assert_approx_eq_does_not_panic_for_close_values() {
        assert_approx_eq(1.0, 1.0001, 0.001);
    }

    #[test]
    #[should_panic]
    fn assert_approx_eq_panics_for_far_values() {
        assert_approx_eq(1.0, 2.0, 0.001);
    }

    #[test]
    fn assert_in_range_does_not_panic_for_valid_range() {
        assert_in_range(0.5, 0.0, 1.0);
    }

    #[test]
    #[should_panic]
    fn assert_in_range_panics_for_out_of_range() {
        assert_in_range(1.5, 0.0, 1.0);
    }
}
