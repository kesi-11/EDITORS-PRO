/// GPU and software rendering pipeline.
/// Compositor handles frame rendering with effect chains.

use serde::{Deserialize, Serialize};

/// Render quality presets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderQuality {
    Draft,    // 360p preview
    Low,      // 480p
    Medium,   // 720p
    High,     // 1080p
    Ultra,    // 4K
}

/// Pixel format for rendered output.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8,
    Rgba16f,
    Rgba32f,
}

/// A single composited frame ready for display or encoding.
#[derive(Debug, Clone)]
pub struct CompositedFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp_ms: f64,
}

/// Compositor that combines video layers with effects.
pub struct Compositor {
    width: u32,
    height: u32,
    quality: RenderQuality,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            quality: RenderQuality::High,
        }
    }

    pub fn set_quality(&mut self, quality: RenderQuality) {
        self.quality = quality;
    }

    pub fn get_quality(&self) -> RenderQuality {
        self.quality
    }

    /// Composite a single frame by blending all layers.
    /// Returns RGBA pixel data.
    pub fn composite_frame(&self, layers: &[Vec<u8>], opacity: &[f32]) -> Vec<u8> {
        let pixel_count = (self.width as usize) * (self.height as usize) * 4;
        let mut output = vec![0u8; pixel_count];

        for (layer_idx, layer) in layers.iter().enumerate() {
            let alpha = opacity.get(layer_idx).copied().unwrap_or(1.0);
            for (i, byte) in layer.iter().enumerate().take(pixel_count) {
                let blended = (*byte as f32 * alpha) + (output[i] as f32 * (1.0 - alpha));
                output[i] = blended.min(255.0) as u8;
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_new() {
        let c = Compositor::new(1920, 1080);
        assert_eq!(c.width, 1920);
        assert_eq!(c.height, 1080);
    }

    #[test]
    fn test_quality_setting() {
        let mut c = Compositor::new(1920, 1080);
        c.set_quality(RenderQuality::Ultra);
        assert_eq!(c.get_quality(), RenderQuality::Ultra);
    }

    #[test]
    fn test_composite_empty() {
        let c = Compositor::new(4, 4);
        let result = c.composite_frame(&[], &[]);
        assert_eq!(result.len(), 64); // 4x4x4
    }

    #[test]
    fn test_composite_single_layer() {
        let c = Compositor::new(2, 2);
        let layer = vec![128u8; 16]; // 2x2x4
        let result = c.composite_frame(&[layer], &[1.0]);
        assert_eq!(result[0], 128);
    }
}
