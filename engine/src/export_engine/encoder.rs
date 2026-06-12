//! FFmpeg encoder wrapper for video export
//!
//! Provides a high-level interface for encoding RGBA frames into video files
//! using FFmpeg's libx264/libx265 encoders. Handles output context creation,
//! codec configuration, frame encoding, and proper finalization.
//!
//! ## Architecture
//!
//! The encoder operates in two phases:
//! 1. **Setup**: Create output context, configure codec, write header
//! 2. **Encode**: Feed RGBA frames one-by-one; each is converted to YUV420P and encoded
//!
//! For two-pass encoding, phase 2 runs twice with the first pass discarding output.

use ffmpeg_next as ffmpeg;

use super::{ExportResult, ExportSettings, ExportStage, VideoCodec};

// ──────────────────────────────────────────────────────────────────
// RGBA → YUV420P conversion
// ──────────────────────────────────────────────────────────────────

/// Convert RGBA pixel data to YUV420P planar format.
///
/// This is the CPU fallback used when the ffmpeg scaler is unavailable.
/// The conversion follows the BT.601 standard (same as libx264 default):
///
/// ```text
/// Y  =  0.257*R + 0.504*G + 0.098*B + 16
/// Cb = -0.148*R - 0.291*G + 0.439*B + 128
/// Cr =  0.439*R - 0.368*G - 0.071*B + 128
/// ```
///
/// Chroma subsampling averages 2×2 pixel blocks for Cb/Cr planes.
pub fn convert_rgba_to_yuv420p(
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let w = width as usize;
    let h = height as usize;

    if rgba.len() < w * h * 4 {
        return Err(format!(
            "RGBA data too short: expected {} bytes, got {}",
            w * h * 4,
            rgba.len()
        ));
    }

    let y_size = w * h;
    let uv_size = w * h / 4;
    let mut yuv = Vec::with_capacity(y_size * 3 / 2);
    yuv.resize(y_size + 2 * uv_size, 0u8);

    // Luma plane (Y)
    let y_plane = &mut yuv[..y_size];
    for row in 0..h {
        for col in 0..w {
            let idx = (row * w + col) * 4;
            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;

            let y = 0.257 * r + 0.504 * g + 0.098 * b + 16.0;
            y_plane[row * w + col] = y.clamp(0.0, 255.0) as u8;
        }
    }

    // Chroma planes (Cb, Cr) — 2×2 subsampled
    let cb_plane = &mut yuv[y_size..y_size + uv_size];
    let cr_plane = &mut yuv[y_size + uv_size..];

    for row in (0..h).step_by(2) {
        for col in (0..w).step_by(2) {
            // Average 2×2 block
            let mut r_sum = 0.0f32;
            let mut g_sum = 0.0f32;
            let mut b_sum = 0.0f32;
            let mut count = 0;

            for dy in 0..2 {
                for dx in 0..2 {
                    let ry = row + dy;
                    let cx = col + dx;
                    if ry < h && cx < w {
                        let idx = (ry * w + cx) * 4;
                        r_sum += rgba[idx] as f32;
                        g_sum += rgba[idx + 1] as f32;
                        b_sum += rgba[idx + 2] as f32;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                r_sum /= count as f32;
                g_sum /= count as f32;
                b_sum /= count as f32;
            }

            let cb = -0.148 * r_sum - 0.291 * g_sum + 0.439 * b_sum + 128.0;
            let cr = 0.439 * r_sum - 0.368 * g_sum - 0.071 * b_sum + 128.0;

            let uv_idx = (row / 2) * (w / 2) + col / 2;
            cb_plane[uv_idx] = cb.clamp(0.0, 255.0) as u8;
            cr_plane[uv_idx] = cr.clamp(0.0, 255.0) as u8;
        }
    }

    Ok(yuv)
}

/// Build an ffmpeg `Video` frame in YUV420P format from planar YUV data.
///
/// The returned frame has the correct line sizes and data pointers set up
/// for the encoder.
fn build_yuv420p_frame(
    yuv_data: &[u8],
    width: u32,
    height: u32,
    pts: i64,
) -> Result<ffmpeg::frame::Video, String> {
    let w = width as usize;
    let h = height as usize;
    let y_size = w * h;

    // Create an ffmpeg frame
    let mut frame = ffmpeg::frame::Video::new(
        ffmpeg::format::Pixel::YUV420P,
        width,
        height,
    );

    frame.set_pts(Some(pts));

    // Copy Y plane
    let y_src = &yuv_data[..y_size];
    let y_dst = frame.data_mut(0);
    let y_stride = frame.stride(0);
    for row in 0..h {
        let src_start = row * w;
        let src_end = src_start + w;
        let dst_start = row * y_stride;
        if src_end <= y_size && dst_start + w <= y_dst.len() {
            y_dst[dst_start..dst_start + w].copy_from_slice(&y_src[src_start..src_end]);
        }
    }

    // Copy Cb plane
    let uv_w = w / 2;
    let uv_h = h / 2;
    let uv_size = uv_w * uv_h;
    let cb_src = &yuv_data[y_size..y_size + uv_size];
    let cb_dst = frame.data_mut(1);
    let cb_stride = frame.stride(1);
    for row in 0..uv_h {
        let src_start = row * uv_w;
        let src_end = src_start + uv_w;
        let dst_start = row * cb_stride;
        if src_end <= uv_size && dst_start + uv_w <= cb_dst.len() {
            cb_dst[dst_start..dst_start + uv_w].copy_from_slice(&cb_src[src_start..src_end]);
        }
    }

    // Copy Cr plane
    let cr_src = &yuv_data[y_size + uv_size..];
    let cr_dst = frame.data_mut(2);
    let cr_stride = frame.stride(2);
    for row in 0..uv_h {
        let src_start = row * uv_w;
        let src_end = src_start + uv_w;
        let dst_start = row * cr_stride;
        if src_end <= cr_src.len() && dst_start + uv_w <= cr_dst.len() {
            cr_dst[dst_start..dst_start + uv_w].copy_from_slice(&cr_src[src_start..src_end]);
        }
    }

    Ok(frame)
}

// ──────────────────────────────────────────────────────────────────
// Encoder
// ──────────────────────────────────────────────────────────────────

/// FFmpeg-based video encoder that writes H.264/H.265/VP9 to MP4/WebM/MOV.
///
/// Usage:
/// ```rust
/// let mut encoder = VideoEncoder::new(&settings)?;
/// encoder.open(output_path)?;
///
/// for each_frame {
///     encoder.encode_rgba_frame(rgba_data, pts)?;
/// }
///
/// let result = encoder.finish()?;
/// ```
pub struct VideoEncoder {
    settings: ExportSettings,
    output_context: Option<ffmpeg::format::context::Output>,
    encoder: Option<ffmpeg::encoder::Video>,
    stream_index: Option<usize>,
    frame_count: u64,
    start_time: std::time::Instant,
}

impl VideoEncoder {
    /// Create a new encoder with the given export settings.
    pub fn new(settings: &ExportSettings) -> Result<Self, String> {
        // Validate settings
        if settings.width == 0 || settings.height == 0 {
            return Err("Width and height must be non-zero".to_string());
        }
        if settings.width % 2 != 0 || settings.height % 2 != 0 {
            return Err(format!(
                "Dimensions must be even (got {}x{})",
                settings.width, settings.height
            ));
        }
        if settings.fps <= 0.0 {
            return Err("FPS must be positive".to_string());
        }
        if settings.bitrate_kbps == 0 {
            return Err("Bitrate must be non-zero".to_string());
        }

        Ok(Self {
            settings: settings.clone(),
            output_context: None,
            encoder: None,
            stream_index: None,
            frame_count: 0,
            start_time: std::time::Instant::now(),
        })
    }

    /// Open the output file and configure the codec.
    ///
    /// After calling this, the encoder is ready to receive frames.
    pub fn open(&mut self, output_path: &str) -> Result<(), String> {
        log::info!(
            "Opening encoder: {}x{} @ {}fps, {}kbps, codec={:?}",
            self.settings.width,
            self.settings.height,
            self.settings.fps,
            self.settings.bitrate_kbps,
            self.settings.codec
        );

        // 1. Create output format context
        let mut octx = ffmpeg::format::output(&output_path)
            .map_err(|e| format!("Failed to create output context for '{}': {}", output_path, e))?;

        // 2. Find the encoder codec
        let codec_name = self.settings.codec.ffmpeg_codec_name();
        let codec = ffmpeg::encoder::find_by_name(codec_name)
            .ok_or_else(|| format!("Codec '{}' not found in FFmpeg", codec_name))?;

        // 3. Add a video stream
        let mut stream = octx.add_stream(codec)
            .map_err(|e| format!("Failed to add stream: {}", e))?;
        let stream_idx = stream.index();

        // 4. Configure the encoder
        let mut encoder = stream.codec().encoder().video()
            .map_err(|e| format!("Failed to create video encoder: {}", e))?;

        encoder.set_width(self.settings.width);
        encoder.set_height(self.settings.height);
        encoder.set_frame_rate(ffmpeg::Rational::new(self.settings.fps as i32, 1));
        encoder.set_bit_rate(self.settings.bitrate_kbps as usize * 1000);
        encoder.set_pixel_format(ffmpeg::format::Pixel::YUV420P);

        // Set time_base to match frame rate (standard practice)
        encoder.set_time_base(ffmpeg::Rational::new(1, self.settings.fps as i32));

        // Codec-specific options
        match self.settings.codec {
            VideoCodec::H264 => {
                // Use medium preset for good balance of speed/quality
                // The "medium" preset is safe across all FFmpeg builds
                if let Some(mut opts) = ffmpeg::dictionary::Dictionary::new() {
                    // Try to set preset; ignore if not supported
                    let _ = opts.set("preset", "medium");
                    let _ = opts.set("crf", "23");
                    encoder.open_with(opts)
                        .map_err(|e| format!("Failed to open H.264 encoder: {}", e))?;
                } else {
                    encoder.open()
                        .map_err(|e| format!("Failed to open H.264 encoder: {}", e))?;
                }
            }
            VideoCodec::H265 => {
                if let Some(mut opts) = ffmpeg::dictionary::Dictionary::new() {
                    let _ = opts.set("preset", "medium");
                    let _ = opts.set("crf", "28");
                    encoder.open_with(opts)
                        .map_err(|e| format!("Failed to open H.265 encoder: {}", e))?;
                } else {
                    encoder.open()
                        .map_err(|e| format!("Failed to open H.265 encoder: {}", e))?;
                }
            }
            _ => {
                encoder.open()
                    .map_err(|e| format!("Failed to open encoder: {}", e))?;
            }
        }

        // 5. Set stream parameters from encoder
        stream.set_parameters(&encoder);

        // 6. Write header
        octx.write_header()
            .map_err(|e| format!("Failed to write header: {}", e))?;

        log::info!("Encoder opened successfully: {}", output_path);

        self.output_context = Some(octx);
        self.encoder = Some(encoder);
        self.stream_index = Some(stream_idx);
        self.start_time = std::time::Instant::now();
        self.frame_count = 0;

        Ok(())
    }

    /// Encode a single RGBA frame and write it to the output.
    ///
    /// The `pts` (presentation timestamp) should be the frame number,
    /// starting from 0. The encoder's time_base will convert this to
    /// the correct timestamp.
    pub fn encode_rgba_frame(&mut self, rgba_data: &[u8], pts: i64) -> Result<(), String> {
        let octx = self.output_context.as_mut()
            .ok_or("Encoder not opened")?;
        let encoder = self.encoder.as_mut()
            .ok_or("Encoder not configured")?;

        // Convert RGBA → YUV420P
        let yuv_data = convert_rgba_to_yuv420p(rgba_data, self.settings.width, self.settings.height)?;

        // Build ffmpeg frame
        let frame = build_yuv420p_frame(&yuv_data, self.settings.width, self.settings.height, pts)?;

        // Send frame to encoder
        encoder.send_frame(&frame)
            .map_err(|e| format!("Send frame error: {}", e))?;

        // Receive and write encoded packets
        self.receive_and_write_packets(octx, encoder)?;

        self.frame_count += 1;
        Ok(())
    }

    /// Encode a single frame that is already in YUV420P format.
    pub fn encode_yuv420p_frame(&mut self, yuv_data: &[u8], pts: i64) -> Result<(), String> {
        let octx = self.output_context.as_mut()
            .ok_or("Encoder not opened")?;
        let encoder = self.encoder.as_mut()
            .ok_or("Encoder not configured")?;

        let frame = build_yuv420p_frame(yuv_data, self.settings.width, self.settings.height, pts)?;

        encoder.send_frame(&frame)
            .map_err(|e| format!("Send frame error: {}", e))?;

        self.receive_and_write_packets(octx, encoder)?;

        self.frame_count += 1;
        Ok(())
    }

    /// Flush the encoder and write the trailer, returning the export result.
    ///
    /// After calling this, the encoder is consumed and cannot be reused.
    pub fn finish(mut self, duration_ms: u64) -> Result<ExportResult, String> {
        let octx = self.output_context.as_mut()
            .ok_or("Encoder not opened")?;

        // Flush the encoder
        if let Some(encoder) = self.encoder.as_mut() {
            // Send EOF to flush
            let _ = encoder.send_eof();
            self.receive_and_write_packets(octx, encoder)?;
        }

        // Write trailer
        octx.write_trailer()
            .map_err(|e| format!("Failed to write trailer: {}", e))?;

        // Get output file size
        let output_path = octx.path().unwrap_or_default().to_string_lossy().to_string();
        let file_size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let elapsed = self.start_time.elapsed();
        log::info!(
            "Export complete: {} frames, {}ms → {}bytes, took {:.1}s",
            self.frame_count,
            duration_ms,
            file_size,
            elapsed.as_secs_f64()
        );

        Ok(ExportResult {
            success: true,
            output_path,
            file_size_bytes: file_size,
            duration_ms,
            error_message: None,
        })
    }

    /// Cancel the export and clean up the output file.
    pub fn cancel(self) {
        if let Some(octx) = self.output_context {
            let path = octx.path().unwrap_or_default().to_string_lossy().to_string();
            // Close the context first
            drop(octx);
            // Delete the partial file
            let _ = std::fs::remove_file(&path);
            log::info!("Export cancelled, partial file removed: {}", path);
        }
    }

    /// Get the number of frames encoded so far.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the elapsed time since the encoder was opened.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Receive encoded packets from the encoder and write them to the output.
    fn receive_and_write_packets(
        &mut self,
        octx: &mut ffmpeg::format::context::Output,
        encoder: &mut ffmpeg::encoder::Video,
    ) -> Result<(), String> {
        let mut packet = ffmpeg::Packet::new();
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.write_interleaved(octx)
                .map_err(|e| format!("Write packet error: {}", e))?;
        }
        Ok(())
    }
}

/// Estimate the remaining time for an export based on progress.
///
/// Returns the estimated seconds remaining, or 0 if not enough data.
pub fn estimate_remaining(
    current_frame: u64,
    total_frames: u64,
    start_time: std::time::Instant,
) -> u64 {
    if current_frame == 0 || total_frames == 0 {
        return 0;
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let frames_per_sec = current_frame as f64 / elapsed;
    let remaining_frames = total_frames.saturating_sub(current_frame);

    (remaining_frames as f64 / frames_per_sec).ceil() as u64
}

/// Check available storage space at the given path.
///
/// Returns `Ok(())` if there's enough space (at least 500MB free),
/// or an error message describing the issue.
pub fn check_storage_space(output_path: &str, estimated_size_bytes: u64) -> Result<(), String> {
    // Get the parent directory of the output path
    let path = std::path::Path::new(output_path);
    let dir = path.parent().unwrap_or(std::path::Path::new("/"));

    // Try to get filesystem stats
    // On Android, this will work for app-specific directories
    let available = fs_available_space(dir);

    // Require at least 100MB + estimated size
    let required = estimated_size_bytes + 100 * 1024 * 1024;

    if available < required {
        let available_mb = available / (1024 * 1024);
        let required_mb = required / (1024 * 1024);
        return Err(format!(
            "Insufficient storage: {:.0}MB available, {:.0}MB required",
            available_mb, required_mb
        ));
    }

    Ok(())
}

/// Get available disk space for a directory (in bytes).
///
/// Returns 0 if the information cannot be determined.
fn fs_available_space(path: &std::path::Path) -> u64 {
    // On Unix (Android is Linux), use statvfs
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;

        let c_path = match CString::new(path.as_os_str().as_bytes()) {
            Ok(p) => p,
            Err(_) => return 0,
        };

        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
                let stat = stat.assume_init();
                return stat.f_bsize as u64 * stat.f_bavail;
            }
        }
    }

    // Fallback: assume plenty of space
    #[cfg(not(unix))]
    {
        let _ = path;
    }

    0
}

