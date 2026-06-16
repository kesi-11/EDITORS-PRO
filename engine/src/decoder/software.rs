//! Software video decoder fallback
//!
//! Pure FFmpeg-based software decoder for environments where
//! hardware acceleration is not available or not needed.

#[cfg(feature = "ffmpeg")]
use ffmpeg_next as ffmpeg;

use super::{FrameData, VideoInfo};

/// Software-only video decoder using FFmpeg
#[cfg(feature = "ffmpeg")]
pub struct SoftwareDecoder {
    format_context: Option<ffmpeg::format::context::Input>,
    video_stream_index: Option<usize>,
    decoder: Option<ffmpeg::decoder::Video>,
    video_info: Option<VideoInfo>,
    is_open: bool,
    current_position_ms: u64,
}

#[cfg(feature = "ffmpeg")]
impl SoftwareDecoder {
    pub fn new() -> Self {
        Self {
            format_context: None,
            video_stream_index: None,
            decoder: None,
            video_info: None,
            is_open: false,
            current_position_ms: 0,
        }
    }

    /// Open a media file for software decoding
    pub fn open(&mut self, file_path: &str) -> Result<(), String> {
        log::info!("Software decoder opening: {}", file_path);

        let format_context = ffmpeg::format::input(&file_path)
            .map_err(|e| format!("Failed to open '{}': {}", file_path, e))?;

        let video_stream = format_context.streams().best(ffmpeg::media::Type::Video)
            .ok_or("No video stream found")?;
        let video_idx = video_stream.index();

        let context = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
            .map_err(|e| format!("Decoder context error: {}", e))?;

        let decoder = context.decoder().video()
            .map_err(|e| format!("Video decoder error: {}", e))?;

        let frame_rate = video_stream.avg_frame_rate();
        let fps = frame_rate.numerator() as f32 / frame_rate.denominator().max(1) as f32;
        let duration_ms = format_context.duration() as u64 * 1000 / ffmpeg::sys::AV_TIME_BASE as u64;

        let audio_stream = format_context.streams()
            .find(|s| s.parameters().medium() == ffmpeg::media::Type::Audio);

        let video_info = VideoInfo {
            width: decoder.width(),
            height: decoder.height(),
            fps,
            duration_ms: if duration_ms > 0 { duration_ms } else {
                let dur = video_stream.duration() as f64
                    * video_stream.time_base().numerator() as f64
                    / video_stream.time_base().denominator().max(1) as f64;
                (dur * 1000.0) as u64
            },
            codec_name: decoder.codec().map(|c| c.name().to_string()).unwrap_or_default(),
            bitrate: format_context.bit_rate() as u64,
            has_audio: audio_stream.is_some(),
            audio_codec: audio_stream.and_then(|s| {
                let ctx = ffmpeg::codec::context::Context::from_parameters(s.parameters()).ok()?;
                Some(ctx.decoder().audio().ok()?.codec()?.name().to_string())
            }),
            audio_sample_rate: audio_stream.and_then(|s| {
                let ctx = ffmpeg::codec::context::Context::from_parameters(s.parameters()).ok()?;
                Some(ctx.decoder().audio().ok()?.rate())
            }),
            audio_channels: audio_stream.and_then(|s| {
                let ctx = ffmpeg::codec::context::Context::from_parameters(s.parameters()).ok()?;
                Some(ctx.decoder().audio().ok()?.channels() as u32)
            }),
        };

        log::info!("Software decoder: {}x{} @ {}fps, {}ms, codec: {}",
            video_info.width, video_info.height, video_info.fps,
            video_info.duration_ms, video_info.codec_name);

        self.format_context = Some(format_context);
        self.video_stream_index = Some(video_idx);
        self.decoder = Some(decoder);
        self.video_info = Some(video_info);
        self.is_open = true;

        Ok(())
    }

    /// Seek to a specific timestamp in milliseconds
    pub fn seek_to(&mut self, time_ms: u64) -> Result<(), String> {
        let format_context = self.format_context.as_mut().ok_or("Not open")?;
        let video_idx = self.video_stream_index.ok_or("No video stream")?;
        let stream = format_context.streams().get(video_idx).ok_or("Stream not found")?;
        let tb = stream.time_base();

        let target = (time_ms as i64 * tb.denominator() as i64) / (1000 * tb.numerator().max(1) as i64);
        format_context.seek(target, ..target + 500)
            .map_err(|e| format!("Seek failed: {}", e))?;

        self.current_position_ms = time_ms;
        // Flush the decoder after seeking
        if let Some(decoder) = &mut self.decoder {
            decoder.flush();
        }

        Ok(())
    }

