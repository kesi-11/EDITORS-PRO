//! Edge case validation and boundary checks
//!
//! Provides validation functions for engine inputs, timeline constraints,
//! and boundary conditions that must be checked before operations.

use crate::timeline::clip::Clip;
use crate::timeline::track::{Track, TrackType};

/// Maximum number of tracks allowed in a timeline
pub const MAX_TRACKS: usize = 32;

/// Maximum number of clips per track
pub const MAX_CLIPS_PER_TRACK: usize = 500;

/// Maximum clip duration in milliseconds (24 hours)
pub const MAX_CLIP_DURATION_MS: u64 = 24 * 60 * 60 * 1000;

/// Maximum timeline duration in milliseconds (24 hours)
pub const MAX_TIMELINE_DURATION_MS: u64 = 24 * 60 * 60 * 1000;

/// Minimum clip duration in milliseconds (1 frame at 60fps ≈ 16.67ms)
pub const MIN_CLIP_DURATION_MS: u64 = 16;

/// Maximum number of undo levels
pub const MAX_UNDO_LEVELS: usize = 200;

/// Maximum number of effects per clip
pub const MAX_EFFECTS_PER_CLIP: usize = 20;

/// Maximum export resolution
pub const MAX_EXPORT_WIDTH: u32 = 3840;
pub const MAX_EXPORT_HEIGHT: u32 = 2160;

/// Minimum export resolution
pub const MIN_EXPORT_WIDTH: u32 = 128;
pub const MIN_EXPORT_HEIGHT: u32 = 128;

/// Maximum export bitrate (100 Mbps)
pub const MAX_EXPORT_BITRATE_KBPS: u64 = 100_000;

/// Validate that a clip's timing is valid
pub fn validate_clip_timing(clip: &Clip) -> Result<(), String> {
    if clip.duration_ms < MIN_CLIP_DURATION_MS {
        return Err(format!(
            "Clip duration {}ms is below minimum {}ms",
            clip.duration_ms, MIN_CLIP_DURATION_MS
        ));
    }

    if clip.duration_ms > MAX_CLIP_DURATION_MS {
        return Err(format!(
            "Clip duration {}ms exceeds maximum {}ms",
            clip.duration_ms, MAX_CLIP_DURATION_MS
        ));
    }

    Ok(())
}

/// Validate that adding a track won't exceed the maximum
pub fn validate_track_count(current_count: usize) -> Result<(), String> {
    if current_count >= MAX_TRACKS {
        return Err(format!(
            "Cannot add more tracks: {} already exist (max: {})",
            current_count, MAX_TRACKS
        ));
    }
    Ok(())
}

/// Validate that a track isn't overfull
pub fn validate_clip_count(track: &Track) -> Result<(), String> {
    if track.clips.len() >= MAX_CLIPS_PER_TRACK {
        return Err(format!(
            "Track '{}' has {} clips (max: {})",
            track.name,
            track.clips.len(),
            MAX_CLIPS_PER_TRACK
        ));
    }
    Ok(())
}

/// Validate export resolution
pub fn validate_export_resolution(width: u32, height: u32) -> Result<(), String> {
    if width < MIN_EXPORT_WIDTH || height < MIN_EXPORT_HEIGHT {
        return Err(format!(
            "Export resolution {}x{} is below minimum {}x{}",
            width, height, MIN_EXPORT_WIDTH, MIN_EXPORT_HEIGHT
        ));
    }

    if width > MAX_EXPORT_WIDTH || height > MAX_EXPORT_HEIGHT {
        return Err(format!(
            "Export resolution {}x{} exceeds maximum {}x{}",
            width, height, MAX_EXPORT_WIDTH, MAX_EXPORT_HEIGHT
        ));
    }

    // Must be even dimensions (H.264/H.265 requirement)
    if width % 2 != 0 || height % 2 != 0 {
        return Err(format!(
            "Export resolution {}x{} must have even dimensions",
            width, height
        ));
    }

    Ok(())
}

