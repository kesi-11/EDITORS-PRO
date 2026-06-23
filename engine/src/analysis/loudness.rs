use serde::{Deserialize, Serialize};

/// Loudness analysis utilities (EBU R128 style).

/// Compute RMS (root mean square) level of audio samples in dB.
pub fn compute_rms_db(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return -f64::INFINITY;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    let rms = (sum_sq / samples.len() as f64).sqrt();
    if rms < f64::EPSILON {
        -f64::INFINITY
    } else {
        20.0 * rms.log10()
    }
}

/// Compute peak level of audio samples in dB.
pub fn compute_peak_db(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return -f64::INFINITY;
    }
    let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    if peak < f32::EPSILON {
        -f64::INFINITY
    } else {
        20.0 * (peak as f64).log10()
    }
}

/// Compute LUFS (Loudness Units Full Scale) - simplified K-weighted.
/// This is a simplified approximation; a full EBU R128 implementation
/// requires K-weighting filters.
pub fn compute_lufs(samples: &[f32], sample_rate: u32) -> f64 {
    // Simplified: just compute RMS and add offset
    let rms = compute_rms_db(samples);
    // Approximate LUFS offset for K-weighting
    rms - 0.691
}

/// Phase F.5: Compute short-term LUFS (3-second window, per EBU R128).
pub fn compute_short_term_lufs(samples: &[f32], sample_rate: u32) -> f64 {
    let window_samples = (sample_rate as usize) * 3;
    if samples.len() < window_samples {
        return compute_lufs(samples, sample_rate);
    }
    let window = &samples[samples.len() - window_samples..];
    compute_lufs(window, sample_rate)
}

/// Phase F.5: Compute momentary LUFS (400ms window, per EBU R128).
pub fn compute_momentary_lufs(samples: &[f32], sample_rate: u32) -> f64 {
    let window_samples = (sample_rate as usize * 400) / 1000;
    if samples.len() < window_samples || window_samples == 0 {
        return compute_lufs(samples, sample_rate);
    }
    let window = &samples[samples.len() - window_samples..];
    compute_lufs(window, sample_rate)
}

/// Audio loudness stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoudnessStats {
    pub rms_db: f64,
    pub peak_db: f64,
    pub lufs: f64,
    pub true_peak: f32,
}

/// Analyze audio loudness.
pub fn analyze_loudness(samples: &[f32], sample_rate: u32) -> LoudnessStats {
    LoudnessStats {
        rms_db: compute_rms_db(samples),
        peak_db: compute_peak_db(samples),
        lufs: compute_lufs(samples, sample_rate),
        true_peak: samples.iter().fold(0.0f32, |a, &b| a.max(b.abs())),
    }
}

/// Phase F.5: Extended loudness stats with windowed measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedLoudnessStats {
    pub rms_db: f64,
    pub peak_db: f64,
    pub integrated_lufs: f64,
    pub short_term_lufs: f64,
    pub momentary_lufs: f64,
    pub true_peak: f32,
}

/// Phase F.5: Analyze audio loudness with windowed short-term + momentary.
pub fn analyze_loudness_extended(samples: &[f32], sample_rate: u32) -> ExtendedLoudnessStats {
    ExtendedLoudnessStats {
        rms_db: compute_rms_db(samples),
        peak_db: compute_peak_db(samples),
        integrated_lufs: compute_lufs(samples, sample_rate),
        short_term_lufs: compute_short_term_lufs(samples, sample_rate),
        momentary_lufs: compute_momentary_lufs(samples, sample_rate),
        true_peak: samples.iter().fold(0.0f32, |a, &b| a.max(b.abs())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_silence() {
        let samples = vec![0.0f32; 1000];
        let rms = compute_rms_db(&samples);
        assert!(rms.is_infinite() && rms.is_sign_negative());
    }

    #[test]
    fn test_peak_silence() {
        let samples = vec![0.0f32; 1000];
        let peak = compute_peak_db(&samples);
        assert!(peak.is_infinite() && peak.is_sign_negative());
    }

    #[test]
    fn test_rms_full_scale() {
        let samples = vec![1.0f32; 100];
        let rms = compute_rms_db(&samples);
        assert!((rms - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_analyze_loudness() {
        let samples = vec![0.5f32; 1000];
        let stats = analyze_loudness(&samples, 48000);
        assert!(!stats.rms_db.is_nan());
        assert!(!stats.peak_db.is_nan());
        assert!(!stats.lufs.is_nan());
    }
}
