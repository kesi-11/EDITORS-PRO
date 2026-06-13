//! Tests for audio/waveform — WaveformData generation and queries

use crate::audio::waveform::WaveformData;

#[test]
fn waveform_from_empty_samples() {
    let wf = WaveformData::from_samples(&[], 44100, 2, 100);
    assert_eq!(wf.peaks.len(), 100);
    assert_eq!(wf.rms_values.len(), 100);
    assert!(wf.peaks.iter().all(|&p| p == 0.0));
    assert_eq!(wf.duration_ms, 0);
}

#[test]
fn waveform_from_zero_sample_rate() {
    let samples = vec![0.5, 0.3, -0.2, 0.8];
    let wf = WaveformData::from_samples(&samples, 0, 2, 50);
    assert_eq!(wf.peaks.len(), 50);
    assert!(wf.peaks.iter().all(|&p| p == 0.0));
}

#[test]
fn waveform_from_silence() {
    let silence = vec![0.0f32; 44100 * 2]; // 1 second stereo
    let wf = WaveformData::from_samples(&silence, 44100, 2, 100);
    assert_eq!(wf.peaks.len(), 100);
    assert!(wf.peaks.iter().all(|&p| p == 0.0));
    assert_eq!(wf.duration_ms, 1000);
}

#[test]
fn waveform_from_constant_signal() {
    let signal: Vec<f32> = vec![0.5; 44100 * 2]; // 1 second stereo at 0.5
    let wf = WaveformData::from_samples(&signal, 44100, 2, 50);
    assert_eq!(wf.peaks.len(), 50);
    // All peaks should be close to 0.5
    for &peak in &wf.peaks {
        assert!((peak - 0.5).abs() < 0.01);
    }
}

#[test]
fn waveform_captures_peak_amplitude() {
    // Create a signal with a single loud sample
    let mut samples = vec![0.1f32; 44100 * 2];
    samples[1000] = 0.9;
    let wf = WaveformData::from_samples(&samples, 44100, 2, 100);
    // At least one peak should be high
    assert!(wf.peaks.iter().any(|&p| p > 0.5));
}

#[test]
fn waveform_duration_calculation() {
    // 22050 samples, mono → 0.5 seconds at 44100 Hz
    let samples = vec![0.5f32; 22050];
    let wf = WaveformData::from_samples(&samples, 44100, 1, 50);
    assert_eq!(wf.duration_ms, 500);
}

#[test]
fn waveform_simple_peaks_empty() {
    let peaks = WaveformData::simple_peaks(&[], 50);
    assert_eq!(peaks.len(), 50);
    assert!(peaks.iter().all(|&p| p == 0.0));
}

#[test]
fn waveform_simple_peaks_basic() {
    let samples = vec![0.5f32; 100];
    let peaks = WaveformData::simple_peaks(&samples, 10);
    assert_eq!(peaks.len(), 10);
    for &p in &peaks {
        assert!((p - 0.5).abs() < 0.01);
    }
}

#[test]
fn waveform_peak_at_position() {
    let mut wf = WaveformData {
        peaks: vec![0.1, 0.3, 0.5, 0.7, 0.9],
        rms_values: vec![0.05, 0.15, 0.25, 0.35, 0.45],
        samples_per_peak: 100,
        duration_ms: 5000,
    };

    assert!((wf.peak_at_position(0.0) - 0.1).abs() < f32::EPSILON);
    assert!((wf.peak_at_position(1.0) - 0.9).abs() < f32::EPSILON);
    assert!((wf.peak_at_position(0.5) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn waveform_peak_at_position_empty() {
    let wf = WaveformData {
        peaks: vec![],
        rms_values: vec![],
        samples_per_peak: 0,
        duration_ms: 0,
    };
    assert!((wf.peak_at_position(0.5) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn waveform_serialization_roundtrip() {
    let wf = WaveformData {
        peaks: vec![0.1, 0.5, 0.9],
        rms_values: vec![0.05, 0.25, 0.45],
        samples_per_peak: 100,
        duration_ms: 3000,
    };
    let json = serde_json::to_string(&wf).unwrap();
    let parsed: WaveformData = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.peaks, wf.peaks);
    assert_eq!(parsed.rms_values, wf.rms_values);
    assert_eq!(parsed.samples_per_peak, 100);
    assert_eq!(parsed.duration_ms, 3000);
}
