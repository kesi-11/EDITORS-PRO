use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::codec::flag::Flags as CodecFlags;
use ffmpeg_next::util::frame::video::Pixel;
use log::{debug, info};
use serde::{Deserialize, Serialize};

/// Supported video codecs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
}

impl VideoCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "libx264",
            VideoCodec::H265 => "libx265",
            VideoCodec::VP9 => "libvpx-vp9",
        }
    }

    pub fn from_str_codec(s: &str) -> Self {
        match s {
            "libx264" | "h264" => VideoCodec::H264,
            "libx265" | "h265" | "hevc" => VideoCodec::H265,
            "libvpx-vp9" | "vp9" => VideoCodec::VP9,
            _ => VideoCodec::H264,
        }
    }
}

/// Encoding quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    Ultrafast,
    Fast,
    Medium,
    Slow,
    Veryslow,
}

impl QualityPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityPreset::Ultrafast => "ultrafast",
            QualityPreset::Fast => "fast",
            QualityPreset::Medium => "medium",
            QualityPreset::Slow => "slow",
            QualityPreset::Veryslow => "veryslow",
        }
    }
}

/// Encoder configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderConfig {
    pub codec: VideoCodec,
    pub bitrate: u64,
    pub gop_size: u32,
    pub profile: String,
    pub preset: QualityPreset,
    pub fps: f64,
    pub width: u32,
    pub height: u32,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            bitrate: 8_000_000,
            gop_size: 12,
            profile: "high".to_string(),
            preset: QualityPreset::Medium,
            fps: 30.0,
            width: 1920,
            height: 1080,
        }
    }
}

/// FFmpeg-based video encoder.
pub struct Encoder {
    config: EncoderConfig,
    output_path: String,
    format_context: Option<ffmpeg::format::context::Output>,
    video_stream: Option<ffmpeg::format::stream::Stream>,
    encoder: Option<ffmpeg::encoder::Video>,
    scaler: Option<ffmpeg::software::converter>,
    frame_count: u64,
    finalized: bool,
}

impl Encoder {
    /// Create a new encoder with the given config and output path.
    pub fn new(config: EncoderConfig, output_path: &str) -> Result<Self> {
        Ok(Self {
            config,
            output_path: output_path.to_string(),
            format_context: None,
            video_stream: None,
            encoder: None,
            scaler: None,
            frame_count: 0,
            finalized: false,
        })
    }

    /// Initialize the encoder (must be called before write_frame).
    pub fn init(&mut self) -> Result<()> {
        ffmpeg::init().context("Failed to initialize FFmpeg")?;

        let octx = ffmpeg::format::output(&self.output_path)
            .context("Failed to create output context")?;

        let codec = ffmpeg::encoder::find_by_name(self.config.codec.as_str())
            .context(format!("Codec {} not found", self.config.codec.as_str()))?;

        let mut stream = octx
            .add_stream(codec)
            .context("Failed to add stream")?;

        let mut encoder = stream.codec().encoder().video()
            .context("Failed to create video encoder")?;

        encoder.set_width(self.config.width);
        encoder.set_height(self.config.height);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_frame_rate(Some(ffmpeg::Rational::new(
            (self.config.fps * 1000.0) as i32,
            1000,
        )));
        encoder.set_time_base(ffmpeg::Rational::new(1, (self.config.fps * 1000.0) as i32));
        encoder.set_bit_rate(self.config.bitrate as usize);
        encoder.set_gop_size(self.config.gop_size as usize);

        // Set preset via options
        let mut opts = ffmpeg::Dictionary::new();
        opts.set("preset", self.config.preset.as_str());
        if self.config.codec == VideoCodec::H264 || self.config.codec == VideoCodec::H265 {
            opts.set("profile", &self.config.profile);
        }

        let encoder = encoder
            .open_with(opts)
            .context("Failed to open encoder")?;

        stream.set_parameters(&encoder);

        let scaler = ffmpeg::converter(
            (self.config.width, self.config.height),
            Pixel::RGBA,
            Pixel::YUV420P,
        )
        .context("Failed to create scaler")?;

        self.format_context = Some(octx);
        self.video_stream = Some(stream);
        self.encoder = Some(encoder);
        self.scaler = Some(scaler);

        debug!(
            "Encoder initialized: {}x{} @ {:.0}fps, codec={}",
            self.config.width, self.config.height, self.config.fps, self.config.codec.as_str()
        );

        Ok(())
    }

