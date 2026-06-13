use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transform properties for a clip on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipTransform {
    pub x: f64,
    pub y: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
    pub opacity: f64,
    pub anchor_x: f64,
    pub anchor_y: f64,
}

impl Default for ClipTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            opacity: 1.0,
            anchor_x: 0.5,
            anchor_y: 0.5,
        }
    }
}

impl ClipTransform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_scale(mut self, sx: f64, sy: f64) -> Self {
        self.scale_x = sx;
        self.scale_y = sy;
        self
    }

    pub fn with_rotation(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

/// A clip on the timeline representing a segment of source media.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: Uuid,
    pub source_path: String,
    pub trim_in: f64,
    pub trim_out: f64,
    pub speed: f64,
    pub transform: ClipTransform,
    pub effects: Vec<String>,
    pub filters: Vec<String>,
    pub volume: f64,
    pub pan: f64,
    pub muted: bool,
    pub locked: bool,
}

impl Default for Clip {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            source_path: String::new(),
            trim_in: 0.0,
            trim_out: 10.0,
            speed: 1.0,
            transform: ClipTransform::default(),
            effects: Vec::new(),
            filters: Vec::new(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            locked: false,
        }
    }
}

impl Clip {
    /// Create a new clip from a source file with default trim (0 to 10 seconds).
    pub fn new(source_path: &str) -> Self {
        Self {
            source_path: source_path.to_string(),
            ..Self::default()
        }
    }

    /// Create a clip with specific trim range.
    pub fn with_trim(source_path: &str, trim_in: f64, trim_out: f64) -> Self {
        Self {
            source_path: source_path.to_string(),
            trim_in,
            trim_out,
            ..Self::default()
        }
    }

    /// Get the duration of this clip on the timeline (accounting for speed).
    pub fn get_duration(&self) -> f64 {
        if self.speed.abs() < f64::EPSILON {
            0.0
        } else {
            (self.trim_out - self.trim_in) / self.speed.abs()
        }
    }

    /// Get the source duration (trim_out - trim_in, before speed adjustment).
    pub fn get_source_duration(&self) -> f64 {
        (self.trim_out - self.trim_in).max(0.0)
    }

    /// Apply speed/time remapping: given a timeline time offset, return the source time.
    pub fn apply_speed(&self, time: f64) -> f64 {
        self.trim_in + time * self.speed
    }

    /// Reverse time remapping: given a source time, return the timeline time.
    pub fn inverse_speed(&self, source_time: f64) -> f64 {
        if self.speed.abs() < f64::EPSILON {
            0.0
        } else {
            (source_time - self.trim_in) / self.speed
        }
    }

    /// Split the clip at the given timeline time offset.
    /// Returns two clips: the left portion and the right portion.
    pub fn split_at(&self, time: f64) -> Result<(Clip, Clip), String> {
        if time <= 0.0 || time >= self.get_duration() {
            return Err(format!(
                "Split time {} is out of clip duration range (0, {})",
                time,
                self.get_duration()
            ));
        }

        let source_split = self.apply_speed(time);

        let mut left = self.clone();
        left.id = Uuid::new_v4();
        left.trim_out = source_split;

        let mut right = self.clone();
        right.id = Uuid::new_v4();
        right.trim_in = source_split;

        Ok((left, right))
    }

    /// Check if the clip contains the given timeline time.
    pub fn contains_time(&self, time: f64) -> bool {
        time >= 0.0 && time < self.get_duration()
    }

