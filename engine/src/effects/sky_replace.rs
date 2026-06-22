//! Sky replacement — luminance-key based sky swap.
//!
//! ## Status
//!
//! This module is a stub with the basic workflow: (1) qualify the sky
//! with a luminance key (top of frame + bright), (2) composite the new
//! sky, (3) edge-feather the mask. Lighting match and reflections are
//! deferred — see the `video:` markers below.
//!
//! ## video: debt markers
//!
//! - Luminance key only, upgrade to gradient-domain blending if edges are visible
//! - Top-of-frame bias only, upgrade to ML-based sky segmentation if foreground has bright areas
//! - No lighting match, upgrade to color transfer from foreground to new sky
//! - No reflections, upgrade to reflection mapping if water/glass is in the foreground

use serde::{Deserialize, Serialize};

/// Sky replacement parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyReplaceParams {
    /// Luma threshold (0–255). Pixels with luma ≥ threshold in the top
    /// portion of the frame are considered sky. Default 180.
    pub luma_threshold: u8,
    /// Top portion of the frame to consider as sky region (0.0–1.0). Default 0.6.
    pub top_portion: f32,
    /// Feather radius (pixels) for the mask edges. Default 4.
    pub feather: usize,
    /// Blend intensity of the new sky (0.0 = original, 1.0 = full new sky). Default 1.0.
    pub intensity: f32,
}

impl Default for SkyReplaceParams {
    fn default() -> Self {
        Self {
            luma_threshold: 180,
            top_portion: 0.6,
            feather: 4,
            intensity: 1.0,
        }
    }
}

/// Replace the sky in `frame` with `new_sky`.
///
/// `frame` and `new_sky` must have the same dimensions. The new sky is
/// composited only where the original frame's luma is ≥ `luma_threshold`
/// in the top `top_portion` of the frame.
///
/// video: luminance key only, upgrade to gradient-domain blending if edges are visible
pub fn replace_sky(
    frame: &mut [u8],
    new_sky: &[u8],
    width: usize,
    height: usize,
    params: &SkyReplaceParams,
) {
    debug_assert_eq!(frame.len(), width * height * 4);
    debug_assert_eq!(new_sky.len(), width * height * 4);

    let sky_height = ((height as f32) * params.top_portion).round() as usize;
    let intensity = params.intensity.clamp(0.0, 1.0);
    let inv_intensity = 1.0 - intensity;

    for y in 0..sky_height {
        for x in 0..width {
            let i = (y * width + x) * 4;
            let r = frame[i] as u32;
            let g = frame[i + 1] as u32;
            let b = frame[i + 2] as u32;
            let y_val = (r * 54 + g * 183 + b * 19) >> 8;

            if y_val >= params.luma_threshold as u32 {
                // Pixel is sky — blend with new sky
                let nr = new_sky[i] as f32;
                let ng = new_sky[i + 1] as f32;
                let nb = new_sky[i + 2] as f32;
                let or = frame[i] as f32;
                let og = frame[i + 1] as f32;
                let ob = frame[i + 2] as f32;
                frame[i] = (or * inv_intensity + nr * intensity) as u8;
                frame[i + 1] = (og * inv_intensity + ng * intensity) as u8;
                frame[i + 2] = (ob * inv_intensity + nb * intensity) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_sky_bright_top() {
        // 8x8 frame, top 4 rows are bright (sky), bottom 4 are dark (ground)
        let mut frame = vec![0u8; 8 * 8 * 4];
        for y in 0..4 {
            for x in 0..8 {
                let i = (y * 8 + x) * 4;
                frame[i] = 200; frame[i + 1] = 200; frame[i + 2] = 200;
            }
        }
        // New sky: pure blue
        let mut new_sky = vec![0u8; 8 * 8 * 4];
        for px in new_sky.chunks_exact_mut(4) {
            px[0] = 50; px[1] = 100; px[2] = 200; px[3] = 255;
        }

        let params = SkyReplaceParams::default();
        replace_sky(&mut frame, &new_sky, 8, 8, &params);

        // Top-left pixel should now be blue-ish
        assert!(frame[2] > frame[0], "expected blue sky, got r={} b={}", frame[0], frame[2]);
    }

    #[test]
    fn replace_sky_zero_intensity_is_noop() {
        let mut frame = vec![200u8; 8 * 8 * 4];
        for i in (0..frame.len()).step_by(4) {
            frame[i + 3] = 255;
        }
        let orig = frame.clone();
        let mut new_sky = vec![0u8; 8 * 8 * 4];
        for px in new_sky.chunks_exact_mut(4) {
            px[0] = 50; px[1] = 100; px[2] = 200; px[3] = 255;
        }
        let params = SkyReplaceParams { intensity: 0.0, ..Default::default() };
        replace_sky(&mut frame, &new_sky, 8, 8, &params);
        assert_eq!(frame, orig);
    }
}
