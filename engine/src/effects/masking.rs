//! Masking system — Bezier/shape masks with feathering, expansion, and blend modes.
//!
//! Supports Rectangle, Ellipse, Bezier, Luminance, Chroma, and Depth mask types.
//! Each mask supports feathering (edge softness), expansion (grow/shrink),
//! inversion, and four compositing modes: Add, Subtract, Intersect, Difference.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Type of mask shape or source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskType {
    Rectangle,
    Ellipse,
    Bezier,
    Luminance,
    Chroma,
    Depth,
}

/// How multiple masks combine on a clip.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskBlendMode {
    Add,
    Subtract,
    Intersect,
    Difference,
}

/// A single Bezier control point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BezierPoint {
    pub x: f32,
    pub y: f32,
    pub handle_in_x: f32,
    pub handle_in_y: f32,
    pub handle_out_x: f32,
    pub handle_out_y: f32,
}

impl BezierPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, handle_in_x: x - 20.0, handle_in_y: y, handle_out_x: x + 20.0, handle_out_y: y }
    }

    pub fn with_handles(x: f32, y: f32, hix: f32, hiy: f32, hox: f32, hoy: f32) -> Self {
        Self { x, y, handle_in_x: hix, handle_in_y: hiy, handle_out_x: hox, handle_out_y: hoy }
    }
}

/// Evaluate a cubic Bezier curve at parameter t.
fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// A mask applied to a clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mask {
    pub id: String,
    pub mask_type: MaskType,
    pub enabled: bool,
    pub inverted: bool,
    pub feather: f32,         // Edge softness 0..1
    pub expansion: f32,       // Grow/shrink -1..1
    pub opacity: f32,         // Mask opacity 0..1
    pub blend_mode: MaskBlendMode,
    // Shape-specific
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_w: f32,
    pub rect_h: f32,
    pub ellipse_cx: f32,
    pub ellipse_cy: f32,
    pub ellipse_rx: f32,
    pub ellipse_ry: f32,
    pub rotation: f32,        // Degrees
    // Bezier-specific
    pub bezier_points: Vec<BezierPoint>,
    // Luminance/Chroma threshold
    pub threshold: f32,
    pub threshold_softness: f32,
}

