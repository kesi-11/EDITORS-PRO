use crate::utils::math::{
    ease_in_back, ease_in_bounce, ease_in_circ, ease_in_cubic, ease_in_elastic, ease_in_expo,
    ease_in_out_back, ease_in_out_bounce, ease_in_out_circ, ease_in_out_cubic, ease_in_out_elastic,
    ease_in_out_expo, ease_in_out_quad, ease_in_out_quart, ease_in_out_quint, ease_in_out_sine,
    ease_in_quad, ease_in_quart, ease_in_quint, ease_in_sine, ease_out_back, ease_out_bounce,
    ease_out_circ, ease_out_cubic, ease_out_elastic, ease_out_expo, ease_out_quad, ease_out_quart,
    ease_out_quint, ease_out_sine, bezier_cubic, lerp, smoothstep, Vector2f, Vector3f, Vector4f,
};
use serde::{Deserialize, Serialize};

// ─── Interpolation types ─────────────────────────────────────────────────────

/// The type of interpolation between keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    Bezier,
    Hold,
    SmoothStep,
    Cubic,
    Elastic,
    Bounce,
    Back,
    Expo,
    Circ,
    Constant,
}

impl Default for InterpolationType {
    fn default() -> Self {
        InterpolationType::Linear
    }
}

// ─── Interpolate trait ───────────────────────────────────────────────────────

/// Trait for values that can be interpolated between two instances.
pub trait Interpolate: Clone {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for f64 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl Interpolate for Vector2f {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Vector2f::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl Interpolate for Vector3f {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Vector3f::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }
}

impl Interpolate for Vector4f {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Vector4f::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
            self.w + (other.w - self.w) * t,
        )
    }
}

// ─── Keyframe ────────────────────────────────────────────────────────────────

/// A single keyframe with time, value, and interpolation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T: Clone + Interpolate> {
    pub time: f64,
    pub value: T,
    pub interpolation_type: InterpolationType,
    pub bezier_in_handle: Vector2f,
    pub bezier_out_handle: Vector2f,
}

impl<T: Clone + Interpolate> Keyframe<T> {
    pub fn new(time: f64, value: T) -> Self {
        Self {
            time,
            value,
            interpolation_type: InterpolationType::Linear,
            bezier_in_handle: Vector2f::new(0.33, 0.0),
            bezier_out_handle: Vector2f::new(0.66, 1.0),
        }
    }

    pub fn with_interpolation(mut self, itype: InterpolationType) -> Self {
        self.interpolation_type = itype;
        self
    }

    pub fn with_bezier_handles(mut self, in_handle: Vector2f, out_handle: Vector2f) -> Self {
        self.bezier_in_handle = in_handle;
        self.bezier_out_handle = out_handle;
        self
    }
}

// ─── Easing curve ────────────────────────────────────────────────────────────

/// Named easing curves with custom Bezier support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingCurve {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    CustomBezier { p1: Vector2f, p2: Vector2f },
}

impl Default for EasingCurve {
    fn default() -> Self {
        EasingCurve::Linear
    }
}

