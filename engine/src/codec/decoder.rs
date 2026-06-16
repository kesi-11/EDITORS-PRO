#[cfg(feature = "ffmpeg")]
use anyhow::{Context, Result};
#[cfg(feature = "ffmpeg")]
use ffmpeg_next as ffmpeg;
#[cfg(feature = "ffmpeg")]
use ffmpeg_next::format::input;
#[cfg(feature = "ffmpeg")]
use ffmpeg_next::media::Type;
#[cfg(feature = "ffmpeg")]
use ffmpeg_next::util::frame::video::Pixel;
#[cfg(feature = "ffmpeg")]
use log::{debug, warn};

#[cfg(not(feature = "ffmpeg"))]
use anyhow::Result;

use std::path::Path;

/// FFmpeg-based video decoder that reads media files and produces RGBA frames.
///
/// When the `ffmpeg` feature is disabled, all methods return errors indicating
/// that FFmpeg is not available. This allows the engine to compile on platforms
/// where FFmpeg libraries are not installed (e.g. during development).
pub struct Decoder {
    #[cfg(feature = "ffmpeg")]
    format_context: Option<ffmpeg::format::context::Input>,
    #[cfg(feature = "ffmpeg")]
    video_stream_idx: Option<usize>,
    #[cfg(feature = "ffmpeg")]
    audio_stream_idx: Option<usize>,
    #[cfg(feature = "ffmpeg")]
    decoder: Option<ffmpeg::decoder::Video>,
    #[cfg(feature = "ffmpeg")]
    audio_decoder: Option<ffmpeg::decoder::Audio>,
    path: String,
    frame_count: u64,
    duration: f64,
    fps: f64,
    width: u32,
    height: u32,
}

