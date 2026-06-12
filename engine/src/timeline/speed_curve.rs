//! Speed curve for variable playback speed within a clip
//!
//! Supports multi-segment speed curves with different easing functions,
//! allowing smooth speed ramps (e.g., slow-motion to normal to fast-forward).

use serde::{Deserialize, Serialize};

/// Easing function type for speed transitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EasingType {
    /// Constant speed (no interpolation)
    Linear,
    /// Gradually speeds up
    EaseIn,
    /// Gradually slows down
    EaseOut,
    /// Speeds up then slows down
    EaseInOut,
    /// Custom cubic bezier (control points stored in segment)
    CubicBezier,
}

impl Default for EasingType {
    fn default() -> Self {
        EasingType::Linear
    }
}

impl EasingType {
    /// Apply easing function to a normalized time value t (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t * t,
            EasingType::EaseOut => 1.0 - (1.0 - t).powi(3),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingType::CubicBezier => {
                // Approximate cubic bezier with ease-in-out for now
                // Full implementation would use bezier control points
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }

    /// Get all easing type variants
    pub fn all() -> Vec<EasingType> {
        vec![
            EasingType::Linear,
            EasingType::EaseIn,
            EasingType::EaseOut,
            EasingType::EaseInOut,
            EasingType::CubicBezier,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            EasingType::Linear => "Linear",
            EasingType::EaseIn => "Ease In",
            EasingType::EaseOut => "Ease Out",
            EasingType::EaseInOut => "Ease In-Out",
            EasingType::CubicBezier => "Cubic Bezier",
        }
    }

    /// Parse an easing type from its display name (case-insensitive)
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "linear" => Some(EasingType::Linear),
            "ease in" | "easein" => Some(EasingType::EaseIn),
            "ease out" | "easeout" => Some(EasingType::EaseOut),
            "ease in-out" | "easeinout" | "ease in out" => Some(EasingType::EaseInOut),
            "cubic bezier" | "cubicbezier" => Some(EasingType::CubicBezier),
            _ => None,
        }
    }
}

/// A single segment of a speed curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSegment {
    /// Start time of this segment in milliseconds
    pub start_ms: u64,
    /// End time of this segment in milliseconds
    pub end_ms: u64,
    /// Speed at the start of this segment (1.0 = normal)
    pub start_speed: f32,
    /// Speed at the end of this segment (1.0 = normal)
    pub end_speed: f32,
    /// Easing function for this segment
    pub easing: EasingType,
}

impl SpeedSegment {
    /// Create a new speed segment
    pub fn new(start_ms: u64, end_ms: u64, start_speed: f32, end_speed: f32, easing: EasingType) -> Self {
        Self {
            start_ms,
            end_ms,
            start_speed,
            end_speed,
            easing,
        }
    }

    /// Create a constant-speed segment
    pub fn constant(start_ms: u64, end_ms: u64, speed: f32) -> Self {
        Self {
            start_ms,
            end_ms,
            start_speed: speed,
            end_speed: speed,
            easing: EasingType::Linear,
        }
    }
}

/// A speed curve composed of multiple segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedCurve {
    /// The segments that make up this curve
    pub segments: Vec<SpeedSegment>,
}

impl Default for SpeedCurve {
    fn default() -> Self {
        Self::constant(1.0)
    }
}

impl SpeedCurve {
    /// Create a constant-speed curve
    pub fn constant(speed: f32) -> Self {
        Self {
            segments: vec![SpeedSegment::constant(0, u64::MAX, speed)],
        }
    }

    /// Create a linear ramp from one speed to another
    pub fn ramp(start_speed: f32, end_speed: f32, duration_ms: u64, easing: EasingType) -> Self {
        Self {
            segments: vec![SpeedSegment::new(
                0,
                duration_ms,
                start_speed,
                end_speed,
                easing,
            )],
        }
    }

    /// Evaluate the speed at a given time
    pub fn evaluate_speed_at(&self, time_ms: u64) -> f32 {
        for segment in &self.segments {
            if time_ms >= segment.start_ms && time_ms < segment.end_ms {
                let duration = (segment.end_ms - segment.start_ms) as f32;
                if duration <= 0.0 {
                    return segment.start_speed;
                }
                let t = (time_ms - segment.start_ms) as f32 / duration;
                let eased_t = segment.easing.apply(t);
                return segment.start_speed + (segment.end_speed - segment.start_speed) * eased_t;
            }
        }
        // Default to the last segment's end speed, or 1.0 if no segments
        self.segments
            .last()
            .map(|s| s.end_speed)
            .unwrap_or(1.0)
    }

