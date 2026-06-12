//! Text rendering for overlays on video frames
//!
//! Handles rendering text with various fonts, sizes, colors, positions,
//! and animation effects onto video frame data.

use serde::{Deserialize, Serialize};

/// A text overlay that can be placed on the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOverlay {
    pub id: String,
    pub content: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: TextColor,
    pub background: Option<TextBackground>,
    pub position: TextPosition,
    pub anchor: TextAnchor,
    pub animation: TextAnimation,
    pub outline: Option<TextOutline>,
    pub shadow: Option<TextShadow>,
}

impl TextOverlay {
    /// Create a simple text overlay with default styling
    pub fn simple(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 48.0,
            color: TextColor::white(),
            background: None,
            position: TextPosition::bottom_center(),
            anchor: TextAnchor::Center,
            animation: TextAnimation::None,
            outline: None,
            shadow: Some(TextShadow::default()),
        }
    }

    /// Create a subtitle-style text overlay
    pub fn subtitle(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 36.0,
            color: TextColor::white(),
            background: Some(TextBackground {
                color: "#000000".to_string(),
                opacity: 0.7,
                padding: 8.0,
                corner_radius: 4.0,
            }),
            position: TextPosition::bottom_center(),
            anchor: TextAnchor::Center,
            animation: TextAnimation::None,
            outline: None,
            shadow: None,
        }
    }

    /// Create a title text overlay
    pub fn title(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 72.0,
            color: TextColor::white(),
            background: None,
            position: TextPosition { x: 0.5, y: 0.3 },
            anchor: TextAnchor::Center,
            animation: TextAnimation::FadeIn { duration_ms: 500 },
            outline: Some(TextOutline {
                color: "#000000".to_string(),
                width: 3.0,
            }),
            shadow: Some(TextShadow::default()),
        }
    }

    /// Get the opacity at a given progress through the clip, considering animation
    pub fn opacity_at_progress(&self, progress: f32, clip_duration_ms: u64) -> f32 {
        match &self.animation {
            TextAnimation::None => 1.0,
            TextAnimation::FadeIn { duration_ms } => {
                let anim_progress = (clip_duration_ms as f32 * progress) / *duration_ms as f32;
                anim_progress.clamp(0.0, 1.0)
            }
            TextAnimation::FadeOut { duration_ms } => {
                let time_from_end = clip_duration_ms as f32 * (1.0 - progress);
                let anim_progress = time_from_end / *duration_ms as f32;
                anim_progress.clamp(0.0, 1.0)
            }
            TextAnimation::Typewriter { .. } => 1.0,
            TextAnimation::SlideIn { .. } => {
                let anim_progress = (clip_duration_ms as f32 * progress / 300.0).clamp(0.0, 1.0);
                smooth_step(anim_progress)
            }
            TextAnimation::Bounce { .. } => 1.0,
            TextAnimation::PopIn { duration_ms } => {
                let anim_progress = (clip_duration_ms as f32 * progress / *duration_ms as f32).clamp(0.0, 1.0);
                if anim_progress < 0.5 {
                    let t = anim_progress * 2.0;
                    t * t // Ease in
                } else {
                    let t = (anim_progress - 0.5) * 2.0;
                    1.0 - (1.0 - t) * (1.0 - t) * 0.2 // Overshoot and settle
                }
            }
        }
    }

    /// Get the visible text content at a given progress (for typewriter effect)
    pub fn visible_text_at_progress(&self, progress: f32) -> &str {
        match &self.animation {
            TextAnimation::Typewriter { chars_per_second } => {
                // Calculate how many characters should be visible
                // This is a simplified version - the actual implementation
                // would need the clip duration to calculate properly
                let visible_chars = (progress * self.content.len() as f32) as usize;
                &self.content[..visible_chars.min(self.content.len())]
            }
            _ => &self.content,
        }
    }
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Text color definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextColor {
    pub hex: String,
    pub opacity: f32,
}

impl TextColor {
    pub fn white() -> Self {
        Self { hex: "#FFFFFF".to_string(), opacity: 1.0 }
    }
    pub fn black() -> Self {
        Self { hex: "#000000".to_string(), opacity: 1.0 }
    }
    pub fn with_hex(hex: &str) -> Self {
        Self { hex: hex.to_string(), opacity: 1.0 }
    }
}

/// Text background
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBackground {
    pub color: String,
    pub opacity: f32,
    pub padding: f32,
    pub corner_radius: f32,
}

/// Position as normalized coordinates (0.0-1.0 relative to frame)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub x: f32,
    pub y: f32,
}

impl TextPosition {
    pub fn center() -> Self { Self { x: 0.5, y: 0.5 } }
    pub fn bottom_center() -> Self { Self { x: 0.5, y: 0.9 } }
    pub fn top_center() -> Self { Self { x: 0.5, y: 0.1 } }
    pub fn at(x: f32, y: f32) -> Self { Self { x, y } }
}

/// Text anchor point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Text animation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAnimation {
    None,
    FadeIn { duration_ms: u64 },
    FadeOut { duration_ms: u64 },
    Typewriter { chars_per_second: f32 },
    SlideIn { direction: SlideDirection, duration_ms: u64 },
    Bounce { height: f32, duration_ms: u64 },
    PopIn { duration_ms: u64 },
}

/// Slide direction for text animations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SlideDirection {
    Left,
    Right,
    Top,
    Bottom,
}

/// Text outline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOutline {
    pub color: String,
    pub width: f32,
}

/// Text shadow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextShadow {
    pub color: String,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
}

impl Default for TextShadow {
    fn default() -> Self {
        Self {
            color: "#000000".to_string(),
            offset_x: 2.0,
            offset_y: 2.0,
            blur: 4.0,
        }
    }
}
