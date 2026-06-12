//! Effects module - Visual effects pipeline
//!
//! Manages the application of visual effects including filters,
//! transitions, and text overlays to timeline frames.

pub mod filters;
pub mod text_render;
pub mod transitions;

use serde::{Deserialize, Serialize};

/// Types of visual effects supported by the engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    Filter,
    Transition,
    TextOverlay,
    ChromaKey,
}

/// An effect that can be applied to a clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub id: String,
    pub name: String,
    pub effect_type: EffectType,
    pub enabled: bool,
    pub order: u32,
    pub parameters: Vec<EffectParameter>,
}

/// A single parameter for an effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectParameter {
    pub name: String,
    pub display_name: String,
    pub value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub step: f32,
}

impl EffectParameter {
    pub fn new(name: &str, display_name: &str, value: f32, min: f32, max: f32, step: f32) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            value: value.clamp(min, max),
            min_value: min,
            max_value: max,
            default_value: value,
            step,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min_value, self.max_value);
    }

    pub fn reset(&mut self) {
        self.value = self.default_value;
    }
}

/// Pipeline that applies effects in order to frame data
pub struct EffectsPipeline {
    effects: Vec<Effect>,
}

impl EffectsPipeline {
    pub fn new() -> Self {
        Self { effects: Vec::new() }
    }

    /// Add an effect to the pipeline
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
        self.effects.sort_by_key(|e| e.order);
    }

    /// Remove an effect by ID
    pub fn remove_effect(&mut self, effect_id: &str) -> Option<Effect> {
        if let Some(pos) = self.effects.iter().position(|e| e.id == effect_id) {
            Some(self.effects.remove(pos))
        } else {
            None
        }
    }

    /// Get all enabled effects in order
    pub fn enabled_effects(&self) -> Vec<&Effect> {
        self.effects.iter().filter(|e| e.enabled).collect()
    }

    /// Apply all enabled effects to frame data (CPU fallback for MVP)
    pub fn apply(&self, frame_data: &mut [u8], width: u32, height: u32) {
        for effect in self.enabled_effects() {
            match effect.effect_type {
                EffectType::Filter => {
                    for param in &effect.parameters {
                        crate::renderer::shader::ShaderManager::apply_cpu_effect(
                            frame_data, &param.name, param.value,
                        );
                    }
                }
                EffectType::TextOverlay => {
                    // Text overlay is handled separately in the renderer
                    log::debug!("Text overlay effect: {} (handled in renderer)", effect.name);
                }
                EffectType::Transition => {
                    // Transitions are handled during clip transitions
                    log::debug!("Transition effect: {} (handled during transitions)", effect.name);
                }
                EffectType::ChromaKey => {
                    // Phase 4: Chroma key implementation
                    log::debug!("Chroma key effect: {} (Phase 4)", effect.name);
                }
            }
        }
    }

    /// Get effects of a specific type
    pub fn effects_of_type(&self, effect_type: &EffectType) -> Vec<&Effect> {
        self.effects.iter().filter(|e| &e.effect_type == effect_type).collect()
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}
