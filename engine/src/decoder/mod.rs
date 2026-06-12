//! Video/Audio decoder module
//!
//! Handles decoding media files using FFmpeg with hardware acceleration
//! support (MediaCodec on Android) and software fallback.

pub mod hardware;
pub mod software;

/// Information about a video file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration_ms: u64,
    pub codec_name: String,
    pub bitrate: u64,
    pub has_audio: bool,
    pub audio_codec: Option<String>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u32>,
}

/// A decoded frame with RGBA pixel data
#[derive(Debug, Clone)]
pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA format, 4 bytes per pixel
    pub timestamp_ms: u64,
    pub is_keyframe: bool,
}

impl FrameData {
    /// Create a blank black frame
    pub fn blank(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0u8; size],
            timestamp_ms: 0,
            is_keyframe: true,
        }
    }

    /// Get the total number of pixels
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    /// Get the data size in bytes
    pub fn data_size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
}

/// Audio sample data
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>, // Interleaved stereo samples
    pub sample_rate: u32,
    pub channels: u32,
    pub timestamp_ms: u64,
    pub duration_ms: u64,
}
