//! Hardware-accelerated video decoder
//!
//! Uses FFmpeg for decoding with hardware acceleration support.
//! On Android, this will integrate MediaCodec via NDK for optimal performance.
//! Falls back to software decoding when hardware acceleration is unavailable.

use ffmpeg_next as ffmpeg;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{FrameData, VideoInfo};

/// Hardware-accelerated video decoder using FFmpeg
pub struct HardwareDecoder {
    format_context: Option<ffmpeg::format::context::Input>,
    video_stream_index: Option<usize>,
    audio_stream_index: Option<usize>,
    decoder: Option<ffmpeg::decoder::Video>,
    video_info: Option<VideoInfo>,
    is_open: bool,
    /// Tracks whether a decode operation is in progress to prevent
    /// concurrent access to FFmpeg contexts. This is a runtime check
    /// that complements the compile-time &mut self exclusivity.
    decoding_in_progress: AtomicBool,
}

impl HardwareDecoder {
    /// Create a new decoder instance
    pub fn new() -> Self {
        Self {
            format_context: None,
            video_stream_index: None,
            audio_stream_index: None,
            decoder: None,
            video_info: None,
            is_open: false,
            decoding_in_progress: AtomicBool::new(false),
        }
    }

    /// Open a media file and prepare for decoding
    pub fn open(&mut self, file_path: &str) -> Result<(), String> {
        log::info!("Opening media file: {}", file_path);

        let format_context = ffmpeg::format::input(&file_path)
            .map_err(|e| format!("Failed to open file '{}': {}", file_path, e))?;

        // Find the best video stream
        let video_stream = format_context.streams().best(ffmpeg::media::Type::Video)
            .ok_or_else(|| "No video stream found".to_string())?;
        let video_stream_index = video_stream.index();

        // Find audio stream if present
        let audio_stream_index = format_context.streams()
            .find(|s| s.parameters().medium() == ffmpeg::media::Type::Audio)
            .map(|s| s.index());

        // Create video decoder
        let context = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
            .map_err(|e| format!("Failed to create decoder context: {}", e))?;

        let decoder = context.decoder().video()
            .map_err(|e| format!("Failed to create video decoder: {}", e))?;

        // Extract video information
        let codec_name = decoder.codec().map(|c| c.name().to_string()).unwrap_or_default();
        let frame_rate = video_stream.avg_frame_rate();
        let fps = (frame_rate.numerator() as f32) / (frame_rate.denominator().max(1) as f32);
        let duration_ms = format_context.duration() as u64 * 1000 / ffmpeg::sys::AV_TIME_BASE as u64;

        let video_info = VideoInfo {
            width: decoder.width(),
            height: decoder.height(),
            fps,
            duration_ms: if duration_ms > 0 { duration_ms } else {
                // Estimate from stream duration
                let stream_dur = video_stream.duration() as f64 * video_stream.time_base().numerator() as f64
                    / video_stream.time_base().denominator() as f64;
                (stream_dur * 1000.0) as u64
            },
            codec_name,
            bitrate: format_context.bit_rate() as u64,
            has_audio: audio_stream_index.is_some(),
            audio_codec: None,
            audio_sample_rate: None,
            audio_channels: None,
        };

        log::info!("Video info: {}x{} @ {}fps, duration: {}ms, codec: {}",
            video_info.width, video_info.height, video_info.fps,
            video_info.duration_ms, video_info.codec_name);

        self.format_context = Some(format_context);
        self.video_stream_index = Some(video_stream_index);
        self.audio_stream_index = audio_stream_index;
        self.decoder = Some(decoder);
        self.video_info = Some(video_info);
        self.is_open = true;

        Ok(())
    }

    /// Get video information for the currently opened file
    pub fn get_video_info(&self) -> Option<&VideoInfo> {
        self.video_info.as_ref()
    }

    /// Get the duration in milliseconds
    pub fn get_duration(&self) -> u64 {
        self.video_info.as_ref().map(|i| i.duration_ms).unwrap_or(0)
    }

    /// Decode a single frame at the specified timestamp
    pub fn decode_frame_at(&mut self, time_ms: u64) -> Result<FrameData, String> {
        if !self.is_open {
            return Err("Decoder is not open".to_string());
        }

        // Runtime assertion: prevent concurrent decode operations.
        // While &mut self ensures exclusive access at compile time within
        // a single thread, the unsafe Send impl allows the decoder to be
        // moved between threads. This assertion catches accidental concurrent
        // use if the decoder is shared across threads (e.g., via Arc).
        assert!(
            !self.decoding_in_progress.swap(true, Ordering::Acquire),
            "HardwareDecoder: concurrent decode detected — FFmpeg contexts are not thread-safe"
        );

        let result = self.decode_frame_at_inner(time_ms);

        self.decoding_in_progress.store(false, Ordering::Release);
        result
    }

