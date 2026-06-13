//! Comprehensive tests for the timeline Track model
//!
//! Covers: construction, clip management, overlap detection,
//! volume control, visibility toggling, and temporal ordering.

use super::clip::Clip;
use super::track::{Track, TrackType};

/// Helper to create a basic clip with predictable values
fn make_clip(start_ms: u64, duration_ms: u64) -> Clip {
    Clip::new("asset-1", start_ms, duration_ms)
}

#[test]
fn track_new_has_expected_defaults() {
    let track = Track::new("Video 1".into(), TrackType::Video, 0);
    assert_eq!(track.name, "Video 1");
    assert_eq!(track.track_type, TrackType::Video);
    assert!(track.clips.is_empty());
    assert!(!track.locked);
    assert!(track.visible);
    assert!((track.volume - 1.0).abs() < f32::EPSILON);
    assert_eq!(track.order_index, 0);
    assert!(!track.id.is_empty());
}

#[test]
fn track_type_display_formatting() {
    assert_eq!(TrackType::Video.to_string(), "Video");
    assert_eq!(TrackType::Audio.to_string(), "Audio");
    assert_eq!(TrackType::Text.to_string(), "Text");
    assert_eq!(TrackType::Effect.to_string(), "Effect");
}

#[test]
fn track_type_serde_roundtrip() {
    for tt in [TrackType::Video, TrackType::Audio, TrackType::Text, TrackType::Effect] {
        let json = serde_json::to_string(&tt).unwrap();
        let parsed: TrackType = serde_json::from_str(&json).unwrap();
        assert_eq!(tt, parsed);
    }
}

#[test]
fn add_clip_maintains_temporal_order() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);

    // Add clips out of order
    let c3 = make_clip(3000, 1000);
    let c1 = make_clip(0, 1000);
    let c2 = make_clip(1000, 1000);

    track.add_clip(c3);
    track.add_clip(c1);
    track.add_clip(c2);

    // Verify temporal ordering
    assert_eq!(track.clips.len(), 3);
    assert_eq!(track.clips[0].start_ms, 0);
    assert_eq!(track.clips[1].start_ms, 1000);
    assert_eq!(track.clips[2].start_ms, 3000);
}

#[test]
fn add_clip_at_end() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.add_clip(make_clip(0, 1000));
    track.add_clip(make_clip(5000, 1000));
    assert_eq!(track.clips.len(), 2);
}

#[test]
fn remove_clip_by_id() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let clip = make_clip(0, 1000);
    let clip_id = clip.id.clone();
    track.add_clip(clip);

    let removed = track.remove_clip(&clip_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id, clip_id);
    assert!(track.clips.is_empty());
}

#[test]
fn remove_clip_not_found() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let removed = track.remove_clip("nonexistent");
    assert!(removed.is_none());
}

#[test]
fn find_clip_returns_correct_reference() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let clip = make_clip(1000, 2000);
    let clip_id = clip.id.clone();
    track.add_clip(clip);

    let found = track.find_clip(&clip_id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().start_ms, 1000);
    assert_eq!(found.unwrap().duration_ms, 2000);
}

#[test]
fn find_clip_mut_allows_modification() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let clip = make_clip(0, 1000);
    let clip_id = clip.id.clone();
    track.add_clip(clip);

    if let Some(c) = track.find_clip_mut(&clip_id) {
        c.duration_ms = 5000;
    }

    let found = track.find_clip(&clip_id).unwrap();
    assert_eq!(found.duration_ms, 5000);
}

#[test]
fn total_duration_empty_track() {
    let track = Track::new("V1".into(), TrackType::Video, 0);
    assert_eq!(track.total_duration_ms(), 0);
}

#[test]
fn total_duration_with_clips() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.add_clip(make_clip(0, 1000));
    track.add_clip(make_clip(1000, 3000));
    // max(start + duration) = max(1000, 4000) = 4000
    assert_eq!(track.total_duration_ms(), 4000);
}

#[test]
fn total_duration_with_gaps() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.add_clip(make_clip(0, 1000));
    track.add_clip(make_clip(5000, 2000)); // gap from 1000-5000
    assert_eq!(track.total_duration_ms(), 7000);
}

