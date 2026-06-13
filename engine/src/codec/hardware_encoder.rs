use anyhow::{Context, Result};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use super::encoder::{EncoderConfig, VideoCodec};

/// Android MediaCodec hardware encoder wrapper with automatic software fallback.
pub struct HardwareEncoder {
    config: EncoderConfig,
    hw_available: bool,
    // In a real Android build, these would hold NDK MediaCodec references.
    // For cross-platform builds, we use software fallback.
    software_fallback: Option<SoftwareFallbackEncoder>,
}

/// Simple software fallback that mimics the encoder interface.
struct SoftwareFallbackEncoder {
    config: EncoderConfig,
    frame_count: u64,
    output_buffer: Vec<Vec<u8>>,
}

impl SoftwareFallbackEncoder {
    fn new(config: EncoderConfig) -> Self {
        Self {
            config,
            frame_count: 0,
            output_buffer: Vec::new(),
        }
    }

    fn encode_frame(&mut self, frame_data: &[u8]) -> Result<Vec<u8>> {
        let stride = self.config.width as usize * 4;
        let expected_size = stride * self.config.height as usize;
        if frame_data.len() < expected_size {
            anyhow::bail!(
                "Frame data too small: got {} bytes, expected {}",
                frame_data.len(),
                expected_size
            );
        }

        // In a real implementation, this would use FFmpeg encoder.
        // For the fallback path, we just compress the frame data using a simple scheme.
        let mut compressed = Vec::with_capacity(frame_data.len() / 2);
        let chunk_size = 16;
        for chunk in frame_data.chunks(chunk_size) {
            // Simple RLE-like compression: store first byte and count
            let first = chunk[0];
            let all_same = chunk.iter().all(|&b| b == first);
            if all_same {
                compressed.push(0xFF); // RLE marker
                compressed.push(first);
                compressed.push(chunk.len() as u8);
            } else {
                compressed.push(chunk.len() as u8);
                compressed.extend_from_slice(chunk);
            }
        }

        self.frame_count += 1;
        Ok(compressed)
    }

    fn encode_with_surface(&mut self, _texture_id: u64) -> Result<Vec<u8>> {
        // In the software fallback, we cannot encode from a surface.
        anyhow::bail!("Surface encoding is not available in software fallback mode")
    }
}

impl HardwareEncoder {
    /// Check whether hardware encoding (MediaCodec NDK) is available.
    /// On non-Android platforms, this always returns false.
    pub fn is_available() -> bool {
        // MediaCodec NDK is only available on Android.
        // Check for the `android` cfg or try to load the library.
        #[cfg(target_os = "android")]
        {
            // On Android, try to detect NDK MediaCodec availability.
            // For now, return true if we're on Android.
            true
        }
        #[cfg(not(target_os = "android"))]
        {
            debug!("HardwareEncoder: not on Android, hardware encoding unavailable");
            false
        }
    }

    /// Create a new hardware encoder. Falls back to software if hardware is unavailable.
    pub fn new(config: EncoderConfig) -> Result<Self> {
        let hw_available = Self::is_available();
        if hw_available {
            debug!("HardwareEncoder: MediaCodec hardware encoding available");
        } else {
            warn!("HardwareEncoder: falling back to software encoding");
        }

        let software_fallback = if !hw_available {
            Some(SoftwareFallbackEncoder::new(config.clone()))
        } else {
            None
        };

        Ok(Self {
            config,
            hw_available,
            software_fallback,
        })
    }

    /// Encode a single frame and return the compressed data.
    pub fn encode_frame(&mut self, frame_data: &[u8]) -> Result<Vec<u8>> {
        if self.hw_available {
            // On Android with MediaCodec, we would call AMediaCodec APIs here.
            // Since we cannot compile NDK code in a non-Android environment,
            // we simulate the hardware encoding path.
            #[cfg(target_os = "android")]
            {
                // Placeholder for actual MediaCodec encoding:
                // let mut codec = AMediaCodec_createEncoderByType("video/avc");
                // ... configure and start the codec
                // ... feed input buffers and dequeue output buffers
                debug!("HardwareEncoder: encoding frame via MediaCodec");
                Ok(frame_data.to_vec()) // Simplified; real impl would return compressed NAL units
            }
            #[cfg(not(target_os = "android"))]
            {
                // This branch shouldn't be reached if hw_available is false on non-Android,
                // but just in case:
                self.software_fallback
                    .as_mut()
                    .context("Software fallback not initialized")?
                    .encode_frame(frame_data)
            }
        } else {
            self.software_fallback
                .as_mut()
                .context("Software fallback not initialized")?
                .encode_frame(frame_data)
        }
    }

