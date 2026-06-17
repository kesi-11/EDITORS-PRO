//! Renderer module - Composites timeline tracks into displayable frames
//!
//! The renderer takes a timeline state and a timestamp, then composites
//! all visible tracks (video, text, effects) into a single frame for display.
//! Clip effects are applied per-clip before compositing. Keyframe animations
//! (position, scale, rotation, opacity) are applied via affine transforms.
//!
//! ## GPU Acceleration
//!
//! When available, the `GpuRenderer` is used for effects processing,
//! providing 10-50x speedup over the CPU path. The renderer
//! automatically falls back to CPU processing when GPU is unavailable.

pub mod gpu;
pub mod shader;
pub mod shaders;

#[cfg(test)]
mod shader_bench;

use crate::decoder::FrameData;
use crate::effects::EffectsPipeline;
use crate::effects::text_rasterizer::TextRasterizer;
use crate::effects::text_render::TextOverlay;
use crate::renderer::gpu::GpuRenderer;
use crate::timeline::keyframe::InterpolatedValues;
use crate::timeline::Timeline;
use crate::timeline::track::TrackType;

/// Preview renderer for real-time frame composition
pub struct PreviewRenderer {
    width: u32,
    height: u32,
    text_rasterizer: TextRasterizer,
    gpu_renderer: GpuRenderer,
    /// Whether to attempt GPU-accelerated effects processing.
    /// Set to true after successful GPU init, false otherwise.
    gpu_available: bool,
}