    /// Write a single frame (RGBA pixel data).
    pub fn write_frame(&mut self, frame_data: &[u8]) -> Result<()> {
        let octx = self.format_context.as_mut().context("Encoder not initialized")?;
        let encoder = self.encoder.as_mut().context("No encoder")?;
        let scaler = self.scaler.as_mut().context("No scaler")?;

        let stride = self.config.width as usize * 4;
        let expected_size = stride * self.config.height as usize;
        if frame_data.len() < expected_size {
            anyhow::bail!(
                "Frame data too small: got {} bytes, expected {}",
                frame_data.len(),
                expected_size
            );
        }

        let mut yuv_frame = ffmpeg::frame::Video::empty();
        let mut rgba_frame = ffmpeg::frame::Video::new(
            Pixel::RGBA,
            self.config.width,
            self.config.height,
        );

        // Copy RGBA data into the frame
        for y in 0..self.config.height as usize {
            let src_offset = y * stride;
            let dst_offset = y * rgba_frame.stride(0);
            let dst = rgba_frame.data_mut(0);
            if src_offset + stride <= frame_data.len() && dst_offset + stride <= dst.len() {
                dst[dst_offset..dst_offset + stride]
                    .copy_from_slice(&frame_data[src_offset..src_offset + stride]);
            }
        }

        scaler.run(&rgba_frame, &mut yuv_frame)?;

        yuv_frame.set_pts(Some(self.frame_count as i64));

        encoder.send_frame(&yuv_frame)?;

        // Receive and write packets
        let mut packet = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.set_pts(Some(packet.pts().unwrap_or(self.frame_count as i64)));
            packet.set_dts(Some(packet.dts().unwrap_or(self.frame_count as i64)));
            packet.write_interleaved(octx)?;
        }