impl EasingCurve {
    /// Evaluate the easing curve at parameter t (0.0 to 1.0).
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingCurve::Linear => t,
            EasingCurve::EaseInQuad => ease_in_quad(t as f64) as f32,
            EasingCurve::EaseOutQuad => ease_out_quad(t as f64) as f32,
            EasingCurve::EaseInOutQuad => ease_in_out_quad(t as f64) as f32,
            EasingCurve::EaseInCubic => ease_in_cubic(t as f64) as f32,
            EasingCurve::EaseOutCubic => ease_out_cubic(t as f64) as f32,
            EasingCurve::EaseInOutCubic => ease_in_out_cubic(t as f64) as f32,
            EasingCurve::EaseInQuart => ease_in_quart(t as f64) as f32,
            EasingCurve::EaseOutQuart => ease_out_quart(t as f64) as f32,
            EasingCurve::EaseInOutQuart => ease_in_out_quart(t as f64) as f32,
            EasingCurve::EaseInQuint => ease_in_quint(t as f64) as f32,
            EasingCurve::EaseOutQuint => ease_out_quint(t as f64) as f32,
            EasingCurve::EaseInOutQuint => ease_in_out_quint(t as f64) as f32,
            EasingCurve::EaseInSine => ease_in_sine(t as f64) as f32,
            EasingCurve::EaseOutSine => ease_out_sine(t as f64) as f32,
            EasingCurve::EaseInOutSine => ease_in_out_sine(t as f64) as f32,
            EasingCurve::EaseInExpo => ease_in_expo(t as f64) as f32,
            EasingCurve::EaseOutExpo => ease_out_expo(t as f64) as f32,
            EasingCurve::EaseInOutExpo => ease_in_out_expo(t as f64) as f32,
            EasingCurve::EaseInCirc => ease_in_circ(t as f64) as f32,
            EasingCurve::EaseOutCirc => ease_out_circ(t as f64) as f32,
            EasingCurve::EaseInOutCirc => ease_in_out_circ(t as f64) as f32,
            EasingCurve::EaseInBack => ease_in_back(t as f64) as f32,
            EasingCurve::EaseOutBack => ease_out_back(t as f64) as f32,
            EasingCurve::EaseInOutBack => ease_in_out_back(t as f64) as f32,
            EasingCurve::EaseInElastic => ease_in_elastic(t as f64) as f32,
            EasingCurve::EaseOutElastic => ease_out_elastic(t as f64) as f32,
            EasingCurve::EaseInOutElastic => ease_in_out_elastic(t as f64) as f32,
            EasingCurve::EaseInBounce => ease_in_bounce(t as f64) as f32,
            EasingCurve::EaseOutBounce => ease_out_bounce(t as f64) as f32,
            EasingCurve::EaseInOutBounce => ease_in_out_bounce(t as f64) as f32,
            EasingCurve::CustomBezier { p1, p2 } => {
                // Approximate cubic bezier easing using binary search on x(t)
                // The Bezier curve is defined by control points:
                // P0 = (0, 0), P1 = p1, P2 = p2, P3 = (1, 1)
                // We need to find t such that the x-coordinate equals our input t,
                // then return the y-coordinate at that parameter.
                let p0x = 0.0f64;
                let p0y = 0.0f64;
                let p1x = p1.x as f64;
                let p1y = p1.y as f64;
                let p2x = p2.x as f64;
                let p2y = p2.y as f64;
                let p3x = 1.0f64;
                let p3y = 1.0f64;

                // Binary search for parameter s where bezier_x(s) = t
                let mut lo = 0.0f64;
                let mut hi = 1.0f64;
                let target = t as f64;
                for _ in 0..20 {
                    let mid = (lo + hi) / 2.0;
                    let x = bezier_cubic(p0x, p1x, p2x, p3x, mid);
                    if x < target {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let s = (lo + hi) / 2.0;
                bezier_cubic(p0y, p1y, p2y, p3y, s) as f32
            }
        }
    }

    /// Get a curve from an interpolation type.
    pub fn from_interpolation(itype: InterpolationType) -> Self {
        match itype {
            InterpolationType::Linear => EasingCurve::Linear,
            InterpolationType::Bezier => EasingCurve::CustomBezier {
                p1: Vector2f::new(0.33, 0.0),
                p2: Vector2f::new(0.66, 1.0),
            },
            InterpolationType::Hold => EasingCurve::Linear, // Hold handled separately
            InterpolationType::SmoothStep => EasingCurve::EaseInOutCubic, // close approximation
            InterpolationType::Cubic => EasingCurve::EaseInOutCubic,
            InterpolationType::Elastic => EasingCurve::EaseInOutElastic,
            InterpolationType::Bounce => EasingCurve::EaseInOutBounce,
            InterpolationType::Back => EasingCurve::EaseInOutBack,
            InterpolationType::Expo => EasingCurve::EaseInOutExpo,
            InterpolationType::Circ => EasingCurve::EaseInOutCirc,
            InterpolationType::Constant => EasingCurve::Linear,
        }
    }
}

// ─── Keyframe track ──────────────────────────────────────────────────────────

/// A track of keyframes that can be evaluated at any time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeTrack<T: Clone + Interpolate> {
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone + Interpolate> KeyframeTrack<T> {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    /// Add a keyframe. Keyframes are automatically sorted by time.
    pub fn add_keyframe(&mut self, kf: Keyframe<T>) {
        let pos = self.keyframes.iter().position(|k| k.time > kf.time);
        match pos {
            Some(idx) => self.keyframes.insert(idx, kf),
            None => self.keyframes.push(kf),
        }
    }

    /// Remove a keyframe at the given index.
    pub fn remove_keyframe(&mut self, index: usize) -> Option<Keyframe<T>> {
        if index < self.keyframes.len() {
            Some(self.keyframes.remove(index))
        } else {
            None
        }
    }

    /// Evaluate the track at the given time, performing full interpolation.
    pub fn evaluate_at(&self, time: f64) -> T {
        if self.keyframes.is_empty() {
            // Return a default value — requires T: Default
            // Since we can't require Default, we handle this per-type below.
            // For f32/f64, return 0. This is a design trade-off.
            return self.evaluate_empty_default();
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].value.clone();
        }

        // Before the first keyframe
        if time <= self.keyframes[0].time {
            return self.keyframes[0].value.clone();
        }

        // After the last keyframe
        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value.clone();
        }

        // Find the two keyframes surrounding the time
        let mut idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time > time {
                idx = i;
                break;
            }
        }

        let kf_left = &self.keyframes[idx - 1];
        let kf_right = &self.keyframes[idx];

        let duration = kf_right.time - kf_left.time;
        if duration.abs() < f64::EPSILON {
            return kf_right.value.clone();
        }

        let t_raw = ((time - kf_left.time) / duration) as f32;

        // Apply interpolation type
        match kf_left.interpolation_type {
            InterpolationType::Hold => kf_left.value.clone(),
            InterpolationType::Constant => kf_left.value.clone(),
            InterpolationType::Linear => kf_left.value.interpolate(&kf_right.value, t_raw),
            _ => {
                // Use easing curve for other interpolation types
                let curve = EasingCurve::from_interpolation(kf_left.interpolation_type);
                let eased_t = curve.evaluate(t_raw);
                kf_left.value.interpolate(&kf_right.value, eased_t)
            }
        }
    }

    /// Get the number of keyframes.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if the track has no keyframes.
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Get the time range of the track.
    pub fn time_range(&self) -> Option<(f64, f64)> {
        if self.keyframes.is_empty() {
            return None;
        }
        Some((
            self.keyframes.first().unwrap().time,
            self.keyframes.last().unwrap().time,
        ))
    }
}