    /// Decode the next available frame
    pub fn decode_next_frame(&mut self) -> Result<Option<FrameData>, String> {
        if !self.is_open {
            return Err("Decoder not open".to_string());
        }

        let format_context = self.format_context.as_mut().ok_or("Not open")?;
        let video_idx = self.video_stream_index.ok_or("No video stream")?;
        let decoder = self.decoder.as_mut().ok_or("No decoder")?;
        let stream = format_context.streams().get(video_idx).ok_or("Stream not found")?;
        let tb = stream.time_base();

        let mut scaler = ffmpeg::software::scaling::context::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg::format::Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            ffmpeg::software::scaling::Flags::BILINEAR,
        ).map_err(|e| format!("Scaler error: {}", e))?;

        let mut decoded = ffmpeg::util::frame::Video::empty();
        let mut scaled = ffmpeg::util::frame::Video::empty();

        for (s, packet) in format_context.packets() {
            if s.index() != video_idx {
                continue;
            }

            decoder.send_packet(&packet)
                .map_err(|e| format!("Send packet error: {}", e))?;

            while decoder.receive_frame(&mut decoded).is_ok() {
                let pts_ms = (decoded.pts().unwrap_or(0) as i64 * tb.numerator() as i64 * 1000)
                    / tb.denominator().max(1) as i64;

                scaler.run(&decoded, &mut scaled)
                    .map_err(|e| format!("Scale error: {}", e))?;

                self.current_position_ms = pts_ms as u64;

                return Ok(Some(FrameData {
                    width: scaled.width(),
                    height: scaled.height(),
                    data: scaled.data(0).to_vec(),
                    timestamp_ms: pts_ms as u64,
                    is_keyframe: decoded.is_key_frame(),
                }));
            }
        }

        Ok(None)
    }

    /// Decode a frame at a specific timestamp
    pub fn decode_frame_at(&mut self, time_ms: u64) -> Result<FrameData, String> {
        self.seek_to(time_ms)?;

        // Decode frames until we reach the target
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 30;

        while attempts < MAX_ATTEMPTS {
            match self.decode_next_frame()? {
                Some(frame) => {
                    // Accept frame if it's close enough to the target time
                    if frame.timestamp_ms >= time_ms.saturating_sub(50) {
                        return Ok(frame);
                    }
                }
                None => break,
            }
            attempts += 1;
        }

        Err(format!("No frame found at {}ms after {} attempts", time_ms, attempts))
    }

    /// Get the current position in milliseconds
    pub fn current_position(&self) -> u64 {
        self.current_position_ms
    }

    /// Get video info
    pub fn get_video_info(&self) -> Option<&VideoInfo> {
        self.video_info.as_ref()
    }

    /// Close the decoder
    pub fn close(&mut self) {
        self.format_context = None;
        self.video_stream_index = None;
        self.decoder = None;
        self.video_info = None;
        self.is_open = false;
        log::info!("Software decoder closed");
    }
}

#[cfg(feature = "ffmpeg")]
impl Drop for SoftwareDecoder {
    fn drop(&mut self) {
        if self.is_open {
            self.close();
        }
    }
}

// SAFETY: Same rationale as HardwareDecoder - FFmpeg contexts are not
// thread-safe. Send allows moving between threads, but Sync is intentionally
// omitted to prevent shared concurrent access.
#[cfg(feature = "ffmpeg")]
unsafe impl Send for SoftwareDecoder {}

// ─── Stub implementation when FFmpeg is not available ──────────────────────────

#[cfg(not(feature = "ffmpeg"))]
pub struct SoftwareDecoder;

#[cfg(not(feature = "ffmpeg"))]
impl SoftwareDecoder {
    pub fn new() -> Self {
        Self
    }

    pub fn open(&mut self, _file_path: &str) -> Result<(), String> {
        Err("FFmpeg is not available. Enable the 'ffmpeg' feature.".to_string())
    }

    pub fn seek_to(&mut self, _time_ms: u64) -> Result<(), String> {
        Err("FFmpeg is not available. Enable the 'ffmpeg' feature.".to_string())
    }

    pub fn decode_next_frame(&mut self) -> Result<Option<FrameData>, String> {
        Err("FFmpeg is not available. Enable the 'ffmpeg' feature.".to_string())
    }

    pub fn decode_frame_at(&mut self, _time_ms: u64) -> Result<FrameData, String> {
        Err("FFmpeg is not available. Enable the 'ffmpeg' feature.".to_string())
    }

    pub fn current_position(&self) -> u64 {
        0
    }

    pub fn get_video_info(&self) -> Option<&VideoInfo> {
        None
    }

    pub fn close(&mut self) {}
}