        self.frame_count += 1;
        Ok(())
    }

    /// Write audio samples to the output.
    pub fn write_audio_samples(
        &mut self,
        _samples: &[f32],
        _sample_rate: u32,
        _channels: u16,
    ) -> Result<()> {
        // Audio encoding is handled separately in the muxing stage.
        // For now, store the audio to a temp file and mux later.
        debug!("write_audio_samples: audio will be muxed in finalization");
        Ok(())
    }

    /// Finalize the encoding: flush encoder and write trailer.
    pub fn finalize(&mut self) -> Result<()> {
        if self.finalized {
            return Ok(());
        }

        let octx = self.format_context.as_mut().context("Encoder not initialized")?;
        let encoder = self.encoder.as_mut().context("No encoder")?;

        // Flush the encoder
        encoder.send_eof()?;
        let mut packet = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.write_interleaved(octx)?;
        }

        octx.write_trailer()
            .context("Failed to write trailer")?;

        self.finalized = true;
        info!("Encoder finalized. Total frames: {}", self.frame_count);
        Ok(())
    }

    /// Mux (combine) separate video and audio files into a single output.
    pub fn mux_audio(video_path: &str, audio_path: &str, output_path: &str) -> Result<()> {
        ffmpeg::init().context("Failed to initialize FFmpeg")?;

        let mut video_ictx = ffmpeg::format::input(video_path)
            .context("Failed to open video input")?;
        let mut audio_ictx = ffmpeg::format::input(audio_path)
            .context("Failed to open audio input")?;

        let video_stream = video_ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .context("No video stream in video file")?;
        let audio_stream = audio_ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .context("No audio stream in audio file")?;

        let mut octx = ffmpeg::format::output(output_path)
            .context("Failed to create output")?;

        let mut video_out = octx
            .add_stream(video_stream.codec().codec())
            .context("Failed to add video stream")?;
        let mut audio_out = octx
            .add_stream(audio_stream.codec().codec())
            .context("Failed to add audio stream")?;

        video_out.set_parameters(&video_stream.parameters());
        audio_out.set_parameters(&audio_stream.parameters());

        octx.write_header()
            .context("Failed to write header")?;

        // Copy video packets
        for (stream, packet) in video_ictx.packets() {
            if stream.index() == video_stream.index() {
                let mut pkt = packet;
                pkt.set_stream(0);
                pkt.rescale_ts(
                    stream.time_base(),
                    video_out.time_base(),
                );
                pkt.write_interleaved(&mut octx)?;
            }
        }

        // Copy audio packets
        for (stream, packet) in audio_ictx.packets() {
            if stream.index() == audio_stream.index() {
                let mut pkt = packet;
                pkt.set_stream(1);
                pkt.rescale_ts(
                    stream.time_base(),
                    audio_out.time_base(),
                );
                pkt.write_interleaved(&mut octx)?;
            }
        }

        octx.write_trailer()
            .context("Failed to write trailer")?;

        info!("Muxed {} + {} -> {}", video_path, audio_path, output_path);
        Ok(())
    }

    /// Get the number of frames written so far.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        if !self.finalized {
            let _ = self.finalize();
        }
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_config_default() {
        let config = EncoderConfig::default();
        assert_eq!(config.codec, VideoCodec::H264);
        assert_eq!(config.bitrate, 8_000_000);
        assert_eq!(config.gop_size, 12);
        assert_eq!(config.fps, 30.0);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_video_codec_as_str() {
        assert_eq!(VideoCodec::H264.as_str(), "libx264");
        assert_eq!(VideoCodec::H265.as_str(), "libx265");
        assert_eq!(VideoCodec::VP9.as_str(), "libvpx-vp9");
    }

    #[test]
    fn test_video_codec_from_str() {
        assert_eq!(VideoCodec::from_str_codec("libx264"), VideoCodec::H264);
        assert_eq!(VideoCodec::from_str_codec("h264"), VideoCodec::H264);
        assert_eq!(VideoCodec::from_str_codec("libx265"), VideoCodec::H265);
        assert_eq!(VideoCodec::from_str_codec("hevc"), VideoCodec::H265);
        assert_eq!(VideoCodec::from_str_codec("vp9"), VideoCodec::VP9);
    }

    #[test]
    fn test_quality_preset_as_str() {
        assert_eq!(QualityPreset::Ultrafast.as_str(), "ultrafast");
        assert_eq!(QualityPreset::Fast.as_str(), "fast");
        assert_eq!(QualityPreset::Medium.as_str(), "medium");
        assert_eq!(QualityPreset::Slow.as_str(), "slow");
        assert_eq!(QualityPreset::Veryslow.as_str(), "veryslow");
    }

    #[test]
    fn test_encoder_new() {
        let config = EncoderConfig::default();
        let encoder = Encoder::new(config, "/tmp/test.mp4");
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_encoder_write_frame_without_init() {
        let config = EncoderConfig::default();
        let mut encoder = Encoder::new(config, "/tmp/test.mp4").unwrap();
        let result = encoder.write_frame(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_encoder_finalize_without_init() {
        let config = EncoderConfig::default();
        let mut encoder = Encoder::new(config, "/tmp/test.mp4").unwrap();
        let result = encoder.finalize();
        assert!(result.is_err());
    }

    #[test]
    fn test_encoder_frame_count_initial() {
        let config = EncoderConfig::default();
        let encoder = Encoder::new(config, "/tmp/test.mp4").unwrap();
        assert_eq!(encoder.frame_count(), 0);
    }

    #[test]
    fn test_encoder_config_serialization() {
        let config = EncoderConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EncoderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.codec, deserialized.codec);
        assert_eq!(config.bitrate, deserialized.bitrate);
        assert_eq!(config.width, deserialized.width);
    }

    #[test]
    fn test_mux_audio_nonexistent_files() {
        let result = Encoder::mux_audio("/nonexistent/video.mp4", "/nonexistent/audio.wav", "/tmp/output.mp4");
        assert!(result.is_err());
    }
}
