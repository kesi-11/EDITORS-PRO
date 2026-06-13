//! Transition effects between clips
//!
//! Handles blending between the end of one clip and the start of another,
//! including cut, fade, dissolve, wipe, slide, and zoom transitions.
//! All blending operations use rayon for parallel pixel processing.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Types of transitions between clips
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    Cut,
    Fade,
    Dissolve,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    SlideLeft,
    SlideRight,
    ZoomIn,
    ZoomOut,
    Spin,
}

impl TransitionType {
    pub fn display_name(&self) -> &str {
        match self {
            TransitionType::Cut => "Cut",
            TransitionType::Fade => "Fade",
            TransitionType::Dissolve => "Dissolve",
            TransitionType::WipeLeft => "Wipe Left",
            TransitionType::WipeRight => "Wipe Right",
            TransitionType::WipeUp => "Wipe Up",
            TransitionType::WipeDown => "Wipe Down",
            TransitionType::SlideLeft => "Slide Left",
            TransitionType::SlideRight => "Slide Right",
            TransitionType::ZoomIn => "Zoom In",
            TransitionType::ZoomOut => "Zoom Out",
            TransitionType::Spin => "Spin",
        }
    }

    /// Get an icon identifier for UI rendering
    pub fn icon(&self) -> &str {
        match self {
            TransitionType::Cut => "cut",
            TransitionType::Fade => "fade",
            TransitionType::Dissolve => "dissolve",
            TransitionType::WipeLeft => "wipe_left",
            TransitionType::WipeRight => "wipe_right",
            TransitionType::WipeUp => "wipe_up",
            TransitionType::WipeDown => "wipe_down",
            TransitionType::SlideLeft => "slide_left",
            TransitionType::SlideRight => "slide_right",
            TransitionType::ZoomIn => "zoom_in",
            TransitionType::ZoomOut => "zoom_out",
            TransitionType::Spin => "spin",
        }
    }

    /// Get the default duration for this transition type
    pub fn default_duration_ms(&self) -> u64 {
        match self {
            TransitionType::Cut => 0,
            TransitionType::Fade => 500,
            TransitionType::Dissolve => 700,
            TransitionType::WipeLeft | TransitionType::WipeRight => 500,
            TransitionType::WipeUp | TransitionType::WipeDown => 500,
            TransitionType::SlideLeft | TransitionType::SlideRight => 400,
            TransitionType::ZoomIn | TransitionType::ZoomOut => 600,
            TransitionType::Spin => 500,
        }
    }

    pub fn all_transitions() -> Vec<TransitionType> {
        vec![
            TransitionType::Cut,
            TransitionType::Fade,
            TransitionType::Dissolve,
            TransitionType::WipeLeft,
            TransitionType::WipeRight,
            TransitionType::WipeUp,
            TransitionType::WipeDown,
            TransitionType::SlideLeft,
            TransitionType::SlideRight,
            TransitionType::ZoomIn,
            TransitionType::ZoomOut,
            TransitionType::Spin,
        ]
    }

    /// Parse from a string (case-insensitive)
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cut" => Some(TransitionType::Cut),
            "fade" => Some(TransitionType::Fade),
            "dissolve" => Some(TransitionType::Dissolve),
            "wipeleft" | "wipe_left" => Some(TransitionType::WipeLeft),
            "wiperight" | "wipe_right" => Some(TransitionType::WipeRight),
            "wipeup" | "wipe_up" => Some(TransitionType::WipeUp),
            "wipedown" | "wipe_down" => Some(TransitionType::WipeDown),
            "slideleft" | "slide_left" => Some(TransitionType::SlideLeft),
            "slideright" | "slide_right" => Some(TransitionType::SlideRight),
            "zoomin" | "zoom_in" => Some(TransitionType::ZoomIn),
            "zoomout" | "zoom_out" => Some(TransitionType::ZoomOut),
            "spin" => Some(TransitionType::Spin),
            _ => None,
        }
    }
}

/// A transition between two clips
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub transition_type: TransitionType,
    pub duration_ms: u64,
    pub from_clip_id: String,
    pub to_clip_id: String,
}

