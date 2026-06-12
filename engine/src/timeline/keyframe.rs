//! Keyframe animation system
//!
//! Provides keyframe-based animation for clip properties including
//! position, scale, rotation, and opacity. Supports multiple easing
//! functions and smooth interpolation between keyframes.

use serde::{Deserialize, Serialize};

use super::speed_curve::EasingType;

/// A single keyframe with a time, value, and easing function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    /// Unique identifier
    pub id: String,
    /// Time position in milliseconds
    pub time_ms: u64,
    /// The value at this keyframe
    pub value: f32,
    /// Easing function used to interpolate TO this keyframe from the previous
    pub easing: EasingType,
}

impl Keyframe {
    /// Create a new keyframe
    pub fn new(time_ms: u64, value: f32, easing: EasingType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            time_ms,
            value,
            easing,
        }
    }

    /// Create a linear keyframe
    pub fn linear(time_ms: u64, value: f32) -> Self {
        Self::new(time_ms, value, EasingType::Linear)
    }
}

/// A track of keyframes for an animatable property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeTrack {
    /// The property name this track animates
    pub property: String,
    /// Keyframes sorted by time
    pub keyframes: Vec<Keyframe>,
    /// Default value when no keyframes exist
    pub default_value: f32,
}

impl KeyframeTrack {
    /// Create a new keyframe track for a property
    pub fn new(property: &str, default_value: f32) -> Self {
        Self {
            property: property.to_string(),
            keyframes: Vec::new(),
            default_value,
        }
    }

    /// Add a keyframe. If a keyframe already exists at the same time, it's replaced.
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        // Remove existing keyframe at the same time
        self.keyframes.retain(|k| k.time_ms != keyframe.time_ms);
        self.keyframes.push(keyframe);
        self.sort();
    }

    /// Remove a keyframe by ID
    pub fn remove_keyframe(&mut self, id: &str) -> bool {
        let before = self.keyframes.len();
        self.keyframes.retain(|k| k.id != id);
        self.keyframes.len() < before
    }

    /// Update a keyframe's value and easing
    pub fn update_keyframe(&mut self, id: &str, value: Option<f32>, easing: Option<EasingType>) -> bool {
        if let Some(kf) = self.keyframes.iter_mut().find(|k| k.id == id) {
            if let Some(v) = value {
                kf.value = v;
            }
            if let Some(e) = easing {
                kf.easing = e;
            }
            true
        } else {
            false
        }
    }

    /// Move a keyframe to a new time
    pub fn move_keyframe(&mut self, id: &str, new_time_ms: u64) -> bool {
        if let Some(kf) = self.keyframes.iter_mut().find(|k| k.id == id) {
            kf.time_ms = new_time_ms;
            self.sort();
            true
        } else {
            false
        }
    }

    /// Interpolate the value at a given time
    pub fn interpolate(&self, time_ms: u64) -> f32 {
        if self.keyframes.is_empty() {
            return self.default_value;
        }

        // Before first keyframe
        if time_ms <= self.keyframes[0].time_ms {
            return self.keyframes[0].value;
        }

        // After last keyframe
        if time_ms >= self.keyframes.last().unwrap().time_ms {
            return self.keyframes.last().unwrap().value;
        }

        // Find surrounding keyframes
        let (before_idx, after_idx) = self.find_surrounding(time_ms);
        let before = &self.keyframes[before_idx];
        let after = &self.keyframes[after_idx];

        // Calculate interpolation parameter
        let duration = (after.time_ms - before.time_ms) as f32;
        if duration <= 0.0 {
            return before.value;
        }

        let t = (time_ms - before.time_ms) as f32 / duration;
        let eased_t = before.easing.apply(t);

        // Linear interpolation with eased parameter
        before.value + (after.value - before.value) * eased_t
    }

    /// Find the keyframe indices surrounding the given time
    fn find_surrounding(&self, time_ms: u64) -> (usize, usize) {
        let mut before_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time_ms <= time_ms {
                before_idx = i;
            } else {
                break;
            }
        }
        let after_idx = (before_idx + 1).min(self.keyframes.len() - 1);
        (before_idx, after_idx)
    }

    /// Sort keyframes by time
    fn sort(&mut self) {
        self.keyframes.sort_by_key(|k| k.time_ms);
    }

    /// Get the number of keyframes
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if the track is empty
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Get a keyframe by ID
    pub fn get(&self, id: &str) -> Option<&Keyframe> {
        self.keyframes.iter().find(|k| k.id == id)
    }

    /// Get all keyframe times
    pub fn times(&self) -> Vec<u64> {
        self.keyframes.iter().map(|k| k.time_ms).collect()
    }
}

/// Keyframe tracks for a clip's animatable properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipKeyframes {
    /// Position X keyframes
    pub position_x: KeyframeTrack,
    /// Position Y keyframes
    pub position_y: KeyframeTrack,
    /// Scale keyframes (1.0 = 100%)
    pub scale: KeyframeTrack,
    /// Rotation keyframes (degrees)
    pub rotation: KeyframeTrack,
    /// Opacity keyframes (0.0-1.0)
    pub opacity: KeyframeTrack,
}

impl Default for ClipKeyframes {
    fn default() -> Self {
        Self {
            position_x: KeyframeTrack::new("position_x", 0.0),
            position_y: KeyframeTrack::new("position_y", 0.0),
            scale: KeyframeTrack::new("scale", 1.0),
            rotation: KeyframeTrack::new("rotation", 0.0),
            opacity: KeyframeTrack::new("opacity", 1.0),
        }
    }
}

