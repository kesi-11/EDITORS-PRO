//! WGSL shader modules for GPU compute effects
//!
//! Each shader is loaded at compile time via `include_str!` so the
//! GPU pipeline can create wgpu shader modules directly from source.
//!
//! ## Shaders
//!
//! - **BRIGHTNESS** — Universal color adjustment shader. Handles brightness,
//!   contrast, saturation, grayscale, sepia, invert, hue_rotate, temperature,
//!   vignette, and sharpen by varying the `mode_flag` uniform parameter.
//! - **BLUR** — 9-tap separable Gaussian blur. Run twice (horizontal then
//!   vertical) for a full two-pass blur.
//! - **COMPOSITE** — Multi-layer alpha compositing using source-over blending.
//! - **CHROMA_KEY** — Green/blue screen color keying with HSV color space,
//!   smoothstep edge feathering, and spill suppression.

pub const BRIGHTNESS: &str = include_str!("brightness.wgsl");
pub const BLUR: &str = include_str!("blur.wgsl");
pub const COMPOSITE: &str = include_str!("composite.wgsl");
pub const CHROMA_KEY: &str = include_str!("chroma_key.wgsl");
pub const MASKING: &str = include_str!("masking.wgsl");
pub const BLEND_MODES: &str = include_str!("blend_modes.wgsl");
pub const NOISE_REDUCTION: &str = include_str!("noise_reduction.wgsl");
pub const LENS_CORRECTION: &str = include_str!("lens_correction.wgsl");
pub const COLOR_SPACE: &str = include_str!("color_space.wgsl");
pub const GRAIN: &str = include_str!("grain.wgsl");