impl Transition {
    pub fn new(transition_type: TransitionType, duration_ms: u64, from_clip_id: &str, to_clip_id: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            transition_type,
            duration_ms: if duration_ms == 0 { transition_type.default_duration_ms() } else { duration_ms },
            from_clip_id: from_clip_id.to_string(),
            to_clip_id: to_clip_id.to_string(),
        }
    }

    /// Create a transition with default duration
    pub fn with_default_duration(transition_type: TransitionType, from_clip_id: &str, to_clip_id: &str) -> Self {
        Self::new(transition_type, 0, from_clip_id, to_clip_id)
    }

    /// Calculate the blend factor for a given progress through the transition
    /// Returns a value between 0.0 (fully from_clip) and 1.0 (fully to_clip)
    pub fn blend_at_progress(&self, progress: f32) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        match self.transition_type {
            TransitionType::Cut => {
                // Instant cut at 50% progress
                if p >= 0.5 { 1.0 } else { 0.0 }
            }
            TransitionType::Fade => {
                // Linear fade
                p
            }
            TransitionType::Dissolve => {
                // Smooth dissolve with easing
                smooth_step(p)
            }
            TransitionType::WipeLeft => p,
            TransitionType::WipeRight => p,
            TransitionType::WipeUp => p,
            TransitionType::WipeDown => p,
            TransitionType::SlideLeft => p,
            TransitionType::SlideRight => p,
            TransitionType::ZoomIn => smooth_step(p),
            TransitionType::ZoomOut => smooth_step(p),
            TransitionType::Spin => smooth_step(p),
        }
    }

    /// Check if this transition is spatial (wipes, slides, zooms)
    /// as opposed to simple alpha-blended (fade, dissolve).
    pub fn is_spatial(&self) -> bool {
        matches!(
            self.transition_type,
            TransitionType::WipeLeft
                | TransitionType::WipeRight
                | TransitionType::WipeUp
                | TransitionType::WipeDown
                | TransitionType::SlideLeft
                | TransitionType::SlideRight
                | TransitionType::ZoomIn
                | TransitionType::ZoomOut
        )
    }

    /// Blend two RGBA frames based on the transition using rayon for parallelism.
    ///
    /// `from_frame` and `to_frame` must both be `width * height * 4` bytes.
    /// `progress` ranges from 0.0 (fully from_frame) to 1.0 (fully to_frame).
    /// Returns a new Vec<u8> with the blended frame.
    pub fn blend_frames(
        &self,
        from_frame: &[u8],
        to_frame: &[u8],
        width: u32,
        height: u32,
        progress: f32,
    ) -> Vec<u8> {
        let blend = self.blend_at_progress(progress);
        let total_pixels = (width * height) as usize;

        match self.transition_type {
            TransitionType::Cut | TransitionType::Fade | TransitionType::Dissolve => {
                // Alpha blend — fully parallelizable
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let idx = i * 4;
                    pixel[0] = (from_frame[idx] as f32 * (1.0 - blend) + to_frame[idx] as f32 * blend) as u8;
                    pixel[1] = (from_frame[idx + 1] as f32 * (1.0 - blend) + to_frame[idx + 1] as f32 * blend) as u8;
                    pixel[2] = (from_frame[idx + 2] as f32 * (1.0 - blend) + to_frame[idx + 2] as f32 * blend) as u8;
                    pixel[3] = 255;
                });
                result
            }

            TransitionType::WipeLeft => {
                let split_x = (blend * width as f32) as u32;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let x = i as u32 % width;
                    let idx = i * 4;
                    let src = if x < split_x { to_frame } else { from_frame };
                    pixel.copy_from_slice(&src[idx..idx + 4]);
                });
                result
            }

            TransitionType::WipeRight => {
                let split_x = ((1.0 - blend) * width as f32) as u32;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let x = i as u32 % width;
                    let idx = i * 4;
                    let src = if x >= split_x { to_frame } else { from_frame };
                    pixel.copy_from_slice(&src[idx..idx + 4]);
                });
                result
            }

            TransitionType::WipeUp => {
                let split_y = (blend * height as f32) as u32;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let y = (i as u32) / width;
                    let idx = i * 4;
                    let src = if y < split_y { to_frame } else { from_frame };
                    pixel.copy_from_slice(&src[idx..idx + 4]);
                });
                result
            }

            TransitionType::WipeDown => {
                let split_y = ((1.0 - blend) * height as f32) as u32;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let y = (i as u32) / width;
                    let idx = i * 4;
                    let src = if y >= split_y { to_frame } else { from_frame };
                    pixel.copy_from_slice(&src[idx..idx + 4]);
                });
                result
            }

            TransitionType::SlideLeft => {
                let offset_x = (blend * width as f32) as i32;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let x = i as i32 % width as i32;
                    let y = i as i32 / width as i32;
                    // From frame slides left, to frame comes from the right
                    let from_x = x + offset_x;
                    let to_x = x - (width as i32 - offset_x);
                    let idx = i * 4;

                    if from_x >= 0 && from_x < width as i32 {
                        let src_idx = (y * width as i32 + from_x) as usize * 4;
                        pixel.copy_from_slice(&from_frame[src_idx..src_idx + 4]);
                    } else if to_x >= 0 && to_x < width as i32 {
                        let src_idx = (y * width as i32 + to_x) as usize * 4;
                        pixel.copy_from_slice(&to_frame[src_idx..src_idx + 4]);
                    } else {
                        pixel[0] = 0; pixel[1] = 0; pixel[2] = 0; pixel[3] = 255;
                    }
                });
                result
            }

            TransitionType::SlideRight => {
                let offset_x = (blend * width as f32) as i32;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let x = i as i32 % width as i32;
                    let y = i as i32 / width as i32;
                    let from_x = x - offset_x;
                    let to_x = x + (width as i32 - offset_x);

                    if from_x >= 0 && from_x < width as i32 {
                        let src_idx = (y * width as i32 + from_x) as usize * 4;
                        pixel.copy_from_slice(&from_frame[src_idx..src_idx + 4]);
                    } else if to_x >= 0 && to_x < width as i32 {
                        let src_idx = (y * width as i32 + to_x) as usize * 4;
                        pixel.copy_from_slice(&to_frame[src_idx..src_idx + 4]);
                    } else {
                        pixel[0] = 0; pixel[1] = 0; pixel[2] = 0; pixel[3] = 255;
                    }
                });
                result
            }

            TransitionType::ZoomIn => {
                // From clip shrinks (zoom out), to clip zooms in from center
                let scale = blend;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let x = i as u32 % width;
                    let y = i as u32 / width;
                    let cx = width as f32 / 2.0;
                    let cy = height as f32 / 2.0;

                    // Sample from the 'to' frame at a zoomed position
                    let sample_x = cx + (x as f32 - cx) / scale.max(0.01);
                    let sample_y = cy + (y as f32 - cy) / scale.max(0.01);

                    let idx = i * 4;
                    if sample_x >= 0.0 && sample_x < width as f32 && sample_y >= 0.0 && sample_y < height as f32 {
                        let src_idx = (sample_y as usize * width as usize + sample_x as usize) * 4;
                        let alpha_to = blend;
                        let alpha_from = 1.0 - blend;
                        pixel[0] = (from_frame[idx] as f32 * alpha_from + to_frame[src_idx] as f32 * alpha_to) as u8;
                        pixel[1] = (from_frame[idx + 1] as f32 * alpha_from + to_frame[src_idx + 1] as f32 * alpha_to) as u8;
                        pixel[2] = (from_frame[idx + 2] as f32 * alpha_from + to_frame[src_idx + 2] as f32 * alpha_to) as u8;
                        pixel[3] = 255;
                    } else {
                        // Outside zoom area: show from_frame with fading
                        let alpha = 1.0 - blend;
                        pixel[0] = (from_frame[idx] as f32 * alpha) as u8;
                        pixel[1] = (from_frame[idx + 1] as f32 * alpha) as u8;
                        pixel[2] = (from_frame[idx + 2] as f32 * alpha) as u8;
                        pixel[3] = 255;
                    }
                });
                result
            }

            TransitionType::ZoomOut => {
                // To clip starts zoomed in and shrinks to normal
                let scale = 1.0 - blend;
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let x = i as u32 % width;
                    let y = i as u32 / width;
                    let cx = width as f32 / 2.0;
                    let cy = height as f32 / 2.0;

                    let sample_x = cx + (x as f32 - cx) / scale.max(0.01);
                    let sample_y = cy + (y as f32 - cy) / scale.max(0.01);

                    let idx = i * 4;
                    if sample_x >= 0.0 && sample_x < width as f32 && sample_y >= 0.0 && sample_y < height as f32 {
                        let src_idx = (sample_y as usize * width as usize + sample_x as usize) * 4;
                        let alpha_from = 1.0 - blend;
                        let alpha_to = blend;
                        pixel[0] = (from_frame[src_idx] as f32 * alpha_from + to_frame[idx] as f32 * alpha_to) as u8;
                        pixel[1] = (from_frame[src_idx + 1] as f32 * alpha_from + to_frame[idx + 1] as f32 * alpha_to) as u8;
                        pixel[2] = (from_frame[src_idx + 2] as f32 * alpha_from + to_frame[idx + 2] as f32 * alpha_to) as u8;
                        pixel[3] = 255;
                    } else {
                        let alpha = blend;
                        pixel[0] = (to_frame[idx] as f32 * alpha) as u8;
                        pixel[1] = (to_frame[idx + 1] as f32 * alpha) as u8;
                        pixel[2] = (to_frame[idx + 2] as f32 * alpha) as u8;
                        pixel[3] = 255;
                    }
                });
                result
            }

            TransitionType::Spin => {
                // Simple rotational blend — cross-dissolve with smooth easing
                // Full spatial spin requires 3D transforms (future GPU phase)
                let mut result = vec![0u8; total_pixels * 4];
                result.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
                    let idx = i * 4;
                    pixel[0] = (from_frame[idx] as f32 * (1.0 - blend) + to_frame[idx] as f32 * blend) as u8;
                    pixel[1] = (from_frame[idx + 1] as f32 * (1.0 - blend) + to_frame[idx + 1] as f32 * blend) as u8;
                    pixel[2] = (from_frame[idx + 2] as f32 * (1.0 - blend) + to_frame[idx + 2] as f32 * blend) as u8;
                    pixel[3] = 255;
                });
                result
            }
        }
    }
}

