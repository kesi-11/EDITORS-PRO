//! Shader management for GPU effects
//!
//! Handles loading, caching, and applying WGSL shaders
//! for visual effects processing.

use std::collections::HashMap;

/// Manages GPU shaders for effects processing
pub struct ShaderManager {
    shaders: HashMap<String, String>,
    loaded: bool,
}

impl ShaderManager {
    pub fn new() -> Self {
        let mut shaders = HashMap::new();

        // Built-in shader source code for basic effects (WGSL)
        shaders.insert("brightness".to_string(), BRIGHTNESS_SHADER.to_string());
        shaders.insert("contrast".to_string(), CONTRAST_SHADER.to_string());
        shaders.insert("saturation".to_string(), SATURATION_SHADER.to_string());
        shaders.insert("grayscale".to_string(), GRAYSCALE_SHADER.to_string());
        shaders.insert("blur".to_string(), BLUR_SHADER.to_string());

        Self {
            shaders,
            loaded: true,
        }
    }

    /// Load a shader by name
    pub fn load_shader(&self, name: &str) -> Option<&str> {
        self.shaders.get(name).map(|s| s.as_str())
    }

    /// Register a custom shader
    pub fn register_shader(&mut self, name: &str, source: &str) {
        self.shaders.insert(name.to_string(), source.to_string());
    }

    /// List all available shader names
    pub fn available_shaders(&self) -> Vec<&str> {
        self.shaders.keys().map(|s| s.as_str()).collect()
    }

    /// Apply a CPU-based effect to frame data (fallback when GPU is not available)
    /// This is used in the MVP before GPU rendering is implemented
    pub fn apply_cpu_effect(data: &mut [u8], effect: &str, intensity: f32) {
        match effect {
            "brightness" => apply_brightness(data, intensity),
            "contrast" => apply_contrast(data, intensity),
            "saturation" => apply_saturation(data, intensity),
            "grayscale" => apply_grayscale(data, intensity),
            "sepia" => apply_sepia(data, intensity),
            "invert" => apply_invert(data),
            _ => log::warn!("Unknown CPU effect: {}", effect),
        }
    }
}

// CPU effect implementations for MVP

fn apply_brightness(data: &mut [u8], intensity: f32) {
    let adjustment = (intensity * 255.0) as i16;
    for chunk in data.chunks_exact_mut(4) {
        chunk[0] = (chunk[0] as i16 + adjustment).clamp(0, 255) as u8;
        chunk[1] = (chunk[1] as i16 + adjustment).clamp(0, 255) as u8;
        chunk[2] = (chunk[2] as i16 + adjustment).clamp(0, 255) as u8;
        // Alpha stays unchanged
    }
}

fn apply_contrast(data: &mut [u8], intensity: f32) {
    let factor = (1.0 + intensity).max(0.0);
    for chunk in data.chunks_exact_mut(4) {
        chunk[0] = (((chunk[0] as f32 - 128.0) * factor) + 128.0).clamp(0.0, 255.0) as u8;
        chunk[1] = (((chunk[1] as f32 - 128.0) * factor) + 128.0).clamp(0.0, 255.0) as u8;
        chunk[2] = (((chunk[2] as f32 - 128.0) * factor) + 128.0).clamp(0.0, 255.0) as u8;
    }
}

fn apply_saturation(data: &mut [u8], intensity: f32) {
    for chunk in data.chunks_exact_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let gray = 0.299 * r + 0.587 * g + 0.114 * b;

        chunk[0] = (gray + (r - gray) * intensity).clamp(0.0, 255.0) as u8;
        chunk[1] = (gray + (g - gray) * intensity).clamp(0.0, 255.0) as u8;
        chunk[2] = (gray + (b - gray) * intensity).clamp(0.0, 255.0) as u8;
    }
}

