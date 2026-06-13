use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::clip::Clip;

/// The type of a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    Video,
    Audio,
    Subtitle,
}

impl std::fmt::Display for TrackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackType::Video => write!(f, "Video"),
            TrackType::Audio => write!(f, "Audio"),
            TrackType::Subtitle => write!(f, "Subtitle"),
        }
    }
}

/// A track on the timeline containing clips of the same type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub track_type: TrackType,
    pub clips: Vec<Clip>,
    pub muted: bool,
    pub solo: bool,
    pub locked: bool,
    pub volume: f64,
    pub pan: f64,
}

impl Track {
    /// Create a new track with the given name and type.
    pub fn new(name: &str, track_type: TrackType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            track_type,
            clips: Vec::new(),
            muted: false,
            solo: false,
            locked: false,
            volume: 1.0,
            pan: 0.0,
        }
    }

    /// Add a clip to the end of the track.
    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
    }

    /// Remove a clip at the given index.
    pub fn remove_clip(&mut self, index: usize) -> Result<Clip, String> {
        if index >= self.clips.len() {
            return Err(format!(
                "Clip index {} out of bounds (len={})",
                index,
                self.clips.len()
            ));
        }
        Ok(self.clips.remove(index))
    }

    /// Move a clip from one index to another.
    pub fn move_clip(&mut self, from_idx: usize, to_idx: usize) -> Result<(), String> {
        if from_idx >= self.clips.len() || to_idx >= self.clips.len() {
            return Err(format!(
                "Clip index out of bounds: from={}, to={}, len={}",
                from_idx,
                to_idx,
                self.clips.len()
            ));
        }
        let clip = self.clips.remove(from_idx);
        self.clips.insert(to_idx, clip);
        Ok(())
    }

    /// Reorder clips by sorting them by their start time.
    pub fn reorder_clips(&mut self) {
        // In a real implementation, clips would have timeline positions.
        // For now, this is a no-op since clips are stored in order.
    }

    /// Get the total duration of all clips on this track.
    pub fn get_total_duration(&self) -> f64 {
        self.clips.iter().map(|c| c.get_duration()).sum()
    }

    /// Find a clip at the given time offset from the track start.
    pub fn find_clip_at_time(&self, time: f64) -> Option<&Clip> {
        let mut accumulated = 0.0;
        for clip in &self.clips {
            let clip_dur = clip.get_duration();
            if time >= accumulated && time < accumulated + clip_dur {
                return Some(clip);
            }
            accumulated += clip_dur;
        }
        None
    }

    /// Find the index of a clip by its ID.
    pub fn find_clip_index_by_id(&self, clip_id: &uuid::Uuid) -> Option<usize> {
        self.clips.iter().position(|c| &c.id == clip_id)
    }

    /// Get the number of clips.
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Check if the track is audible (not muted, and either solo or no track is solo).
    pub fn is_audible(&self, any_solo: bool) -> bool {
        if self.muted {
            return false;
        }
        if any_solo && !self.solo {
            return false;
        }
        true
    }

    /// Set the track volume (0.0 to 2.0).
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 2.0);
    }

    /// Set the track pan (-1.0 to 1.0).
    pub fn set_pan(&mut self, pan: f64) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Toggle mute.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Toggle solo.
    pub fn toggle_solo(&mut self) {
        self.solo = !self.solo;
    }

    /// Toggle lock.
    pub fn toggle_lock(&mut self) {
        self.locked = !self.locked;
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_new() {
        let track = Track::new("Video 1", TrackType::Video);
        assert_eq!(track.name, "Video 1");
        assert_eq!(track.track_type, TrackType::Video);
        assert!(track.clips.is_empty());
        assert!(!track.muted);
        assert!(!track.solo);
        assert!(!track.locked);
    }

    #[test]
    fn test_track_add_clip() {
        let mut track = Track::new("V1", TrackType::Video);
        track.add_clip(Clip::new("a.mp4"));
        track.add_clip(Clip::new("b.mp4"));
        assert_eq!(track.clip_count(), 2);
    }

    #[test]
    fn test_track_remove_clip() {
        let mut track = Track::new("V1", TrackType::Video);
        track.add_clip(Clip::new("a.mp4"));
        track.add_clip(Clip::new("b.mp4"));
        let removed = track.remove_clip(0);
        assert!(removed.is_ok());
        assert_eq!(track.clip_count(), 1);
        assert_eq!(track.clips[0].source_path, "b.mp4");
    }

    #[test]
    fn test_track_remove_clip_out_of_bounds() {
        let mut track = Track::new("V1", TrackType::Video);
        let result = track.remove_clip(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_track_move_clip() {
        let mut track = Track::new("V1", TrackType::Video);
        track.add_clip(Clip::new("a.mp4"));
        track.add_clip(Clip::new("b.mp4"));
        track.add_clip(Clip::new("c.mp4"));
        track.move_clip(0, 2).unwrap();
        assert_eq!(track.clips[0].source_path, "b.mp4");
        assert_eq!(track.clips[1].source_path, "c.mp4");
        assert_eq!(track.clips[2].source_path, "a.mp4");
    }

    #[test]
    fn test_track_total_duration() {
        let mut track = Track::new("V1", TrackType::Video);
        track.add_clip(Clip::with_trim("a.mp4", 0.0, 5.0));
        track.add_clip(Clip::with_trim("b.mp4", 0.0, 10.0));
        assert!((track.get_total_duration() - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_track_find_clip_at_time() {
        let mut track = Track::new("V1", TrackType::Video);
        track.add_clip(Clip::with_trim("a.mp4", 0.0, 5.0));
        track.add_clip(Clip::with_trim("b.mp4", 0.0, 10.0));
        let clip = track.find_clip_at_time(3.0);
        assert!(clip.is_some());
        assert_eq!(clip.unwrap().source_path, "a.mp4");

        let clip = track.find_clip_at_time(7.0);
        assert!(clip.is_some());
        assert_eq!(clip.unwrap().source_path, "b.mp4");
    }

    #[test]
    fn test_track_find_clip_at_time_empty() {
        let track = Track::new("V1", TrackType::Video);
        assert!(track.find_clip_at_time(5.0).is_none());
    }

    #[test]
    fn test_track_is_audible() {
        let mut track = Track::new("A1", TrackType::Audio);
        assert!(track.is_audible(false));
        track.muted = true;
        assert!(!track.is_audible(false));
        track.muted = false;
        track.solo = true;
        assert!(track.is_audible(true));
    }

    #[test]
    fn test_track_toggle() {
        let mut track = Track::new("V1", TrackType::Video);
        track.toggle_mute();
        assert!(track.muted);
        track.toggle_mute();
        assert!(!track.muted);
        track.toggle_solo();
        assert!(track.solo);
        track.toggle_lock();
        assert!(track.locked);
    }
}