impl Mask {
    pub fn new_rectangle(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            mask_type: MaskType::Rectangle,
            enabled: true,
            inverted: false,
            feather: 0.0,
            expansion: 0.0,
            opacity: 1.0,
            blend_mode: MaskBlendMode::Add,
            rect_x: x, rect_y: y, rect_w: w, rect_h: h,
            ellipse_cx: 0.0, ellipse_cy: 0.0, ellipse_rx: 0.0, ellipse_ry: 0.0,
            rotation: 0.0,
            bezier_points: Vec::new(),
            threshold: 0.5,
            threshold_softness: 0.1,
        }
    }

    pub fn new_ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            mask_type: MaskType::Ellipse,
            enabled: true,
            inverted: false,
            feather: 0.0,
            expansion: 0.0,
            opacity: 1.0,
            blend_mode: MaskBlendMode::Add,
            rect_x: 0.0, rect_y: 0.0, rect_w: 0.0, rect_h: 0.0,
            ellipse_cx: cx, ellipse_cy: cy, ellipse_rx: rx, ellipse_ry: ry,
            rotation: 0.0,
            bezier_points: Vec::new(),
            threshold: 0.5,
            threshold_softness: 0.1,
        }
    }

    pub fn new_bezier(points: Vec<BezierPoint>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            mask_type: MaskType::Bezier,
            enabled: true,
            inverted: false,
            feather: 0.0,
            expansion: 0.0,
            opacity: 1.0,
            blend_mode: MaskBlendMode::Add,
            rect_x: 0.0, rect_y: 0.0, rect_w: 0.0, rect_h: 0.0,
            ellipse_cx: 0.0, ellipse_cy: 0.0, ellipse_rx: 0.0, ellipse_ry: 0.0,
            rotation: 0.0,
            bezier_points: points,
            threshold: 0.5,
            threshold_softness: 0.1,
        }
    }

    /// Compute mask value (0..1) for a pixel at normalized coordinates (0..1).
    pub fn evaluate_at(&self, nx: f32, ny: f32, pixel_luma: f32) -> f32 {
        let raw = match self.mask_type {
            MaskType::Rectangle => self.eval_rectangle(nx, ny),
            MaskType::Ellipse => self.eval_ellipse(nx, ny),
            MaskType::Bezier => self.eval_bezier(nx, ny),
            MaskType::Luminance => self.eval_luminance(pixel_luma),
            MaskType::Chroma => self.eval_luminance(pixel_luma), // Simplified: uses threshold
            MaskType::Depth => self.eval_luminance(pixel_luma),  // Simplified: uses threshold
        };

        let expanded = apply_expansion(raw, self.expansion);
        let feathered = apply_feather(expanded, self.feather);
        let alpha = feathered * self.opacity;
        if self.inverted { 1.0 - alpha } else { alpha }
    }

    fn eval_rectangle(&self, nx: f32, ny: f32) -> f32 {
        let rot = self.rotation * PI / 180.0;
        let cos_r = rot.cos();
        let sin_r = rot.sin();
        let cx = self.rect_x + self.rect_w / 2.0;
        let cy = self.rect_y + self.rect_h / 2.0;
        let dx = nx - cx;
        let dy = ny - cy;
        let rx = dx * cos_r + dy * sin_r + cx;
        let ry = -dx * sin_r + dy * cos_r + cy;

        if rx >= self.rect_x && rx <= self.rect_x + self.rect_w &&
           ry >= self.rect_y && ry <= self.rect_y + self.rect_h { 1.0 } else { 0.0 }
    }

    fn eval_ellipse(&self, nx: f32, ny: f32) -> f32 {
        let dx = (nx - self.ellipse_cx) / self.ellipse_rx.max(0.001);
        let dy = (ny - self.ellipse_cy) / self.ellipse_ry.max(0.001);
        let dist_sq = dx * dx + dy * dy;
        if dist_sq <= 1.0 { 1.0 } else { 0.0 }
    }

    fn eval_bezier(&self, nx: f32, ny: f32) -> f32 {
        if self.bezier_points.len() < 3 { return 0.0; }
        // Point-in-polygon via ray casting on the Bezier path
        let n = self.bezier_points.len();
        let mut inside = false;
        let steps = 8; // Subdivisions per Bezier segment
        let mut j = n - 1;
        for i in 0..n {
            let pi = &self.bezier_points[i];
            let pj = &self.bezier_points[j];
            for s in 0..steps {
                let t0 = s as f32 / steps as f32;
                let t1 = (s + 1) as f32 / steps as f32;
                let x0 = cubic_bezier(pj.x, pj.handle_out_x, pi.handle_in_x, pi.x, t0);
                let y0 = cubic_bezier(pj.y, pj.handle_out_y, pi.handle_in_y, pi.y, t0);
                let x1 = cubic_bezier(pj.x, pj.handle_out_x, pi.handle_in_x, pi.x, t1);
                let y1 = cubic_bezier(pj.y, pj.handle_out_y, pi.handle_in_y, pi.y, t1);

                if ((y0 > ny) != (y1 > ny)) &&
                   (nx < (x1 - x0) * (ny - y0) / (y1 - y0 + 1e-10) + x0) {
                    inside = !inside;
                }
            }
            j = i;
        }
        if inside { 1.0 } else { 0.0 }
    }

    fn eval_luminance(&self, luma: f32) -> f32 {
        let lo = self.threshold - self.threshold_softness;
        let hi = self.threshold + self.threshold_softness;
        if luma < lo { 0.0 }
        else if luma > hi { 1.0 }
        else { (luma - lo) / (hi - lo + 1e-10) }
    }
}

/// Apply expansion (grow/shrink) to a mask value.
fn apply_expansion(value: f32, expansion: f32) -> f32 {
    if expansion > 0.0 {
        // Grow: push values toward 1
        (value + expansion).min(1.0)
    } else {
        // Shrink: push values toward 0
        (value + expansion).max(0.0)
    }
}