// Helper trait for default values in empty tracks
trait EmptyDefault {
    fn empty_default() -> Self;
}

impl EmptyDefault for f32 {
    fn empty_default() -> Self { 0.0 }
}

impl EmptyDefault for f64 {
    fn empty_default() -> Self { 0.0 }
}

impl EmptyDefault for Vector2f {
    fn empty_default() -> Self { Vector2f::ZERO }
}

impl EmptyDefault for Vector3f {
    fn empty_default() -> Self { Vector3f::ZERO }
}

impl EmptyDefault for Vector4f {
    fn empty_default() -> Self { Vector4f::ZERO }
}

impl<T: Clone + Interpolate + EmptyDefault> KeyframeTrack<T> {
    fn evaluate_empty_default(&self) -> T {
        T::empty_default()
    }
}

// ─── Animatable properties ───────────────────────────────────────────────────

/// Properties that can be animated with keyframes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimatableProperty {
    PositionX,
    PositionY,
    ScaleX,
    ScaleY,
    Rotation,
    Opacity,
    Volume,
    Pan,
    EffectParam(String),
    MaskFeather,
    MaskExpansion,
    SpeedRamp,
}

// ─── Keyframe animation ──────────────────────────────────────────────────────

/// A keyframe animation for a single property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeAnimation {
    pub property: AnimatableProperty,
    pub track: KeyframeTrack<f32>,
    pub easing: EasingCurve,
}

impl KeyframeAnimation {
    pub fn new(property: AnimatableProperty) -> Self {
        Self {
            property,
            track: KeyframeTrack::new(),
            easing: EasingCurve::Linear,
        }
    }

    pub fn with_easing(mut self, easing: EasingCurve) -> Self {
        self.easing = easing;
        self
    }

    /// Add a keyframe at the given time with the given value.
    pub fn add_keyframe(&mut self, time: f64, value: f32) {
        let kf = Keyframe::new(time, value)
            .with_interpolation(InterpolationType::Linear);
        self.track.add_keyframe(kf);
    }

    /// Add a keyframe with a specific interpolation type.
    pub fn add_keyframe_with_interpolation(&mut self, time: f64, value: f32, itype: InterpolationType) {
        let kf = Keyframe::new(time, value)
            .with_interpolation(itype);
        self.track.add_keyframe(kf);
    }

    /// Evaluate the animation at the given time.
    pub fn evaluate_at(&self, time: f64) -> f32 {
        self.track.evaluate_at(time)
    }

    /// Check if this animation has any keyframes.
    pub fn is_animated(&self) -> bool {
        !self.track.is_empty()
    }

