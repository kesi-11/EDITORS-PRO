//! Template instantiation — replaces placeholders with user media
//!
//! Takes a Template and a map of slot_id → media_path, then builds
//! a complete Timeline with all effects, transitions, and timing preserved.

use std::collections::HashMap;

use super::{PlaceholderSlot, Template};
use crate::timeline::clip::Clip;
use crate::timeline::track::TrackType;
use crate::timeline::Timeline;

/// Map of placeholder slot ID to user-provided media path
pub type MediaAssignments = HashMap<String, String>;

/// Result of template instantiation
#[derive(Debug, Clone)]
pub struct InstantiatedTemplate {
    pub timeline: Timeline,
    pub unfilled_slots: Vec<String>,
}

/// Build a project from a template by filling placeholders with user media
///
/// For each placeholder slot:
/// 1. Look up the user's media path in the assignments map
/// 2. Create a clip from the media asset
/// 3. Place it on the correct track at the correct position
/// 4. Apply any per-slot effects from the template
///
/// Slots without assignments are left as black/color fill clips.
pub fn instantiate_template(
    template: &Template,
    assignments: &MediaAssignments,
) -> Result<InstantiatedTemplate, String> {
    // Clone the template timeline as the starting point
    let mut timeline = template.timeline_template.clone();
    let mut unfilled_slots: Vec<String> = Vec::new();

    for slot in &template.placeholder_slots {
        // Get the track where this slot belongs
        let track_index = slot.track_index;
        if track_index >= timeline.tracks.len() {
            return Err(format!(
                "Slot '{}' references track index {} but timeline only has {} tracks",
                slot.id,
                track_index,
                timeline.tracks.len()
            ));
        }

        let track_id = timeline.tracks[track_index].id.clone();

        if let Some(media_path) = assignments.get(&slot.id) {
            // Create a clip from the user's media
            let clip_duration = if slot.expected_duration_ms > 0 {
                slot.expected_duration_ms
            } else {
                5000 // Default 5s if no expected duration
            };

            // Use the file path as the asset_id (the engine will resolve it)
            let asset_id = derive_asset_id(media_path);
            let clip = Clip::new(&asset_id, slot.start_ms, clip_duration);

            if let Err(e) = timeline.add_clip_to_track(&track_id, clip) {
                log::warn!(
                    "Failed to add clip for slot '{}' on track '{}': {}",
                    slot.id,
                    track_id,
                    e
                );
            }
        } else {
            // No assignment — create a placeholder clip (black fill)
            unfilled_slots.push(slot.id.clone());

            let clip_duration = if slot.expected_duration_ms > 0 {
                slot.expected_duration_ms
            } else {
                3000
            };

            // Create a black-fill placeholder clip with special asset ID
            let placeholder_id = format!("__placeholder_{}__", slot.id);
            let mut clip = Clip::new(&placeholder_id, slot.start_ms, clip_duration);
            clip.opacity = 0.0; // Make it transparent/black

            if let Err(e) = timeline.add_clip_to_track(&track_id, clip) {
                log::warn!(
                    "Failed to add placeholder clip for slot '{}' on track '{}': {}",
                    slot.id,
                    track_id,
                    e
                );
            }
        }
    }

    // Set the timeline duration to match the template
    timeline.duration_ms = template.duration_ms;
    timeline.recalculate_duration();

    Ok(InstantiatedTemplate {
        timeline,
        unfilled_slots,
    })
}

/// Validate that all required slots have media assignments
///
/// Returns `Ok(())` if all slots have assignments, or `Err` with a list
/// of unfilled slot labels.
pub fn validate_assignments(
    template: &Template,
    assignments: &MediaAssignments,
) -> Result<(), Vec<String>> {
    let unfilled: Vec<String> = template
        .placeholder_slots
        .iter()
        .filter(|slot| !assignments.contains_key(&slot.id))
        .map(|slot| format!("{} ({})", slot.label, slot.id))
        .collect();

    if unfilled.is_empty() {
        Ok(())
    } else {
        Err(unfilled)
    }
}

/// Derive an asset ID from a media file path.
///
/// This is a simple implementation that uses the file path as the asset ID.
/// In production, this would be resolved by the engine's media asset system.
fn derive_asset_id(file_path: &str) -> String {
    // Use a hash of the path for a consistent ID
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    file_path.hash(&mut hasher);
    format!("asset-{:016x}", hasher.finish())
}

