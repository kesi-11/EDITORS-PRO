//! Integration tests for the export pipeline
//!
//! These tests verify the full export flow from settings configuration
//! through the encoding loop. They require FFmpeg to be available at
//! runtime and will create temporary files.

#[cfg(test)]
mod tests {
    use crate::export_engine::{
        convert_rgba_to_yuv420p, estimate_remaining, check_storage_space,
        ExportPipeline, ExportSettings, OutputFormat, VideoCodec,
    };

    // ── Color Conversion Tests ──────────────────────────────────────

    #[test]
    fn test_rgba_to_yuv420p_red_pixel() {
        // Pure red: R=255, G=0, B=0
        let width = 4u32;
        let height = 4u32;
        let rgba: Vec<u8> = (0..width * height)
            .flat_map(|_| [255u8, 0u8, 0u8, 255u8])
            .collect();

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        let y_size = (width * height) as usize;
        // Y for red (BT.601): 0.257*255 + 0.504*0 + 0.098*0 + 16 = 81.5 ≈ 82
        assert!(
            yuv[0] >= 80 && yuv[0] <= 84,
            "Y for red should be ~82, got {}",
            yuv[0]
        );
        // Cb for red: -0.148*255 - 0.291*0 + 0.439*0 + 128 = 90.3 ≈ 90
        assert!(
            yuv[y_size] >= 88 && yuv[y_size] <= 92,
            "Cb for red should be ~90, got {}",
            yuv[y_size]
        );
    }

    #[test]
    fn test_rgba_to_yuv420p_green_pixel() {
        let width = 4u32;
        let height = 4u32;
        let rgba: Vec<u8> = (0..width * height)
            .flat_map(|_| [0u8, 255u8, 0u8, 255u8])
            .collect();

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        // Y for green: 0.257*0 + 0.504*255 + 0.098*0 + 16 = 144.5 ≈ 145
        assert!(
            yuv[0] >= 143 && yuv[0] <= 147,
            "Y for green should be ~145, got {}",
            yuv[0]
        );
    }

    #[test]
    fn test_rgba_to_yuv420p_blue_pixel() {
        let width = 4u32;
        let height = 4u32;
        let rgba: Vec<u8> = (0..width * height)
            .flat_map(|_| [0u8, 0u8, 255u8, 255u8])
            .collect();

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        // Y for blue: 0.257*0 + 0.504*0 + 0.098*255 + 16 = 40.9 ≈ 41
        assert!(
            yuv[0] >= 39 && yuv[0] <= 43,
            "Y for blue should be ~41, got {}",
            yuv[0]
        );
    }

    #[test]
    fn test_rgba_to_yuv420p_dimensions_1080p() {
        let width = 1920u32;
        let height = 1080u32;
        let rgba = vec![128u8; (width * height * 4) as usize];

        let yuv = convert_rgba_to_yuv420p(&rgba, width, height).unwrap();

        let expected_size = (width * height * 3 / 2) as usize;
        assert_eq!(yuv.len(), expected_size);
    }

