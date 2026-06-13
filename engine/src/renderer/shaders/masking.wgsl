// Masking compute shader — GPU mask compositing with feather/expansion/inversion
// Workgroup: 8x8

struct MaskParams {
    mask_type: u32,      // 0=Rect, 1=Ellipse, 2=Bezier, 3=Luminance
    inverted: u32,
    feather: f32,
    expansion: f32,
    opacity: f32,
    blend_mode: u32,     // 0=Add, 1=Subtract, 2=Intersect, 3=Difference
    rect_x: f32,
    rect_y: f32,
    rect_w: f32,
    rect_h: f32,
    ellipse_cx: f32,
    ellipse_cy: f32,
    ellipse_rx: f32,
    ellipse_ry: f32,
    threshold: f32,
    threshold_softness: f32,
    _padding: f32,
    rotation: f32,
}

@group(0) @binding(0) var<uniform> params: MaskParams;
@group(0) @binding(1) var<storage, read> existing_mask: array<f32>;
@group(0) @binding(2) var<storage, read_write> output_mask: array<f32>;
@group(0) @binding(3) var<storage, read> luma_data: array<f32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = params.rect_w; // Using rect_w as width placeholder
    let height = params.rect_h; // Using rect_h as height placeholder
    
    let x = f32(gid.x);
    let y = f32(gid.y);
    let idx = gid.y * u32(width) + gid.x;
    
    let nx = x / width;
    let ny = y / height;
    
    var mask_val: f32 = 0.0;
    
    switch params.mask_type {
        case 0u: { // Rectangle
            let rot = params.rotation * 3.14159265 / 180.0;
            let cos_r = cos(rot);
            let sin_r = sin(rot);
            let cx = params.rect_x + params.rect_w / 2.0;
            let cy = params.rect_y + params.rect_h / 2.0;
            let dx = nx - cx;
            let dy = ny - cy;
            let rx = dx * cos_r + dy * sin_r + cx;
            let ry = -dx * sin_r + dy * cos_r + cy;
            if (rx >= params.rect_x && rx <= params.rect_x + params.rect_w &&
                ry >= params.rect_y && ry <= params.rect_y + params.rect_h) {
                mask_val = 1.0;
            }
        }
        case 1u: { // Ellipse
            let dx = (nx - params.ellipse_cx) / max(params.ellipse_rx, 0.001);
            let dy = (ny - params.ellipse_cy) / max(params.ellipse_ry, 0.001);
            if (dx * dx + dy * dy <= 1.0) {
                mask_val = 1.0;
            }
        }
        case 3u: { // Luminance
            let luma = luma_data[idx];
            let lo = params.threshold - params.threshold_softness;
            let hi = params.threshold + params.threshold_softness;
            if (luma < lo) { mask_val = 0.0; }
            else if (luma > hi) { mask_val = 1.0; }
            else { mask_val = (luma - lo) / max(hi - lo, 0.001); }
        }
        default: { mask_val = 0.0; }
    }
    
    // Apply expansion
    if (params.expansion > 0.0) {
        mask_val = min(mask_val + params.expansion, 1.0);
    } else {
        mask_val = max(mask_val + params.expansion, 0.0);
    }
    
    // Apply feather (smoothstep around 0.5 boundary)
    if (params.feather > 0.0) {
        let transition = params.feather * 0.5;
        let center = 0.5;
        let dist = abs(mask_val - center);
        if (dist <= transition) {
            let t = clamp((mask_val - (center - transition)) / (2.0 * transition + 0.0001), 0.0, 1.0);
            mask_val = t * t * (3.0 - 2.0 * t);
        }
    }
    
    // Apply opacity
    mask_val *= params.opacity;
    
    // Apply inversion
    if (params.inverted == 1u) {
        mask_val = 1.0 - mask_val;
    }
    
    // Composite with existing mask
    let existing = existing_mask[idx];
    var result: f32;
    switch params.blend_mode {
        case 0u: { result = min(existing + mask_val, 1.0); } // Add
        case 1u: { result = max(existing - mask_val, 0.0); } // Subtract
        case 2u: { result = existing * mask_val; } // Intersect
        case 3u: { result = abs(existing - mask_val); } // Difference
        default: { result = min(existing + mask_val, 1.0); }
    }
    
    output_mask[idx] = result;
}
