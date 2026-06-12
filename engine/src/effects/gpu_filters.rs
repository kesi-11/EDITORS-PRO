//! GPU-accelerated filter implementations
//!
//! Maps each filter effect to its corresponding WGSL compute shader
//! and provides parameter marshaling for the GPU pipeline.
//!
//! ## Architecture
//!
//! Most effects share the **brightness** shader by varying the `mode_flag`
//! parameter, which selects the per-pixel operation inside that shader:
//!
//! | mode_flag | Effect       | Primary param (brightness field) |
//! |-----------|-------------|----------------------------------|
//! | 0.0       | brightness  | brightness offset                |
//! | 0.0       | contrast    | contrast factor (brightness=0)   |
//! | 0.0       | saturation  | saturation factor (brightness=0, contrast=0) |
//! | 1.0       | grayscale   | (unused)                         |
//! | 2.0       | sepia       | sepia intensity (via saturation) |
//! | 3.0       | invert      | (unused)                         |
//! | 4.0       | hue_rotate  | rotation angle (0..1 → 0..360°)  |
//! | 5.0       | temperature | temperature shift                |
//! | 6.0       | vignette    | vignette intensity               |
//! | 7.0       | sharpen     | sharpen strength                 |
//!
//! The **blur** shader handles the blur effect with separate
//! horizontal and vertical dispatches.

use crate::effects::EffectParameter;

/// A GPU filter descriptor that pairs a shader name with its parameters
#[derive(Debug, Clone)]
pub struct GpuFilterDescriptor {
    /// The WGSL shader name (must match a loaded shader module)
    pub shader_name: String,
    /// Uniform parameters as f32 values (packed for GPU upload)
    pub params: Vec<f32>,
}

impl GpuFilterDescriptor {
    /// Create a new descriptor referencing the given shader
    pub fn new(shader_name: &str, params: Vec<f32>) -> Self {
        Self {
            shader_name: shader_name.to_string(),
            params,
        }
    }
}

/// GPU filter dispatcher — maps effect names to shader + params
pub struct GpuFilterDispatcher;

impl GpuFilterDispatcher {
    /// Mode flags for the brightness shader (must match brightness.wgsl)
    pub const MODE_BRIGHTNESS: f32 = 0.0;
    pub const MODE_CONTRAST: f32 = 0.0;
    pub const MODE_SATURATION: f32 = 0.0;
    pub const MODE_GRAYSCALE: f32 = 1.0;
    pub const MODE_SEPIA: f32 = 2.0;
    pub const MODE_INVERT: f32 = 3.0;
    pub const MODE_HUE_ROTATE: f32 = 4.0;
    pub const MODE_TEMPERATURE: f32 = 5.0;
    pub const MODE_VIGNETTE: f32 = 6.0;
    pub const MODE_SHARPEN: f32 = 7.0;

