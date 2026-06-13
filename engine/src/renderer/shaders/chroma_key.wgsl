/// Chroma key compositing compute shader
///
/// Bind group 0:
///   [0] uniform Params — target_hue, hue_tolerance, saturation_tolerance, softness
///   [1] input texture_2d<f32>
///   [2] output texture_storage_2d<rgba8unorm, write>
///   [3] uniform SpillParams — spill_suppression, pad1, pad2, pad3
///
/// Converts each pixel from RGB to HSV, calculates hue distance from
/// the target color, and modifies the alpha channel based on the
/// distance and softness parameters. Pixels within the key color
/// range become transparent (alpha = 0) with smoothstep feathering
/// at the edges. Spill suppression removes key-color fringing from
/// semi-transparent edge pixels.

struct Params {
    target_hue: f32,
    hue_tolerance: f32,
    saturation_tolerance: f32,
    softness: f32,
};

struct SpillParams {
    spill_suppression: f32,
    pad1: f32,
    pad2: f32,
    pad3: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> spill_params: SpillParams;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let color = textureLoad(input_tex, id.xy, 0);
    var result = color;

    // Skip fully transparent pixels
    if (color.a <= 0.0) {
        textureStore(output_tex, id.xy, result);
        return;
    }

    // ── Convert RGB to HSV ───────────────────────────────────────────
    let r = color.r;
    let g = color.g;
    let b = color.b;

    let max_c = max(r, max(g, b));
    let min_c = min(r, min(g, b));
    let delta = max_c - min_c;

    // Hue (0–360)
    var hue: f32 = 0.0;
    if (delta > 0.0) {
        if (max_c == r) {
            hue = 60.0 * (((g - b) / delta) % 6.0);
        } else if (max_c == g) {
            hue = 60.0 * (((b - r) / delta) + 2.0);
        } else {
            hue = 60.0 * (((r - g) / delta) + 4.0);
        }
        if (hue < 0.0) {
            hue = hue + 360.0;
        }
    }

    // Saturation (0–1)
    let saturation = select(delta / max_c, 0.0, max_c == 0.0);

    // Value (0–1) — same as max_c
    let value = max_c;

    // ── Calculate circular hue distance (0–180) ─────────────────────
    let diff = abs(hue - params.target_hue);
    let hue_dist = min(diff, 360.0 - diff);

    // ── Determine if pixel is within key color range ────────────────
    let in_hue = hue_dist <= params.hue_tolerance;
    let sat_threshold = clamp(1.0 - params.saturation_tolerance, 0.0, 1.0);
    let in_sat = saturation >= sat_threshold;
    // Require some minimum value to avoid keying very dark pixels
    let in_val = value >= 0.15;

    if (in_hue && in_sat && in_val) {
        // ── Pixel is within the key color range ─────────────────────
        let hue_ratio = hue_dist / max(params.hue_tolerance, 0.001);
        let sat_ratio = select(
            (1.0 - saturation) / params.saturation_tolerance,
            0.0,
            params.saturation_tolerance <= 0.0,
        );
        let distance_ratio = max(hue_ratio, sat_ratio);

        // Apply softness: smoothstep from 0 (at distance_ratio=0) to 1 (at distance_ratio=1)
        let alpha: f32 = if params.softness > 0.0 {
            let t = clamp(distance_ratio / params.softness, 0.0, 1.0);
            t * t * (3.0 - 2.0 * t) // smoothstep
        } else {
            0.0 // Hard edge: fully transparent
        };

        result.a = alpha * color.a;

        // Apply spill suppression on edge pixels
        if (spill_params.spill_suppression > 0.0 && result.a > 0.0) {
            result = apply_spill_suppression(result, params.target_hue, spill_params.spill_suppression);
        }
    } else if (in_hue && in_val) {
        // ── Near the edge — partial keying based on hue distance ────
        let hue_ratio = hue_dist / max(params.hue_tolerance, 0.001);
        if (hue_ratio <= 1.0 + params.softness) {
            let t = clamp(
                (hue_ratio - 1.0 + params.softness) / max(params.softness, 0.001),
                0.0,
                1.0,
            );
            let alpha = t * t * (3.0 - 2.0 * t);
            result.a = min(alpha * color.a, color.a);

            if (spill_params.spill_suppression > 0.0 && result.a > 0.0) {
                result = apply_spill_suppression(result, params.target_hue, spill_params.spill_suppression);
            }
        }
    }

    textureStore(output_tex, id.xy, result);
}

/// Apply spill suppression to remove key-color fringing from edge pixels.
///
/// Determines which RGB channel to suppress based on the target hue
/// (green range → reduce green, blue range → reduce blue, generic →
/// reduce dominant channel). The strength parameter controls how
/// aggressively the target color is suppressed.
fn apply_spill_suppression(color: vec4f, target_hue: f32, strength: f32) -> vec4f {
    var r = color.r;
    var g = color.g;
    var b = color.b;

    if (target_hue >= 90.0 && target_hue <= 150.0) {
        // Green screen: reduce green channel
        let factor = 1.0 - strength * 0.5;
        g = min(g * factor + (r + b) * 0.5 * strength * 0.3, 1.0);
    } else if (target_hue >= 210.0 && target_hue <= 270.0) {
        // Blue screen: reduce blue channel
        let factor = 1.0 - strength * 0.5;
        b = min(b * factor + (r + g) * 0.5 * strength * 0.3, 1.0);
    } else {
        // Generic: reduce the channel that most matches the target hue
        let factors = hue_to_rgb_factors(target_hue);
        let dominant = r * factors.x + g * factors.y + b * factors.z;
        let factor = 1.0 - strength * 0.4;
        let mix_val = dominant * (1.0 - factor) * 0.3;
        r = max(r - mix_val * factors.x, 0.0);
        g = max(g - mix_val * factors.y, 0.0);
        b = max(b - mix_val * factors.z, 0.0);
    }

    return vec4f(r, g, b, color.a);
}

/// Convert a hue angle to RGB factor weights.
///
/// Returns (r_factor, g_factor, b_factor) indicating how much each
/// channel contributes to the given hue. Used for spill suppression.
fn hue_to_rgb_factors(hue: f32) -> vec3f {
    let h = hue % 360.0;
    if (h < 60.0) {
        return vec3f(1.0, h / 60.0, 0.0);
    } else if (h < 120.0) {
        return vec3f((120.0 - h) / 60.0, 1.0, 0.0);
    } else if (h < 180.0) {
        return vec3f(0.0, 1.0, (h - 120.0) / 60.0);
    } else if (h < 240.0) {
        return vec3f(0.0, (240.0 - h) / 60.0, 1.0);
    } else if (h < 300.0) {
        return vec3f((h - 240.0) / 60.0, 0.0, 1.0);
    } else {
        return vec3f(1.0, 0.0, (360.0 - h) / 60.0);
    }
}
