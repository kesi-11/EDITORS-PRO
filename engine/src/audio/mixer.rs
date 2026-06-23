//! Audio mixer - Combines multiple audio tracks with volume control
//!
//! Mixes audio from multiple tracks, applying volume, fade in/out,
//! and ducking effects to produce a final audio output.
//! Supports resampling to a common sample rate for mixing.

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
        let sample_count =
            (sample_rate as f64 * channels as f64 * duration_ms as f64 / 1000.0) as usize;
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

    /// Get samples at a specific time offset
    ///
    /// Returns a slice of interleaved samples for the given time range.
    pub fn samples_at_time(&self, time_ms: u64, duration_ms: u64) -> &[f32] {
        let start_sample =
            (time_ms as f64 * self.sample_rate as f64 * self.channels as f64 / 1000.0) as usize;
        let sample_count =
            (duration_ms as f64 * self.sample_rate as f64 * self.channels as f64 / 1000.0) as usize;

        let end = (start_sample + sample_count).min(self.samples.len());
        if start_sample >= self.samples.len() {
            return &[];
        }
        &self.samples[start_sample..end]
    }

    /// Calculate the peak amplitude of the buffer
    pub fn peak_amplitude(&self) -> f32 {
        self.samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    /// Calculate the RMS (root mean square) energy of the buffer
    pub fn rms_energy(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        (self.samples.iter().map(|s| s * s).sum::<f32>() / self.samples.len() as f32).sqrt()
    }

    /// Get a segment of the audio between start_ms and end_ms
    ///
    /// Returns a new AudioBuffer containing only the samples
    /// in the specified time range.
    pub fn segment(&self, start_ms: u64, end_ms: u64) -> AudioBuffer {
        if self.samples.is_empty() || self.sample_rate == 0 {
            return AudioBuffer::new(self.sample_rate, self.channels, 0);
        }

        let start_sample =
            (start_ms as f64 * self.sample_rate as f64 * self.channels as f64 / 1000.0) as usize;
        let end_sample =
            (end_ms as f64 * self.sample_rate as f64 * self.channels as f64 / 1000.0) as usize;

        let start = start_sample.min(self.samples.len());
        let end = end_sample.min(self.samples.len());

        let segment_duration = if end > start {
            ((end - start) as f64 * 1000.0 / (self.sample_rate as f64 * self.channels as f64))
                as u64
        } else {
            0
        };

        AudioBuffer {
            samples: self.samples[start..end].to_vec(),
            sample_rate: self.sample_rate,
            channels: self.channels,
            start_ms: self.start_ms + start_ms,
            duration_ms: segment_duration,
        }
    }
}