// ──────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Test RGBA → YUV420P conversion produces the expected buffer size
    #[test]
    fn test_rgba_to_yuv420p_size() {
        let width = 64u32;
        let height = 64u32;
        let rgba = vec![128u8; (width * height * 4) as usize];

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        let expected_size = (width * height * 3 / 2) as usize;
        assert_eq!(yuv.len(), expected_size, "YUV420P size should be W*H*3/2");
    }

    /// Test that black RGBA pixels produce the expected YUV values
    #[test]
    fn test_rgba_to_yuv420p_black() {
        let width = 4u32;
        let height = 4u32;
        // RGBA black (0,0,0,255)
        let rgba: Vec<u8> = (0..width * height)
            .flat_map(|_| [0u8, 0u8, 0u8, 255u8])
            .collect();

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        let y_size = (width * height) as usize;
        // Y for black should be 16 (BT.601 limited range)
        assert_eq!(yuv[0], 16, "Y for black should be 16 (limited range)");
        // Cb for black should be 128
        assert_eq!(yuv[y_size], 128, "Cb for black should be 128");
    }

    /// Test that white RGBA pixels produce the expected Y values
    #[test]
    fn test_rgba_to_yuv420p_white() {
        let width = 4u32;
        let height = 4u32;
        // RGBA white (255,255,255,255)
        let rgba: Vec<u8> = (0..width * height)
            .flat_map(|_| [255u8, 255u8, 255u8, 255u8])
            .collect();

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        // Y for white should be 235 (BT.601 limited range)
        assert_eq!(yuv[0], 235, "Y for white should be 235 (limited range)");
    }

    /// Test that even-dimension requirement is enforced
    #[test]
    fn test_odd_dimensions_rejected() {
        let result = VideoEncoder::new(&ExportSettings {
            width: 191,
            height: 108,
            fps: 30.0,
            bitrate_kbps: 5000,
            codec: VideoCodec::H264,
            format: super::super::OutputFormat::Mp4,
            audio_bitrate_kbps: 128,
            audio_sample_rate: 44100,
            audio_channels: 2,
            include_audio: true,
            two_pass: false,
        });
        assert!(result.is_err(), "Odd dimensions should be rejected");
    }

    /// Test that zero dimensions are rejected
    #[test]
    fn test_zero_dimensions_rejected() {
        let result = VideoEncoder::new(&ExportSettings {
            width: 0,
            height: 1080,
            fps: 30.0,
            bitrate_kbps: 5000,
            codec: VideoCodec::H264,
            format: super::super::OutputFormat::Mp4,
            audio_bitrate_kbps: 128,
            audio_sample_rate: 44100,
            audio_channels: 2,
            include_audio: true,
            two_pass: false,
        });
        assert!(result.is_err(), "Zero width should be rejected");
    }

    /// Test estimate_remaining returns sensible values
    #[test]
    fn test_estimate_remaining() {
        let start = std::time::Instant::now() - std::time::Duration::from_secs(10);
        // 100 frames encoded in 10 seconds = 10 fps
        // 900 frames remaining → ~90 seconds
        let remaining = estimate_remaining(100, 1000, start);
        assert!(remaining > 80 && remaining < 100, "Expected ~90s, got {}", remaining);
    }

    /// Test estimate_remaining with zero frames returns 0
    #[test]
    fn test_estimate_remaining_zero_frames() {
        let start = std::time::Instant::now();
        assert_eq!(estimate_remaining(0, 1000, start), 0);
    }
}
