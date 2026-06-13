//! Custom Workspace — User-defined panel layouts, keyboard shortcuts, and preferences.
//!
//! Allows editors to save and switch between custom workspace layouts,
//! define keyboard shortcuts, and persist user preferences.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position of a panel in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelPosition {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    Floating,
}

/// A panel in the workspace layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePanel {
    pub panel_id: String,
    pub panel_name: String,
    pub position: PanelPosition,
    pub width_ratio: f32,     // 0..1 relative to container
    pub height_ratio: f32,
    pub is_collapsed: bool,
    pub tab_index: u32,
}

/// A complete workspace layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub id: String,
    pub name: String,
    pub panels: Vec<WorkspacePanel>,
    pub is_builtin: bool,
}

impl WorkspaceLayout {
    pub fn new(name: &str) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), name: name.to_string(), panels: Vec::new(), is_builtin: false }
    }

    pub fn add_panel(&mut self, panel: WorkspacePanel) { self.panels.push(panel); }

    pub fn get_panel(&self, id: &str) -> Option<&WorkspacePanel> {
        self.panels.iter().find(|p| p.panel_id == id)
    }

    pub fn panels_at(&self, position: PanelPosition) -> Vec<&WorkspacePanel> {
        self.panels.iter().filter(|p| p.position == position).collect()
    }

    /// Built-in "Edit" workspace layout.
    pub fn edit_layout() -> Self {
        let mut layout = Self { id: "builtin_edit".to_string(), name: "Edit".to_string(), panels: Vec::new(), is_builtin: true };
        layout.add_panel(WorkspacePanel { panel_id: "inspector".into(), panel_name: "Inspector".into(), position: PanelPosition::Right, width_ratio: 0.25, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "timeline".into(), panel_name: "Timeline".into(), position: PanelPosition::Bottom, width_ratio: 1.0, height_ratio: 0.35, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "preview".into(), panel_name: "Preview".into(), position: PanelPosition::Center, width_ratio: 0.5, height_ratio: 0.65, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "effects".into(), panel_name: "Effects".into(), position: PanelPosition::Left, width_ratio: 0.25, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        layout
    }

    /// Built-in "Color" workspace layout.
    pub fn color_layout() -> Self {
        let mut layout = Self { id: "builtin_color".to_string(), name: "Color".to_string(), panels: Vec::new(), is_builtin: true };
        layout.add_panel(WorkspacePanel { panel_id: "color_grading".into(), panel_name: "Color Grading".into(), position: PanelPosition::Right, width_ratio: 0.3, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "scopes".into(), panel_name: "Scopes".into(), position: PanelPosition::Left, width_ratio: 0.25, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "preview".into(), panel_name: "Preview".into(), position: PanelPosition::Center, width_ratio: 0.45, height_ratio: 0.65, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "timeline".into(), panel_name: "Timeline".into(), position: PanelPosition::Bottom, width_ratio: 1.0, height_ratio: 0.35, is_collapsed: false, tab_index: 0 });
        layout
    }

    /// Built-in "Audio" workspace layout.
    pub fn audio_layout() -> Self {
        let mut layout = Self { id: "builtin_audio".to_string(), name: "Audio".to_string(), panels: Vec::new(), is_builtin: true };
        layout.add_panel(WorkspacePanel { panel_id: "mixer".into(), panel_name: "Mixer".into(), position: PanelPosition::Right, width_ratio: 0.3, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "waveform".into(), panel_name: "Waveform".into(), position: PanelPosition::Center, width_ratio: 0.7, height_ratio: 0.65, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "timeline".into(), panel_name: "Timeline".into(), position: PanelPosition::Bottom, width_ratio: 1.0, height_ratio: 0.35, is_collapsed: false, tab_index: 0 });
        layout
    }
}

/// Keyboard shortcut binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub action: String,
    pub key: String,          // e.g., "Ctrl+S"
    pub context: String,      // e.g., "global", "timeline", "preview"
    pub description: String,
}

/// User preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,                // "dark", "light", "system"
    pub language: String,             // ISO 639-1
    pub auto_save_interval_sec: u32,
    pub max_undo_levels: u32,
    pub default_fps: f64,
    pub default_resolution: (u32, u32),
    pub preview_quality: String,
    pub show_waveform: bool,
    pub show_scopes: bool,
    pub gpu_acceleration: bool,
    pub proxy_mode: String,
    pub key_bindings: Vec<KeyBinding>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            auto_save_interval_sec: 120,
            max_undo_levels: 100,
            default_fps: 30.0,
            default_resolution: (1920, 1080),
            preview_quality: "medium".to_string(),
            show_waveform: true,
            show_scopes: false,
            gpu_acceleration: true,
            proxy_mode: "auto".to_string(),
            key_bindings: Self::default_key_bindings(),
        }
    }
}