    /// Encode from a GPU surface/texture (Android-only feature).
    pub fn encode_with_surface(&mut self, texture_id: u64) -> Result<Vec<u8>> {
        if !self.hw_available {
            return self
                .software_fallback
                .as_mut()
                .context("Software fallback not initialized")?
                .encode_with_surface(texture_id);
        }

        #[cfg(target_os = "android")]
        {
            debug!("HardwareEncoder: encoding from surface texture {}", texture_id);
            // Real implementation would:
            // 1. Create an input surface via AMediaCodec_createInputSurface
            // 2. Render the texture to the surface
            // 3. Dequeue the encoded output buffer
            Ok(Vec::new()) // Placeholder
        }
        #[cfg(not(target_os = "android"))]
        {
            anyhow::bail!("Surface encoding requires Android MediaCodec")
        }
    }

    /// Check if this encoder is using hardware acceleration.
    pub fn is_hardware(&self) -> bool {
        self.hw_available
    }

    /// Get the encoder configuration.
    pub fn config(&self) -> &EncoderConfig {
        &self.config
    }

    /// Get the number of frames encoded so far.
    pub fn frame_count(&self) -> u64 {
        if let Some(ref fallback) = self.software_fallback {
            fallback.frame_count
        } else {
            0
        }
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> EncoderConfig {
        EncoderConfig {
            codec: VideoCodec::H264,
            bitrate: 4_000_000,
            gop_size: 12,
            profile: "high".to_string(),
            preset: super::super::encoder::QualityPreset::Medium,
            fps: 30.0,
            width: 640,
            height: 480,
        }
    }

    #[test]
    fn test_hardware_encoder_is_available() {
        // On non-Android, this should be false
        let available = HardwareEncoder::is_available();
        #[cfg(not(target_os = "android"))]
        assert!(!available);
    }

    #[test]
    fn test_hardware_encoder_new() {
        let config = test_config();
        let encoder = HardwareEncoder::new(config);
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_hardware_encoder_is_hardware() {
        let config = test_config();
        let encoder = HardwareEncoder::new(config).unwrap();
        // On non-Android, should not be hardware
        #[cfg(not(target_os = "android"))]
        assert!(!encoder.is_hardware());
    }

    #[test]
    fn test_hardware_encoder_encode_frame_software() {
        let config = test_config();
        let mut encoder = HardwareEncoder::new(config).unwrap();
        let frame_data = vec![0u8; 640 * 480 * 4]; // RGBA frame
        let result = encoder.encode_frame(&frame_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hardware_encoder_encode_frame_too_small() {
        let config = test_config();
        let mut encoder = HardwareEncoder::new(config).unwrap();
        let frame_data = vec![0u8; 100]; // Too small
        let result = encoder.encode_frame(&frame_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_hardware_encoder_encode_with_surface_fallback() {
        let config = test_config();
        let mut encoder = HardwareEncoder::new(config).unwrap();
        let result = encoder.encode_with_surface(0);
        // Should fail in software fallback mode
        assert!(result.is_err());
    }

    #[test]
    fn test_hardware_encoder_frame_count() {
        let config = test_config();
        let mut encoder = HardwareEncoder::new(config).unwrap();
        assert_eq!(encoder.frame_count(), 0);
        let frame_data = vec![128u8; 640 * 480 * 4];
        let _ = encoder.encode_frame(&frame_data);
        assert_eq!(encoder.frame_count(), 1);
    }

    #[test]
    fn test_hardware_encoder_config() {
        let config = test_config();
        let encoder = HardwareEncoder::new(config.clone()).unwrap();
        assert_eq!(encoder.config().width, 640);
        assert_eq!(encoder.config().height, 480);
    }
}
