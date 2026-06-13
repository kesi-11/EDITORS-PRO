/// Audio effects processing.

/// Apply a simple low-pass filter to audio samples.
pub fn low_pass_filter(samples: &[f32], cutoff_ratio: f32) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_ratio);
    let dt = 1.0;
    let alpha = dt / (rc + dt);

    let mut output = Vec::with_capacity(samples.len());
    let mut prev = 0.0f32;
    for &s in samples {
        prev = prev + alpha * (s - prev);
        output.push(prev);
    }
    output
}

/// Apply a simple high-pass filter to audio samples.
pub fn high_pass_filter(samples: &[f32], cutoff_ratio: f32) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_ratio);
    let dt = 1.0;
    let alpha = rc / (rc + dt);

    let mut output = Vec::with_capacity(samples.len());
    let mut prev_in = 0.0f32;
    let mut prev_out = 0.0f32;
    for &s in samples {
        let y = alpha * (prev_out + s - prev_in);
        prev_in = s;
        prev_out = y;
        output.push(y);
    }
    output
}

/// Normalize audio samples to a target peak level.
pub fn normalize(samples: &[f32], target_peak: f32) -> Vec<f32> {
    let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    if peak < f32::EPSILON {
        return samples.to_vec();
    }
    let gain = target_peak / peak;
    samples.iter().map(|&s| s * gain).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_pass_filter() {
        let samples = vec![1.0f32, 0.0, 0.0, 0.0, 0.0];
        let result = low_pass_filter(&samples, 0.1);
        assert!(!result.is_empty());
        // Should gradually decrease from 1.0
        assert!(result[0] > 0.0);
    }

    #[test]
    fn test_high_pass_filter() {
        let samples = vec![1.0f32, 1.0, 1.0, 1.0, 1.0];
        let result = high_pass_filter(&samples, 0.1);
        assert!(!result.is_empty());
        // Constant input should produce near-zero output after initial transient
    }

    #[test]
    fn test_normalize() {
        let samples = vec![0.5f32, -0.5, 0.25];
        let result = normalize(&samples, 1.0);
        assert!((result[0] - 1.0).abs() < 1e-5);
        assert!((result[1] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_silence() {
        let samples = vec![0.0f32; 100];
        let result = normalize(&samples, 1.0);
        assert!(result.iter().all(|&s| s == 0.0));
    }
}
