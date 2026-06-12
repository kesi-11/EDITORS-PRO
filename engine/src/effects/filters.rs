//! Filter effects - Color adjustments and visual filters
//!
//! Provides a catalog of built-in filters that can be applied to video frames.

use serde::{Deserialize, Serialize};

use super::{Effect, EffectParameter, EffectType};

/// Built-in filter types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    Brightness,
    Contrast,
    Saturation,
    Hue,
    Blur,
    Sharpen,
    Grayscale,
    Sepia,
    Invert,
    Vignette,
    Temperature,
}

impl FilterType {
    /// Get the display name for this filter
    pub fn display_name(&self) -> &str {
        match self {
            FilterType::Brightness => "Brightness",
            FilterType::Contrast => "Contrast",
            FilterType::Saturation => "Saturation",
            FilterType::Hue => "Hue",
            FilterType::Blur => "Blur",
            FilterType::Sharpen => "Sharpen",
            FilterType::Grayscale => "Grayscale",
            FilterType::Sepia => "Sepia",
            FilterType::Invert => "Invert",
            FilterType::Vignette => "Vignette",
            FilterType::Temperature => "Temperature",
        }
    }

    /// Create an Effect instance for this filter type
    pub fn to_effect(&self) -> Effect {
        let params = self.default_parameters();
        Effect {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.display_name().to_string(),
            effect_type: EffectType::Filter,
            enabled: true,
            order: 0,
            parameters: params,
        }
    }

    /// Get the default parameters for this filter
    pub fn default_parameters(&self) -> Vec<EffectParameter> {
        match self {
            FilterType::Brightness => vec![
                EffectParameter::new("brightness", "Brightness", 0.0, -1.0, 1.0, 0.01),
            ],
            FilterType::Contrast => vec![
                EffectParameter::new("contrast", "Contrast", 0.0, -1.0, 1.0, 0.01),
            ],
            FilterType::Saturation => vec![
                EffectParameter::new("saturation", "Saturation", 1.0, 0.0, 3.0, 0.01),
            ],
            FilterType::Hue => vec![
                EffectParameter::new("hue", "Hue Shift", 0.0, -180.0, 180.0, 1.0),
            ],
            FilterType::Blur => vec![
                EffectParameter::new("blur", "Blur Radius", 0.0, 0.0, 20.0, 0.5),
            ],
            FilterType::Sharpen => vec![
                EffectParameter::new("sharpen", "Sharpness", 0.0, 0.0, 2.0, 0.05),
            ],
            FilterType::Grayscale => vec![
                EffectParameter::new("grayscale", "Intensity", 1.0, 0.0, 1.0, 0.01),
            ],
            FilterType::Sepia => vec![
                EffectParameter::new("sepia", "Intensity", 1.0, 0.0, 1.0, 0.01),
            ],
            FilterType::Invert => vec![],
            FilterType::Vignette => vec![
                EffectParameter::new("vignette", "Intensity", 0.5, 0.0, 1.0, 0.01),
                EffectParameter::new("vignette_radius", "Radius", 0.5, 0.0, 1.0, 0.01),
            ],
            FilterType::Temperature => vec![
                EffectParameter::new("temperature", "Temperature", 0.0, -1.0, 1.0, 0.01),
            ],
        }
    }

    /// Get all available filter types
    pub fn all_filters() -> Vec<FilterType> {
        vec![
            FilterType::Brightness,
            FilterType::Contrast,
            FilterType::Saturation,
            FilterType::Hue,
            FilterType::Blur,
            FilterType::Sharpen,
            FilterType::Grayscale,
            FilterType::Sepia,
            FilterType::Invert,
            FilterType::Vignette,
            FilterType::Temperature,
        ]
    }
}

/// Preset filter combinations for quick application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filters: Vec<FilterType>,
    pub parameter_overrides: Vec<(String, f32)>,
}

impl FilterPreset {
    /// Get built-in filter presets
    pub fn built_in_presets() -> Vec<FilterPreset> {
        vec![
            FilterPreset {
                id: "cinematic".to_string(),
                name: "Cinematic".to_string(),
                description: "Warm cinematic look with reduced saturation".to_string(),
                filters: vec![FilterType::Saturation, FilterType::Contrast],
                parameter_overrides: vec![
                    ("saturation".to_string(), 0.8),
                    ("contrast".to_string(), 0.15),
                ],
            },
            FilterPreset {
                id: "vintage".to_string(),
                name: "Vintage".to_string(),
                description: "Faded vintage look with sepia tones".to_string(),
                filters: vec![FilterType::Sepia, FilterType::Contrast, FilterType::Brightness],
                parameter_overrides: vec![
                    ("sepia".to_string(), 0.6),
                    ("contrast".to_string(), -0.1),
                    ("brightness".to_string(), 0.05),
                ],
            },
            FilterPreset {
                id: "dramatic".to_string(),
                name: "Dramatic".to_string(),
                description: "High contrast dramatic look".to_string(),
                filters: vec![FilterType::Contrast, FilterType::Saturation, FilterType::Vignette],
                parameter_overrides: vec![
                    ("contrast".to_string(), 0.4),
                    ("saturation".to_string(), 1.3),
                    ("vignette".to_string(), 0.6),
                ],
            },
            FilterPreset {
                id: "cool".to_string(),
                name: "Cool".to_string(),
                description: "Cool blue tones".to_string(),
                filters: vec![FilterType::Temperature, FilterType::Contrast],
                parameter_overrides: vec![
                    ("temperature".to_string(), -0.3),
                    ("contrast".to_string(), 0.1),
                ],
            },
            FilterPreset {
                id: "warm".to_string(),
                name: "Warm".to_string(),
                description: "Warm golden tones".to_string(),
                filters: vec![FilterType::Temperature, FilterType::Saturation],
                parameter_overrides: vec![
                    ("temperature".to_string(), 0.3),
                    ("saturation".to_string(), 1.2),
                ],
            },
            FilterPreset {
                id: "noir".to_string(),
                name: "Noir".to_string(),
                description: "Classic black and white with high contrast".to_string(),
                filters: vec![FilterType::Grayscale, FilterType::Contrast, FilterType::Vignette],
                parameter_overrides: vec![
                    ("grayscale".to_string(), 1.0),
                    ("contrast".to_string(), 0.3),
                    ("vignette".to_string(), 0.4),
                ],
            },
        ]
    }
}
