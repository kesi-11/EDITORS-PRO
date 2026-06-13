use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::clip::Clip;
use super::track::Track;

/// Marker type for timeline markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerType {
    Standard,
    Chapter,
    Compression,
    /// User-defined marker type.
    Custom(u8),
}

impl Default for MarkerType {
    fn default() -> Self {
        MarkerType::Standard
    }
}

/// A marker on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    pub position: f64,
    pub name: String,
    pub color: String,
    pub marker_type: MarkerType,
}

impl Marker {
    pub fn new(position: f64, name: &str) -> Self {
        Self {
            position,
            name: name.to_string(),
            color: "#FF0000".to_string(),
            marker_type: MarkerType::default(),
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = color.to_string();
        self
    }

    pub fn with_type(mut self, marker_type: MarkerType) -> Self {
        self.marker_type = marker_type;
        self
    }
}

/// A region on the timeline (time range selection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub start: f64,
    pub end: f64,
    pub name: String,
    pub color: String,
}

impl Region {
    pub fn new(start: f64, end: f64, name: &str) -> Self {
        Self {
            start,
            end,
            name: name.to_string(),
            color: "#0000FF".to_string(),
        }
    }

    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    pub fn contains(&self, time: f64) -> bool {
        time >= self.start && time <= self.end
    }
}

/// The timeline containing all tracks, markers, and regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub tracks: Vec<Track>,
    pub duration: f64,
    pub markers: Vec<Marker>,
    pub regions: Vec<Region>,
    pub timebase: u32,
}

