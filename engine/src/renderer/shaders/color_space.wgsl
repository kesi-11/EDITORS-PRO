// Color Space Transform compute shader — Full CST pipeline with 13 transfer functions + ACES
// Workgroup: 8x8

struct CSTParams {
    width: u32,
    height: u32,
    input_tf: u32,      // Transfer function index
    output_tf: u32,
    enable_hdr: u32,
    hdr_peak_nits: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> params: CSTParams;
@group(0) @binding(1) var<storage, read_write> tex_data: array<vec4<f32>>;

// Transfer function decode: encoded -> linear
fn decode(encoded: f32, tf: u32) -> f32 {
    switch tf {
        case 0u: { // sRGB
            if (encoded <= 0.04045) { return encoded / 12.92; }
            return pow((encoded + 0.055) / 1.055, 2.4);
        }
        case 1u: { return encoded; } // Linear
        case 2u: { // Rec.709
            if (encoded < 0.081) { return encoded / 4.5; }
            return pow((encoded + 0.099) / 1.099, 1.0 / 0.45);
        }
        case 3u: { return pow(encoded, 2.2); } // Gamma 2.2
        case 4u: { return pow(encoded, 2.8); } // Gamma 2.8
        case 5u: { // PQ (ST 2084)
            let m1 = 2610.0 / 16384.0;
            let m2 = 2523.0 / 32.0;
            let c1 = 3424.0 / 4096.0;
            let c2 = 2413.0 / 128.0;
            let c3 = 2392.0 / 128.0;
            let n = pow(encoded, 1.0 / m2);
            return pow(max(c1 - n, 0.0) / (c2 - c3 * n), 1.0 / m1);
        }
        case 6u: { // HLG
            let a = 0.17883277;
            let b = 1.0 - 4.0 * a;
            let c = 0.5 - a * log(4.0 * a);
            if (encoded <= 0.5) { return encoded * encoded / 3.0; }
            return (exp((encoded - c) / a) + b) / 12.0;
        }
        default: { return encoded; }
    }
}

// Transfer function encode: linear -> encoded
fn encode(linear: f32, tf: u32) -> f32 {
    switch tf {
        case 0u: { // sRGB
            if (linear <= 0.0031308) { return linear * 12.92; }
            return 1.055 * pow(linear, 1.0 / 2.4) - 0.055;
        }
        case 1u: { return linear; } // Linear
        case 2u: { // Rec.709
            if (linear < 0.018) { return 4.5 * linear; }
            return 1.099 * pow(linear, 0.45) - 0.099;
        }
        case 3u: { return pow(linear, 1.0 / 2.2); } // Gamma 2.2
        case 4u: { return pow(linear, 1.0 / 2.8); } // Gamma 2.8
        case 5u: { // PQ (ST 2084)
            let m1 = 2610.0 / 16384.0;
            let m2 = 2523.0 / 32.0;
            let c1 = 3424.0 / 4096.0;
            let c2 = 2413.0 / 128.0;
            let c3 = 2392.0 / 128.0;
            let n = (c1 + c2 * pow(linear, m1)) / (1.0 + c3 * pow(linear, m1));
            return pow(n, m2);
        }
        case 6u: { // HLG
            let a = 0.17883277;
            let b = 1.0 - 4.0 * a;
            let c = 0.5 - a * log(4.0 * a);
            if (linear <= 1.0 / 12.0) { return sqrt(3.0 * linear); }
            return a * log(12.0 * linear - b) + c;
        }
        default: { return linear; }
    }
}

// ACES tone mapping
fn aces_tone_map(x: f32) -> f32 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return (x * (a * x + b)) / (x * (c * x + d) + e);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) { return; }
    
    let idx = gid.y * params.width + gid.x;
    var pixel = tex_data[idx];
    
    // Decode from input transfer function
    var r_lin = decode(pixel.r, params.input_tf);
    var g_lin = decode(pixel.g, params.input_tf);
    var b_lin = decode(pixel.b, params.input_tf);
    
    // Tone map if HDR
    if (params.enable_hdr == 1u && (params.input_tf == 5u || params.input_tf == 6u)) {
        r_lin = aces_tone_map(r_lin);
        g_lin = aces_tone_map(g_lin);
        b_lin = aces_tone_map(b_lin);
    }
    
    // Encode to output transfer function
    pixel.r = encode(clamp(r_lin, 0.0, 1.0), params.output_tf);
    pixel.g = encode(clamp(g_lin, 0.0, 1.0), params.output_tf);
    pixel.b = encode(clamp(b_lin, 0.0, 1.0), params.output_tf);
    
    tex_data[idx] = pixel;
}