    /// Get the number of keyframes.
    pub fn keyframe_count(&self) -> usize {
        self.track.len()
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_f32() {
        let result = 0.0f32.interpolate(&10.0f32, 0.5);
        assert!((result - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_interpolate_f64() {
        let result = 0.0f64.interpolate(&10.0f64, 0.5);
        assert!((result - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_interpolate_vector2f() {
        let a = Vector2f::new(0.0, 0.0);
        let b = Vector2f::new(10.0, 20.0);
        let result = a.interpolate(&b, 0.5);
        assert!((result.x - 5.0).abs() < 1e-5);
        assert!((result.y - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_interpolate_vector3f() {
        let a = Vector3f::new(0.0, 0.0, 0.0);
        let b = Vector3f::new(6.0, 12.0, 18.0);
        let result = a.interpolate(&b, 0.5);
        assert!((result.x - 3.0).abs() < 1e-5);
        assert!((result.y - 6.0).abs() < 1e-5);
        assert!((result.z - 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_interpolate_vector4f() {
        let a = Vector4f::ZERO;
        let b = Vector4f::new(4.0, 8.0, 12.0, 16.0);
        let result = a.interpolate(&b, 0.5);
        assert!((result.x - 2.0).abs() < 1e-5);
        assert!((result.w - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_keyframe_track_evaluate_linear() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(0.0, 0.0f32));
        track.add_keyframe(Keyframe::new(1.0, 100.0f32));
        assert!((track.evaluate_at(0.5) - 50.0).abs() < 1e-3);
    }

    #[test]
    fn test_keyframe_track_evaluate_hold() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(0.0, 0.0f32).with_interpolation(InterpolationType::Hold));
        track.add_keyframe(Keyframe::new(1.0, 100.0f32));
        // At 0.5, Hold should return the left value
        assert!((track.evaluate_at(0.5) - 0.0).abs() < 1e-3);
    }

    #[test]
    fn test_keyframe_track_evaluate_before_first() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(1.0, 10.0f32));
        track.add_keyframe(Keyframe::new(2.0, 20.0f32));
        assert!((track.evaluate_at(0.5) - 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_keyframe_track_evaluate_after_last() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(0.0, 10.0f32));
        track.add_keyframe(Keyframe::new(1.0, 20.0f32));
        assert!((track.evaluate_at(2.0) - 20.0).abs() < 1e-3);
    }

    #[test]
    fn test_keyframe_track_single_keyframe() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(1.0, 42.0f32));
        assert!((track.evaluate_at(0.0) - 42.0).abs() < 1e-3);
        assert!((track.evaluate_at(5.0) - 42.0).abs() < 1e-3);
    }

    #[test]
    fn test_keyframe_track_empty() {
        let track: KeyframeTrack<f32> = KeyframeTrack::new();
        assert!((track.evaluate_at(0.0)).abs() < 1e-3);
    }

    #[test]
    fn test_easing_curve_linear() {
        let curve = EasingCurve::Linear;
        assert!((curve.evaluate(0.0)).abs() < 1e-5);
        assert!((curve.evaluate(1.0) - 1.0).abs() < 1e-5);
        assert!((curve.evaluate(0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_easing_curve_ease_in_out_cubic() {
        let curve = EasingCurve::EaseInOutCubic;
        assert!((curve.evaluate(0.0)).abs() < 1e-5);
        assert!((curve.evaluate(1.0) - 1.0).abs() < 1e-5);
        // Midpoint should be ~0.5
        assert!((curve.evaluate(0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_easing_curve_custom_bezier() {
        let curve = EasingCurve::CustomBezier {
            p1: Vector2f::new(0.25, 0.1),
            p2: Vector2f::new(0.25, 1.0),
        };
        assert!((curve.evaluate(0.0)).abs() < 0.01);
        assert!((curve.evaluate(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_keyframe_animation_is_animated() {
        let mut anim = KeyframeAnimation::new(AnimatableProperty::Opacity);
        assert!(!anim.is_animated());
        anim.add_keyframe(0.0, 1.0);
        assert!(anim.is_animated());
    }

    #[test]
    fn test_keyframe_animation_evaluate() {
        let mut anim = KeyframeAnimation::new(AnimatableProperty::PositionX);
        anim.add_keyframe(0.0, 0.0);
        anim.add_keyframe(1.0, 100.0);
        assert!((anim.evaluate_at(0.5) - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_keyframe_track_sorted_insert() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(2.0, 20.0f32));
        track.add_keyframe(Keyframe::new(0.0, 0.0f32));
        track.add_keyframe(Keyframe::new(1.0, 10.0f32));
        assert_eq!(track.keyframes[0].time, 0.0);
        assert_eq!(track.keyframes[1].time, 1.0);
        assert_eq!(track.keyframes[2].time, 2.0);
    }

    #[test]
    fn test_keyframe_track_remove() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        track.add_keyframe(Keyframe::new(0.0, 0.0f32));
        track.add_keyframe(Keyframe::new(1.0, 10.0f32));
        let removed = track.remove_keyframe(0);
        assert!(removed.is_some());
        assert_eq!(track.len(), 1);
    }

    #[test]
    fn test_keyframe_track_time_range() {
        let mut track: KeyframeTrack<f32> = KeyframeTrack::new();
        assert!(track.time_range().is_none());
        track.add_keyframe(Keyframe::new(2.0, 20.0f32));
        track.add_keyframe(Keyframe::new(5.0, 50.0f32));
        let range = track.time_range().unwrap();
        assert!((range.0 - 2.0).abs() < 1e-9);
        assert!((range.1 - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_easing_curve_bounce_boundary() {
        let curve = EasingCurve::EaseOutBounce;
        assert!((curve.evaluate(0.0)).abs() < 1e-5);
        assert!((curve.evaluate(1.0) - 1.0).abs() < 1e-5);
    }
}