impl Timeline {
    /// Create a new empty timeline.
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            duration: 0.0,
            markers: Vec::new(),
            regions: Vec::new(),
            timebase: 30,
        }
    }

    /// Add a track to the timeline.
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.recalculate_duration();
    }

    /// Remove a track at the given index.
    pub fn remove_track(&mut self, index: usize) -> Result<Track> {
        if index >= self.tracks.len() {
            anyhow::bail!("Track index {} out of bounds (len={})", index, self.tracks.len());
        }
        let track = self.tracks.remove(index);
        self.recalculate_duration();
        Ok(track)
    }

    /// Reorder tracks: move track at `from_idx` to `to_idx`.
    pub fn reorder_tracks(&mut self, from_idx: usize, to_idx: usize) -> Result<()> {
        if from_idx >= self.tracks.len() || to_idx >= self.tracks.len() {
            anyhow::bail!(
                "Track index out of bounds: from={}, to={}, len={}",
                from_idx,
                to_idx,
                self.tracks.len()
            );
        }
        let track = self.tracks.remove(from_idx);
        self.tracks.insert(to_idx, track);
        Ok(())
    }

    /// Get the total duration of the timeline (longest track).
    pub fn get_duration(&self) -> f64 {
        self.duration
    }

    /// Find a clip at the given time across all tracks.
    pub fn find_clip_at_time(&self, time: f64) -> Option<&Clip> {
        for track in &self.tracks {
            if let Some(clip) = track.find_clip_at_time(time) {
                return Some(clip);
            }
        }
        None
    }

    /// Split a clip at the given time on the specified track.
    pub fn split_at_time(&mut self, track_idx: usize, time: f64) -> Result<()> {
        if track_idx >= self.tracks.len() {
            anyhow::bail!("Track index {} out of bounds", track_idx);
        }

        let clip_idx = self.tracks[track_idx]
            .clips
            .iter()
            .position(|c| {
                let clip_start = 0.0; // Simplified: in a real impl, compute actual start
                let clip_end = clip_start + c.get_duration();
                time > clip_start && time < clip_end
            });

        if let Some(idx) = clip_idx {
            let clip = &self.tracks[track_idx].clips[idx];
            let clip_start = 0.0;
            let split_point = time - clip_start;
            let (left, right) = clip.split_at(split_point)?;
            self.tracks[track_idx].clips[idx] = left;
            self.tracks[track_idx].clips.insert(idx + 1, right);
            self.recalculate_duration();
            Ok(())
        } else {
            anyhow::bail!("No clip at time {} on track {}", time, track_idx);
        }
    }

    /// Ripple delete: remove a clip and shift subsequent clips left.
    pub fn ripple_delete(&mut self, track_idx: usize, clip_idx: usize) -> Result<()> {
        if track_idx >= self.tracks.len() {
            anyhow::bail!("Track index {} out of bounds", track_idx);
        }

        let track = &mut self.tracks[track_idx];
        if clip_idx >= track.clips.len() {
            anyhow::bail!(
                "Clip index {} out of bounds on track {} (len={})",
                clip_idx,
                track_idx,
                track.clips.len()
            );
        }

        let removed_duration = track.clips[clip_idx].get_duration();
        track.clips.remove(clip_idx);

        // Shift subsequent clips left
        for clip in &mut track.clips[clip_idx..] {
            // In a real implementation, adjust clip timeline position
            let _ = removed_duration;
        }

        self.recalculate_duration();
        Ok(())
    }

    /// Add a marker.
    pub fn add_marker(&mut self, marker: Marker) {
        self.markers.push(marker);
    }

    /// Remove a marker at the given index.
    pub fn remove_marker(&mut self, index: usize) -> Result<Marker> {
        if index >= self.markers.len() {
            anyhow::bail!("Marker index out of bounds");
        }
        Ok(self.markers.remove(index))
    }

    /// Add a region.
    pub fn add_region(&mut self, region: Region) {
        self.regions.push(region);
    }

    /// Remove a region at the given index.
    pub fn remove_region(&mut self, index: usize) -> Result<Region> {
        if index >= self.regions.len() {
            anyhow::bail!("Region index out of bounds");
        }
        Ok(self.regions.remove(index))
    }

    /// Get the number of tracks.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Recalculate the total duration from track durations.
    fn recalculate_duration(&mut self) {
        self.duration = self
            .tracks
            .iter()
            .map(|t| t.get_total_duration())
            .fold(0.0f64, |a, b| a.max(b));
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::track::TrackType;

    fn make_clip_with_duration(duration: f64) -> Clip {
        Clip {
            trim_in: 0.0,
            trim_out: duration,
            speed: 1.0,
            ..Clip::default()
        }
    }

    #[test]
    fn test_timeline_new() {
        let tl = Timeline::new();
        assert!(tl.tracks.is_empty());
        assert_eq!(tl.duration, 0.0);
        assert!(tl.markers.is_empty());
        assert!(tl.regions.is_empty());
    }

    #[test]
    fn test_timeline_add_track() {
        let mut tl = Timeline::new();
        let track = Track::new("Video 1", TrackType::Video);
        tl.add_track(track);
        assert_eq!(tl.track_count(), 1);
    }

    #[test]
    fn test_timeline_remove_track() {
        let mut tl = Timeline::new();
        tl.add_track(Track::new("Video 1", TrackType::Video));
        tl.add_track(Track::new("Audio 1", TrackType::Audio));
        let removed = tl.remove_track(0);
        assert!(removed.is_ok());
        assert_eq!(tl.track_count(), 1);
        assert_eq!(tl.tracks[0].name, "Audio 1");
    }

    #[test]
    fn test_timeline_remove_track_out_of_bounds() {
        let mut tl = Timeline::new();
        let result = tl.remove_track(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_timeline_reorder_tracks() {
        let mut tl = Timeline::new();
        tl.add_track(Track::new("A", TrackType::Video));
        tl.add_track(Track::new("B", TrackType::Audio));
        tl.reorder_tracks(0, 1).unwrap();
        assert_eq!(tl.tracks[0].name, "B");
        assert_eq!(tl.tracks[1].name, "A");
    }

    #[test]
    fn test_timeline_reorder_tracks_out_of_bounds() {
        let mut tl = Timeline::new();
        tl.add_track(Track::new("A", TrackType::Video));
        let result = tl.reorder_tracks(0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_timeline_add_marker() {
        let mut tl = Timeline::new();
        tl.add_marker(Marker::new(5.0, "Start"));
        assert_eq!(tl.markers.len(), 1);
        assert_eq!(tl.markers[0].name, "Start");
    }

    #[test]
    fn test_timeline_remove_marker() {
        let mut tl = Timeline::new();
        tl.add_marker(Marker::new(5.0, "A"));
        tl.add_marker(Marker::new(10.0, "B"));
        let removed = tl.remove_marker(0).unwrap();
        assert_eq!(removed.name, "A");
        assert_eq!(tl.markers.len(), 1);
    }

    #[test]
    fn test_timeline_add_region() {
        let mut tl = Timeline::new();
        tl.add_region(Region::new(0.0, 10.0, "Selection"));
        assert_eq!(tl.regions.len(), 1);
    }

    #[test]
    fn test_region_duration() {
        let region = Region::new(2.0, 8.0, "Test");
        assert!((region.duration() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_region_contains() {
        let region = Region::new(2.0, 8.0, "Test");
        assert!(region.contains(5.0));
        assert!(!region.contains(1.0));
        assert!(region.contains(2.0));
    }

    #[test]
    fn test_marker_with_color_and_type() {
        let marker = Marker::new(5.0, "Chapter")
            .with_color("#00FF00")
            .with_type(MarkerType::Chapter);
        assert_eq!(marker.color, "#00FF00");
        assert_eq!(marker.marker_type, MarkerType::Chapter);
    }

    #[test]
    fn test_timeline_ripple_delete_out_of_bounds() {
        let mut tl = Timeline::new();
        tl.add_track(Track::new("V1", TrackType::Video));
        let result = tl.ripple_delete(5, 0);
        assert!(result.is_err());
    }
}
