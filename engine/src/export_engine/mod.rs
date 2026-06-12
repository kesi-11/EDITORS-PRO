//! Export engine - Video export pipeline
//!
//! Handles rendering the timeline and encoding the final video output
//! with configurable resolution, bitrate, codec, and format settings.

use serde::{Deserialize, Serialize};

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub bitrate_kbps: u64,
    pub codec: VideoCodec,
    pub format: OutputFormat,
    pub audio_bitrate_kbps: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: u32,
    pub include_audio: bool,
    pub two_pass: bool,
}

impl ExportSettings {
    /// 720p export preset
    pub fn hd_720p() -> Self {
        Self {
            width: 1280,
            height: 720,
            fps: 30.0,
            bitrate_kbps: 5000,
            codec: VideoCodec::H264,
            format: OutputFormat::Mp4,
            audio_bitrate_kbps: 128,
            audio_sample_rate: 44100,
            audio_channels: 2,
            include_audio: true,
            two_pass: false,
        }
    }

    /// 1080p export preset
    pub fn full_hd_1080p() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            bitrate_kbps: 10000,
            codec: VideoCodec::H264,
            format: OutputFormat::Mp4,
            audio_bitrate_kbps: 192,
            audio_sample_rate: 44100,
            audio_channels: 2,
            include_audio: true,
            two_pass: false,
        }
    }

    /// 4K export preset
    pub fn ultra_hd_4k() -> Self {
        Self {
            width: 3840,
            height: 2160,
            fps: 30.0,
            bitrate_kbps: 40000,
            codec: VideoCodec::H265,
            format: OutputFormat::Mp4,
            audio_bitrate_kbps: 256,
            audio_sample_rate: 48000,
            audio_channels: 2,
            include_audio: true,
            two_pass: true,
        }
    }

    /// TikTok/Reels vertical format
    pub fn social_vertical() -> Self {
        Self {
            width: 1080,
            height: 1920,
            fps: 30.0,
            bitrate_kbps: 8000,
            codec: VideoCodec::H264,
            format: OutputFormat::Mp4,
            audio_bitrate_kbps: 128,
            audio_sample_rate: 44100,
            audio_channels: 2,
            include_audio: true,
            two_pass: false,
        }
    }

    /// Instagram square format
    pub fn social_square() -> Self {
        Self {
            width: 1080,
            height: 1080,
            fps: 30.0,
            bitrate_kbps: 6000,
            codec: VideoCodec::H264,
            format: OutputFormat::Mp4,
            audio_bitrate_kbps: 128,
            audio_sample_rate: 44100,
            audio_channels: 2,
            include_audio: true,
            two_pass: false,
        }
    }
}

/// Video codec options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    H265,
    Vp9,
    Av1,
}

impl VideoCodec {
    pub fn ffmpeg_codec_name(&self) -> &str {
        match self {
            VideoCodec::H264 => "libx264",
            VideoCodec::H265 => "libx265",
            VideoCodec::Vp9 => "libvpx-vp9",
            VideoCodec::Av1 => "libaom-av1",
    }
    }

    pub fn display_name(&self) -> &str {
        match self {
            VideoCodec::H264 => "H.264 (AVC)",
            VideoCodec::H265 => "H.265 (HEVC)",
            VideoCodec::Vp9 => "VP9",
            VideoCodec::Av1 => "AV1",
        }
    }
}

/// Output container format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Mp4,
    WebM,
    Mov,
    Avi,
    Gif,
}

impl OutputFormat {
    pub fn ffmpeg_format_name(&self) -> &str {
        match self {
            OutputFormat::Mp4 => "mp4",
            OutputFormat::WebM => "webm",
            OutputFormat::Mov => "mov",
            OutputFormat::Avi => "avi",
            OutputFormat::Gif => "gif",
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            OutputFormat::Mp4 => ".mp4",
            OutputFormat::WebM => ".webm",
            OutputFormat::Mov => ".mov",
            OutputFormat::Avi => ".avi",
            OutputFormat::Gif => ".gif",
        }
    }
}

/// Progress report during export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgress {
    /// Percentage complete (0.0 to 1.0)
    pub progress: f32,
    /// Current frame being processed
    pub current_frame: u64,
    /// Total frames to process
    pub total_frames: u64,
    /// Estimated time remaining in seconds
    pub estimated_seconds_remaining: u64,
    /// Current processing stage
    pub stage: ExportStage,
}

/// Stages of the export process
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExportStage {
    Preparing,
    Rendering,
    Encoding,
    Finalizing,
    Complete,
    Error,
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub output_path: String,
    pub file_size_bytes: u64,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

/// The export pipeline orchestrates the rendering and encoding process
pub struct ExportPipeline {
    settings: ExportSettings,
}

impl ExportPipeline {
    pub fn new(settings: ExportSettings) -> Self {
        Self { settings }
    }

    /// Get the export settings
    pub fn settings(&self) -> &ExportSettings {
        &self.settings
    }

    /// Calculate the total number of frames for the given duration
    pub fn total_frames(&self, duration_ms: u64) -> u64 {
        (duration_ms as f64 * self.settings.fps as f64 / 1000.0).ceil() as u64
    }

    /// Calculate the frame number for a given timestamp
    pub fn frame_at_time(&self, time_ms: u64) -> u64 {
        (time_ms as f64 * self.settings.fps as f64 / 1000.0) as u64
    }

    /// Calculate the timestamp for a given frame number
    pub fn time_at_frame(&self, frame: u64) -> u64 {
        (frame as f64 * 1000.0 / self.settings.fps as f64) as u64
    }
}