/// Validate export bitrate
pub fn validate_export_bitrate(bitrate_kbps: u64) -> Result<(), String> {
    if bitrate_kbps == 0 {
        return Err("Export bitrate cannot be zero".to_string());
    }

    if bitrate_kbps > MAX_EXPORT_BITRATE_KBPS {
        return Err(format!(
            "Export bitrate {}kbps exceeds maximum {}kbps",
            bitrate_kbps, MAX_EXPORT_BITRATE_KBPS
        ));
    }

    Ok(())
}

/// Validate FPS value
pub fn validate_fps(fps: f64) -> Result<(), String> {
    if fps <= 0.0 {
        return Err("FPS must be positive".to_string());
    }

    if fps > 240.0 {
        return Err(format!("FPS {} exceeds maximum 240", fps));
    }

    Ok(())
}

/// Validate a seek position within timeline bounds
pub fn validate_seek_position(position_ms: u64, duration_ms: u64) -> Result<(), String> {
    if position_ms > duration_ms {
        return Err(format!(
            "Seek position {}ms exceeds timeline duration {}ms",
            position_ms, duration_ms
        ));
    }
    Ok(())
}

/// Validate opacity value
pub fn validate_opacity(opacity: f32) -> Result<(), String> {
    if opacity < 0.0 || opacity > 1.0 {
        return Err(format!("Opacity {} must be between 0.0 and 1.0", opacity));
    }
    Ok(())
}

/// Validate speed value
pub fn validate_speed(speed: f32) -> Result<(), String> {
    if speed <= 0.0 {
        return Err("Speed must be positive".to_string());
    }

    if speed > 100.0 {
        return Err(format!("Speed {} exceeds maximum 100x", speed));
    }

    Ok(())
}

/// Validate volume value (in dB)
pub fn validate_volume_db(volume_db: f32) -> Result<(), String> {
    if volume_db < -60.0 {
        return Err(format!("Volume {}dB is below minimum -60dB", volume_db));
    }

    if volume_db > 12.0 {
        return Err(format!("Volume {}dB exceeds maximum +12dB", volume_db));
    }

    Ok(())
}

/// Validate pan value
pub fn validate_pan(pan: f32) -> Result<(), String> {
    if pan < -1.0 || pan > 1.0 {
        return Err(format!("Pan {} must be between -1.0 and 1.0", pan));
    }
    Ok(())
}