impl From<crate::audio::decoder::DecodedAudio> for AudioBuffer {
    /// Convert DecodedAudio from the FFmpeg decoder into an AudioBuffer
    /// for use in the mixer and audio pipeline.
    fn from(audio: crate::audio::decoder::DecodedAudio) -> Self {
        AudioBuffer {
            samples: audio.samples,
            sample_rate: audio.sample_rate,
            channels: audio.channels,
            start_ms: 0,
            duration_ms: audio.duration_ms,
        }
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

/// Represents a track's audio contribution for mixing
#[derive(Debug, Clone)]
pub struct TrackAudioSource {
    /// The audio buffer for this track
    pub buffer: AudioBuffer,
    /// Volume level for this track (0.0 to 2.0, 1.0 = normal)
    pub volume: f32,
    /// Offset in milliseconds from the timeline start
    pub offset_ms: u64,
    /// Volume envelope for fade effects
    pub envelope: VolumeEnvelope,
    /// Whether this track is muted
    pub muted: bool,
    /// Phase F.4: Pan (-1.0 = full left, 0.0 = center, +1.0 = full right).
    /// Applied after volume + envelope, before mixing into the output.
    /// For mono buffers, this attenuates the opposite channel.
    /// For stereo buffers, this crossfades between channels.
    /// video: pan uses constant-power panning law; upgrade to true stereo
    /// panner if surround sound is needed.
    pub pan: f32,
    /// Phase F.4: Whether this track is soloed. When any source has
    /// `solo = true`, all sources with `solo = false` are muted during mixing.
    pub solo: bool,
    /// Phase F.4: Optional per-track EQ settings. Applied before volume +
    /// pan, after envelope. None = no EQ processing.
    /// video: EQ applied per-track during mix, upgrade to real-time IIR
    /// streaming when the audio pipeline supports per-sample effect chains.
    pub eq_settings: Option<crate::audio::effects::EqSettings>,
}

/// The audio mixer combines multiple audio sources
pub struct AudioMixer {
    sample_rate: u32,
    channels: u32,
}

impl AudioMixer {
    pub fn new(sample_rate: u32, channels: u32) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    /// Mix multiple audio track sources into a single output
    ///
    /// Each source has its own volume, offset, and envelope settings.
    /// The output is at the mixer's configured sample rate and channel count.
    /// Soft clipping (tanh) is applied to prevent harsh distortion.
    pub fn mix_sources(
        &self,
        sources: &[TrackAudioSource],
        output_duration_ms: u64,
    ) -> AudioBuffer {
        if sources.is_empty() || output_duration_ms == 0 {
            return AudioBuffer::new(self.sample_rate, self.channels, 0);
        }

        // Phase F.4: if any source is soloed, mute all non-soloed sources.
        let any_soloed = sources.iter().any(|s| s.solo);

        let output_sample_count =
            (self.sample_rate as f64 * self.channels as f64 * output_duration_ms as f64 / 1000.0)
                as usize;
        let mut output = vec![0.0f32; output_sample_count];

        for source in sources {
            // Phase F.4: respect solo — if any track is soloed, skip non-soloed.
            if any_soloed && !source.solo {
                continue;
            }
            if source.muted || source.volume <= 0.0 {
                continue;
            }

            // Calculate the offset in samples
            let offset_samples =
                (source.offset_ms as f64 * self.sample_rate as f64 * self.channels as f64 / 1000.0)
                    as usize;

            // Apply volume envelope first
            let mut processed = source.buffer.clone();
            self.apply_envelope(&mut processed, &source.envelope);

            // Phase F.4: apply per-track EQ if configured
            if let Some(eq) = &source.eq_settings {
                if eq.enabled {
                    processed.samples = crate::audio::effects::apply_eq_chain(
                        &processed.samples,
                        self.sample_rate,
                        eq,
                    );
                }
            }

            // Phase F.4: apply pan (constant-power panning law).
            // For stereo output: left = cos(theta), right = sin(theta)
            // where theta = (pan + 1) * pi/4 (maps -1..+1 to 0..pi/2)
            // At pan=0 (center): left_gain = right_gain = cos(pi/4) ≈ 0.707
            // At pan=-1 (full left): left_gain = 1.0, right_gain = 0.0
            // At pan=+1 (full right): left_gain = 0.0, right_gain = 1.0
            let pan = source.pan.clamp(-1.0, 1.0);
            let theta = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
            let left_gain = theta.cos();
            let right_gain = theta.sin();

            if self.channels == 2 && processed.channels == 2 {
                // Stereo-to-stereo: preserve width at center, collapse at extremes
                for i in (0..processed.samples.len()).step_by(2) {
                    let out_idx = offset_samples + i;
                    if out_idx + 1 < output.len() {
                        let l = processed.samples[i] * source.volume;
                        let r = processed.samples[i + 1] * source.volume;
                        output[out_idx] += l * left_gain + r * (1.0 - left_gain);
                        output[out_idx + 1] += r * right_gain + l * (1.0 - right_gain);
                    }
                }
            } else if self.channels == 2 && processed.channels == 1 {
                // Mono-to-stereo: apply pan directly
                for (i, &sample) in processed.samples.iter().enumerate() {
                    let out_idx = offset_samples + i * 2;
                    if out_idx + 1 < output.len() {
                        let s = sample * source.volume;
                        output[out_idx] += s * left_gain;
                        output[out_idx + 1] += s * right_gain;
                    }
                }
            } else {
                for (i, &sample) in processed.samples.iter().enumerate() {
                    let out_idx = offset_samples + i;
                    if out_idx < output.len() {
                        output[out_idx] += sample * source.volume;
                    }
                }
            }
        }

        // Soft clipping using tanh to prevent harsh distortion
        for sample in output.iter_mut() {
            *sample = sample.tanh();
        }

        AudioBuffer {
            samples: output,
            sample_rate: self.sample_rate,
            channels: self.channels,
            start_ms: 0,
            duration_ms: output_duration_ms,
        }
    }

    /// Mix multiple audio buffers into a single output (simple version)
    pub fn mix(&self, sources: &[(&AudioBuffer, f32)]) -> AudioBuffer {
        if sources.is_empty() {
            return AudioBuffer::new(self.sample_rate, self.channels, 0);
        }

        // Find the longest buffer
        let max_duration = sources
            .iter()
            .map(|(buf, _)| buf.duration_ms)
            .max()
            .unwrap_or(0);
        let max_samples = (self.sample_rate as f64 * self.channels as f64 * max_duration as f64
            / 1000.0) as usize;

        let mut output = vec![0.0f32; max_samples];

        for (buffer, volume) in sources {
            let samples_to_mix = buffer.samples.len().min(max_samples);
            for i in 0..samples_to_mix {
                output[i] += buffer.samples[i] * volume;
            }
        }

        // Soft clipping
        for sample in output.iter_mut() {
            *sample = sample.tanh();
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
        let fade_out_start = total_samples
            .saturating_sub((envelope.fade_out_ms as usize * samples_per_ms).min(total_samples));

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
        let window_size =
            (50.0 * main_buffer.sample_rate as f64 * channels as f64 / 1000.0) as usize;

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
        let peak = buffer
            .samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        if peak > 0.0 {
            let gain = target_peak / peak;
            for sample in buffer.samples.iter_mut() {
                *sample *= gain;
            }
        }
    }

    /// Resample audio to a different sample rate
    ///
    /// Uses linear interpolation for simplicity. For production use,
    /// a proper resampling library (rubato) should be used.
    pub fn resample(&self, buffer: &AudioBuffer, target_sample_rate: u32) -> AudioBuffer {
        if buffer.sample_rate == target_sample_rate {
            return buffer.clone();
        }

        let ratio = target_sample_rate as f64 / buffer.sample_rate as f64;
        let channels = buffer.channels as usize;
        let input_frames = buffer.samples.len() / channels;
        let output_frames = (input_frames as f64 * ratio) as usize;
        let output_duration_ms = (output_frames as f64 * 1000.0 / target_sample_rate as f64) as u64;

        let mut output = vec![0.0f32; output_frames * channels];

        for frame in 0..output_frames {
            let src_frame = frame as f64 / ratio;
            let src_frame_floor = src_frame.floor() as usize;
            let src_frame_ceil = (src_frame_floor + 1).min(input_frames - 1);
            let frac = src_frame - src_frame_floor as f64;

            for ch in 0..channels {
                let src_idx_floor = src_frame_floor * channels + ch;
                let src_idx_ceil = src_frame_ceil * channels + ch;
                let value = buffer.samples[src_idx_floor] * (1.0 - frac as f32)
                    + buffer.samples[src_idx_ceil] * frac as f32;
                output[frame * channels + ch] = value;
            }
        }

        AudioBuffer {
            samples: output,
            sample_rate: target_sample_rate,
            channels: buffer.channels,
            start_ms: buffer.start_ms,
            duration_ms: output_duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_empty_sources() {
        let mixer = AudioMixer::new(44100, 2);
        let result = mixer.mix(&[]);
        assert_eq!(result.samples.len(), 0);
    }

    #[test]
    fn test_mix_single_source() {
        let mixer = AudioMixer::new(44100, 2);
        let buffer = AudioBuffer {
            samples: vec![0.5; 100],
            sample_rate: 44100,
            channels: 2,
            start_ms: 0,
            duration_ms: 100,
        };

        let result = mixer.mix(&[(&buffer, 1.0)]);
        // tanh(0.5) ≈ 0.4621
        assert!((result.samples[0] - 0.4621).abs() < 0.01);
    }

    #[test]
    fn test_mix_two_sources() {
        let mixer = AudioMixer::new(44100, 2);
        let buf1 = AudioBuffer {
            samples: vec![0.5; 100],
            sample_rate: 44100,
            channels: 2,
            start_ms: 0,
            duration_ms: 100,
        };
        let buf2 = AudioBuffer {
            samples: vec![0.3; 100],
            sample_rate: 44100,
            channels: 2,
            start_ms: 0,
            duration_ms: 100,
        };

        let result = mixer.mix(&[(&buf1, 1.0), (&buf2, 1.0)]);
        // 0.5 + 0.3 = 0.8, tanh(0.8) ≈ 0.6640
        assert!((result.samples[0] - 0.664).abs() < 0.01);
    }

    #[test]
    fn test_mix_sources_with_offset() {
        let mixer = AudioMixer::new(44100, 2);
        let buf1 = AudioBuffer {
            samples: vec![1.0; 100],
            sample_rate: 44100,
            channels: 2,
            start_ms: 0,
            duration_ms: 100,
        };

        let sources = vec![TrackAudioSource {
            buffer: buf1,
            volume: 1.0,
            offset_ms: 0,
            envelope: VolumeEnvelope::default(),
            muted: false,
            pan: 0.0,
            solo: false,
            eq_settings: None,
        }];

        let result = mixer.mix_sources(&sources, 200);
        // Output should be 200ms long, first 100ms has signal, rest is silence
        assert!(result.samples.len() > 100);
        assert!(result.samples[0].abs() > 0.0);
    }

    #[test]
    fn test_volume_envelope_fade_in() {
        let mixer = AudioMixer::new(44100, 2);
        let mut buffer = AudioBuffer::new(44100, 2, 1000);
        buffer.samples.fill(1.0);

        let envelope = VolumeEnvelope {
            volume: 1.0,
            fade_in_ms: 500,
            fade_out_ms: 0,
        };

        mixer.apply_envelope(&mut buffer, &envelope);

        // First sample should be near 0 (fade in start)
        assert!(buffer.samples[0].abs() < 0.01);
        // Middle should be at full volume
        let mid = buffer.samples.len() / 2;
        assert!((buffer.samples[mid] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize() {
        let mixer = AudioMixer::new(44100, 2);
        let mut buffer = AudioBuffer {
            samples: vec![0.5, -0.5, 0.3, -0.3],
            sample_rate: 44100,
            channels: 2,
            start_ms: 100,
            duration_ms: 100,
        };

        mixer.normalize(&mut buffer, 1.0);
        // Peak should be 1.0
        let peak = buffer
            .samples
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        assert!((peak - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_resample_up() {
        let mixer = AudioMixer::new(44100, 2);
        let buffer = AudioBuffer {
            samples: vec![0.5; 44100], // 0.5s at 44100Hz mono
            sample_rate: 44100,
            channels: 1,
            start_ms: 0,
            duration_ms: 500,
        };

        let resampled = mixer.resample(&buffer, 48000);
        assert_eq!(resampled.sample_rate, 48000);
        // Should have more samples (upsampled)
        assert!(resampled.samples.len() > buffer.samples.len());
    }

    #[test]
    fn test_peak_amplitude() {
        let buffer = AudioBuffer {
            samples: vec![0.1, -0.5, 0.3, -0.8, 0.2],
            sample_rate: 44100,
            channels: 1,
            start_ms: 0,
            duration_ms: 100,
        };

        assert!((buffer.peak_amplitude() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_rms_energy() {
        let buffer = AudioBuffer {
            samples: vec![1.0, -1.0, 1.0, -1.0],
            sample_rate: 44100,
            channels: 1,
            start_ms: 0,
            duration_ms: 100,
        };

        assert!((buffer.rms_energy() - 1.0).abs() < 0.001);
    }
}
