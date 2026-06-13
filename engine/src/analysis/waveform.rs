/// Waveform analysis utilities.

/// Compute a waveform overview from audio samples.
/// Returns a vector of (min, max) pairs for display purposes.
pub fn compute_waveform(samples: &[f32], samples_per_pixel: usize) -> Vec<(f32, f32)> {
    let mut waveform = Vec::new();
    for chunk in samples.chunks(samples_per_pixel.max(1)) {
        let min = chunk.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = chunk.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        waveform.push((min, max));
    }
    waveform
}

/// Normalize a waveform to -1.0 to 1.0 range.
pub fn normalize_waveform(waveform: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let max_abs = waveform
        .iter()
        .flat_map(|(min, max)| [min.abs(), max.abs()])
        .fold(0.0f32, |a, b| a.max(b));

    if max_abs < f32::EPSILON {
        return waveform.to_vec();
    }

    waveform
        .iter()
        .map(|(min, max)| (min / max_abs, max / max_abs))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_waveform_empty() {
        let result = compute_waveform(&[], 100);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_waveform_silence() {
        let samples = vec![0.0f32; 1000];
        let result = compute_waveform(&samples, 100);
        assert_eq!(result.len(), 10);
        for (min, max) in &result {
            assert!((min - 0.0).abs() < 1e-5);
            assert!((max - 0.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_normalize_waveform() {
        let waveform = vec![(-0.5f32, 0.5f32), (-1.0f32, 1.0f32)];
        let normalized = normalize_waveform(&waveform);
        assert!((normalized[0].0 - (-0.5)).abs() < 1e-5);
        assert!((normalized[1].0 - (-1.0)).abs() < 1e-5);
    }
}