/// Apply feathering (Gaussian-like edge softness) to a mask value.
fn apply_feather(value: f32, feather: f32) -> f32 {
    if feather <= 0.0 { return value; }
    // Soft transition around the 0.5 boundary
    let transition = feather * 0.5;
    let center = 0.5;
    let dist = (value - center).abs();
    if dist > transition { value }
    else {
        // Smoothstep in the transition zone
        let t = (value - (center - transition)) / (2.0 * transition + 1e-10);
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// Composite two mask values using a blend mode.
pub fn composite_masks(existing: f32, incoming: f32, mode: MaskBlendMode) -> f32 {
    match mode {
        MaskBlendMode::Add => (existing + incoming).min(1.0),
        MaskBlendMode::Subtract => (existing - incoming).max(0.0),
        MaskBlendMode::Intersect => existing * incoming,
        MaskBlendMode::Difference => (existing - incoming).abs(),
    }
}

/// Apply a stack of masks to compute the final mask value for a pixel.
pub fn evaluate_mask_stack(masks: &[Mask], nx: f32, ny: f32, pixel_luma: f32) -> f32 {
    let mut result = 0.0;
    for mask in masks {
        if !mask.enabled { continue; }
        let val = mask.evaluate_at(nx, ny, pixel_luma);
        result = composite_masks(result, val, mask.blend_mode);
    }
    result
}

/// Apply masks to RGBA frame data.
pub fn apply_masks(frame: &mut [u8], width: u32, height: u32, masks: &[Mask]) {
    let w = width as f32;
    let h = height as f32;
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let nx = x as f32 / w;
            let ny = y as f32 / h;
            let luma = (frame[idx] as f32 * 0.299 + frame[idx + 1] as f32 * 0.587 + frame[idx + 2] as f32 * 0.114) / 255.0;
            let mask_val = evaluate_mask_stack(masks, nx, ny, luma);
            frame[idx + 3] = (frame[idx + 3] as f32 * mask_val) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_mask_inside() {
        let mask = Mask::new_rectangle(0.2, 0.2, 0.6, 0.6);
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.5), 1.0);
    }

    #[test]
    fn test_rectangle_mask_outside() {
        let mask = Mask::new_rectangle(0.2, 0.2, 0.6, 0.6);
        assert_eq!(mask.evaluate_at(0.1, 0.1, 0.5), 0.0);
    }

    #[test]
    fn test_ellipse_mask_center() {
        let mask = Mask::new_ellipse(0.5, 0.5, 0.3, 0.3);
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.5), 1.0);
    }

    #[test]
    fn test_ellipse_mask_outside() {
        let mask = Mask::new_ellipse(0.5, 0.5, 0.1, 0.1);
        assert_eq!(mask.evaluate_at(0.9, 0.9, 0.5), 0.0);
    }

    #[test]
    fn test_mask_inversion() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 1.0, 1.0);
        mask.inverted = true;
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.5), 0.0);
    }

    #[test]
    fn test_mask_feather() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 1.0, 1.0);
        mask.feather = 0.3;
        let val = mask.evaluate_at(0.5, 0.5, 0.5);
        assert!(val > 0.0);
    }

    #[test]
    fn test_mask_expansion_grow() {
        let mut mask = Mask::new_ellipse(0.5, 0.5, 0.3, 0.3);
        mask.expansion = 0.5;
        let val = mask.evaluate_at(0.5, 0.5, 0.5);
        assert_eq!(val, 1.0);
    }

    #[test]
    fn test_composite_add() {
        assert_eq!(composite_masks(0.5, 0.6, MaskBlendMode::Add), 1.0);
    }

    #[test]
    fn test_composite_subtract() {
        assert_eq!(composite_masks(0.8, 0.3, MaskBlendMode::Subtract), 0.5);
    }

    #[test]
    fn test_composite_intersect() {
        assert!((composite_masks(0.5, 0.6, MaskBlendMode::Intersect) - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_composite_difference() {
        assert_eq!(composite_masks(0.8, 0.3, MaskBlendMode::Difference), 0.5);
    }

    #[test]
    fn test_luminance_mask_below() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 0.0, 0.0);
        mask.mask_type = MaskType::Luminance;
        mask.threshold = 0.5;
        mask.threshold_softness = 0.1;
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.3), 0.0);
    }

    #[test]
    fn test_luminance_mask_above() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 0.0, 0.0);
        mask.mask_type = MaskType::Luminance;
        mask.threshold = 0.5;
        mask.threshold_softness = 0.1;
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.7), 1.0);
    }

    #[test]
    fn test_mask_opacity() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 1.0, 1.0);
        mask.opacity = 0.5;
        let val = mask.evaluate_at(0.5, 0.5, 0.5);
        assert!((val - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_mask_disabled() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 1.0, 1.0);
        mask.enabled = false;
        let masks = vec![mask];
        assert_eq!(evaluate_mask_stack(&masks, 0.5, 0.5, 0.5), 0.0);
    }

    #[test]
    fn test_bezier_point_in_polygon() {
        let pts = vec![
            BezierPoint::new(0.2, 0.2),
            BezierPoint::new(0.8, 0.2),
            BezierPoint::new(0.8, 0.8),
            BezierPoint::new(0.2, 0.8),
        ];
        let mask = Mask::new_bezier(pts);
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.5), 1.0);
    }

    #[test]
    fn test_bezier_point_outside() {
        let pts = vec![
            BezierPoint::new(0.2, 0.2),
            BezierPoint::new(0.8, 0.2),
            BezierPoint::new(0.8, 0.8),
            BezierPoint::new(0.2, 0.8),
        ];
        let mask = Mask::new_bezier(pts);
        assert_eq!(mask.evaluate_at(0.1, 0.1, 0.5), 0.0);
    }

    #[test]
    fn test_apply_masks_to_frame() {
        let mut frame = vec![255u8; 4 * 4 * 4]; // 4x4 RGBA
        let mask = Mask::new_rectangle(0.0, 0.0, 1.0, 1.0);
        apply_masks(&mut frame, 4, 4, &[mask]);
        assert_eq!(frame[3], 255); // Alpha unchanged for full mask
    }

    #[test]
    fn test_mask_rotation() {
        let mut mask = Mask::new_rectangle(0.3, 0.3, 0.4, 0.4);
        mask.rotation = 45.0;
        let val = mask.evaluate_at(0.5, 0.5, 0.5);
        assert_eq!(val, 1.0); // Center still inside
    }

    #[test]
    fn test_cubic_bezier_curve() {
        let result = cubic_bezier(0.0, 0.0, 1.0, 1.0, 0.5);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_bezier_point_constructors() {
        let p = BezierPoint::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
        let p2 = BezierPoint::with_handles(10.0, 20.0, 5.0, 15.0, 15.0, 25.0);
        assert_eq!(p2.handle_in_x, 5.0);
    }

    #[test]
    fn test_mask_stack_multiple() {
        let m1 = Mask::new_rectangle(0.0, 0.0, 1.0, 1.0);
        let mut m2 = Mask::new_ellipse(0.5, 0.5, 0.3, 0.3);
        m2.blend_mode = MaskBlendMode::Intersect;
        let val = evaluate_mask_stack(&[m1, m2], 0.1, 0.1, 0.5);
        assert!(val < 0.01); // Outside ellipse, intersect gives 0
    }

    #[test]
    fn test_apply_expansion_shrink() {
        let val = apply_expansion(0.3, -0.5);
        assert_eq!(val, 0.0);
    }

    #[test]
    fn test_apply_feather_zero() {
        assert_eq!(apply_feather(0.7, 0.0), 0.7);
    }

    #[test]
    fn test_empty_mask_stack() {
        assert_eq!(evaluate_mask_stack(&[], 0.5, 0.5, 0.5), 0.0);
    }

    #[test]
    fn test_bezier_too_few_points() {
        let pts = vec![BezierPoint::new(0.5, 0.5)];
        let mask = Mask::new_bezier(pts);
        assert_eq!(mask.evaluate_at(0.5, 0.5, 0.5), 0.0);
    }

    #[test]
    fn test_luminance_mask_softness() {
        let mut mask = Mask::new_rectangle(0.0, 0.0, 0.0, 0.0);
        mask.mask_type = MaskType::Luminance;
        mask.threshold = 0.5;
        mask.threshold_softness = 0.2;
        let val = mask.evaluate_at(0.5, 0.5, 0.45);
        assert!(val > 0.0 && val < 1.0); // In the soft transition zone
    }

    #[test]
    fn test_rectangle_mask_default_values() {
        let mask = Mask::new_rectangle(0.1, 0.1, 0.8, 0.8);
        assert!(mask.enabled);
        assert!(!mask.inverted);
        assert_eq!(mask.feather, 0.0);
        assert_eq!(mask.expansion, 0.0);
        assert_eq!(mask.opacity, 1.0);
    }

    #[test]
    fn test_ellipse_mask_default_values() {
        let mask = Mask::new_ellipse(0.5, 0.5, 0.3, 0.3);
        assert_eq!(mask.mask_type, MaskType::Ellipse);
        assert_eq!(mask.ellipse_cx, 0.5);
    }
}