impl ClipKeyframes {
    /// Create new empty keyframe set
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the interpolated values at a given time
    pub fn interpolate_all(&self, time_ms: u64) -> InterpolatedValues {
        InterpolatedValues {
            position_x: self.position_x.interpolate(time_ms),
            position_y: self.position_y.interpolate(time_ms),
            scale: self.scale.interpolate(time_ms),
            rotation: self.rotation.interpolate(time_ms),
            opacity: self.opacity.interpolate(time_ms),
        }
    }

    /// Check if any track has keyframes
    pub fn has_any_keyframes(&self) -> bool {
        !self.position_x.is_empty()
            || !self.position_y.is_empty()
            || !self.scale.is_empty()
            || !self.rotation.is_empty()
            || !self.opacity.is_empty()
    }

    /// Get the track for a property name
    pub fn track_for(&mut self, property: &str) -> Option<&mut KeyframeTrack> {
        match property {
            "position_x" => Some(&mut self.position_x),
            "position_y" => Some(&mut self.position_y),
            "scale" => Some(&mut self.scale),
            "rotation" => Some(&mut self.rotation),
            "opacity" => Some(&mut self.opacity),
            _ => None,
        }
    }

    /// Get an immutable reference to the track for a property name
    pub fn track_for_ref(&self, property: &str) -> Option<&KeyframeTrack> {
        match property {
            "position_x" => Some(&self.position_x),
            "position_y" => Some(&self.position_y),
            "scale" => Some(&self.scale),
            "rotation" => Some(&self.rotation),
            "opacity" => Some(&self.opacity),
            _ => None,
        }
    }
}

/// Interpolated values from keyframe tracks
#[derive(Debug, Clone, Copy)]
pub struct InterpolatedValues {
    pub position_x: f32,
    pub position_y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
}

/// Supported keyframe property names
pub const KEYFRAME_PROPERTIES: &[&str] = &[
    "position_x",
    "position_y",
    "scale",
    "rotation",
    "opacity",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_track_empty() {
        let track = KeyframeTrack::new("position_x", 0.0);
        assert_eq!(track.interpolate(0), 0.0);
        assert_eq!(track.interpolate(5000), 0.0);
    }

    #[test]
    fn test_keyframe_track_single() {
        let mut track = KeyframeTrack::new("position_x", 0.0);
        track.add_keyframe(Keyframe::linear(0, 100.0));
        assert_eq!(track.interpolate(0), 100.0);
        assert_eq!(track.interpolate(5000), 100.0); // Holds last value
    }

    #[test]
    fn test_keyframe_track_two_linear() {
        let mut track = KeyframeTrack::new("position_x", 0.0);
        track.add_keyframe(Keyframe::linear(0, 0.0));
        track.add_keyframe(Keyframe::linear(1000, 100.0));
        
        assert!((track.interpolate(0) - 0.0).abs() < 0.01);
        assert!((track.interpolate(500) - 50.0).abs() < 0.01);
        assert!((track.interpolate(1000) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_keyframe_track_ease_in() {
        let mut track = KeyframeTrack::new("position_x", 0.0);
        track.add_keyframe(Keyframe::new(0, 0.0, EasingType::EaseIn));
        track.add_keyframe(Keyframe::linear(1000, 100.0));
        
        let mid = track.interpolate(500);
        // Ease-in: mid value should be less than linear 50.0
        assert!(mid < 50.0, "Ease-in at midpoint ({}) should be less than linear 50.0", mid);
    }

    #[test]
    fn test_keyframe_replacement() {
        let mut track = KeyframeTrack::new("position_x", 0.0);
        track.add_keyframe(Keyframe::linear(500, 50.0));
        track.add_keyframe(Keyframe::linear(500, 75.0)); // Replace at same time
        assert_eq!(track.len(), 1);
        assert!((track.interpolate(500) - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_keyframe_remove() {
        let mut track = KeyframeTrack::new("position_x", 0.0);
        let kf = Keyframe::linear(500, 50.0);
        let id = kf.id.clone();
        track.add_keyframe(kf);
        assert!(track.remove_keyframe(&id));
        assert!(track.is_empty());
    }

    #[test]
    fn test_keyframe_update() {
        let mut track = KeyframeTrack::new("position_x", 0.0);
        let kf = Keyframe::linear(500, 50.0);
        let id = kf.id.clone();
        track.add_keyframe(kf);
        track.update_keyframe(&id, Some(75.0), Some(EasingType::EaseOut));
        assert!((track.interpolate(500) - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_clip_keyframes_default() {
        let kf = ClipKeyframes::default();
        let values = kf.interpolate_all(500);
        assert!((values.position_x - 0.0).abs() < 0.01);
        assert!((values.scale - 1.0).abs() < 0.01);
        assert!((values.rotation - 0.0).abs() < 0.01);
        assert!((values.opacity - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_clip_keyframes_position_animation() {
        let mut kf = ClipKeyframes::default();
        kf.position_x.add_keyframe(Keyframe::linear(0, 0.0));
        kf.position_x.add_keyframe(Keyframe::linear(5000, 100.0));
        kf.position_y.add_keyframe(Keyframe::linear(0, 0.0));
        kf.position_y.add_keyframe(Keyframe::linear(5000, 200.0));
        
        let values = kf.interpolate_all(2500);
        assert!((values.position_x - 50.0).abs() < 0.1);
        assert!((values.position_y - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_three_keyframes() {
        let mut track = KeyframeTrack::new("scale", 1.0);
        track.add_keyframe(Keyframe::linear(0, 1.0));
        track.add_keyframe(Keyframe::linear(5000, 2.0));
        track.add_keyframe(Keyframe::linear(10000, 1.0));
        
        assert!((track.interpolate(2500) - 1.5).abs() < 0.01);
        assert!((track.interpolate(7500) - 1.5).abs() < 0.01);
    }
}
