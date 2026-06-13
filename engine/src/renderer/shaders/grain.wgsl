// Film Grain compute shader — GPU film grain with PCG hash, temporal blending
// Workgroup: 8x8

struct GrainShaderParams {
    width: u32,
    height: u32,
    intensity: f32,
    size: f32,
    color_grain: u32,
    red_weight: f32,
    green_weight: f32,
    blue_weight: f32,
    frame_num: u32,
    temporal_blend: f32,
    vhs_enabled: u32,
    vhs_tracking: f32,
    vhs_scanlines: f32,
    vhs_color_bleed: f32,
    halation_enabled: u32,
    halation_threshold: f32,
    halation_intensity: f32,
}

@group(0) @binding(0) var<uniform> params: GrainShaderParams;
@group(0) @binding(1) var<storage, read_write> tex_data: array<vec4<f32>>;

// PCG hash for high-quality pseudo-random numbers
fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn hash_pixel(x: u32, y: u32, frame: u32) -> f32 {
    let h = pcg_hash(x * 374761393u + y * 668265263u + frame * 1274126177u);
    return f32(h) / 4294967295.0 * 2.0 - 1.0; // -1..1
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) { return; }
    
    let idx = gid.y * params.width + gid.x;
    var pixel = tex_data[idx];
    
    let noise = hash_pixel(gid.x, gid.y, params.frame_num) * params.intensity;
    
    if (params.color_grain == 1u) {
        pixel.r = clamp(pixel.r + noise * params.red_weight, 0.0, 1.0);
        pixel.g = clamp(pixel.g + noise * params.green_weight, 0.0, 1.0);
        pixel.b = clamp(pixel.b + noise * params.blue_weight, 0.0, 1.0);
    } else {
        pixel.r = clamp(pixel.r + noise, 0.0, 1.0);
        pixel.g = clamp(pixel.g + noise, 0.0, 1.0);
        pixel.b = clamp(pixel.b + noise, 0.0, 1.0);
    }
    
    // VHS scanlines
    if (params.vhs_enabled == 1u) {
        if (gid.y % 2u == 0u) {
            let dim = 1.0 - params.vhs_scanlines * 0.5;
            pixel.r *= dim;
            pixel.g *= dim;
            pixel.b *= dim;
        }
    }
    
    // Halation (simplified: bloom highlights)
    if (params.halation_enabled == 1u) {
        let luma = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        if (luma > params.halation_threshold) {
            let bloom = (luma - params.halation_threshold) / (1.0 - params.halation_threshold + 0.001) * params.halation_intensity;
            pixel.r = clamp(pixel.r + bloom * 1.0, 0.0, 1.0);
            pixel.g = clamp(pixel.g + bloom * 0.6, 0.0, 1.0);
            pixel.b = clamp(pixel.b + bloom * 0.4, 0.0, 1.0);
        }
    }
    
    tex_data[idx] = pixel;
}
