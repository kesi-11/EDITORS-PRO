//! Tests for the decoder module — VideoInfo, FrameData, AudioData structs
//!
//! Note: SoftwareDecoder and HardwareDecoder require actual media files
//! to test open/decode operations. These tests cover the data structures
//! and edge cases that can be tested without FFmpeg file I/O.

use crate::decoder::{AudioData, FrameData, VideoInfo};

#[test]
fn frame_data_blank_creates_correct_sized_frame() {
    let frame = FrameData::blank(1920, 1080);
    assert_eq!(frame.width, 1920);
    assert_eq!(frame.height, 1080);
    assert_eq!(frame.data.len(), 1920 * 1080 * 4);
    assert!(frame.data.iter().all(|&b| b == 0));
    assert_eq!(frame.timestamp_ms, 0);
    assert!(frame.is_keyframe);
}

#[test]
fn frame_data_blank_small_dimensions() {
    let frame = FrameData::blank(2, 2);
    assert_eq!(frame.data.len(), 16); // 2*2*4
}

#[test]
fn frame_data_pixel_count() {
    let frame = FrameData::blank(1920, 1080);
    assert_eq!(frame.pixel_count(), 1920 * 1080);
}

#[test]
fn frame_data_data_size() {
    let frame = FrameData::blank(640, 480);
    assert_eq!(frame.data_size(), 640 * 480 * 4);
}

#[test]
fn video_info_serialization() {
    let info = VideoInfo {
        width: 1920,
        height: 1080,
        fps: 29.97,
        duration_ms: 60000,
        codec_name: "h264".to_string(),
        bitrate: 5000000,
        has_audio: true,
        audio_codec: Some("aac".to_string()),
        audio_sample_rate: Some(44100),
        audio_channels: Some(2),
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: VideoInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.width, 1920);
    assert_eq!(parsed.height, 1080);
    assert!((parsed.fps - 29.97).abs() < 0.01);
    assert_eq!(parsed.duration_ms, 60000);
    assert_eq!(parsed.codec_name, "h264");
    assert_eq!(parsed.bitrate, 5000000);
    assert!(parsed.has_audio);
    assert_eq!(parsed.audio_codec.as_deref(), Some("aac"));
    assert_eq!(parsed.audio_sample_rate, Some(44100));
    assert_eq!(parsed.audio_channels, Some(2));
}

#[test]
fn video_info_minimal() {
    let info = VideoInfo {
        width: 640,
        height: 480,
        fps: 30.0,
        duration_ms: 10000,
        codec_name: "vp8".to_string(),
        bitrate: 1000000,
        has_audio: false,
        audio_codec: None,
        audio_sample_rate: None,
        audio_channels: None,
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: VideoInfo = serde_json::from_str(&json).unwrap();
    assert!(!parsed.has_audio);
    assert!(parsed.audio_codec.is_none());
}

#[test]
fn audio_data_construction() {
    let audio = AudioData {
        samples: vec![0.0, 0.5, -0.5, 1.0],
        sample_rate: 44100,
        channels: 2,
        timestamp_ms: 1000,
        duration_ms: 500,
    };

    assert_eq!(audio.samples.len(), 4);
    assert_eq!(audio.sample_rate, 44100);
    assert_eq!(audio.channels, 2);
}

#[test]
fn software_decoder_new_is_not_open() {
    use crate::decoder::software::SoftwareDecoder;
    let decoder = SoftwareDecoder::new();
    assert!(decoder.get_video_info().is_none());
    assert_eq!(decoder.current_position(), 0);
}

#[test]
fn software_decoder_close_resets_state() {
    use crate::decoder::software::SoftwareDecoder;
    let mut decoder = SoftwareDecoder::new();
    decoder.close(); // Should not panic
    assert!(decoder.get_video_info().is_none());
}

#[test]
fn hardware_decoder_new_is_not_open() {
    use crate::decoder::hardware::HardwareDecoder;
    let decoder = HardwareDecoder::new();
    assert!(decoder.get_video_info().is_none());
    assert_eq!(decoder.get_duration(), 0);
}

#[test]
fn hardware_decoder_close_resets_state() {
    use crate::decoder::hardware::HardwareDecoder;
    let mut decoder = HardwareDecoder::new();
    decoder.close(); // Should not panic
    assert!(decoder.get_video_info().is_none());
    assert_eq!(decoder.get_duration(), 0);
}

#[test]
fn hardware_decoder_decode_not_open_errors() {
    use crate::decoder::hardware::HardwareDecoder;
    let mut decoder = HardwareDecoder::new();
    let result = decoder.decode_frame_at(1000);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not open"));
}

#[test]
fn hardware_decoder_thumbnails_not_open_errors() {
    use crate::decoder::hardware::HardwareDecoder;
    let mut decoder = HardwareDecoder::new();
    let result = decoder.generate_thumbnails(5);
    assert!(result.is_err());
}

#[test]
fn frame_data_equality_by_value() {
    let f1 = FrameData::blank(100, 100);
    let f2 = FrameData::blank(100, 100);
    assert_eq!(f1.width, f2.width);
    assert_eq!(f1.height, f2.height);
    assert_eq!(f1.data, f2.data);
}
