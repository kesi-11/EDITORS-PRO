//! Color legalizer — clamps color values to broadcast-legal range.
//!
//! Rec.709 SDI/broadcast legal range:
//! - Luma: 16–235 (8-bit), 64–940 (10-bit)
//! - Chroma: 16–240 (8-bit), 64–960 (10-bit)
//!
//! Full-range RGB (0–255) is for web/JPEG/computer-graphics only.
//! Never ship to broadcast without legalizing.
//!
//! ## video: debt markers
//!
//! - 8-bit clamping, upgrade to 10-bit if banding appears after legalization
//! - Hard clamp by default, upgrade to soft-clip with knee if highlight roll-off is needed
//! - No gamut compression (Rec.709 only), upgrade to Rec.2020 gamut compression for HDR delivery

use serde::{Deserialize, Serialize};

/// Legalizer parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalizerParams {
    /// If true, soft-clip at the knee instead of hard clamping.
    /// Soft-clip preserves gradient detail near the limits.
    pub soft_clip: bool,
    /// Knee point (0.0–1.0). Below this, pass-through. Above, soft-clip.
    /// Default 0.9 = soft-clip starts at 90% of legal range.
    pub knee: f32,
}

impl Default for LegalizerParams {
    fn default() -> Self {
        Self {
            soft_clip: true,
            knee: 0.9,
        }
    }
}

/// Legalize a packed RGBA8 frame to Rec.709 broadcast-legal range.
///
/// - Luma is clamped/soft-clipped to [16, 235].
/// - Chroma is clamped to [16, 240].
/// - Alpha is preserved.
///
/// video: 8-bit clamping, upgrade to 10-bit if banding appears after legalization
pub fn legalize_rgba8(pixels: &mut [u8], params: &LegalizerParams) {
    for px in pixels.chunks_exact_mut(4) {
        let r = px[0] as i32;
        let g = px[1] as i32;
        let b = px[2] as i32;

        // Compute YCbCr
        let y_val = (54 * r + 183 * g + 19 * b) >> 8;
        let cb = (-43 * r - 85 * g + 128 * b + 128 * 256) >> 8;
        let cr = (128 * r - 107 * g - 21 * b + 128 * 256) >> 8;

        // Legalize Y to [16, 235]
        let y_legal = legalize_channel(y_val, 16, 235, params);
        // Legalize Cb, Cr to [16, 240]
        let cb_legal = legalize_channel(cb, 16, 240, params);
        let cr_legal = legalize_channel(cr, 16, 240, params);

        // Convert back to RGB
        let y_new = y_legal as i32 - 16;
        let cb_new = cb_legal as i32 - 128;
        let cr_new = cr_legal as i32 - 128;

        // Rec.709 inverse:
        //   R = 1.164(Y - 16) + 1.793(Cr - 128)
        //   G = 1.164(Y - 16) - 0.534(Cr - 128) - 0.213(Cb - 128)
        //   B = 1.164(Y - 16) + 2.115(Cb - 128)
        // Using scaled integers (×256):
        let r_new = (298 * y_new + 459 * cr_new) >> 8;
        let g_new = (298 * y_new - 137 * cr_new - 55 * cb_new) >> 8;
        let b_new = (298 * y_new + 541 * cb_new) >> 8;

        // Clamp RGB to [0, 255] (full range — the legal range is enforced via YCbCr)
        px[0] = r_new.clamp(0, 255) as u8;
        px[1] = g_new.clamp(0, 255) as u8;
        px[2] = b_new.clamp(0, 255) as u8;
        // alpha unchanged
    }
}

/// Legalize a single channel value to `[lo, hi]`.
///
/// If `soft_clip` is true and `knee < 1.0`, values above `lo + knee * (hi - lo)`
/// (and below `lo + (1 - knee) * (hi - lo)`) are soft-clipped with a smooth curve
/// rather than hard-clamped.
fn legalize_channel(value: i32, lo: i32, hi: i32, params: &LegalizerParams) -> i32 {
    if !params.soft_clip {
        return value.clamp(lo, hi);
    }
    let range = (hi - lo) as f32;
    let knee = params.knee.clamp(0.5, 0.99);
    let high_knee = lo as f32 + knee * range;
    let low_knee = lo as f32 + (1.0 - knee) * range;

    let v = value as f32;
    let out = if v > high_knee {
        // Soft-clip the top
        let excess = v - high_knee;
        let headroom = (hi as f32) - high_knee;
        let compressed = headroom * (1.0 - (-excess / headroom).exp());
        high_knee + compressed
    } else if v < low_knee {
        // Soft-clip the bottom
        let excess = low_knee - v;
        let headroom = low_knee - (lo as f32);
        let compressed = headroom * (1.0 - (-excess / headroom).exp());
        low_knee - compressed
    } else {
        v
    };
    out.round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legalize_pure_black() {
        let mut pixels = vec![0u8, 0, 0, 255];
        legalize_rgba8(&mut pixels, &LegalizerParams::default());
        // Y of pure black is 0, which is below 16 — should be lifted to ~16
        let y = (54 * pixels[0] as i32 + 183 * pixels[1] as i32 + 19 * pixels[2] as i32) >> 8;
        assert!(y >= 14 && y <= 20, "expected legal black ~16, got y={}", y);
    }

    #[test]
    fn legalize_pure_white() {
        let mut pixels = vec![255u8, 255, 255, 255];
        legalize_rgba8(&mut pixels, &LegalizerParams::default());
        // Y of pure white is 255, which is above 235 — should be lowered
        let y = (54 * pixels[0] as i32 + 183 * pixels[1] as i32 + 19 * pixels[2] as i32) >> 8;
        assert!(y <= 238, "expected legal white ~235, got y={}", y);
    }

    #[test]
    fn legalize_mid_gray_unchanged() {
        let mut pixels = vec![128u8, 128, 128, 255];
        let orig_y = (54 * 128 + 183 * 128 + 19 * 128) >> 8;
        legalize_rgba8(&mut pixels, &LegalizerParams::default());
        let new_y = (54 * pixels[0] as i32 + 183 * pixels[1] as i32 + 19 * pixels[2] as i32) >> 8;
        // Mid-gray (Y ~128) is well inside legal range — should be unchanged
        assert!((new_y - orig_y).abs() <= 2, "expected ~unchanged, orig={} new={}", orig_y, new_y);
    }

    #[test]
    fn hard_clamp_mode() {
        let params = LegalizerParams { soft_clip: false, knee: 0.9 };
        let mut pixels = vec![255u8, 255, 255, 255];
        legalize_rgba8(&mut pixels, &params);
        let y = (54 * pixels[0] as i32 + 183 * pixels[1] as i32 + 19 * pixels[2] as i32) >> 8;
        // Hard-clamp mode should pull Y down to exactly 235 (with rounding)
        assert!(y <= 236, "expected ~235, got y={}", y);
    }
}
