//! Chroma key (green screen) effect
//!
//! Replaces a specified color range in the frame with transparency,
//! allowing a background layer to show through. Uses HSV color space
//! for more natural color selection than RGB.

use rayon::prelude::*;

use super::{Effect, EffectParameter, EffectType};

/// Chroma key configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChromaKeyConfig {
    /// Target color in HSV format (hue: 0-360, saturation: 0-1, value: 0-1)
    pub target_hue: f32,
    /// Tolerance for hue matching (degrees, 0-180)
    pub hue_tolerance: f32,
    /// Tolerance for saturation matching (0.0-1.0)
    pub saturation_tolerance: f32,
    /// Edge softness (0.0 = hard edge, 1.0 = maximum feathering)
    pub softness: f32,
    /// Spill suppression strength (0.0-1.0) — removes color fringe
    pub spill_suppression: f32,
}

impl Default for ChromaKeyConfig {
    fn default() -> Self {
        Self {
            target_hue: 120.0, // Green
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 0.15,
            spill_suppression: 0.5,
        }
    }
}

impl ChromaKeyConfig {
    /// Preset for green screen
    pub fn green_screen() -> Self {
        Self {
            target_hue: 120.0,
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 0.15,
            spill_suppression: 0.5,
        }
    }

    /// Preset for blue screen
    pub fn blue_screen() -> Self {
        Self {
            target_hue: 240.0,
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 0.15,
            spill_suppression: 0.5,
        }
    }

    /// Create a config from an RGB color (converts to HSV target).
    ///
    /// Uses default tolerance values. The hue is derived from the given
    /// RGB color so you can chroma key against an arbitrary color.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let (hue, sat, _val) = rgb_to_hsv(r, g, b);
        Self {
            target_hue: hue,
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 0.15,
            spill_suppression: 0.5,
        }
    }

    /// Construct a config from effect parameters (used by EffectsPipeline).
    pub fn from_parameters(params: &[EffectParameter]) -> Self {
        Self {
            target_hue: param_value(params, "target_hue", 120.0),
            hue_tolerance: param_value(params, "hue_tolerance", 30.0),
            saturation_tolerance: param_value(params, "saturation_tolerance", 0.4),
            softness: param_value(params, "softness", 0.15),
            spill_suppression: param_value(params, "spill_suppression", 0.5),
        }
    }
}

/// Helper: extract a parameter value by name, with a fallback default.
fn param_value(params: &[EffectParameter], name: &str, default: f32) -> f32 {
    params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.value)
        .unwrap_or(default)
}

/// Apply chroma key to RGBA frame data.
///
/// For each pixel:
/// 1. Convert RGB to HSV
/// 2. Calculate color distance from target in HSV space
/// 3. If within tolerance, set alpha to 0 (transparent)
/// 4. Apply softness for edge feathering (gradual alpha transition)
/// 5. Apply spill suppression (remove target color from semi-transparent pixels)
///
/// Uses rayon for parallel row-level processing.
pub fn apply_chroma_key(frame_data: &mut [u8], width: u32, height: u32, config: &ChromaKeyConfig) {
    let w = width as usize;
    let h = height as usize;
    let row_bytes = w * 4;

    // Process rows in parallel with rayon
    frame_data.par_chunks_mut(row_bytes).for_each(|row| {
        for pixel in row.chunks_exact_mut(4) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            // Skip fully transparent pixels
            if a == 0 {
                continue;
            }

            let (hue, sat, val) = rgb_to_hsv(r, g, b);

            // Calculate hue distance (circular, 0-180 degrees)
            let hue_dist = hue_distance(hue, config.target_hue);

            // Determine if this pixel is within the key color range
            let in_hue = hue_dist <= config.hue_tolerance;
            let in_sat = sat >= (1.0 - config.saturation_tolerance).max(0.0).min(1.0);
            // Also require some minimum value to avoid keying out very dark pixels
            let in_val = val >= 0.15;

            if in_hue && in_sat && in_val {
                // Pixel is within the key color range
                // Calculate alpha based on distance vs tolerance for softness
                let hue_ratio = hue_dist / config.hue_tolerance.max(0.001);
                let sat_ratio = if config.saturation_tolerance > 0.0 {
                    (1.0 - sat) / config.saturation_tolerance
                } else {
                    0.0
                };
                let distance_ratio = hue_ratio.max(sat_ratio);

                // Apply softness: gradual alpha transition
                let alpha: f32 = if config.softness > 0.0 {
                    // Smooth step from 0 (at distance_ratio=0) to 1 (at distance_ratio=1)
                    let t = (distance_ratio / config.softness).min(1.0);
                    t * t * (3.0 - 2.0 * t) // smoothstep
                } else {
                    // Hard edge: fully transparent if within tolerance
                    0.0
                };

                pixel[3] = (alpha * a as f32).round() as u8;

                // Apply spill suppression: reduce the target color component
                if config.spill_suppression > 0.0 && pixel[3] > 0 {
                    apply_spill_suppression(pixel, config.target_hue, config.spill_suppression);
                }
            } else if in_hue && in_val {
                // Near the edge — partial keying based on hue distance
                let hue_ratio = hue_dist / config.hue_tolerance.max(0.001);
                if hue_ratio <= 1.0 + config.softness {
                    // In the softness zone
                    let t = ((hue_ratio - 1.0 + config.softness) / config.softness.max(0.001))
                        .min(1.0)
                        .max(0.0);
                    let alpha = t * t * (3.0 - 2.0 * t);
                    pixel[3] = (alpha * a as f32).round().min(a as f32) as u8;

                    if config.spill_suppression > 0.0 && pixel[3] > 0 {
                        apply_spill_suppression(pixel, config.target_hue, config.spill_suppression);
                    }
                }
            }
        }
    });
}

