// Lens Correction compute shader — GPU distortion/CA/vignette correction
// Workgroup: 8x8

struct LensParams {
    width: u32,
    height: u32,
    k1: f32,
    k2: f32,
    k3: f32,
    p1: f32,
    p2: f32,
    ca_red_x: f32,
    ca_red_y: f32,
    ca_blue_x: f32,
    ca_blue_y: f32,
    ca_radial: f32,
    vignette_amount: f32,
    vignette_midpoint: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> params: LensParams;
@group(0) @binding(1) var<storage, read> input_tex: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> output_tex: array<vec4<f32>>;

fn undistort(nx: f32, ny: f32) -> vec2<f32> {
    let r2 = nx * nx + ny * ny;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    
    let radial = 1.0 + params.k1 * r2 + params.k2 * r4 + params.k3 * r6;
    let dx_tang = 2.0 * params.p1 * nx * ny + params.p2 * (r2 + 2.0 * nx * nx);
    let dy_tang = params.p1 * (r2 + 2.0 * ny * ny) + 2.0 * params.p2 * nx * ny;
    
    return vec2<f32>(nx * radial + dx_tang, ny * radial + dy_tang);
}

fn sample_tex(nx: f32, ny: f32) -> vec4<f32> {
    let w = f32(params.width);
    let h = f32(params.height);
    let scale = min(w, h) / 2.0;
    let cx = w / 2.0;
    let cy = h / 2.0;
    
    let px = clamp(u32(nx * scale + cx), 0u, params.width - 1u);
    let py = clamp(u32(ny * scale + cy), 0u, params.height - 1u);
    return input_tex[py * params.width + px];
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) { return; }
    
    let w = f32(params.width);
    let h = f32(params.height);
    let scale = min(w, h) / 2.0;
    let cx = w / 2.0;
    let cy = h / 2.0;
    
    let nx = (f32(gid.x) - cx) / scale;
    let ny = (f32(gid.y) - cy) / scale;
    
    // Undistort
    let undistorted = undistort(nx, ny);
    
    // Chromatic aberration offsets
    let ca_factor = abs(nx) * params.ca_radial * 0.001;
    let r_nx = nx + params.ca_red_x * ca_factor;
    let r_ny = ny + params.ca_red_y * ca_factor;
    let b_nx = nx + params.ca_blue_x * ca_factor;
    let b_ny = ny + params.ca_blue_y * ca_factor;
    
    let r_sample = sample_tex(r_nx, r_ny).r;
    let g_sample = sample_tex(undistorted.x, undistorted.y).g;
    let b_sample = sample_tex(b_nx, b_ny).b;
    let a_sample = sample_tex(undistorted.x, undistorted.y).a;
    
    // Vignette
    let vnx = f32(gid.x) / w * 2.0 - 1.0;
    let vny = f32(gid.y) / h * 2.0 - 1.0;
    let dist = sqrt(vnx * vnx + vny * vny);
    let midpoint = max(params.vignette_midpoint, 0.01);
    let scale_v = 1.0 - midpoint;
    let normalized_dist = max(dist - midpoint, 0.0) / max(scale_v, 0.01);
    let vignette = clamp(1.0 - params.vignette_amount * normalized_dist * normalized_dist, 0.0, 1.0);
    
    let idx = gid.y * params.width + gid.x;
    output_tex[idx] = vec4<f32>(
        r_sample * vignette,
        g_sample * vignette,
        b_sample * vignette,
        a_sample
    );
}