impl PreviewRenderer {
    /// Create a new preview renderer at the specified resolution.
    ///
    /// GPU acceleration is lazily initialized on the first frame render.
    /// If GPU init fails, it silently falls back to CPU rendering.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            text_rasterizer: TextRasterizer::new(),
            gpu_renderer: GpuRenderer::new(),
            gpu_available: false,
        }
    }

    /// Attempt to initialize the GPU renderer.
    ///
    /// Call this once during engine startup. If it fails, the renderer
    /// will use CPU-only effects processing. This method is non-blocking
    /// and safe to call from any thread.
    pub async fn init_gpu(&mut self) {
        self.gpu_renderer.init_or_fallback().await;
        self.gpu_available = self.gpu_renderer.is_available();
        if self.gpu_available {
            log::info!("Preview renderer: GPU acceleration ENABLED");
        } else {
            log::info!("Preview renderer: using CPU-only effects");
        }
    }

    /// Compose a single frame from the timeline at the given timestamp.
    ///
    /// This method:
    /// 1. Takes the decoded video frame (or a blank frame)
    /// 2. Applies per-clip effects (GPU if available, CPU fallback)
    /// 3. Applies keyframe transforms (position, scale, rotation, opacity)
    /// 4. Applies opacity blending for non-opaque clips
    /// 5. Renders text overlays from text tracks
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

        // Apply per-clip effects, keyframe transforms, and opacity
        for (_track, clip) in &active_clips {
            // Apply effects from the clip's effect chain
            if !clip.effects.is_empty() {
                self.apply_effects(&mut frame, &clip.effects);
            }

            // Apply keyframe transforms if the clip has keyframes
            if clip.has_keyframes() {
                let values = clip.interpolate_at(time_ms);
                frame = apply_clip_transform(&frame, &values, self.width, self.height);
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

                let progress = if clip.duration_ms > 0 {
                    (time_ms - clip.start_ms) as f32 / clip.duration_ms as f32
                } else {
                    0.0
                };

                // Build TextOverlay from clip properties instead of hardcoded "Text"
                let content = clip.properties.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Text");
                let font_family = clip.properties.get("font_family")
                    .and_then(|v| v.as_str())
                    .unwrap_or("sans-serif");
                let font_size = clip.properties.get("font_size")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(48.0);
                let color_hex = clip.properties.get("color_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("#FFFFFF");

                let mut overlay = TextOverlay::simple(content);
                overlay.font_family = font_family.to_string();
                overlay.font_size = font_size;
                overlay.color = crate::effects::text_render::TextColor::with_hex(color_hex);

                // Apply position from clip properties if specified
                if let Some(pos_x) = clip.properties.get("position_x").and_then(|v| v.as_f64()) {
                    if let Some(pos_y) = clip.properties.get("position_y").and_then(|v| v.as_f64()) {
                        overlay.position = crate::effects::text_render::TextPosition::at(pos_x as f32, pos_y as f32);
                    }
                }
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

    /// Apply effects to a frame, using GPU acceleration when available.
    fn apply_effects(&mut self, frame: &mut FrameData, effects: &[crate::effects::Effect]) {
        if effects.is_empty() {
            return;
        }

        // Collect GPU-compatible effects for batch processing
        let mut gpu_effects: Vec<(String, Vec<f32>)> = Vec::new();
        let mut cpu_only_effects: Vec<crate::effects::Effect> = Vec::new();

        for effect in effects {
            if !effect.enabled {
                continue;
            }

            match effect.effect_type {
                crate::effects::EffectType::Filter => {
                    // Check if this effect has a GPU shader
                    let shader_name = effect.name.to_lowercase().replace(' ', "_");
                    if self.gpu_available
                        && self.gpu_renderer.gpu_accelerated_effects().contains(&shader_name.as_str())
                    {
                        let params: Vec<f32> = effect.parameters.iter()
                            .map(|p| p.value)
                            .collect();
                        gpu_effects.push((shader_name, params));
                    } else {
                        cpu_only_effects.push(effect.clone());
                    }
                }
                _ => {
                    // Non-filter effects (transitions, text, chroma key)
                    // are handled elsewhere
                    cpu_only_effects.push(effect.clone());
                }
            }
        }

        // Apply GPU effects in batch
        if !gpu_effects.is_empty() {
            if let Err(e) = self.gpu_renderer.apply_effects_chain(frame, &gpu_effects) {
                log::warn!("GPU effects failed ({}), falling back to CPU", e);
                // Fall back to CPU for all effects
                let pipeline = EffectsPipeline::new(effects.to_vec());
                pipeline.apply(&mut frame.data, frame.width, frame.height);
                return;
            }
        }

        // Apply remaining CPU-only effects
        if !cpu_only_effects.is_empty() {
            let pipeline = EffectsPipeline::new(cpu_only_effects);
            pipeline.apply(&mut frame.data, frame.width, frame.height);
        }
    }

    /// Apply effects from a clip's effect chain to frame data.
    pub fn apply_clip_effects(&self, frame: &mut FrameData, effects: &[crate::effects::Effect]) {
        if effects.is_empty() { return; }
        let pipeline = EffectsPipeline::new(effects.to_vec());
        pipeline.apply(&mut frame.data, frame.width, frame.height);
    }

    /// Resize a frame to the target dimensions using bilinear interpolation.
    pub fn resize_frame(&self, frame: &FrameData, target_width: u32, target_height: u32) -> FrameData {
        if frame.width == target_width && frame.height == target_height {
            return frame.clone();
        }

        let mut data = vec![0u8; (target_width * target_height * 4) as usize];
        let x_ratio = frame.width as f32 / target_width as f32;
        let y_ratio = frame.height as f32 / target_height as f32;

        // Bilinear interpolation for better quality during export
        for y in 0..target_height {
            for x in 0..target_width {
                let src_x = x as f32 * x_ratio;
                let src_y = y as f32 * y_ratio;

                let x0 = src_x.floor() as u32;
                let y0 = src_y.floor() as u32;
                let x1 = (x0 + 1).min(frame.width - 1);
                let y1 = (y0 + 1).min(frame.height - 1);

                let fx = src_x - x0 as f32;
                let fy = src_y - y0 as f32;

                let dst_idx = ((y * target_width + x) * 4) as usize;

                for ch in 0..4 {
                    let v00 = frame.data[((y0 * frame.width + x0) * 4 + ch as u32) as usize] as f32;
                    let v10 = frame.data[((y0 * frame.width + x1) * 4 + ch as u32) as usize] as f32;
                    let v01 = frame.data[((y1 * frame.width + x0) * 4 + ch as u32) as usize] as f32;
                    let v11 = frame.data[((y1 * frame.width + x1) * 4 + ch as u32) as usize] as f32;

                    let top = v00 * (1.0 - fx) + v10 * fx;
                    let bottom = v01 * (1.0 - fx) + v11 * fx;
                    let value = top * (1.0 - fy) + bottom * fy;

                    data[dst_idx + ch as usize] = value.clamp(0.0, 255.0) as u8;
                }
            }
        }

        FrameData {
            width: target_width,
            height: target_height,
            data,
            timestamp_ms: frame.timestamp_ms,
            is_keyframe: frame.is_keyframe,
            pooled: false,
        }
    }

    /// Check if GPU acceleration is currently active.
    pub fn is_gpu_accelerated(&self) -> bool {
        self.gpu_available
    }

    /// Get the name of the GPU adapter, if available.
    pub fn gpu_adapter_name(&self) -> Option<String> {
        self.gpu_renderer.adapter_name()
    }

    /// Get the name of the GPU backend (e.g., "Vulkan", "Metal"), if available.
    pub fn gpu_backend_name(&self) -> Option<String> {
        self.gpu_renderer.backend_name()
    }

    /// Get the list of effects that have GPU shader pipelines.
    pub fn gpu_accelerated_effects(&self) -> Vec<&str> {
        self.gpu_renderer.gpu_accelerated_effects()
    }

    /// Enable or disable GPU acceleration at runtime.
    ///
    /// When `enabled` is `false`, the renderer will use CPU-only effects
    /// even if a GPU is available. This is useful for debugging.
    pub fn set_gpu_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.gpu_available = false;
        } else {
            // Only re-enable if the GPU was actually initialized
            self.gpu_available = self.gpu_renderer.is_available();
        }
    }

    /// Get the rendering resolution.
    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Apply affine transforms (translation, scaling, rotation) to a frame
/// based on keyframe-interpolated values.
///
/// This creates a new frame with the transforms applied. The transform
/// pipeline is:
/// 1. Scale the frame by the scale factor
/// 2. Rotate the frame by the rotation angle (degrees)
/// 3. Translate (offset) the frame by position_x and position_y pixels
/// 4. Apply opacity from the interpolated values
///
/// The output frame has the same dimensions as the input. Pixels that
/// are transformed outside the frame boundary are clipped.
fn apply_clip_transform(
    frame: &FrameData,
    values: &InterpolatedValues,
    _output_width: u32,
    _output_height: u32,
) -> FrameData {
    let w = frame.width as usize;
    let h = frame.height as usize;
    let mut output = vec![0u8; w * h * 4];

    // Convert rotation from degrees to radians
    let angle_rad = values.rotation.to_radians();
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Scale factor
    let scale = values.scale;
    if scale <= 0.0 {
        // Fully scaled down — nothing visible
        return FrameData {
            width: frame.width,
            height: frame.height,
            data: output,
            timestamp_ms: frame.timestamp_ms,
            is_keyframe: frame.is_keyframe,
            pooled: false,
        };
    }

    // Center of the frame
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    // Translation offsets (in pixels)
    let tx = values.position_x;
    let ty = values.position_y;

    // Opacity multiplier
    let opacity = values.opacity.clamp(0.0, 1.0);

    // Inverse transform: for each output pixel, find the source pixel
    let inv_scale = 1.0 / scale;

    for dst_y in 0..h {
        for dst_x in 0..w {
            // Translate destination pixel to center-origin
            let dx = dst_x as f32 - cx - tx;
            let dy = dst_y as f32 - cy - ty;

            // Inverse rotate
            let rot_x = dx * cos_a + dy * sin_a;
            let rot_y = -dx * sin_a + dy * cos_a;

            // Inverse scale
            let src_x = rot_x * inv_scale + cx;
            let src_y = rot_y * inv_scale + cy;

            // Nearest-neighbor sampling for speed
            let sx = src_x.round() as isize;
            let sy = src_y.round() as isize;

            let dst_idx = (dst_y * w + dst_x) * 4;

            if sx >= 0 && sx < w as isize && sy >= 0 && sy < h as isize {
                let src_idx = (sy as usize * w + sx as usize) * 4;
                output[dst_idx] = (frame.data[src_idx] as f32 * opacity) as u8;
                output[dst_idx + 1] = (frame.data[src_idx + 1] as f32 * opacity) as u8;
                output[dst_idx + 2] = (frame.data[src_idx + 2] as f32 * opacity) as u8;
                output[dst_idx + 3] = (frame.data[src_idx + 3] as f32 * opacity) as u8;
            }
            // else: leave as 0 (transparent black)
        }
    }

    FrameData {
        width: frame.width,
        height: frame.height,
        data: output,
        timestamp_ms: frame.timestamp_ms,
        is_keyframe: frame.is_keyframe,
        pooled: false,
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

    /// Whether GPU effects should be used at this quality level.
    pub fn use_gpu(&self) -> bool {
        match self {
            RenderQuality::PreviewLow => false, // CPU is fast enough for small frames
            RenderQuality::PreviewMedium => true,
            RenderQuality::ExportHigh => true,
            RenderQuality::ExportUltra => true,
        }
    }
}
