//! Transition effects between clips
//!
//! Handles blending between the end of one clip and the start of another,
//! including cut, fade, dissolve, wipe, slide, and zoom transitions.

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
            duration_ms,
            from_clip_id: from_clip_id.to_string(),
            to_clip_id: to_clip_id.to_string(),
        }
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

    /// Blend two RGBA frames based on the transition
    /// Returns the blended frame data
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
                // Simple alpha blend
                let mut result = vec![0u8; total_pixels * 4];
                for i in 0..total_pixels {
                    let idx = i * 4;
                    result[idx] = (from_frame[idx] as f32 * (1.0 - blend) + to_frame[idx] as f32 * blend) as u8;
                    result[idx + 1] = (from_frame[idx + 1] as f32 * (1.0 - blend) + to_frame[idx + 1] as f32 * blend) as u8;
                    result[idx + 2] = (from_frame[idx + 2] as f32 * (1.0 - blend) + to_frame[idx + 2] as f32 * blend) as u8;
                    result[idx + 3] = 255;
                }
                result
            }
            TransitionType::WipeLeft => {
                let split_x = (blend * width as f32) as u32;
                let mut result = vec![0u8; total_pixels * 4];
                for y in 0..height {
                    for x in 0..width {
                        let idx = ((y * width + x) * 4) as usize;
                        let src = if x < split_x { to_frame } else { from_frame };
                        result[idx..idx + 4].copy_from_slice(&src[idx..idx + 4]);
                    }
                }
                result
            }
            _ => {
                // Default: simple blend for unimplemented transitions
                let mut result = vec![0u8; total_pixels * 4];
                for i in 0..total_pixels {
                    let idx = i * 4;
                    result[idx] = (from_frame[idx] as f32 * (1.0 - blend) + to_frame[idx] as f32 * blend) as u8;
                    result[idx + 1] = (from_frame[idx + 1] as f32 * (1.0 - blend) + to_frame[idx + 1] as f32 * blend) as u8;
                    result[idx + 2] = (from_frame[idx + 2] as f32 * (1.0 - blend) + to_frame[idx + 2] as f32 * blend) as u8;
                    result[idx + 3] = 255;
                }
                result
            }
        }
    }
}

/// Smooth step interpolation (ease in-out)
fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