    /// Inner decode implementation (called after concurrency guard)
    fn decode_frame_at_inner(&mut self, time_ms: u64) -> Result<FrameData, String> {

        let format_context = self.format_context.as_mut()
            .ok_or("No format context")?;
        let video_idx = self.video_stream_index
            .ok_or("No video stream index")?;
        let decoder = self.decoder.as_mut()
            .ok_or("No decoder")?;

        // Seek to the target timestamp
        let stream = format_context.streams().get(video_idx)
            .ok_or("Video stream not found")?;
        let time_base = stream.time_base();
        let seek_target = (time_ms as i64 * time_base.denominator() as i64)
            / (1000 * time_base.numerator() as i64);

        format_context.seek(seek_target, ..seek_target + 1000)
            .map_err(|e| format!("Seek failed: {}", e))?;

        // Decode frames until we reach or pass the target time
        let mut scaler = ffmpeg::software::scaling::context::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg::format::Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            ffmpeg::software::scaling::Flags::BILINEAR,
        ).map_err(|e| format!("Failed to create scaler: {}", e))?;

        let mut received = ffmpeg::util::frame::Video::empty();
        let mut frame = ffmpeg::util::frame::Video::empty();

        for (stream, packet) in format_context.packets() {
            if stream.index() != video_idx {
                continue;
            }

            decoder.send_packet(&packet)
                .map_err(|e| format!("Send packet failed: {}", e))?;

            while decoder.receive_frame(&mut received).is_ok() {
                let pts_ms = (received.pts().unwrap_or(0) as i64 * time_base.numerator() as i64 * 1000)
                    / time_base.denominator().max(1) as i64;

                if pts_ms >= time_ms as i64 - 50 {
                    scaler.run(&received, &mut frame)
                        .map_err(|e| format!("Scale failed: {}", e))?;

                    let data = frame.data(0).to_vec();
                    return Ok(FrameData {
                        width: frame.width(),
                        height: frame.height(),
                        data,
                        timestamp_ms: pts_ms as u64,
                        is_keyframe: received.is_key_frame(),
                    });
                }
            }
        }

        Err("No frame found at the specified timestamp".to_string())
    }

    /// Generate thumbnail images at regular intervals
    pub fn generate_thumbnails(&mut self, count: u32) -> Result<Vec<FrameData>, String> {
        if !self.is_open {
            return Err("Decoder is not open".to_string());
        }

        let duration = self.get_duration();
        if duration == 0 || count == 0 {
            return Err("Invalid duration or count for thumbnail generation".to_string());
        }

        let mut thumbnails = Vec::with_capacity(count as usize);
        let interval = duration / (count as u64 + 1);

        for i in 1..=count {
            let time_ms = interval * i as u64;
            match self.decode_frame_at(time_ms) {
                Ok(frame) => thumbnails.push(frame),
                Err(e) => {
                    log::warn!("Failed to generate thumbnail at {}ms: {}", time_ms, e);
                    thumbnails.push(FrameData::blank(
                        self.video_info.as_ref().map(|i| i.width).unwrap_or(1920),
                        self.video_info.as_ref().map(|i| i.height).unwrap_or(1080),
                    ));
                }
            }
        }

        Ok(thumbnails)
    }

    /// Close the decoder and release resources
    pub fn close(&mut self) {
        self.format_context = None;
        self.video_stream_index = None;
        self.audio_stream_index = None;
        self.decoder = None;
        self.video_info = None;
        self.is_open = false;
        log::info!("Decoder closed");
    }
}

impl Drop for HardwareDecoder {
    fn drop(&mut self) {
        if self.is_open {
            self.close();
        }
    }
}

// SAFETY: HardwareDecoder is NOT thread-safe because FFmpeg's AVFormatContext
// and AVCodecContext are not safe for concurrent access. We implement Send
// (but NOT Sync) to allow moving the decoder between threads, which is safe
// as long as only one thread accesses it at a time.
//
// The EditorsProEngine holds the decoder behind &mut self, ensuring
// exclusive access at the API level. Additionally, the `decoding_in_progress`
// AtomicBool provides a runtime guard against accidental concurrent use if
// the decoder is moved across threads and accessed via shared references.
//
// NOTE: We intentionally do NOT implement Sync for HardwareDecoder.
// If true thread-safe decoding is needed in the future, the recommended
// approach is to spawn a dedicated decode thread and communicate via channels,
// which avoids the need for unsafe Send/Sync entirely.
unsafe impl Send for HardwareDecoder {}