impl UserPreferences {
    fn default_key_bindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding { action: "save".into(), key: "Ctrl+S".into(), context: "global".into(), description: "Save project".into() },
            KeyBinding { action: "undo".into(), key: "Ctrl+Z".into(), context: "global".into(), description: "Undo".into() },
            KeyBinding { action: "redo".into(), key: "Ctrl+Shift+Z".into(), context: "global".into(), description: "Redo".into() },
            KeyBinding { action: "play_pause".into(), key: "Space".into(), context: "global".into(), description: "Play/Pause".into() },
            KeyBinding { action: "split".into(), key: "S".into(), context: "timeline".into(), description: "Split clip".into() },
            KeyBinding { action: "delete".into(), key: "Delete".into(), context: "timeline".into(), description: "Delete selection".into() },
            KeyBinding { action: "copy".into(), key: "Ctrl+C".into(), context: "global".into(), description: "Copy".into() },
            KeyBinding { action: "paste".into(), key: "Ctrl+V".into(), context: "global".into(), description: "Paste".into() },
            KeyBinding { action: "zoom_in".into(), key: "Ctrl+=".into(), context: "timeline".into(), description: "Zoom in".into() },
            KeyBinding { action: "zoom_out".into(), key: "Ctrl+-".into(), context: "timeline".into(), description: "Zoom out".into() },
            KeyBinding { action: "add_marker".into(), key: "M".into(), context: "timeline".into(), description: "Add marker".into() },
            KeyBinding { action: "export".into(), key: "Ctrl+E".into(), context: "global".into(), description: "Export".into() },
        ]
    }

    pub fn find_binding(&self, action: &str) -> Option<&KeyBinding> {
        self.key_bindings.iter().find(|k| k.action == action)
    }

    pub fn set_binding(&mut self, action: &str, key: &str) {
        if let Some(binding) = self.key_bindings.iter_mut().find(|k| k.action == action) {
            binding.key = key.to_string();
        } else {
            self.key_bindings.push(KeyBinding { action: action.to_string(), key: key.to_string(), context: "global".to_string(), description: String::new() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_layout_new() {
        let layout = WorkspaceLayout::new("Custom");
        assert_eq!(layout.name, "Custom");
        assert!(layout.panels.is_empty());
    }

    #[test]
    fn test_workspace_add_panel() {
        let mut layout = WorkspaceLayout::new("Test");
        layout.add_panel(WorkspacePanel { panel_id: "test".into(), panel_name: "Test".into(), position: PanelPosition::Left, width_ratio: 0.25, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        assert_eq!(layout.panels.len(), 1);
    }

    #[test]
    fn test_workspace_panels_at() {
        let mut layout = WorkspaceLayout::new("Test");
        layout.add_panel(WorkspacePanel { panel_id: "left1".into(), panel_name: "L1".into(), position: PanelPosition::Left, width_ratio: 0.25, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        layout.add_panel(WorkspacePanel { panel_id: "right1".into(), panel_name: "R1".into(), position: PanelPosition::Right, width_ratio: 0.25, height_ratio: 1.0, is_collapsed: false, tab_index: 0 });
        assert_eq!(layout.panels_at(PanelPosition::Left).len(), 1);
    }

    #[test]
    fn test_builtin_edit_layout() {
        let layout = WorkspaceLayout::edit_layout();
        assert!(layout.is_builtin);
        assert!(layout.panels.len() >= 4);
    }

    #[test]
    fn test_builtin_color_layout() {
        let layout = WorkspaceLayout::color_layout();
        assert!(layout.is_builtin);
    }

    #[test]
    fn test_builtin_audio_layout() {
        let layout = WorkspaceLayout::audio_layout();
        assert!(layout.is_builtin);
    }

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.theme, "dark");
        assert_eq!(prefs.max_undo_levels, 100);
        assert!(prefs.gpu_acceleration);
    }

    #[test]
    fn test_default_key_bindings() {
        let prefs = UserPreferences::default();
        assert!(prefs.key_bindings.len() >= 10);
        let save = prefs.find_binding("save");
        assert!(save.is_some());
        assert_eq!(save.unwrap().key, "Ctrl+S");
    }

    #[test]
    fn test_set_binding_existing() {
        let mut prefs = UserPreferences::default();
        prefs.set_binding("save", "Ctrl+Shift+S");
        assert_eq!(prefs.find_binding("save").unwrap().key, "Ctrl+Shift+S");
    }

    #[test]
    fn test_set_binding_new() {
        let mut prefs = UserPreferences::default();
        let count = prefs.key_bindings.len();
        prefs.set_binding("custom_action", "F12");
        assert_eq!(prefs.key_bindings.len(), count + 1);
    }

    #[test]
    fn test_get_panel_by_id() {
        let layout = WorkspaceLayout::edit_layout();
        assert!(layout.get_panel("inspector").is_some());
        assert!(layout.get_panel("nonexistent").is_none());
    }
}