/// Validate effect count per clip
pub fn validate_effect_count(count: usize) -> Result<(), String> {
    if count > MAX_EFFECTS_PER_CLIP {
        return Err(format!(
            "Clip has {} effects (max: {})",
            count, MAX_EFFECTS_PER_CLIP
        ));
    }
    Ok(())
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_clip_timing_valid() {
        let clip = Clip::new("asset-1", 0, 5000);
        assert!(validate_clip_timing(&clip).is_ok());
    }

    #[test]
    fn test_validate_clip_timing_too_short() {
        let mut clip = Clip::new("asset-1", 0, 5);
        clip.duration_ms = 5;
        assert!(validate_clip_timing(&clip).is_err());
    }

    #[test]
    fn test_validate_track_count_ok() {
        assert!(validate_track_count(10).is_ok());
    }

    #[test]
    fn test_validate_track_count_max() {
        assert!(validate_track_count(MAX_TRACKS).is_err());
    }

    #[test]
    fn test_validate_export_resolution_valid() {
        assert!(validate_export_resolution(1920, 1080).is_ok());
        assert!(validate_export_resolution(3840, 2160).is_ok());
        assert!(validate_export_resolution(128, 128).is_ok());
    }

    #[test]
    fn test_validate_export_resolution_too_small() {
        assert!(validate_export_resolution(64, 64).is_err());
    }

    #[test]
    fn test_validate_export_resolution_too_large() {
        assert!(validate_export_resolution(7680, 4320).is_err());
    }

    #[test]
    fn test_validate_export_resolution_odd() {
        assert!(validate_export_resolution(1921, 1081).is_err());
    }

    #[test]
    fn test_validate_export_bitrate_valid() {
        assert!(validate_export_bitrate(5000).is_ok());
    }

    #[test]
    fn test_validate_export_bitrate_zero() {
        assert!(validate_export_bitrate(0).is_err());
    }

    #[test]
    fn test_validate_export_bitrate_too_high() {
        assert!(validate_export_bitrate(200_000).is_err());
    }

    #[test]
    fn test_validate_fps_valid() {
        assert!(validate_fps(24.0).is_ok());
        assert!(validate_fps(60.0).is_ok());
    }

    #[test]
    fn test_validate_fps_zero() {
        assert!(validate_fps(0.0).is_err());
    }

    #[test]
    fn test_validate_fps_negative() {
        assert!(validate_fps(-1.0).is_err());
    }

    #[test]
    fn test_validate_fps_too_high() {
        assert!(validate_fps(300.0).is_err());
    }

    #[test]
    fn test_validate_seek_position_valid() {
        assert!(validate_seek_position(5000, 10000).is_ok());
    }

    #[test]
    fn test_validate_seek_position_over() {
        assert!(validate_seek_position(15000, 10000).is_err());
    }

    #[test]
    fn test_validate_opacity_valid() {
        assert!(validate_opacity(0.0).is_ok());
        assert!(validate_opacity(0.5).is_ok());
        assert!(validate_opacity(1.0).is_ok());
    }

    #[test]
    fn test_validate_opacity_invalid() {
        assert!(validate_opacity(-0.1).is_err());
        assert!(validate_opacity(1.1).is_err());
    }

    #[test]
    fn test_validate_speed_valid() {
        assert!(validate_speed(0.25).is_ok());
        assert!(validate_speed(1.0).is_ok());
        assert!(validate_speed(4.0).is_ok());
    }

    #[test]
    fn test_validate_speed_invalid() {
        assert!(validate_speed(0.0).is_err());
        assert!(validate_speed(-1.0).is_err());
        assert!(validate_speed(101.0).is_err());
    }

    #[test]
    fn test_validate_volume_db_valid() {
        assert!(validate_volume_db(-60.0).is_ok());
        assert!(validate_volume_db(0.0).is_ok());
        assert!(validate_volume_db(12.0).is_ok());
    }

    #[test]
    fn test_validate_volume_db_invalid() {
        assert!(validate_volume_db(-61.0).is_err());
        assert!(validate_volume_db(13.0).is_err());
    }

    #[test]
    fn test_validate_pan_valid() {
        assert!(validate_pan(-1.0).is_ok());
        assert!(validate_pan(0.0).is_ok());
        assert!(validate_pan(1.0).is_ok());
    }

    #[test]
    fn test_validate_pan_invalid() {
        assert!(validate_pan(-1.1).is_err());
        assert!(validate_pan(1.1).is_err());
    }

    #[test]
    fn test_validate_effect_count_ok() {
        assert!(validate_effect_count(5).is_ok());
    }

    #[test]
    fn test_validate_effect_count_too_many() {
        assert!(validate_effect_count(25).is_err());
    }

    #[test]
    fn test_constants_are_sane() {
        assert!(MIN_CLIP_DURATION_MS > 0);
        assert!(MAX_CLIP_DURATION_MS > MIN_CLIP_DURATION_MS);
        assert!(MIN_EXPORT_WIDTH < MAX_EXPORT_WIDTH);
        assert!(MIN_EXPORT_HEIGHT < MAX_EXPORT_HEIGHT);
        assert!(MAX_TRACKS > 0);
        assert!(MAX_CLIPS_PER_TRACK > 0);
        assert!(MAX_UNDO_LEVELS > 0);
    }
}
