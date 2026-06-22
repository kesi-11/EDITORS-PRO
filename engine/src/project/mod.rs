//! Project management - Save/load project files
//!
//! Handles the serialization and persistence of project data,
//! including timeline state, media references, and settings.

pub mod format;
pub mod interop;

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::timeline::Timeline;

/// A complete editing project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub settings: ProjectSettings,
    pub timeline: Timeline,
    pub media_assets: Vec<MediaAsset>,
    pub thumbnail_path: Option<String>,
}

impl Project {
    /// Create a new empty project
    pub fn new(name: &str) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
            settings: ProjectSettings::default(),
            timeline: Timeline::new(),
            media_assets: Vec::new(),
            thumbnail_path: None,
        }
    }

    /// Create a project with custom settings
    pub fn with_settings(name: &str, settings: ProjectSettings) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
            settings,
            timeline: Timeline::new(),
            media_assets: Vec::new(),
            thumbnail_path: None,
        }
    }

    /// Add a media asset to the project
    pub fn add_media_asset(&mut self, asset: MediaAsset) {
        self.updated_at = chrono::Utc::now().timestamp_millis();
        self.media_assets.push(asset);
    }

    /// Remove a media asset by ID
    pub fn remove_media_asset(&mut self, asset_id: &str) -> Option<MediaAsset> {
        if let Some(pos) = self.media_assets.iter().position(|a| a.id == asset_id) {
            Some(self.media_assets.remove(pos))
        } else {
            None
        }
    }

    /// Find a media asset by ID
    pub fn find_media_asset(&self, asset_id: &str) -> Option<&MediaAsset> {
        self.media_assets.iter().find(|a| a.id == asset_id)
    }

    /// Save the project to a JSON file
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        log::info!("Project saved to {:?}", path);
        Ok(())
    }

    /// Load a project from a JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let project: Project = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization failed: {}", e))?;

        log::info!("Project loaded from {:?}", path);
        Ok(project)
    }

    /// Save to the custom .epp format (JSON wrapped in a zip)
    pub fn save_as_epp(&self, path: &Path) -> Result<(), String> {
        let save_data = format::EppFormat::from_project(self);
        save_data.save(path)
    }

    /// Load from .epp format
    pub fn load_from_epp(path: &Path) -> Result<Self, String> {
        let save_data = format::EppFormat::load(path)?;
        Ok(save_data.to_project())
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }
}

/// Project-level settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub sample_rate: u32,
    pub background_color: String,
    pub auto_save_interval_ms: u64,
    pub proxy_enabled: bool,
    pub proxy_quality: ProxyQuality,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            sample_rate: 44100,
            background_color: "#000000".to_string(),
            auto_save_interval_ms: 30000, // 30 seconds
            proxy_enabled: true,
            proxy_quality: ProxyQuality::Half,
        }
    }
}

/// Proxy quality for smooth editing on weak devices
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyQuality {
    Quarter,
    Half,
    ThreeQuarter,
}

/// A media asset imported into the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub media_type: MediaType,
    pub duration_ms: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size_bytes: u64,
    pub codec: Option<String>,
    pub bitrate: Option<u64>,
    pub import_date: i64,
    pub thumbnail_path: Option<String>,
}

impl MediaAsset {
    /// Create a new media asset from a file path
    pub fn new(file_path: &str, media_type: MediaType) -> Self {
        let path = Path::new(file_path);
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_size = std::fs::metadata(file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: file_path.to_string(),
            file_name,
            media_type,
            duration_ms: None,
            width: None,
            height: None,
            file_size_bytes: file_size,
            codec: None,
            bitrate: None,
            import_date: chrono::Utc::now().timestamp_millis(),
            thumbnail_path: None,
        }
    }

    /// Check if this asset is a video
    pub fn is_video(&self) -> bool {
        matches!(self.media_type, MediaType::Video)
    }

    /// Check if this asset is audio
    pub fn is_audio(&self) -> bool {
        matches!(self.media_type, MediaType::Audio)
    }

    /// Check if this asset is an image
    pub fn is_image(&self) -> bool {
        matches!(self.media_type, MediaType::Image)
    }
}

/// Type of media
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Video,
    Audio,
    Image,
}