impl Decoder {
    /// Create a new Decoder for the given file path (does not open immediately).
    pub fn new(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            anyhow::bail!("File not found: {}", path);
        }
        Ok(Self {
            #[cfg(feature = "ffmpeg")]
            format_context: None,
            #[cfg(feature = "ffmpeg")]
            video_stream_idx: None,
            #[cfg(feature = "ffmpeg")]
            audio_stream_idx: None,
            #[cfg(feature = "ffmpeg")]
            decoder: None,
            #[cfg(feature = "ffmpeg")]
            audio_decoder: None,
            path: path.to_string(),
            frame_count: 0,
            duration: 0.0,
            fps: 0.0,
            width: 0,
            height: 0,
        })
    }

    /// Open the media file and initialize the decoder contexts.
    #[cfg(feature = "ffmpeg")]
    pub fn open(&mut self) -> Result<()> {
        ffmpeg::init().context("Failed to initialize FFmpeg")?;

        let ictx = input(&self.path).context("Failed to open input file")?;
        self.duration = ictx.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE);

        // Find video stream
        let video_stream = ictx
            .streams()
            .best(Type::Video)
            .context("No video stream found")?;
        let video_idx = video_stream.index();
        self.video_stream_idx = Some(video_idx);

        let video_codec = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
            .context("Failed to get video codec context")?
            .decoder()
            .video()
            .context("Failed to create video decoder")?;

        self.fps = f64::from(video_stream.avg_frame_rate());
        if self.fps <= 0.0 || self.fps.is_nan() {
            self.fps = f64::from(video_stream.r_frame_rate());
        }
        if self.fps <= 0.0 || self.fps.is_nan() {
            self.fps = 30.0;
        }

        self.width = video_codec.width();
        self.height = video_codec.height();

        if self.duration > 0.0 && self.fps > 0.0 {
            self.frame_count = (self.duration * self.fps) as u64;
        } else {
            self.frame_count = video_stream.frames();
        }

        self.decoder = Some(video_codec);

        // Find audio stream
        if let Some(audio_stream) = ictx.streams().best(Type::Audio) {
            let audio_idx = audio_stream.index();
            self.audio_stream_idx = Some(audio_idx);

            let audio_codec =
                ffmpeg::codec::context::Context::from_parameters(audio_stream.parameters())
                    .ok()
                    .and_then(|ctx| ctx.decoder().audio().ok());
            self.audio_decoder = audio_codec;
        }

        self.format_context = Some(ictx);
        debug!(
            "Opened decoder for {}: {}x{} @ {:.2}fps, {} frames, {:.2}s",
            self.path, self.width, self.height, self.fps, self.frame_count, self.duration
        );

        Ok(())
    }

    /// Open the media file — stub when FFmpeg is not available.
    #[cfg(not(feature = "ffmpeg"))]
    pub fn open(&mut self) -> Result<()> {
        anyhow::bail!("FFmpeg is not available. Enable the 'ffmpeg' feature and install FFmpeg libraries.")
    }

    /// Close the decoder and release resources.
    pub fn close(&mut self) {
        #[cfg(feature = "ffmpeg")]
        {
            self.decoder = None;
            self.audio_decoder = None;
            self.format_context = None;
            self.video_stream_idx = None;
            self.audio_stream_idx = None;
        }
        log::debug!("Decoder closed for {}", self.path);
    }

    /// Get the total frame count.
    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the duration in seconds.
    pub fn get_duration(&self) -> f64 {
        self.duration
    }

    /// Get the resolution as (width, height).
    pub fn get_resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the frames per second.
    pub fn get_fps(&self) -> f64 {
        self.fps
    }

    /// Decode a single frame at the given index and return RGBA pixel data.
    #[cfg(feature = "ffmpeg")]
    pub fn decode_frame(&mut self, frame_idx: u64) -> Result<Vec<u8>> {
        self.seek_to_frame(frame_idx)?;

        let ictx = self.format_context.as_mut().context("Decoder not open")?;
        let video_idx = self.video_stream_idx.context("No video stream")?;
        let decoder = self.decoder.as_mut().context("No video decoder")?;

        let mut scaler = ffmpeg::converter(
            (decoder.width(), decoder.height()),
            decoder.format(),
            Pixel::RGBA,
        )
        .context("Failed to create scaler")?;

        let mut receive_done = false;
        for (stream, packet) in ictx.packets() {
            if stream.index() == video_idx {
                decoder.send_packet(&packet)?;
                let mut frame = ffmpeg::frame::Video::empty();
                if decoder.receive_frame(&mut frame).is_ok() && !receive_done {
                    let mut rgba_frame = ffmpeg::frame::Video::empty();
                    scaler.run(&frame, &mut rgba_frame)?;
                    let data = rgba_frame.data(0).to_vec();
                    return Ok(data);
                }
            }
        }

        // Flush the decoder
        decoder.send_eof()?;
        let mut frame = ffmpeg::frame::Video::empty();
        while decoder.receive_frame(&mut frame).is_ok() {
            let mut rgba_frame = ffmpeg::frame::Video::empty();
            scaler.run(&frame, &mut rgba_frame)?;
            let data = rgba_frame.data(0).to_vec();
            return Ok(data);
        }

        anyhow::bail!("Failed to decode frame at index {}", frame_idx)
    }

    /// Decode frame — stub when FFmpeg is not available.
    #[cfg(not(feature = "ffmpeg"))]
    pub fn decode_frame(&mut self, _frame_idx: u64) -> Result<Vec<u8>> {
        anyhow::bail!("FFmpeg is not available. Enable the 'ffmpeg' feature.")
    }

    /// Decode a range of frames [start, end).
    pub fn decode_range(&mut self, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        let mut frames = Vec::with_capacity((end - start) as usize);
        for idx in start..end {
            match self.decode_frame(idx) {
                Ok(data) => frames.push(data),
                Err(e) => {
                    log::warn!("Failed to decode frame {}: {}", idx, e);
                    break;
                }
            }
        }
        Ok(frames)
    }

    /// Seek to a specific frame index.
    #[cfg(feature = "ffmpeg")]
    pub fn seek_to_frame(&mut self, frame_idx: u64) -> Result<()> {
        let ictx = self.format_context.as_mut().context("Decoder not open")?;

        if self.fps > 0.0 {
            let timestamp_us = (frame_idx as f64 / self.fps * f64::from(ffmpeg::ffi::AV_TIME_BASE))
                as i64;
            ictx.seek(timestamp_us, ..timestamp_us + 1)
                .context("Seek failed")?;
        }

        // Flush decoder after seek
        if let Some(decoder) = self.decoder.as_mut() {
            decoder.flush();
        }

        debug!("Seeked to frame {}", frame_idx);
        Ok(())
    }

    /// Seek to frame — stub when FFmpeg is not available.
    #[cfg(not(feature = "ffmpeg"))]
    pub fn seek_to_frame(&mut self, _frame_idx: u64) -> Result<()> {
        anyhow::bail!("FFmpeg is not available. Enable the 'ffmpeg' feature.")
    }

    /// Extract all audio samples as interleaved f32.
    #[cfg(feature = "ffmpeg")]
    pub fn get_audio_samples(&mut self) -> Result<Vec<f32>> {
        let ictx = self.format_context.as_mut().context("Decoder not open")?;
        let audio_idx = self.audio_stream_idx.context("No audio stream")?;
        let audio_decoder = self.audio_decoder.as_mut().context("No audio decoder")?;

        let mut samples = Vec::new();
        let mut frame = ffmpeg::frame::Audio::empty();

        for (stream, packet) in ictx.packets() {
            if stream.index() == audio_idx {
                audio_decoder.send_packet(&packet)?;
                while audio_decoder.receive_frame(&mut frame).is_ok() {
                    let channels = frame.channels();
                    let nb_samples = frame.samples();
                    for ch in 0..channels {
                        let plane = frame.data(ch);
                        let sample_size = nb_samples * std::mem::size_of::<f32>();
                        if plane.len() >= sample_size {
                            let ch_samples: Vec<f32> = plane[..sample_size]
                                .chunks_exact(4)
                                .map(|chunk| {
                                    f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                                })
                                .collect();
                            if ch == 0 {
                                samples.resize(samples.len() + ch_samples.len() * channels as usize, 0.0);
                            }
                            for (i, &s) in ch_samples.iter().enumerate() {
                                samples[i * channels as usize + ch as usize] = s;
                            }
                        }
                    }
                }
            }
        }

        audio_decoder.send_eof()?;
        while audio_decoder.receive_frame(&mut frame).is_ok() {
            let channels = frame.channels();
            let nb_samples = frame.samples();
            for ch in 0..channels {
                let plane = frame.data(ch);
                let sample_size = nb_samples * std::mem::size_of::<f32>();
                if plane.len() >= sample_size {
                    let ch_samples: Vec<f32> = plane[..sample_size]
                        .chunks_exact(4)
                        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect();
                    if ch == 0 {
                        samples.resize(samples.len() + ch_samples.len() * channels as usize, 0.0);
                    }
                    for (i, &s) in ch_samples.iter().enumerate() {
                        samples[i * channels as usize + ch as usize] = s;
                    }
                }
            }
        }

        Ok(samples)
    }

    /// Get audio samples — stub when FFmpeg is not available.
    #[cfg(not(feature = "ffmpeg"))]
    pub fn get_audio_samples(&mut self) -> Result<Vec<f32>> {
        anyhow::bail!("FFmpeg is not available. Enable the 'ffmpeg' feature.")
    }

    /// Extract a thumbnail image (RGBA) at the given time in milliseconds.
    #[cfg(feature = "ffmpeg")]
    pub fn extract_thumbnail(&mut self, time_ms: u64) -> Result<Vec<u8>> {
        let timestamp_us = (time_ms as f64 * 1000.0) as i64;
        {
            let ictx = self.format_context.as_mut().context("Decoder not open")?;
            ictx.seek(timestamp_us, ..timestamp_us + 1)
                .context("Seek for thumbnail failed")?;
        }

        if let Some(decoder) = self.decoder.as_mut() {
            decoder.flush();
        }

        let frame_idx = ((time_ms as f64 / 1000.0) * self.fps) as u64;
        self.decode_frame(frame_idx)
    }

    /// Extract thumbnail — stub when FFmpeg is not available.
    #[cfg(not(feature = "ffmpeg"))]
    pub fn extract_thumbnail(&mut self, _time_ms: u64) -> Result<Vec<u8>> {
        anyhow::bail!("FFmpeg is not available. Enable the 'ffmpeg' feature.")
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        self.close();
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_new_nonexistent_file() {
        let result = Decoder::new("/nonexistent/file.mp4");
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_new_with_valid_path_structure() {
        let tmp = std::env::temp_dir().join("test_decoder_dummy.mp4");
        std::fs::write(&tmp, b"").ok();
        let result = Decoder::new(tmp.to_str().unwrap());
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_default_values() {
        let tmp = std::env::temp_dir().join("test_decoder_defaults.mp4");
        std::fs::write(&tmp, b"").ok();
        let decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        assert_eq!(decoder.get_frame_count(), 0);
        assert_eq!(decoder.get_duration(), 0.0);
        assert_eq!(decoder.get_resolution(), (0, 0));
        assert_eq!(decoder.get_fps(), 0.0);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_close_without_open() {
        let tmp = std::env::temp_dir().join("test_decoder_close.mp4");
        std::fs::write(&tmp, b"").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        decoder.close();
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_open_without_ffmpeg() {
        let tmp = std::env::temp_dir().join("test_decoder_invalid.mp4");
        std::fs::write(&tmp, b"not a real video").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        let result = decoder.open();
        // Should fail — either FFmpeg is available and file is invalid,
        // or FFmpeg is not available and open() returns a stub error
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_seek_without_open() {
        let tmp = std::env::temp_dir().join("test_decoder_seek.mp4");
        std::fs::write(&tmp, b"").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        let result = decoder.seek_to_frame(0);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_decode_without_open() {
        let tmp = std::env::temp_dir().join("test_decoder_decode.mp4");
        std::fs::write(&tmp, b"").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        let result = decoder.decode_frame(0);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_get_audio_without_open() {
        let tmp = std::env::temp_dir().join("test_decoder_audio.mp4");
        std::fs::write(&tmp, b"").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        let result = decoder.get_audio_samples();
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_extract_thumbnail_without_open() {
        let tmp = std::env::temp_dir().join("test_decoder_thumb.mp4");
        std::fs::write(&tmp, b"").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        let result = decoder.extract_thumbnail(1000);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_decoder_decode_range_without_open() {
        let tmp = std::env::temp_dir().join("test_decoder_range.mp4");
        std::fs::write(&tmp, b"").ok();
        let mut decoder = Decoder::new(tmp.to_str().unwrap()).unwrap();
        let result = decoder.decode_range(0, 10);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }
}