fn apply_grayscale(data: &mut [u8], _intensity: f32) {
    for chunk in data.chunks_exact_mut(4) {
        let gray = (0.299 * chunk[0] as f32 + 0.587 * chunk[1] as f32 + 0.114 * chunk[2] as f32) as u8;
        chunk[0] = gray;
        chunk[1] = gray;
        chunk[2] = gray;
    }
}

fn apply_sepia(data: &mut [u8], intensity: f32) {
    for chunk in data.chunks_exact_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;

        let sepia_r = (r * 0.393 + g * 0.769 + b * 0.189).min(255.0);
        let sepia_g = (r * 0.349 + g * 0.686 + b * 0.168).min(255.0);
        let sepia_b = (r * 0.272 + g * 0.534 + b * 0.131).min(255.0);

        chunk[0] = (r + (sepia_r - r) * intensity).clamp(0.0, 255.0) as u8;
        chunk[1] = (g + (sepia_g - g) * intensity).clamp(0.0, 255.0) as u8;
        chunk[2] = (b + (sepia_b - b) * intensity).clamp(0.0, 255.0) as u8;
    }
}

fn apply_invert(data: &mut [u8]) {
    for chunk in data.chunks_exact_mut(4) {
        chunk[0] = 255 - chunk[0];
        chunk[1] = 255 - chunk[1];
        chunk[2] = 255 - chunk[2];
    }
}

// WGSL shader source code for GPU effects (Phase 3)

const BRIGHTNESS_SHADER: &str = r#"
@group(0) @binding(0) var<uniform> params: vec4f;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) { return; }
    let color = textureLoad(input_tex, id.xy, 0);
    let adjusted = clamp(color + vec4f(params.x, params.x, params.x, 0.0), vec4f(0.0), vec4f(1.0));
    textureStore(output_tex, id.xy, adjusted);
}
"#;

const CONTRAST_SHADER: &str = r#"
@group(0) @binding(0) var<uniform> params: vec4f;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) { return; }
    let color = textureLoad(input_tex, id.xy, 0);
    let factor = 1.0 + params.x;
    let adjusted = clamp((color - vec4f(0.5)) * factor + vec4f(0.5), vec4f(0.0), vec4f(1.0));
    textureStore(output_tex, id.xy, adjusted);
}
"#;

const SATURATION_SHADER: &str = r#"
@group(0) @binding(0) var<uniform> params: vec4f;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) { return; }
    let color = textureLoad(input_tex, id.xy, 0);
    let gray = dot(color.rgb, vec3f(0.299, 0.587, 0.114));
    let saturated = clamp(vec3f(gray) + (color.rgb - vec3f(gray)) * params.x, vec3f(0.0), vec3f(1.0));
    textureStore(output_tex, id.xy, vec4f(saturated, color.a));
}
"#;

const GRAYSCALE_SHADER: &str = r#"
@group(0) @binding(0) var<uniform> params: vec4f;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) { return; }
    let color = textureLoad(input_tex, id.xy, 0);
    let gray = dot(color.rgb, vec3f(0.299, 0.587, 0.114));
    textureStore(output_tex, id.xy, vec4f(vec3f(gray), color.a));
}
"#;

const BLUR_SHADER: &str = r#"
@group(0) @binding(0) var<uniform> params: vec4f;
@group(0) @binding(1) var input_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let dims = textureDimensions(input_tex);
    if (id.x >= dims.x || id.y >= dims.y) { return; }
    let radius = i32(params.x);
    var sum = vec4f(0.0);
    var count = 0.0;
    for (var dx = -radius; dx <= radius; dx++) {
        for (var dy = -radius; dy <= radius; dy++) {
            let offset = vec2i(dx, dy);
            let coord = clamp(vec2i(id.xy) + offset, vec2i(0), vec2i(i32(dims.x) - 1, i32(dims.y) - 1));
            sum += textureLoad(input_tex, coord, 0);
            count += 1.0;
        }
    }
    textureStore(output_tex, id.xy, sum / count);
}
"#;
