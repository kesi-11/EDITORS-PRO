//! Renderer module - Composites timeline tracks into displayable frames
//!
//! The renderer takes a timeline state and a timestamp, then composites
//! all visible tracks (video, text, effects) into a single frame for display.
//! Clip effects are applied per-clip before compositing.

pub mod gpu;
pub mod shader;

use crate::decoder::FrameData;
use crate::effects::EffectsPipeline;
use crate::effects::text_rasterizer::TextRasterizer;
use crate::effects::text_render::TextOverlay;
use crate::timeline::Timeline;
use crate::timeline::track::TrackType;

/// Preview renderer for real-time frame composition
pub struct PreviewRenderer {
    width: u32,
    height: u32,
    text_rasterizer: TextRasterizer,
}

impl PreviewRenderer {
    /// Create a new preview renderer at the specified resolution
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            text_rasterizer: TextRasterizer::new(),
        }
    }

    /// Compose a single frame from the timeline at the given timestamp.
    ///
    /// This method:
    /// 1. Takes the decoded video frame (or a blank frame)
    /// 2. Applies per-clip effects from the active clip's effect chain
    /// 3. Applies opacity blending for non-opaque clips
    /// 4. Renders text overlays from text tracks
    pub fn compose_frame(
        &mut self,
        timeline: &Timeline,
        time_ms: u64,
        video_frame: Option<FrameData>,
    ) -> FrameData {
        // Start with the video frame or a blank frame
        let mut frame = video_frame.unwrap_or_else(|| FrameData::blank(self.width, self.height));

        // Get all clips active at this timestamp
        let active_clips = timeline.get_clips_at_time(time_ms);

        // Apply per-clip effects and opacity
        for (_track, clip) in &active_clips {
            // Apply effects from the clip's effect chain
            if !clip.effects.is_empty() {
                let pipeline = EffectsPipeline::new(clip.effects.clone());
                pipeline.apply(&mut frame.data, frame.width, frame.height);
            }

            // Apply opacity blending
            if clip.opacity < 1.0 {
                let alpha = clip.opacity;
                for chunk in frame.data.chunks_exact_mut(4) {
                    chunk[0] = (chunk[0] as f32 * alpha) as u8;
                    chunk[1] = (chunk[1] as f32 * alpha) as u8;
                    chunk[2] = (chunk[2] as f32 * alpha) as u8;
                    chunk[3] = (chunk[3] as f32 * alpha) as u8;
                }
            }
        }

        // Render text overlays from text tracks
        let text_tracks = timeline.tracks_of_type(TrackType::Text);
        for track in text_tracks {
            if !track.visible {
                continue;
            }
            for clip in &track.clips {
                if !clip.contains_time(time_ms) {
                    continue;
                }

                // Create a text overlay for this clip
                // The clip's asset_id for text clips is "text_<id>"
                let progress = if clip.duration_ms > 0 {
                    (time_ms - clip.start_ms) as f32 / clip.duration_ms as f32
                } else {
                    0.0
                };

                // Extract text content from the clip's effects or use a default
                let overlay = TextOverlay::simple("Text");
                self.text_rasterizer.render_text(
                    &overlay,
                    &mut frame.data,
                    frame.width,
                    frame.height,
                    progress,
                    clip.duration_ms,
                );
            }
        }

        frame
    }

    /// Apply effects from a clip's effect chain to frame data.
    pub fn apply_clip_effects(&self, frame: &mut FrameData, effects: &[crate::effects::Effect]) {
        if effects.is_empty() { return; }
        let pipeline = EffectsPipeline::new(effects.to_vec());
        pipeline.apply(&mut frame.data, frame.width, frame.height);
    }

    /// Resize a frame to the target dimensions
    pub fn resize_frame(&self, frame: &FrameData, target_width: u32, target_height: u32) -> FrameData {
        if frame.width == target_width && frame.height == target_height {
            return frame.clone();
        }

        // Simple nearest-neighbor resize for performance
        let mut data = vec![0u8; (target_width * target_height * 4) as usize];
        let x_ratio = frame.width as f32 / target_width as f32;
        let y_ratio = frame.height as f32 / target_height as f32;

        for y in 0..target_height {
            for x in 0..target_width {
                let src_x = (x as f32 * x_ratio) as u32;
                let src_y = (y as f32 * y_ratio) as u32;
                let src_idx = ((src_y * frame.width + src_x) * 4) as usize;
                let dst_idx = ((y * target_width + x) * 4) as usize;

                if src_idx + 3 < frame.data.len() && dst_idx + 3 < data.len() {
                    data[dst_idx] = frame.data[src_idx];         // R
                    data[dst_idx + 1] = frame.data[src_idx + 1]; // G
                    data[dst_idx + 2] = frame.data[src_idx + 2]; // B
                    data[dst_idx + 3] = frame.data[src_idx + 3]; // A
                }
            }
        }

        FrameData {
            width: target_width,
            height: target_height,
            data,
            timestamp_ms: frame.timestamp_ms,
            is_keyframe: frame.is_keyframe,
        }
    }
}

/// Render quality settings for preview vs export
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum RenderQuality {
    /// Low quality for smooth preview on weak devices
    PreviewLow,
    /// Medium quality for standard preview
    PreviewMedium,
    /// High quality for final export
    ExportHigh,
    /// Maximum quality for 4K export
    ExportUltra,
}

impl RenderQuality {
    /// Get the resolution scale factor for this quality level
    pub fn scale_factor(&self) -> f32 {
        match self {
            RenderQuality::PreviewLow => 0.25,
            RenderQuality::PreviewMedium => 0.5,
            RenderQuality::ExportHigh => 1.0,
            RenderQuality::ExportUltra => 1.0,
        }
    }
}