#[test]
fn would_overlap_no_overlap() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let c1 = make_clip(0, 1000);
    let c1_id = c1.id.clone();
    track.add_clip(c1);

    // Place after existing clip — no overlap
    assert!(!track.would_overlap(&c1_id, 1000, 1000));
}

#[test]
fn would_overlap_yes_overlap() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let c1 = make_clip(0, 2000);
    let c1_id = c1.id.clone();
    track.add_clip(c1);

    // Overlap: new clip starts at 1500, existing ends at 2000
    assert!(track.would_overlap(&c1_id, 1500, 1000));
}

#[test]
fn would_overlap_excludes_self() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let c1 = make_clip(0, 2000);
    let c1_id = c1.id.clone();
    track.add_clip(c1);

    // Moving the same clip — it should not count as overlapping with itself
    assert!(!track.would_overlap(&c1_id, 0, 2000));
}

#[test]
fn clips_in_range_basic() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.add_clip(make_clip(0, 1000));
    track.add_clip(make_clip(1000, 1000));
    track.add_clip(make_clip(5000, 1000));

    let in_range = track.clips_in_range(0, 2000);
    assert_eq!(in_range.len(), 2);
}

#[test]
fn clips_in_range_boundary() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.add_clip(make_clip(0, 1000));
    track.add_clip(make_clip(1000, 1000));

    // Time 1000 is the end of clip 0 and start of clip 1
    // clip 0: start=0, end=1000 → 1000 < 1000 is false, so not in range
    // clip 1: start=1000, end=2000 → 1000 < 2000 && 1000 > 1000 → false (start_ms is not > start)
    // Actually let me re-check: start_ms < clip_end && end_ms > clip.start_ms
    // clip0: 1000 < 1000 → false
    // clip1: 1000 < 2000 → true && 1000 > 1000 → false
    // So exact boundary should return empty
    let exact_boundary = track.clips_in_range(1000, 1000);
    assert_eq!(exact_boundary.len(), 0);
}

#[test]
fn set_volume_clamps_high() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.set_volume(5.0);
    assert!((track.volume - 2.0).abs() < f32::EPSILON);
}

#[test]
fn set_volume_clamps_low() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.set_volume(-1.0);
    assert!((track.volume - 0.0).abs() < f32::EPSILON);
}

#[test]
fn set_volume_normal_range() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    track.set_volume(0.5);
    assert!((track.volume - 0.5).abs() < f32::EPSILON);
}

#[test]
fn toggle_lock_flips_state() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    assert!(!track.locked);
    track.toggle_lock();
    assert!(track.locked);
    track.toggle_lock();
    assert!(!track.locked);
}

#[test]
fn toggle_visibility_flips_state() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    assert!(track.visible);
    track.toggle_visibility();
    assert!(!track.visible);
    track.toggle_visibility();
    assert!(track.visible);
}

#[test]
fn reorder_clips_fixes_disorder() {
    let mut track = Track::new("V1".into(), TrackType::Video, 0);
    let c1 = make_clip(5000, 1000);
    let c2 = make_clip(0, 1000);
    let c3 = make_clip(2000, 1000);

    // Manually insert out-of-order
    track.clips.push(c1);
    track.clips.push(c2);
    track.clips.push(c3);

    assert_eq!(track.clips[0].start_ms, 5000); // Before reorder

    track.reorder_clips();

    assert_eq!(track.clips[0].start_ms, 0);
    assert_eq!(track.clips[1].start_ms, 2000);
    assert_eq!(track.clips[2].start_ms, 5000);
}

#[test]
fn track_serialization_roundtrip() {
    let mut track = Track::new("V1".into(), TrackType::Audio, 2);
    track.add_clip(make_clip(0, 3000));
    track.set_volume(1.5);
    track.toggle_lock();

    let json = serde_json::to_string(&track).unwrap();
    let parsed: Track = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "V1");
    assert_eq!(parsed.track_type, TrackType::Audio);
    assert_eq!(parsed.clips.len(), 1);
    assert!((parsed.volume - 1.5).abs() < f32::EPSILON);
    assert!(parsed.locked);
    assert_eq!(parsed.order_index, 2);
}
