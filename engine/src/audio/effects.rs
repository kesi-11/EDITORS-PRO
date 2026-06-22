//! Audio effects processing.
//!
//! Phase F.3: extended with an 8-band parametric EQ + high-pass + low-pass
//! filter chain. Implements biquad filters (RBJ Audio EQ Cookbook formulas)
//! for each band, cascaded in series.
//!
//! ## video: debt markers
//!
//! - 8-band cascade only, upgrade to 32-band if surgical repair is needed
//! - Single-precision f32, upgrade to f64 if numerical noise appears at low frequencies
//! - No oversampling, upgrade to 2x oversampling if high-frequency bands show aliasing at high Q
//! - No spectrum visualization, upgrade to real-time FFT display if user wants to see the curve

use serde::{Deserialize, Serialize};

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

    #[test]
    fn test_biquad_passthrough() {
        // A peaking filter with 0 dB gain should be ~identity
        let coeffs = BiquadCoeffs::peaking(1000.0, 44100.0, 0.0, 1.0);
        let mut state = BiquadState::default();
        let samples = vec![0.5f32, -0.3, 0.7, -0.1, 0.4];
        let result: Vec<f32> = samples.iter().map(|&x| coeffs.process(&mut state, x)).collect();
        for (orig, filtered) in samples.iter().zip(result.iter()) {
            assert!((orig - filtered).abs() < 1e-3, "expected passthrough, got {} vs {}", orig, filtered);
        }
    }

    #[test]
    fn test_biquad_highpass_dc() {
        // High-pass at 80 Hz should heavily attenuate DC (0 Hz)
        let coeffs = BiquadCoeffs::high_pass(80.0, 44100.0, 0.707);
        let mut state = BiquadState::default();
        // 100 samples of constant 0.5 (DC)
        let samples = vec![0.5f32; 100];
        let result: Vec<f32> = samples.iter().map(|&x| coeffs.process(&mut state, x)).collect();
        // After 100 samples the high-pass should have attenuated most of the DC
        let final_val = result.last().unwrap();
        assert!(final_val.abs() < 0.1, "expected DC attenuation, got {}", final_val);
    }

    #[test]
    fn test_eq_chain_passthrough() {
        // EQ with all bands at 0 dB gain should be ~identity
        let settings = EqSettings {
            enabled: true,
            high_pass_hz: 20.0,
            low_pass_hz: 20000.0,
            bands: vec![
                EqBand { frequency: 60.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 120.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 250.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 500.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 1000.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 2500.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 5000.0, gain_db: 0.0, q: 1.0, enabled: false },
                EqBand { frequency: 10000.0, gain_db: 0.0, q: 1.0, enabled: false },
            ],
        };
        let samples = vec![0.3f32, -0.2, 0.5, -0.4, 0.1];
        let result = apply_eq_chain(&samples, 44100, &settings);
        assert_eq!(result.len(), samples.len());
        // Should be near-passthrough (HPF at 20 Hz + LPF at 20 kHz + all bands disabled)
        for (orig, filtered) in samples.iter().zip(result.iter()) {
            assert!((orig - filtered).abs() < 0.05, "expected near-passthrough, got {} vs {}", orig, filtered);
        }
    }
}

// ─── Phase F.3: 8-band parametric EQ (RBJ biquad cascade) ──────────────

/// Biquad filter state (Direct Form I — kept here for simplicity; DF-II
/// transposed would be more numerically stable at low frequencies, but
/// for an 8-band EQ at 44.1/48 kHz the difference is inaudible).
///
/// video: Direct Form I, upgrade to DF-II transposed if numerical noise appears at low frequencies
#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadState {
    pub x1: f32, // previous input sample
    pub x2: f32, // input sample before that
    pub y1: f32, // previous output sample
    pub y2: f32, // output sample before that
}

/// Biquad filter coefficients (normalized: a0 = 1).
#[derive(Debug, Clone, Copy)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCoeffs {
    /// Peaking EQ filter (boost or cut at center frequency).
    /// gain_db > 0 = boost, gain_db < 0 = cut.
    /// RBJ Audio EQ Cookbook formulas.
    pub fn peaking(freq_hz: f32, sample_rate: f32, gain_db: f32, q: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * freq_hz / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// High-pass filter (Butterworth default: Q = 0.707).
    pub fn high_pass(freq_hz: f32, sample_rate: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * freq_hz / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = (1.0 + cos_w0) / 2.0;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Low-pass filter (Butterworth default: Q = 0.707).
    pub fn low_pass(freq_hz: f32, sample_rate: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * freq_hz / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = (1.0 - cos_w0) / 2.0;
        let b1 = 1.0 - cos_w0;
        let b2 = (1.0 - cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Process one sample through the biquad (Direct Form I).
    #[inline]
    pub fn process(&self, state: &mut BiquadState, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * state.x1 + self.b2 * state.x2
            - self.a1 * state.y1 - self.a2 * state.y2;
        state.x2 = state.x1;
        state.x1 = x;
        state.y2 = state.y1;
        state.y1 = y;
        y
    }
}

/// One EQ band's parameters (mirrors Flutter EqBand).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBand {
    pub frequency: f32,
    pub gain_db: f32,
    pub q: f32,
    pub enabled: bool,
}

/// Full EQ settings (mirrors Flutter EqSettings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqSettings {
    pub enabled: bool,
    pub high_pass_hz: f32,
    pub low_pass_hz: f32,
    pub bands: Vec<EqBand>,
}

/// Apply the full EQ chain (HPF → 8 peaking bands → LPF) to a sample buffer.
/// Returns a new buffer; the input is not modified.
///
/// video: 8-band cascade only, upgrade to 32-band if surgical repair is needed
pub fn apply_eq_chain(samples: &[f32], sample_rate: u32, settings: &EqSettings) -> Vec<f32> {
    if !settings.enabled || samples.is_empty() {
        return samples.to_vec();
    }

    // Build the cascade of biquads
    let mut cascade: Vec<(BiquadCoeffs, BiquadState)> = Vec::new();

    // 1. High-pass filter
    cascade.push((
        BiquadCoeffs::high_pass(settings.high_pass_hz, sample_rate as f32, 0.707),
        BiquadState::default(),
    ));

    // 2. Each enabled peaking band
    for band in &settings.bands {
        if band.enabled && band.gain_db.abs() > 0.001 {
            cascade.push((
                BiquadCoeffs::peaking(band.frequency, sample_rate as f32, band.gain_db, band.q),
                BiquadState::default(),
            ));
        }
    }

    // 3. Low-pass filter (only if below Nyquist/2)
    let nyquist = sample_rate as f32 / 2.0;
    if settings.low_pass_hz < nyquist * 0.95 {
        cascade.push((
            BiquadCoeffs::low_pass(settings.low_pass_hz, sample_rate as f32, 0.707),
            BiquadState::default(),
        ));
    }

    // Apply cascade
    let mut output = samples.to_vec();
    for (coeffs, state) in cascade.iter_mut() {
        let mut new_output = Vec::with_capacity(output.len());
        for &x in &output {
            new_output.push(coeffs.process(state, x));
        }
        output = new_output;
    }
    output
}