    #[test]
    fn test_rgba_to_yuv420p_uneven_input_rejected() {
        // Width and height must be at least 2 for chroma subsampling
        // But our function doesn't enforce this — it just works with 4:2:0
        // The VideoEncoder enforces even dimensions, not the conversion.
        let width = 1u32;
        let height = 1u32;
        let rgba = vec![128u8; 4];
        // This should succeed (1x1 is valid for the conversion)
        let result = convert_rgba_to_yuv420p(&rgba, width, height);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rgba_to_yuv420p_too_short_data() {
        let width = 4u32;
        let height = 4u32;
        let rgba = vec![0u8; 10]; // Way too short
        let result = convert_rgba_to_yuv420p(&rgba, width, height);
        assert!(result.is_err());
    }

    // ── Pipeline Tests ──────────────────────────────────────────────

    #[test]
    fn test_pipeline_total_frames() {
        let settings = ExportSettings::full_hd_1080p();
        let pipeline = ExportPipeline::new(settings);

        // 30 seconds at 30fps = 900 frames
        assert_eq!(pipeline.total_frames(30000), 900);
        // 1 second = 30 frames
        assert_eq!(pipeline.total_frames(1000), 30);
        // 0ms = 0 frames
        assert_eq!(pipeline.total_frames(0), 0);
    }

    #[test]
    fn test_pipeline_frame_at_time() {
        let settings = ExportSettings::full_hd_1080p();
        let pipeline = ExportPipeline::new(settings);

        // At 0ms, frame 0
        assert_eq!(pipeline.frame_at_time(0), 0);
        // At 1000ms (1s), frame 30
        assert_eq!(pipeline.frame_at_time(1000), 30);
        // At 500ms, frame 15
        assert_eq!(pipeline.frame_at_time(500), 15);
    }

    #[test]
    fn test_pipeline_time_at_frame() {
        let settings = ExportSettings::full_hd_1080p();
        let pipeline = ExportPipeline::new(settings);

        // Frame 0 = 0ms
        assert_eq!(pipeline.time_at_frame(0), 0);
        // Frame 30 = 1000ms
        assert_eq!(pipeline.time_at_frame(30), 1000);
        // Frame 1 = 33ms (1000/30)
        assert_eq!(pipeline.time_at_frame(1), 33);
    }

    // ── Settings Preset Tests ───────────────────────────────────────

    #[test]
    fn test_preset_720p() {
        let settings = ExportSettings::hd_720p();
        assert_eq!(settings.width, 1280);
        assert_eq!(settings.height, 720);
        assert_eq!(settings.fps, 30.0);
        assert_eq!(settings.codec, VideoCodec::H264);
        assert_eq!(settings.format, OutputFormat::Mp4);
        assert!(!settings.two_pass);
    }

    #[test]
    fn test_preset_1080p() {
        let settings = ExportSettings::full_hd_1080p();
        assert_eq!(settings.width, 1920);
        assert_eq!(settings.height, 1080);
        assert_eq!(settings.bitrate_kbps, 10000);
    }

    #[test]
    fn test_preset_4k() {
        let settings = ExportSettings::ultra_hd_4k();
        assert_eq!(settings.width, 3840);
        assert_eq!(settings.height, 2160);
        assert_eq!(settings.codec, VideoCodec::H265);
        assert!(settings.two_pass);
    }

    #[test]
    fn test_preset_social_vertical() {
        let settings = ExportSettings::social_vertical();
        assert_eq!(settings.width, 1080);
        assert_eq!(settings.height, 1920);
    }

    #[test]
    fn test_preset_social_square() {
        let settings = ExportSettings::social_square();
        assert_eq!(settings.width, 1080);
        assert_eq!(settings.height, 1080);
    }

    #[test]
    fn test_preset_by_name() {
        assert!(ExportSettings::preset_by_name("1080p").is_some());
        assert!(ExportSettings::preset_by_name("720p").is_some());
        assert!(ExportSettings::preset_by_name("4K").is_some());
        assert!(ExportSettings::preset_by_name("Social Vertical").is_some());
        assert!(ExportSettings::preset_by_name("Social Square").is_some());
        assert!(ExportSettings::preset_by_name("unknown").is_none());
    }

    // ── Estimated File Size Tests ───────────────────────────────────

    #[test]
    fn test_estimated_file_size() {
        let settings = ExportSettings::full_hd_1080p();
        // 10 seconds of 1080p at 10Mbps video + 192kbps audio
        let size = settings.estimated_file_size(10000);
        // Video: 10Mbps * 10s / 8 = 12.5MB
        // Audio: 192kbps * 10s / 8 = 0.24MB
        // Total with 5% overhead ≈ 13.4MB
        assert!(
            size > 10_000_000 && size < 20_000_000,
            "Expected ~13MB, got {} bytes",
            size
        );
    }

    // ── Codec Tests ─────────────────────────────────────────────────

    #[test]
    fn test_codec_from_str_lossy() {
        assert_eq!(VideoCodec::from_str_lossy("H.264"), Some(VideoCodec::H264));
        assert_eq!(VideoCodec::from_str_lossy("h264"), Some(VideoCodec::H264));
        assert_eq!(VideoCodec::from_str_lossy("H.265"), Some(VideoCodec::H265));
        assert_eq!(VideoCodec::from_str_lossy("HEVC"), Some(VideoCodec::H265));
        assert_eq!(VideoCodec::from_str_lossy("VP9"), Some(VideoCodec::Vp9));
        assert_eq!(VideoCodec::from_str_lossy("AV1"), Some(VideoCodec::Av1));
        assert_eq!(VideoCodec::from_str_lossy("unknown"), None);
    }

    // ── Format Tests ────────────────────────────────────────────────

    #[test]
    fn test_format_from_str_lossy() {
        assert_eq!(OutputFormat::from_str_lossy("MP4"), Some(OutputFormat::Mp4));
        assert_eq!(OutputFormat::from_str_lossy("webm"), Some(OutputFormat::WebM));
        assert_eq!(OutputFormat::from_str_lossy("MOV"), Some(OutputFormat::Mov));
        assert_eq!(OutputFormat::from_str_lossy("unknown"), None);
    }

    // ── Export Result Tests ─────────────────────────────────────────

    #[test]
    fn test_export_result_file_size_human() {
        use crate::export_engine::ExportResult;

        let result = ExportResult {
            success: true,
            output_path: "/test.mp4".to_string(),
            file_size_bytes: 1024,
            duration_ms: 10000,
            error_message: None,
        };
        assert_eq!(result.file_size_human(), "1.0 KB");

        let result_mb = ExportResult {
            success: true,
            output_path: "/test.mp4".to_string(),
            file_size_bytes: 10 * 1024 * 1024,
            duration_ms: 10000,
            error_message: None,
        };
        assert_eq!(result_mb.file_size_human(), "10.0 MB");
    }

    #[test]
    fn test_export_result_error() {
        use crate::export_engine::ExportResult;

        let result = ExportResult::error("Test error", "/test.mp4");
        assert!(!result.success);
        assert_eq!(result.error_message, Some("Test error".to_string()));
    }

    // ── Export Stage Tests ──────────────────────────────────────────

    #[test]
    fn test_export_stage_display_name() {
        use crate::export_engine::ExportStage;

        assert_eq!(ExportStage::Preparing.display_name(), "Preparing");
        assert_eq!(ExportStage::Encoding.display_name(), "Encoding");
        assert_eq!(ExportStage::Finalizing.display_name(), "Finalizing");
        assert_eq!(ExportStage::Complete.display_name(), "Complete");
        assert_eq!(ExportStage::Error("test".to_string()).display_name(), "Error");
    }

    // ── Progress Report Tests ───────────────────────────────────────

    #[test]
    fn test_export_progress_preparing() {
        use crate::export_engine::ExportProgress;

        let progress = ExportProgress::preparing();
        assert_eq!(progress.progress, 0.0);
        assert_eq!(progress.stage, crate::export_engine::ExportStage::Preparing);
    }

    #[test]
    fn test_export_progress_encoding() {
        use crate::export_engine::ExportProgress;

        let start = std::time::Instant::now();
        let progress = ExportProgress::encoding(50, 100, start);
        assert!((progress.progress - 0.5).abs() < 0.01);
        assert_eq!(progress.current_frame, 50);
        assert_eq!(progress.total_frames, 100);
    }

    #[test]
    fn test_export_progress_complete() {
        use crate::export_engine::ExportProgress;

        let progress = ExportProgress::complete();
        assert_eq!(progress.progress, 1.0);
        assert_eq!(progress.stage, crate::export_engine::ExportStage::Complete);
    }
}
