/// Separable Gaussian blur compute shader (9-tap single-pass approximation)
///
/// Bind group 0:
///   [0] uniform Params — radius, sigma, direction, pad
///   [1] input texture_2d<f32>
///   [2] output texture_storage_2d<rgba8unorm, write>
///
/// direction: 0.0 = horizontal, 1.0 = vertical
/// For a full separable blur, run horizontal pass first, then vertical pass
/// on the output of the horizontal pass.

struct Params {
    radius: f32,
    sigma: f32,
    direction: f32,
    pad: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

/// Compute a Gaussian weight for the given offset
fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma));
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let sigma = max(params.sigma, 0.001);
    let is_horizontal = params.direction < 0.5;

    // 9-tap Gaussian kernel offsets: -4, -3, -2, -1, 0, 1, 2, 3, 4
    var color_sum = vec4f(0.0);
    var weight_sum = 0.0;

    for (var i: i32 = -4; i <= 4; i++) {
        let weight = gaussian(f32(i), sigma);

        var offset_coord: vec2i;
        if (is_horizontal) {
            let ox = clamp(i32(id.x) + i, 0, i32(dims.x) - 1);
            offset_coord = vec2i(ox, i32(id.y));
        } else {
            let oy = clamp(i32(id.y) + i, 0, i32(dims.y) - 1);
            offset_coord = vec2i(i32(id.x), oy);
        }

        let sample = textureLoad(input_tex, offset_coord, 0);
        color_sum += sample * weight;
        weight_sum += weight;
    }

    let result = color_sum / weight_sum;
    textureStore(output_tex, id.xy, result);
}
