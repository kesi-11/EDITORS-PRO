//! Preset System — Save/load/share effect presets and workspace configurations.
//!
//! Professional preset management for effects, color grades, and workspace layouts.
//! Supports import/export, categories, favorites, and community sharing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetType {
    ColorGrade,
    Effect,
    Audio,
    Mask,
    Transition,
    Workspace,
    SpeedRamp,
    Grain,
    Export,
}

/// A saved preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub preset_type: PresetType,
    pub parameters: HashMap<String, f64>,
    pub thumbnail_path: Option<String>,
    pub is_builtin: bool,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub author: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

impl Preset {
    pub fn new(name: &str, preset_type: PresetType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            category: "Custom".to_string(),
            preset_type,
            parameters: HashMap::new(),
            thumbnail_path: None,
            is_builtin: false,
            is_favorite: false,
            tags: Vec::new(),
            author: "User".to_string(),
            created_at: now,
            modified_at: now,
        }
    }

    pub fn set_param(&mut self, key: &str, value: f64) {
        self.parameters.insert(key.to_string(), value);
        self.modified_at = chrono::Utc::now();
    }

    pub fn get_param(&self, key: &str) -> Option<f64> {
        self.parameters.get(key).copied()
    }

    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }

    /// Export preset as JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))
    }

    /// Import preset from JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))
    }
}

/// Preset manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetManager {
    pub presets: HashMap<String, Preset>,
}

impl PresetManager {
    pub fn new() -> Self { Self { presets: HashMap::new() } }

    pub fn add(&mut self, preset: Preset) { self.presets.insert(preset.id.clone(), preset); }
    pub fn get(&self, id: &str) -> Option<&Preset> { self.presets.get(id) }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Preset> { self.presets.get_mut(id) }
    pub fn remove(&mut self, id: &str) -> Option<Preset> { self.presets.remove(id) }

    pub fn by_type(&self, preset_type: PresetType) -> Vec<&Preset> {
        self.presets.values().filter(|p| p.preset_type == preset_type).collect()
    }

    pub fn by_category(&self, category: &str) -> Vec<&Preset> {
        self.presets.values().filter(|p| p.category == category).collect()
    }

    pub fn favorites(&self) -> Vec<&Preset> {
        self.presets.values().filter(|p| p.is_favorite).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Preset> {
        let q = query.to_lowercase();
        self.presets.values().filter(|p| {
            p.name.to_lowercase().contains(&q) ||
            p.description.to_lowercase().contains(&q) ||
            p.tags.iter().any(|t| t.to_lowercase().contains(&q))
        }).collect()
    }

    /// Load built-in presets for all types.
    pub fn load_builtin_presets(&mut self) {
        // Color Grade presets
        let cg_presets = vec![
            ("Cinematic Teal & Orange", "cinematic", vec![
                ("lift_r", -0.02), ("lift_g", -0.01), ("lift_b", 0.02),
                ("gamma_r", 0.04), ("gamma_g", 0.0), ("gamma_b", -0.03),
                ("gain_r", 0.06), ("gain_g", 0.02), ("gain_b", -0.04),
                ("saturation", 1.1), ("contrast", 1.15),
            ]),
            ("Vintage Fade", "vintage", vec![
                ("lift_r", 0.03), ("lift_g", 0.02), ("lift_b", 0.01),
                ("gamma_r", -0.01), ("gamma_g", 0.0), ("gamma_b", 0.02),
                ("saturation", 0.85), ("contrast", 0.9),
            ]),
            ("High Contrast BW", "bw", vec![
                ("saturation", 0.0), ("contrast", 1.5), ("gamma_r", 0.1),
            ]),
            ("Soft Pastel", "soft", vec![
                ("saturation", 0.7), ("contrast", 0.85), ("lift_r", 0.05),
                ("lift_g", 0.04), ("lift_b", 0.06),
            ]),
        ];

        for (name, category, params) in cg_presets {
            let mut preset = Preset::new(name, PresetType::ColorGrade);
            preset.category = category.to_string();
            preset.is_builtin = true;
            for (key, value) in params {
                preset.set_param(key, value);
            }
            self.add(preset);
        }
    }

    pub fn count(&self) -> usize { self.presets.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_new() {
        let p = Preset::new("Test", PresetType::ColorGrade);
        assert_eq!(p.name, "Test");
        assert!(!p.is_builtin);
        assert!(!p.is_favorite);
    }

    #[test]
    fn test_preset_params() {
        let mut p = Preset::new("Test", PresetType::Effect);
        p.set_param("intensity", 0.75);
        assert_eq!(p.get_param("intensity"), Some(0.75));
        assert_eq!(p.get_param("missing"), None);
    }

    #[test]
    fn test_preset_toggle_favorite() {
        let mut p = Preset::new("Test", PresetType::ColorGrade);
        p.toggle_favorite();
        assert!(p.is_favorite);
        p.toggle_favorite();
        assert!(!p.is_favorite);
    }

    #[test]
    fn test_preset_json_roundtrip() {
        let mut p = Preset::new("Test", PresetType::Audio);
        p.set_param("gain", 1.5);
        let json = p.to_json().unwrap();
        let restored = Preset::from_json(&json).unwrap();
        assert_eq!(restored.name, "Test");
        assert_eq!(restored.get_param("gain"), Some(1.5));
    }

    #[test]
    fn test_preset_manager_add() {
        let mut mgr = PresetManager::new();
        mgr.add(Preset::new("Test", PresetType::ColorGrade));
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_preset_manager_by_type() {
        let mut mgr = PresetManager::new();
        mgr.add(Preset::new("CG1", PresetType::ColorGrade));
        mgr.add(Preset::new("FX1", PresetType::Effect));
        assert_eq!(mgr.by_type(PresetType::ColorGrade).len(), 1);
    }

    #[test]
    fn test_preset_manager_favorites() {
        let mut mgr = PresetManager::new();
        let mut p = Preset::new("Fav", PresetType::ColorGrade);
        p.is_favorite = true;
        mgr.add(p);
        mgr.add(Preset::new("NotFav", PresetType::Effect));
        assert_eq!(mgr.favorites().len(), 1);
    }

    #[test]
    fn test_preset_manager_search() {
        let mut mgr = PresetManager::new();
        let mut p = Preset::new("Cinematic Look", PresetType::ColorGrade);
        p.tags.push("cinematic".to_string());
        mgr.add(p);
        assert_eq!(mgr.search("cinematic").len(), 1);
        assert_eq!(mgr.search("missing").len(), 0);
    }

    #[test]
    fn test_builtin_presets() {
        let mut mgr = PresetManager::new();
        mgr.load_builtin_presets();
        assert!(mgr.count() >= 4);
        assert!(mgr.by_type(PresetType::ColorGrade).len() >= 4);
    }

    #[test]
    fn test_preset_manager_remove() {
        let mut mgr = PresetManager::new();
        let p = Preset::new("Test", PresetType::ColorGrade);
        let id = p.id.clone();
        mgr.add(p);
        assert!(mgr.remove(&id).is_some());
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_preset_by_category() {
        let mut mgr = PresetManager::new();
        let mut p = Preset::new("Test", PresetType::ColorGrade);
        p.category = "cinematic".to_string();
        mgr.add(p);
        assert_eq!(mgr.by_category("cinematic").len(), 1);
    }
}