    /// Create a filter descriptor for the given effect name and parameters.
    ///
    /// Returns `None` if the effect does not have a GPU implementation.
    pub fn create_descriptor(
        effect_name: &str,
        params: &[EffectParameter],
    ) -> Option<GpuFilterDescriptor> {
        match effect_name.to_lowercase().as_str() {
            "brightness" => {
                let value = param_value(params, "value").unwrap_or(0.0);
                // mode 0: brightness=value, contrast=0, saturation=1
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![value, 0.0, 1.0, Self::MODE_BRIGHTNESS],
                ))
            }
            "contrast" => {
                let value = param_value(params, "value").unwrap_or(0.0);
                // mode 0: brightness=0, contrast=value, saturation=1
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![0.0, value, 1.0, Self::MODE_CONTRAST],
                ))
            }
            "saturation" => {
                let value = param_value(params, "value").unwrap_or(1.0);
                // mode 0: brightness=0, contrast=0, saturation=value
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![0.0, 0.0, value, Self::MODE_SATURATION],
                ))
            }
            "grayscale" => {
                // mode 1: brightness=0, contrast=0, saturation=0, mode=1
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![0.0, 0.0, 0.0, Self::MODE_GRAYSCALE],
                ))
            }
            "blur" => {
                let radius = param_value(params, "radius").unwrap_or(5.0);
                let sigma = param_value(params, "sigma").unwrap_or(radius / 2.0);
                // Horizontal pass first (direction=0)
                Some(GpuFilterDescriptor::new(
                    "blur",
                    vec![radius, sigma, 0.0, 0.0],
                ))
                // Note: caller must also dispatch a vertical pass (direction=1)
            }
            "sepia" => {
                let intensity = param_value(params, "intensity").unwrap_or(1.0);
                // mode 2: saturation field carries sepia intensity
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![0.0, 0.0, intensity, Self::MODE_SEPIA],
                ))
            }
            "invert" => {
                // mode 3: all params unused
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![0.0, 0.0, 0.0, Self::MODE_INVERT],
                ))
            }
            "vignette" => {
                let intensity = param_value(params, "intensity").unwrap_or(0.5);
                // mode 6: brightness field carries vignette intensity
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![intensity, 0.0, 0.0, Self::MODE_VIGNETTE],
                ))
            }
            "sharpen" => {
                let strength = param_value(params, "strength").unwrap_or(1.0);
                // mode 7: brightness field carries sharpen strength
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![strength, 0.0, 0.0, Self::MODE_SHARPEN],
                ))
            }
            "hue_rotate" => {
                let angle = param_value(params, "angle").unwrap_or(0.0);
                // mode 4: brightness field carries hue angle (0..1 → 0..360°)
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![angle, 0.0, 0.0, Self::MODE_HUE_ROTATE],
                ))
            }
            "temperature" => {
                let shift = param_value(params, "shift").unwrap_or(0.0);
                // mode 5: brightness field carries temperature shift
                Some(GpuFilterDescriptor::new(
                    "brightness",
                    vec![shift, 0.0, 0.0, Self::MODE_TEMPERATURE],
                ))
            }
            "chroma_key" => {
                let target_hue = param_value(params, "target_hue").unwrap_or(120.0);
                let hue_tolerance = param_value(params, "hue_tolerance").unwrap_or(30.0);
                let saturation_tolerance = param_value(params, "saturation_tolerance").unwrap_or(0.4);
                let softness = param_value(params, "softness").unwrap_or(0.15);
                let spill_suppression = param_value(params, "spill_suppression").unwrap_or(0.5);
                // chroma_key shader uses its own uniform layout:
                //   Params: target_hue, hue_tolerance, saturation_tolerance, softness
                //   SpillParams: spill_suppression, pad1, pad2, pad3
                Some(GpuFilterDescriptor::new(
                    "chroma_key",
                    vec![target_hue, hue_tolerance, saturation_tolerance, softness, spill_suppression],
                ))
            }
            _ => None,
        }
    }

    /// Create a vertical-pass blur descriptor (to be dispatched after horizontal).
    ///
    /// Call this after the horizontal blur descriptor to complete the
    /// separable two-pass Gaussian blur.
    pub fn create_blur_vertical_descriptor(
        radius: f32,
        sigma: f32,
    ) -> GpuFilterDescriptor {
        GpuFilterDescriptor::new("blur", vec![radius, sigma, 1.0, 0.0])
    }

    /// List all GPU-accelerated effect names
    pub fn gpu_accelerated_effects() -> &'static [&'static str] {
        &[
            "brightness",
            "contrast",
            "saturation",
            "grayscale",
            "blur",
            "sepia",
            "invert",
            "vignette",
            "sharpen",
            "hue_rotate",
            "temperature",
            "chroma_key",
        ]
    }

    /// Check if an effect has GPU acceleration
    pub fn is_gpu_accelerated(effect_name: &str) -> bool {
        Self::gpu_accelerated_effects()
            .iter()
            .any(|&name| name.eq_ignore_ascii_case(effect_name))
    }
}