    /// Calculate the total duration of the curve, accounting for speed changes.
    /// This converts "source time" to "display time" by integrating the speed.
    ///
    /// For example, 1000ms at 0.5x speed = 2000ms display time
    /// 1000ms at 2.0x speed = 500ms display time
    pub fn source_to_display_time(&self, source_ms: u64) -> u64 {
        let speed = self.evaluate_speed_at(source_ms);
        if speed <= 0.0 {
            return source_ms;
        }
        (source_ms as f64 / speed as f64) as u64
    }

    /// Add a segment to the curve
    pub fn add_segment(&mut self, segment: SpeedSegment) {
        self.segments.push(segment);
        self.segments.sort_by_key(|s| s.start_ms);
    }

    /// Remove overlapping segments and insert a new one
    pub fn set_segment(&mut self, segment: SpeedSegment) {
        // Remove segments that overlap with the new one
        self.segments.retain(|s| {
            s.end_ms <= segment.start_ms || s.start_ms >= segment.end_ms
        });
        self.segments.push(segment);
        self.segments.sort_by_key(|s| s.start_ms);
    }

    /// Get the number of segments
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Check if this is a constant-speed curve
    pub fn is_constant(&self) -> bool {
        if self.segments.len() <= 1 {
            return true;
        }
        let speed = self.segments[0].start_speed;
        self.segments.iter().all(|s| {
            (s.start_speed - speed).abs() < 0.001 && (s.end_speed - speed).abs() < 0.001
        })
    }

    /// Get the overall speed factor (average)
    pub fn average_speed(&self) -> f32 {
        if self.segments.is_empty() {
            return 1.0;
        }
        let total: f32 = self.segments.iter().map(|s| (s.start_speed + s.end_speed) / 2.0).sum();
        total / self.segments.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_speed() {
        let curve = SpeedCurve::constant(1.0);
        assert_eq!(curve.evaluate_speed_at(0), 1.0);
        assert_eq!(curve.evaluate_speed_at(5000), 1.0);
    }

    #[test]
    fn test_half_speed() {
        let curve = SpeedCurve::constant(0.5);
        assert_eq!(curve.evaluate_speed_at(0), 0.5);
        assert_eq!(curve.evaluate_speed_at(5000), 0.5);
    }

    #[test]
    fn test_speed_ramp_linear() {
        let curve = SpeedCurve::ramp(1.0, 2.0, 1000, EasingType::Linear);
        assert!((curve.evaluate_speed_at(0) - 1.0).abs() < 0.01);
        assert!((curve.evaluate_speed_at(500) - 1.5).abs() < 0.01);
        assert!((curve.evaluate_speed_at(999) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_speed_ramp_ease_in() {
        let curve = SpeedCurve::ramp(0.5, 1.5, 1000, EasingType::EaseIn);
        // At t=0, speed should be 0.5 (start)
        assert!((curve.evaluate_speed_at(0) - 0.5).abs() < 0.01);
        // At t=500, ease-in should be slower than linear
        let speed_at_mid = curve.evaluate_speed_at(500);
        let linear_at_mid = 1.0; // 0.5 + (1.5 - 0.5) * 0.5
        assert!(speed_at_mid < linear_at_mid, "Ease-in should be slower at midpoint");
    }

    #[test]
    fn test_easing_linear() {
        assert!((EasingType::Linear.apply(0.0) - 0.0).abs() < 0.001);
        assert!((EasingType::Linear.apply(0.5) - 0.5).abs() < 0.001);
        assert!((EasingType::Linear.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_functions_range() {
        for easing in EasingType::all() {
            let start = easing.apply(0.0);
            let end = easing.apply(1.0);
            assert!((start - 0.0).abs() < 0.01, "Easing {:?} start should be 0", easing);
            assert!((end - 1.0).abs() < 0.01, "Easing {:?} end should be 1", easing);
        }
    }

    #[test]
    fn test_is_constant() {
        assert!(SpeedCurve::constant(1.0).is_constant());
        assert!(SpeedCurve::constant(0.5).is_constant());
        assert!(!SpeedCurve::ramp(1.0, 2.0, 1000, EasingType::Linear).is_constant());
    }

    #[test]
    fn test_add_segment() {
        let mut curve = SpeedCurve::constant(1.0);
        curve.add_segment(SpeedSegment::new(1000, 2000, 1.0, 2.0, EasingType::Linear));
        assert_eq!(curve.segment_count(), 2);
    }

    #[test]
    fn test_source_to_display_time() {
        let curve = SpeedCurve::constant(2.0);
        // 1000ms source at 2x speed = 500ms display time
        assert_eq!(curve.source_to_display_time(1000), 500);
        
        let half_curve = SpeedCurve::constant(0.5);
        // 1000ms source at 0.5x speed = 2000ms display time
        assert_eq!(half_curve.source_to_display_time(1000), 2000);
    }
}