/// Smooth step interpolation (ease in-out)
fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solid_frame(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(255);
        }
        data
    }

    #[test]
    fn test_cross_dissolve_at_50_percent() {
        let from = make_solid_frame(100, 100, 200, 0, 0);
        let to = make_solid_frame(100, 100, 0, 0, 200);
        let transition = Transition::new(TransitionType::Dissolve, 500, "a", "b");
        let result = transition.blend_frames(&from, &to, 100, 100, 0.5);

        // At 50% with smooth step: blend = smooth_step(0.5) = 0.5
        // R ≈ 100, B ≈ 100
        let pixel = &result[0..4];
        assert!(pixel[0] > 50 && pixel[0] < 200, "R should be blended: got {}", pixel[0]);
        assert!(pixel[2] > 50 && pixel[2] < 200, "B should be blended: got {}", pixel[2]);
    }

    #[test]
    fn test_cut_transition() {
        let from = make_solid_frame(10, 10, 255, 0, 0);
        let to = make_solid_frame(10, 10, 0, 255, 0);
        let transition = Transition::new(TransitionType::Cut, 0, "a", "b");

        // Below 50%: should be from_frame
        let result_before = transition.blend_frames(&from, &to, 10, 10, 0.4);
        assert_eq!(result_before[0], 255, "Before cut: should show from frame R");
        assert_eq!(result_before[1], 0, "Before cut: should show from frame G");

        // Above 50%: should be to_frame
        let result_after = transition.blend_frames(&from, &to, 10, 10, 0.6);
        assert_eq!(result_after[0], 0, "After cut: should show to frame R");
        assert_eq!(result_after[1], 255, "After cut: should show to frame G");
    }

    #[test]
    fn test_wipe_left() {
        let from = make_solid_frame(100, 100, 255, 0, 0);
        let to = make_solid_frame(100, 100, 0, 255, 0);
        let transition = Transition::new(TransitionType::WipeLeft, 500, "a", "b");

        let result = transition.blend_frames(&from, &to, 100, 100, 0.5);
        // At 50%, left half should be 'to' (green), right half 'from' (red)
        let left_pixel = &result[0..4]; // x=0
        let right_pixel = &result[(99 * 4)..(99 * 4 + 4)]; // x=99

        assert_eq!(left_pixel[1], 255, "Left side should be to_frame (green)");
        assert_eq!(right_pixel[0], 255, "Right side should be from_frame (red)");
    }

    #[test]
    fn test_all_transitions_have_names() {
        for t in TransitionType::all_transitions() {
            assert!(!t.display_name().is_empty());
            assert!(!t.icon().is_empty());
        }
    }

    #[test]
    fn test_default_duration_ms() {
        assert_eq!(TransitionType::Cut.default_duration_ms(), 0);
        assert!(TransitionType::Fade.default_duration_ms() > 0);
        assert!(TransitionType::Dissolve.default_duration_ms() > 0);
    }

    #[test]
    fn test_from_str_lossy() {
        assert_eq!(TransitionType::from_str_lossy("fade"), Some(TransitionType::Fade));
        assert_eq!(TransitionType::from_str_lossy("Dissolve"), Some(TransitionType::Dissolve));
        assert_eq!(TransitionType::from_str_lossy("wipe_left"), Some(TransitionType::WipeLeft));
        assert_eq!(TransitionType::from_str_lossy("unknown"), None);
    }
}
