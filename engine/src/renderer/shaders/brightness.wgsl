/// Universal brightness/contrast/saturation compute shader
///
/// Bind group 0:
///   [0] uniform Params — brightness, contrast, saturation, mode_flag
///   [1] input texture_2d<f32>
///   [2] output texture_storage_2d<rgba8unorm, write>
///
/// The mode_flag (params.w) selects additional per-pixel operations
/// that reuse the same uniform layout:
///   0.0 — standard brightness / contrast / saturation
///   1.0 — grayscale  (brightness & contrast ignored, saturation forced to 0)
///   2.0 — sepia       (brightness & contrast applied, then sepia mix using saturation as intensity)
///   3.0 — invert      (brightness & contrast ignored, saturation ignored)
///   4.0 — hue_rotate  (brightness=angle, contrast & saturation ignored)
///   5.0 — temperature (brightness=shift, contrast ignored, saturation ignored)
///   6.0 — vignette    (brightness=intensity, contrast & saturation ignored)
///   7.0 — sharpen     (brightness=strength, contrast & saturation ignored)

struct Params {
    brightness: f32,
    contrast: f32,
    saturation: f32,
    mode_flag: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let color = textureLoad(input_tex, id.xy, 0);
    var result: vec4f;

    if (params.mode_flag < 0.5) {
        // ── Mode 0: brightness / contrast / saturation ──────────────
        // Apply contrast (center at 0.5, scale, add back)
        var c = (color - vec4f(0.5)) * (1.0 + params.contrast) + vec4f(0.5);
        // Apply brightness offset
        c = c + vec4f(params.brightness, params.brightness, params.brightness, 0.0);
        // Apply saturation (luminance-based desaturation)
        let lum = dot(c.rgb, vec3f(0.299, 0.587, 0.114));
        c = vec4f(
            clamp(lum + (c.r - lum) * params.saturation, 0.0, 1.0),
            clamp(lum + (c.g - lum) * params.saturation, 0.0, 1.0),
            clamp(lum + (c.b - lum) * params.saturation, 0.0, 1.0),
            c.a,
        );
        result = clamp(c, vec4f(0.0), vec4f(1.0));

    } else if (params.mode_flag < 1.5) {
        // ── Mode 1: grayscale ───────────────────────────────────────
        let gray = dot(color.rgb, vec3f(0.299, 0.587, 0.114));
        result = vec4f(vec3f(gray), color.a);

    } else if (params.mode_flag < 2.5) {
        // ── Mode 2: sepia ───────────────────────────────────────────
        let r = color.r;
        let g = color.g;
        let b = color.b;
        let sepia_r = clamp(r * 0.393 + g * 0.769 + b * 0.189, 0.0, 1.0);
        let sepia_g = clamp(r * 0.349 + g * 0.686 + b * 0.168, 0.0, 1.0);
        let sepia_b = clamp(r * 0.272 + g * 0.534 + b * 0.131, 0.0, 1.0);
        let intensity = params.saturation; // reuse saturation field as sepia intensity
        let blended = vec3f(r, g, b) + (vec3f(sepia_r, sepia_g, sepia_b) - vec3f(r, g, b)) * intensity;
        result = vec4f(clamp(blended, vec3f(0.0), vec3f(1.0)), color.a);

    } else if (params.mode_flag < 3.5) {
        // ── Mode 3: invert ──────────────────────────────────────────
        result = vec4f(1.0 - color.rgb, color.a);

    } else if (params.mode_flag < 4.5) {
        // ── Mode 4: hue_rotate ──────────────────────────────────────
        // brightness field reused as rotation angle (0..1 maps to 0..360°)
        let angle = params.brightness * 6.28318530718;
        let cos_a = cos(angle);
        let sin_a = sin(angle);
        let r = color.r;
        let g = color.g;
        let b = color.b;
        let new_r = (0.213 + 0.787 * cos_a - 0.213 * sin_a) * r
                  + (0.715 - 0.715 * cos_a - 0.715 * sin_a) * g
                  + (0.072 - 0.072 * cos_a + 0.928 * sin_a) * b;
        let new_g = (0.213 - 0.213 * cos_a + 0.143 * sin_a) * r
                  + (0.715 + 0.285 * cos_a + 0.140 * sin_a) * g
                  + (0.072 - 0.072 * cos_a - 0.283 * sin_a) * b;
        let new_b = (0.213 - 0.213 * cos_a - 0.787 * sin_a) * r
                  + (0.715 - 0.715 * cos_a + 0.715 * sin_a) * g
                  + (0.072 + 0.928 * cos_a + 0.072 * sin_a) * b;
        result = clamp(vec4f(new_r, new_g, new_b, color.a), vec4f(0.0), vec4f(1.0));

    } else if (params.mode_flag < 5.5) {
        // ── Mode 5: temperature ─────────────────────────────────────
        // brightness field reused as temperature shift
        let temp = params.brightness;
        result = clamp(vec4f(
            color.r + temp * 0.12,
            color.g,
            color.b - temp * 0.12,
            color.a,
        ), vec4f(0.0), vec4f(1.0));

    } else if (params.mode_flag < 6.5) {
        // ── Mode 6: vignette ────────────────────────────────────────
        // brightness field reused as vignette intensity
        let center = vec2f(f32(dims.x) * 0.5, f32(dims.y) * 0.5);
        let pos = vec2f(f32(id.x), f32(id.y));
        let max_dist = distance(vec2f(0.0), center);
        let dist = distance(pos, center) / max_dist;
        let vignette = 1.0 - params.brightness * smoothstep(0.5, 1.5, dist);
        result = vec4f(clamp(color.rgb * vignette, vec3f(0.0), vec3f(1.0)), color.a);

    } else if (params.mode_flag < 7.5) {
        // ── Mode 7: sharpen ─────────────────────────────────────────
        // brightness field reused as sharpen strength
        let strength = params.brightness;
        let center_color = color;
        var sum = vec4f(0.0);
        let center_weight = 1.0 + 4.0 * strength;
        let neighbor_weight = -strength;

        for (var dy: i32 = -1; dy <= 1; dy++) {
            for (var dx: i32 = -1; dx <= 1; dx++) {
                let coord = clamp(
                    vec2i(id.xy) + vec2i(dx, dy),
                    vec2i(0, 0),
                    vec2i(i32(dims.x) - 1, i32(dims.y) - 1),
                );
                let weight = select(neighbor_weight, center_weight, dx == 0 && dy == 0);
                sum += textureLoad(input_tex, coord, 0) * weight;
            }
        }
        result = clamp(sum, vec4f(0.0), vec4f(1.0));

    } else {
        // ── Unknown mode: pass through ──────────────────────────────
        result = color;
    }

    textureStore(output_tex, id.xy, result);
}

/// Smoothstep helper for vignette falloff
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