/// Extract a parameter value by name from a slice of EffectParameters.
/// Falls back to checking by index (position) if the name doesn't match.
fn param_value(params: &[EffectParameter], name: &str) -> Option<f32> {
    // Try by name first
    if let Some(p) = params.iter().find(|p| p.name == name) {
        return Some(p.value);
    }
    // Fall back to first parameter (many effects have a single "value" param)
    if !params.is_empty() {
        return Some(params[0].value);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_param(name: &str, value: f32) -> EffectParameter {
        EffectParameter::new(name, name, value, -10.0, 10.0, 0.01)
    }

    #[test]
    fn test_brightness_descriptor() {
        let params = vec![make_param("value", 0.2)];
        let desc = GpuFilterDispatcher::create_descriptor("brightness", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params, vec![0.2, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_contrast_descriptor() {
        let params = vec![make_param("value", 0.5)];
        let desc = GpuFilterDispatcher::create_descriptor("contrast", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params, vec![0.0, 0.5, 1.0, 0.0]);
    }

    #[test]
    fn test_saturation_descriptor() {
        let params = vec![make_param("value", 2.0)];
        let desc = GpuFilterDispatcher::create_descriptor("saturation", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params, vec![0.0, 0.0, 2.0, 0.0]);
    }

    #[test]
    fn test_grayscale_descriptor() {
        let desc = GpuFilterDispatcher::create_descriptor("grayscale", &[]).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[3], 1.0); // mode flag
    }

    #[test]
    fn test_sepia_descriptor() {
        let params = vec![make_param("intensity", 0.8)];
        let desc = GpuFilterDispatcher::create_descriptor("sepia", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[2], 0.8); // sepia intensity in saturation slot
        assert_eq!(desc.params[3], 2.0); // mode flag
    }

    #[test]
    fn test_invert_descriptor() {
        let desc = GpuFilterDispatcher::create_descriptor("invert", &[]).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[3], 3.0); // mode flag
    }

    #[test]
    fn test_blur_descriptor() {
        let params = vec![make_param("radius", 4.0), make_param("sigma", 2.0)];
        let desc = GpuFilterDispatcher::create_descriptor("blur", &params).unwrap();
        assert_eq!(desc.shader_name, "blur");
        assert_eq!(desc.params[0], 4.0); // radius
        assert_eq!(desc.params[2], 0.0); // horizontal
    }

    #[test]
    fn test_blur_vertical_descriptor() {
        let desc = GpuFilterDispatcher::create_blur_vertical_descriptor(5.0, 2.5);
        assert_eq!(desc.shader_name, "blur");
        assert_eq!(desc.params[2], 1.0); // vertical
    }

    #[test]
    fn test_hue_rotate_descriptor() {
        let params = vec![make_param("angle", 0.5)];
        let desc = GpuFilterDispatcher::create_descriptor("hue_rotate", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[0], 0.5); // angle in brightness slot
        assert_eq!(desc.params[3], 4.0); // mode flag
    }

    #[test]
    fn test_temperature_descriptor() {
        let params = vec![make_param("shift", 0.7)];
        let desc = GpuFilterDispatcher::create_descriptor("temperature", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[0], 0.7); // shift in brightness slot
        assert_eq!(desc.params[3], 5.0); // mode flag
    }

    #[test]
    fn test_vignette_descriptor() {
        let params = vec![make_param("intensity", 0.6)];
        let desc = GpuFilterDispatcher::create_descriptor("vignette", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[0], 0.6); // intensity in brightness slot
        assert_eq!(desc.params[3], 6.0); // mode flag
    }

    #[test]
    fn test_sharpen_descriptor() {
        let params = vec![make_param("strength", 1.5)];
        let desc = GpuFilterDispatcher::create_descriptor("sharpen", &params).unwrap();
        assert_eq!(desc.shader_name, "brightness");
        assert_eq!(desc.params[0], 1.5); // strength in brightness slot
        assert_eq!(desc.params[3], 7.0); // mode flag
    }

    #[test]
    fn test_unknown_effect_returns_none() {
        let desc = GpuFilterDispatcher::create_descriptor("unknown_effect", &[]);
        assert!(desc.is_none());
    }

    #[test]
    fn test_is_gpu_accelerated() {
        assert!(GpuFilterDispatcher::is_gpu_accelerated("brightness"));
        assert!(GpuFilterDispatcher::is_gpu_accelerated("Blur"));
        assert!(GpuFilterDispatcher::is_gpu_accelerated("HUE_ROTATE"));
        assert!(GpuFilterDispatcher::is_gpu_accelerated("chroma_key"));
        assert!(!GpuFilterDispatcher::is_gpu_accelerated("unknown_effect"));
    }

    #[test]
    fn test_gpu_accelerated_effects_count() {
        let effects = GpuFilterDispatcher::gpu_accelerated_effects();
        assert_eq!(effects.len(), 12);
    }

    #[test]
    fn test_chroma_key_descriptor() {
        let params = vec![
            make_param("target_hue", 120.0),
            make_param("hue_tolerance", 30.0),
            make_param("saturation_tolerance", 0.4),
            make_param("softness", 0.15),
            make_param("spill_suppression", 0.5),
        ];
        let desc = GpuFilterDispatcher::create_descriptor("chroma_key", &params).unwrap();
        assert_eq!(desc.shader_name, "chroma_key");
        assert_eq!(desc.params.len(), 5);
        assert_eq!(desc.params[0], 120.0); // target_hue
        assert_eq!(desc.params[1], 30.0);  // hue_tolerance
        assert_eq!(desc.params[2], 0.4);   // saturation_tolerance
        assert_eq!(desc.params[3], 0.15);  // softness
        assert_eq!(desc.params[4], 0.5);   // spill_suppression
    }

    #[test]
    fn test_chroma_key_descriptor_defaults() {
        let desc = GpuFilterDispatcher::create_descriptor("chroma_key", &[]).unwrap();
        assert_eq!(desc.shader_name, "chroma_key");
        assert_eq!(desc.params[0], 120.0); // default target_hue (green)
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let params = vec![make_param("value", 0.1)];
        assert!(GpuFilterDispatcher::create_descriptor("Brightness", &params).is_some());
        assert!(GpuFilterDispatcher::create_descriptor("BRIGHTNESS", &params).is_some());
        assert!(GpuFilterDispatcher::create_descriptor("brightness", &params).is_some());
    }

    #[test]
    fn test_param_value_by_name() {
        let params = vec![make_param("radius", 5.0), make_param("sigma", 2.0)];
        assert_eq!(param_value(&params, "radius"), Some(5.0));
        assert_eq!(param_value(&params, "sigma"), Some(2.0));
    }

    #[test]
    fn test_param_value_fallback_to_first() {
        let params = vec![make_param("value", 3.0)];
        assert_eq!(param_value(&params, "nonexistent"), Some(3.0));
    }

    #[test]
    fn test_param_value_empty() {
        let params: Vec<EffectParameter> = vec![];
        assert_eq!(param_value(&params, "anything"), None);
    }
}