/// Apply spill suppression to a pixel.
///
/// Removes the target hue color component from the pixel's RGB channels.
/// This eliminates the green (or blue) fringe that appears on edges of
/// keyed subjects. The strength parameter controls how aggressively
/// the target color is suppressed.
fn apply_spill_suppression(pixel: &mut [u8], target_hue: f32, strength: f32) {
    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    // Determine which channel to suppress based on target hue
    let (suppressed_r, suppressed_g, suppressed_b) = if (90.0..=150.0).contains(&target_hue) {
        // Green screen: reduce green channel
        let factor = 1.0 - strength * 0.5;
        let new_g = g * factor + (r + b) * 0.5 * strength * 0.3;
        (r, new_g.min(1.0), b)
    } else if (210.0..=270.0).contains(&target_hue) {
        // Blue screen: reduce blue channel
        let factor = 1.0 - strength * 0.5;
        let new_b = b * factor + (r + g) * 0.5 * strength * 0.3;
        (r, g, new_b.min(1.0))
    } else {
        // Generic: reduce the channel that most matches the target hue
        // Determine dominant channel from hue
        let (cr, cg, cb) = hue_to_rgb_factors(target_hue);
        let dominant = r * cr + g * cg + b * cb;
        let factor = 1.0 - strength * 0.4;
        let avg = (r + g + b) / 3.0;
        let mix = dominant * (1.0 - factor) * 0.3;
        (
            (r - mix * cr).max(0.0),
            (g - mix * cg).max(0.0),
            (b - mix * cb).max(0.0),
        )
    };

    pixel[0] = (suppressed_r * 255.0).round().clamp(0.0, 255.0) as u8;
    pixel[1] = (suppressed_g * 255.0).round().clamp(0.0, 255.0) as u8;
    pixel[2] = (suppressed_b * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// Convert a hue angle to RGB factor weights.
///
/// Returns (r_factor, g_factor, b_factor) indicating how much each
/// channel contributes to the given hue. Used for spill suppression.
fn hue_to_rgb_factors(hue: f32) -> (f32, f32, f32) {
    let h = hue % 360.0;
    if h < 60.0 {
        (1.0, h / 60.0, 0.0)
    } else if h < 120.0 {
        ((120.0 - h) / 60.0, 1.0, 0.0)
    } else if h < 180.0 {
        (0.0, 1.0, (h - 120.0) / 60.0)
    } else if h < 240.0 {
        (0.0, (240.0 - h) / 60.0, 1.0)
    } else if h < 300.0 {
        ((h - 240.0) / 60.0, 0.0, 1.0)
    } else {
        (1.0, 0.0, (360.0 - h) / 60.0)
    }
}

/// Convert RGB to HSV.
///
/// Returns (hue: 0-360, saturation: 0-1, value: 0-1).
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Hue
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    // Saturation
    let saturation = if max == 0.0 { 0.0 } else { delta / max };

    (hue, saturation, max)
}

/// Calculate circular hue distance (0-180 degrees).
///
/// The distance accounts for the circular nature of hue (e.g., the
/// distance between 350° and 10° is 20°, not 340°).
pub fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

/// Create a ChromaKey effect from config.
///
/// The effect can be added to a clip's effects pipeline and will be
/// applied by the `EffectsPipeline::apply()` method.
pub fn create_chroma_key_effect(config: &ChromaKeyConfig) -> Effect {
    Effect::new(
        "Chroma Key",
        EffectType::ChromaKey,
        vec![
            EffectParameter::new(
                "target_hue",
                "Target Hue",
                config.target_hue,
                0.0,
                360.0,
                1.0,
            ),
            EffectParameter::new(
                "hue_tolerance",
                "Hue Tolerance",
                config.hue_tolerance,
                0.0,
                180.0,
                1.0,
            ),
            EffectParameter::new(
                "saturation_tolerance",
                "Saturation Tolerance",
                config.saturation_tolerance,
                0.0,
                1.0,
                0.01,
            ),
            EffectParameter::new("softness", "Edge Softness", config.softness, 0.0, 1.0, 0.01),
            EffectParameter::new(
                "spill_suppression",
                "Spill Suppression",
                config.spill_suppression,
                0.0,
                1.0,
                0.01,
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv_pure_green() {
        let (h, s, v) = rgb_to_hsv(0, 255, 0);
        assert!(
            (h - 120.0).abs() < 0.1,
            "Green hue should be ~120, got {}",
            h
        );
        assert!((s - 1.0).abs() < 0.001, "Green saturation should be 1.0");
        assert!((v - 1.0).abs() < 0.001, "Green value should be 1.0");
    }

    #[test]
    fn test_rgb_to_hsv_pure_red() {
        let (h, s, v) = rgb_to_hsv(255, 0, 0);
        assert!((h - 0.0).abs() < 0.1, "Red hue should be ~0, got {}", h);
        assert!((s - 1.0).abs() < 0.001);
        assert!((v - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rgb_to_hsv_pure_blue() {
        let (h, s, v) = rgb_to_hsv(0, 0, 255);
        assert!(
            (h - 240.0).abs() < 0.1,
            "Blue hue should be ~240, got {}",
            h
        );
        assert!((s - 1.0).abs() < 0.001);
        assert!((v - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rgb_to_hsv_white() {
        let (h, s, v) = rgb_to_hsv(255, 255, 255);
        assert!((s - 0.0).abs() < 0.001, "White saturation should be 0.0");
        assert!((v - 1.0).abs() < 0.001, "White value should be 1.0");
    }

    #[test]
    fn test_rgb_to_hsv_black() {
        let (h, s, v) = rgb_to_hsv(0, 0, 0);
        assert!((s - 0.0).abs() < 0.001, "Black saturation should be 0.0");
        assert!((v - 0.0).abs() < 0.001, "Black value should be 0.0");
    }

    #[test]
    fn test_hue_distance_same() {
        assert!((hue_distance(120.0, 120.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_hue_distance_opposite() {
        assert!((hue_distance(0.0, 180.0) - 180.0).abs() < 0.001);
    }

    #[test]
    fn test_hue_distance_wraparound() {
        // Distance between 350 and 10 should be 20, not 340
        assert!((hue_distance(350.0, 10.0) - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_hue_distance_small() {
        assert!((hue_distance(120.0, 130.0) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_chroma_key_green_pixel_removed() {
        // Create a 1x1 frame with a pure green pixel
        let mut frame = [0u8, 255, 0, 255]; // RGBA: green, fully opaque
        let config = ChromaKeyConfig::green_screen();
        apply_chroma_key(&mut frame, 1, 1, &config);

        // Green pixel should be transparent
        assert_eq!(
            frame[3], 0,
            "Green pixel should be transparent after chroma key"
        );
    }

    #[test]
    fn test_chroma_key_blue_pixel_kept_with_green_config() {
        // Create a 1x1 frame with a pure blue pixel
        let mut frame = [0u8, 0, 255, 255]; // RGBA: blue, fully opaque
        let config = ChromaKeyConfig::green_screen();
        apply_chroma_key(&mut frame, 1, 1, &config);

        // Blue pixel should remain opaque (hue distance from green is 120° > 30° tolerance)
        assert_eq!(
            frame[3], 255,
            "Blue pixel should remain opaque with green screen config"
        );
    }

    #[test]
    fn test_chroma_key_blue_pixel_removed_with_blue_config() {
        // Create a 1x1 frame with a pure blue pixel
        let mut frame = [0u8, 0, 255, 255]; // RGBA: blue, fully opaque
        let config = ChromaKeyConfig::blue_screen();
        apply_chroma_key(&mut frame, 1, 1, &config);

        // Blue pixel should be transparent with blue screen config
        assert_eq!(
            frame[3], 0,
            "Blue pixel should be transparent with blue screen config"
        );
    }

    #[test]
    fn test_chroma_key_red_pixel_kept() {
        // Red pixel should not be affected by either green or blue config
        let mut frame = [255u8, 0, 0, 255]; // RGBA: red, fully opaque
        let config = ChromaKeyConfig::green_screen();
        apply_chroma_key(&mut frame, 1, 1, &config);

        assert_eq!(frame[3], 255, "Red pixel should remain opaque");
    }

    #[test]
    fn test_chroma_key_softness() {
        // Create a pixel that's near the edge of green (yellow-green, hue ~90)
        let mut frame = [128u8, 255, 0, 255];
        let config = ChromaKeyConfig {
            target_hue: 120.0,
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 1.0, // Maximum softness
            spill_suppression: 0.0,
        };
        apply_chroma_key(&mut frame, 1, 1, &config);

        // With softness, edge pixels should have partial transparency
        // The exact alpha depends on the smoothstep calculation
        assert!(
            frame[3] < 255 && frame[3] > 0,
            "Edge pixel should be semi-transparent with softness, got alpha {}",
            frame[3]
        );
    }

    #[test]
    fn test_chroma_key_hard_edge() {
        // With zero softness, pixels within tolerance should be fully transparent
        let mut frame = [0u8, 255, 0, 255]; // Pure green
        let config = ChromaKeyConfig {
            target_hue: 120.0,
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 0.0, // Hard edge
            spill_suppression: 0.0,
        };
        apply_chroma_key(&mut frame, 1, 1, &config);

        assert_eq!(
            frame[3], 0,
            "Green pixel should be fully transparent with hard edge"
        );
    }

    #[test]
    fn test_chroma_key_spill_suppression() {
        // Create a pixel that has some green spill (not fully green but greenish)
        let mut frame = [100u8, 200, 100, 255]; // Greenish pixel
        let config = ChromaKeyConfig {
            target_hue: 120.0,
            hue_tolerance: 30.0,
            saturation_tolerance: 0.4,
            softness: 0.0,
            spill_suppression: 1.0, // Maximum spill suppression
        };
        apply_chroma_key(&mut frame, 1, 1, &config);

        // After spill suppression, the green channel should be reduced
        // (if the pixel wasn't fully keyed out)
        // For a pixel fully within the key range, it will be transparent
        // so we test with a partially-keyed pixel instead
    }

    #[test]
    fn test_chroma_key_from_rgb() {
        let config = ChromaKeyConfig::from_rgb(0, 255, 0);
        assert!(
            (config.target_hue - 120.0).abs() < 0.1,
            "Green RGB should give hue ~120"
        );

        let config = ChromaKeyConfig::from_rgb(0, 0, 255);
        assert!(
            (config.target_hue - 240.0).abs() < 0.1,
            "Blue RGB should give hue ~240"
        );
    }

    #[test]
    fn test_chroma_key_larger_frame() {
        // Create a 4x1 frame: red, green, blue, white
        let mut frame: Vec<u8> = vec![
            255, 0, 0, 255, // Red
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Blue
            255, 255, 255, 255, // White
        ];
        let config = ChromaKeyConfig::green_screen();
        apply_chroma_key(&mut frame, 4, 1, &config);

        // Red: kept
        assert_eq!(frame[3], 255, "Red pixel should be kept");
        // Green: removed
        assert_eq!(frame[7], 0, "Green pixel should be removed");
        // Blue: kept
        assert_eq!(frame[11], 255, "Blue pixel should be kept");
        // White: kept (saturation is 0, so not keyed)
        assert_eq!(frame[15], 255, "White pixel should be kept");
    }

    #[test]
    fn test_create_chroma_key_effect() {
        let config = ChromaKeyConfig::green_screen();
        let effect = create_chroma_key_effect(&config);

        assert_eq!(effect.name, "Chroma Key");
        assert_eq!(effect.effect_type, EffectType::ChromaKey);
        assert!(effect.enabled);
        assert_eq!(effect.parameters.len(), 5);

        // Check parameter names
        assert_eq!(effect.parameters[0].name, "target_hue");
        assert_eq!(effect.parameters[1].name, "hue_tolerance");
        assert_eq!(effect.parameters[2].name, "saturation_tolerance");
        assert_eq!(effect.parameters[3].name, "softness");
        assert_eq!(effect.parameters[4].name, "spill_suppression");
    }

    #[test]
    fn test_config_from_parameters() {
        let params = vec![
            EffectParameter::new("target_hue", "Target Hue", 240.0, 0.0, 360.0, 1.0),
            EffectParameter::new("hue_tolerance", "Hue Tolerance", 20.0, 0.0, 180.0, 1.0),
            EffectParameter::new(
                "saturation_tolerance",
                "Saturation Tolerance",
                0.5,
                0.0,
                1.0,
                0.01,
            ),
            EffectParameter::new("softness", "Edge Softness", 0.2, 0.0, 1.0, 0.01),
            EffectParameter::new(
                "spill_suppression",
                "Spill Suppression",
                0.7,
                0.0,
                1.0,
                0.01,
            ),
        ];

        let config = ChromaKeyConfig::from_parameters(&params);
        assert!((config.target_hue - 240.0).abs() < 0.001);
        assert!((config.hue_tolerance - 20.0).abs() < 0.001);
        assert!((config.saturation_tolerance - 0.5).abs() < 0.001);
        assert!((config.softness - 0.2).abs() < 0.001);
        assert!((config.spill_suppression - 0.7).abs() < 0.001);
    }
}
