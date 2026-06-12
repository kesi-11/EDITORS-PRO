//! Renderer module - Composites timeline tracks into displayable frames
//!
//! The renderer takes a timeline state and a timestamp, then composites
//! all visible tracks (video, text, effects) into a single frame for display.

pub mod gpu;
pub mod shader;

use crate::decoder::FrameData;
use crate::timeline::Timeline;

/// Preview renderer for real-time frame composition
pub struct PreviewRenderer {
    width: u32,
    height: u32,
}

impl PreviewRenderer {
    /// Create a new preview renderer at the specified resolution
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Compose a single frame from the timeline at the given timestamp
    ///
    /// For MVP: Returns the video frame at the timestamp.
    /// Future: Will composite all visible tracks (video + text + effects).
    pub fn compose_frame(
        &self,
        timeline: &Timeline,
        time_ms: u64,
        video_frame: Option<FrameData>,
    ) -> FrameData {
        // Start with the video frame or a blank frame
        let mut frame = video_frame.unwrap_or_else(|| FrameData::blank(self.width, self.height));

        // Get all clips active at this timestamp
        let active_clips = timeline.get_clips_at_time(time_ms);

        // Apply opacity to clips that are not fully opaque
        for (_track, clip) in &active_clips {
            if clip.opacity < 1.0 {
                // Simple opacity blending - for MVP, just adjust alpha channel
                let alpha = (clip.opacity * 255.0) as u8;
                // In a real implementation, we'd blend with the frame below
                // For now, set alpha on the frame data
                for chunk in frame.data.chunks_exact_mut(4) {
                    chunk[3] = alpha;
                }
            }
        }

        // For MVP: Just return the video frame as-is
        // Phase 3+ will add text overlay compositing and effect application
        frame
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