/// Create a new project-ready Timeline from a template with all slots filled.
///
/// This is a convenience function that combines validation and instantiation.
/// If any slots are unfilled, it returns an error with the list.
pub fn create_project_from_template(
    template: &Template,
    assignments: &MediaAssignments,
) -> Result<Timeline, String> {
    // Validate first
    if let Err(unfilled) = validate_assignments(template, assignments) {
        return Err(format!(
            "Cannot create project: {} unfilled slot(s): {}",
            unfilled.len(),
            unfilled.join(", ")
        ));
    }

    let result = instantiate_template(template, assignments)?;
    if !result.unfilled_slots.is_empty() {
        return Err(format!(
            "Internal error: validation passed but {} slots remain unfilled",
            result.unfilled_slots.len()
        ));
    }

    Ok(result.timeline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{PlaceholderMediaType, PlaceholderSlot, Template, TemplateCategory};

    fn create_test_template() -> Template {
        let mut tmpl = Template::new("Test Template", TemplateCategory::Social);
        tmpl.duration_ms = 20000;
        tmpl.aspect_ratio = (9, 16);
        tmpl.placeholder_slots = vec![
            PlaceholderSlot {
                id: "slot-1".to_string(),
                label: "First video".to_string(),
                track_index: 0,
                start_ms: 0,
                expected_duration_ms: 10000,
                media_type: PlaceholderMediaType::Video,
                is_filled: false,
            },
            PlaceholderSlot {
                id: "slot-2".to_string(),
                label: "Second video".to_string(),
                track_index: 0,
                start_ms: 10000,
                expected_duration_ms: 10000,
                media_type: PlaceholderMediaType::Video,
                is_filled: false,
            },
        ];

        // Add tracks to the template timeline
        tmpl.timeline_template
            .add_track(TrackType::Video, Some("Video".to_string()));

        tmpl
    }

    #[test]
    fn test_validate_assignments_all_filled() {
        let tmpl = create_test_template();
        let mut assignments = MediaAssignments::new();
        assignments.insert("slot-1".to_string(), "/path/to/video1.mp4".to_string());
        assignments.insert("slot-2".to_string(), "/path/to/video2.mp4".to_string());

        assert!(validate_assignments(&tmpl, &assignments).is_ok());
    }

    #[test]
    fn test_validate_assignments_missing() {
        let tmpl = create_test_template();
        let mut assignments = MediaAssignments::new();
        assignments.insert("slot-1".to_string(), "/path/to/video1.mp4".to_string());

        let result = validate_assignments(&tmpl, &assignments);
        assert!(result.is_err());
        let unfilled = result.unwrap_err();
        assert_eq!(unfilled.len(), 1);
        assert!(unfilled[0].contains("slot-2"));
    }

    #[test]
    fn test_instantiate_template_all_filled() {
        let tmpl = create_test_template();
        let mut assignments = MediaAssignments::new();
        assignments.insert("slot-1".to_string(), "/path/to/video1.mp4".to_string());
        assignments.insert("slot-2".to_string(), "/path/to/video2.mp4".to_string());

        let result = instantiate_template(&tmpl, &assignments).unwrap();
        assert!(result.unfilled_slots.is_empty());

        // Check that clips were added to the timeline
        let video_track = &result.timeline.tracks[0];
        assert_eq!(video_track.clips.len(), 2);
    }

    #[test]
    fn test_instantiate_template_partial_fill() {
        let tmpl = create_test_template();
        let mut assignments = MediaAssignments::new();
        assignments.insert("slot-1".to_string(), "/path/to/video1.mp4".to_string());

        let result = instantiate_template(&tmpl, &assignments).unwrap();
        assert_eq!(result.unfilled_slots.len(), 1);
        assert_eq!(result.unfilled_slots[0], "slot-2");

        // Still should have 2 clips (one real + one placeholder)
        let video_track = &result.timeline.tracks[0];
        assert_eq!(video_track.clips.len(), 2);
    }

    #[test]
    fn test_instantiate_template_invalid_track_index() {
        let mut tmpl = Template::new("Bad Track", TemplateCategory::Social);
        tmpl.placeholder_slots = vec![PlaceholderSlot {
            id: "slot-1".to_string(),
            label: "Out of range".to_string(),
            track_index: 5, // No such track
            start_ms: 0,
            expected_duration_ms: 5000,
            media_type: PlaceholderMediaType::Video,
            is_filled: false,
        }];
        tmpl.timeline_template
            .add_track(TrackType::Video, Some("Video".to_string()));

        let mut assignments = MediaAssignments::new();
        assignments.insert("slot-1".to_string(), "/path/to/video.mp4".to_string());

        let result = instantiate_template(&tmpl, &assignments);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_project_from_template_success() {
        let tmpl = create_test_template();
        let mut assignments = MediaAssignments::new();
        assignments.insert("slot-1".to_string(), "/path/to/video1.mp4".to_string());
        assignments.insert("slot-2".to_string(), "/path/to/video2.mp4".to_string());

        let timeline = create_project_from_template(&tmpl, &assignments).unwrap();
        assert_eq!(timeline.tracks[0].clips.len(), 2);
    }

    #[test]
    fn test_create_project_from_template_missing_slots() {
        let tmpl = create_test_template();
        let assignments = MediaAssignments::new();

        let result = create_project_from_template(&tmpl, &assignments);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unfilled slot"));
    }

    #[test]
    fn test_derive_asset_id_consistent() {
        let id1 = derive_asset_id("/path/to/video.mp4");
        let id2 = derive_asset_id("/path/to/video.mp4");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_derive_asset_id_different_paths() {
        let id1 = derive_asset_id("/path/to/video1.mp4");
        let id2 = derive_asset_id("/path/to/video2.mp4");
        assert_ne!(id1, id2);
    }
}
