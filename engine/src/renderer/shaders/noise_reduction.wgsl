// Noise Reduction compute shader — GPU bilateral filter with edge preservation
// Workgroup: 8x8

struct NRShaderParams {
    width: u32,
    height: u32,
    spatial_sigma: f32,
    range_sigma: f32,
    strength: f32,
    channel_mode: u32,  // 0=LumaOnly, 1=ChromaOnly, 2=Both
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> params: NRShaderParams;
@group(0) @binding(1) var<storage, read> input_tex: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> output_tex: array<vec4<f32>>;

fn bilateral_filter(x: u32, y: u32, channel: u32) -> f32 {
    let radius = i32(ceil(params.spatial_sigma * 2.0));
    let spatial_sigma2 = 2.0 * params.spatial_sigma * params.spatial_sigma;
    let range_sigma2 = 2.0 * params.range_sigma * params.range_sigma;
    
    let idx = y * params.width + x;
    let center = input_tex[idx][channel];
    
    var weight_sum = 0.0;
    var value_sum = 0.0;
    
    for (var dy = -radius; dy <= radius; dy++) {
        for (var dx = -radius; dx <= radius; dx++) {
            let nx = clamp(u32(i32(x) + dx), 0u, params.width - 1u);
            let ny = clamp(u32(i32(y) + dy), 0u, params.height - 1u);
            let nidx = ny * params.width + nx;
            let neighbor = input_tex[nidx][channel];
            
            let dist_spatial = f32(dx * dx + dy * dy);
            let dist_range = (center - neighbor) * (center - neighbor);
            let weight = exp(-dist_spatial / spatial_sigma2) * exp(-dist_range / range_sigma2);
            
            weight_sum += weight;
            value_sum += neighbor * weight;
        }
    }
    
    if (weight_sum > 0.0) { return value_sum / weight_sum; }
    return center;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) { return; }
    
    let idx = gid.y * params.width + gid.x;
    let original = input_tex[idx];
    
    var result = original;
    
    // Process channels based on channel_mode
    if (params.channel_mode == 0u || params.channel_mode == 2u) {
        // Luma processing
        let luma = 0.299 * original.r + 0.587 * original.g + 0.114 * original.b;
        let denoised_r = bilateral_filter(gid.x, gid.y, 0u);
        let denoised_g = bilateral_filter(gid.x, gid.y, 1u);
        let denoised_b = bilateral_filter(gid.x, gid.y, 2u);
        
        result.r = mix(original.r, denoised_r, params.strength);
        result.g = mix(original.g, denoised_g, params.strength);
        result.b = mix(original.b, denoised_b, params.strength);
    }
    
    output_tex[idx] = result;
}