    /// Set the playback speed (time remapping).
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.max(0.01); // Prevent zero/negative speed
    }

    /// Set volume (0.0 to 2.0).
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 2.0);
    }

    /// Set pan (-1.0 left to 1.0 right).
    pub fn set_pan(&mut self, pan: f64) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Add an effect by name.
    pub fn add_effect(&mut self, effect_name: &str) {
        if !self.effects.contains(&effect_name.to_string()) {
            self.effects.push(effect_name.to_string());
        }
    }

    /// Remove an effect by name.
    pub fn remove_effect(&mut self, effect_name: &str) {
        self.effects.retain(|e| e != effect_name);
    }

    /// Add a filter by name.
    pub fn add_filter(&mut self, filter_name: &str) {
        if !self.filters.contains(&filter_name.to_string()) {
            self.filters.push(filter_name.to_string());
        }
    }

    /// Remove a filter by name.
    pub fn remove_filter(&mut self, filter_name: &str) {
        self.filters.retain(|f| f != filter_name);
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_new() {
        let clip = Clip::new("video.mp4");
        assert_eq!(clip.source_path, "video.mp4");
        assert_eq!(clip.trim_in, 0.0);
        assert_eq!(clip.trim_out, 10.0);
        assert_eq!(clip.speed, 1.0);
    }

    #[test]
    fn test_clip_with_trim() {
        let clip = Clip::with_trim("video.mp4", 5.0, 15.0);
        assert_eq!(clip.trim_in, 5.0);
        assert_eq!(clip.trim_out, 15.0);
    }

    #[test]
    fn test_clip_get_duration() {
        let clip = Clip::with_trim("video.mp4", 5.0, 15.0);
        assert!((clip.get_duration() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_get_duration_with_speed() {
        let mut clip = Clip::with_trim("video.mp4", 0.0, 10.0);
        clip.speed = 2.0;
        assert!((clip.get_duration() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_get_source_duration() {
        let clip = Clip::with_trim("video.mp4", 3.0, 13.0);
        assert!((clip.get_source_duration() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_apply_speed() {
        let clip = Clip::with_trim("video.mp4", 5.0, 15.0);
        // At timeline time 0, source time should be 5.0
        assert!((clip.apply_speed(0.0) - 5.0).abs() < 1e-9);
        // At timeline time 5, source time should be 10.0
        assert!((clip.apply_speed(5.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_apply_speed_2x() {
        let mut clip = Clip::with_trim("video.mp4", 0.0, 10.0);
        clip.speed = 2.0;
        // At timeline time 0, source time = 0 + 0*2 = 0
        assert!((clip.apply_speed(0.0)).abs() < 1e-9);
        // At timeline time 2, source time = 0 + 2*2 = 4
        assert!((clip.apply_speed(2.0) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_split_at() {
        let clip = Clip::with_trim("video.mp4", 0.0, 10.0);
        let (left, right) = clip.split_at(5.0).unwrap();
        assert!((left.trim_out - 5.0).abs() < 1e-9);
        assert!((left.trim_in).abs() < 1e-9);
        assert!((right.trim_in - 5.0).abs() < 1e-9);
        assert!((right.trim_out - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_split_at_boundary() {
        let clip = Clip::with_trim("video.mp4", 0.0, 10.0);
        assert!(clip.split_at(0.0).is_err());
        assert!(clip.split_at(10.0).is_err());
    }

    #[test]
    fn test_clip_set_speed() {
        let mut clip = Clip::new("video.mp4");
        clip.set_speed(0.5);
        assert!((clip.speed - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_clip_set_volume_pan() {
        let mut clip = Clip::new("video.mp4");
        clip.set_volume(1.5);
        clip.set_pan(-0.5);
        assert!((clip.volume - 1.5).abs() < 1e-9);
        assert!((clip.pan - (-0.5)).abs() < 1e-9);
    }

    #[test]
    fn test_clip_effects() {
        let mut clip = Clip::new("video.mp4");
        clip.add_effect("blur");
        clip.add_effect("sharpen");
        assert_eq!(clip.effects.len(), 2);
        clip.add_effect("blur"); // duplicate, shouldn't add
        assert_eq!(clip.effects.len(), 2);
        clip.remove_effect("blur");
        assert_eq!(clip.effects.len(), 1);
        assert_eq!(clip.effects[0], "sharpen");
    }

    #[test]
    fn test_clip_transform_default() {
        let t = ClipTransform::default();
        assert!((t.scale_x - 1.0).abs() < 1e-9);
        assert!((t.opacity - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_clip_unique_ids() {
        let c1 = Clip::new("a.mp4");
        let c2 = Clip::new("a.mp4");
        assert_ne!(c1.id, c2.id);
    }
}
