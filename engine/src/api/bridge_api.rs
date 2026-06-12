//! Bridge API for flutter_rust_bridge v2
//!
//! This module provides the `EditorsProEngineApi` struct that wraps
//! `EditorsProEngine` in a `std::sync::Mutex` so that all methods can
//! use `&self` instead of `&mut self`, which is required by
//! flutter_rust_bridge v2.
//!
//! All DTOs are re-exported from the parent `api` module so that the
//! generated Dart code has access to them.

use std::sync::Mutex;

use super::{
    ClipInfo, EditorsProEngine, MediaAssetInfo, ProjectInfo, TrackInfo,
};
use crate::project::ProjectSettings;
use crate::timeline::track::TrackType;

/// Re-export DTOs for the bridge
pub use super::{ClipInfo as BridgeClipInfo, MediaAssetInfo as BridgeMediaAssetInfo, ProjectInfo as BridgeProjectInfo, TrackInfo as BridgeTrackInfo};

/// Bridge-compatible project settings
///
/// flutter_rust_bridge v2 requires all types passed across the bridge
/// to derive `Serialize, Deserialize`. The engine's internal
/// `ProjectSettings` may have fields that are not serializable, so we
/// provide a bridge-specific version that only contains the fields
/// Flutter needs to create a project.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BridgeProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
}

impl From<BridgeProjectSettings> for ProjectSettings {
    fn from(s: BridgeProjectSettings) -> Self {
        // Start from defaults and override only the fields Flutter cares about.
        let mut settings = ProjectSettings::default();
        settings.width = s.width;
        settings.height = s.height;
        settings.fps = s.fps;
        settings
    }
}

/// The main API struct exposed to Flutter via flutter_rust_bridge v2.
///
/// Wraps `EditorsProEngine` in a `Mutex` so that every method can
/// take `&self`, satisfying flutter_rust_bridge v2's requirement that
/// the API struct is shared (not exclusively borrowed).
pub struct EditorsProEngineApi {
    inner: Mutex<EditorsProEngine>,
}

impl EditorsProEngineApi {
    /// Create a new engine API instance.
    ///
    /// The underlying `EditorsProEngine` is created but **not**
    /// initialized — you must call `initialize()` before any other
    /// operation.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(EditorsProEngine::new()),
        }
    }

    /// Initialize the engine.
    ///
    /// Must be called once before any other operations.
    /// Sets up logging and FFmpeg libraries.
    pub fn initialize(&self) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.initialize().map_err(|e| format!("{}", e))
    }

    /// Create a new project with the given name and optional settings.
    ///
    /// Returns a `ProjectInfo` DTO describing the newly created project.
    pub fn create_project(
        &self,
        name: String,
        settings: Option<BridgeProjectSettings>,
    ) -> Result<ProjectInfo, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        let project_settings = settings.map(ProjectSettings::from);
        engine
            .create_project(&name, project_settings)
            .map_err(|e| format!("{}", e))
    }

    /// Import a media file into the current project.
    ///
    /// The file at `file_path` must be accessible from the native side.
    /// Returns a `MediaAssetInfo` DTO with metadata extracted from the file.
    pub fn import_media(&self, file_path: String) -> Result<MediaAssetInfo, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.import_media(&file_path)
    }

    /// Add a track to the timeline.
    ///
    /// `track_type` must be one of "Video", "Audio", "Text", or "Effect".
    /// Returns a `TrackInfo` DTO for the new track.
    pub fn add_track(
        &self,
        track_type: String,
        name: Option<String>,
    ) -> Result<TrackInfo, String> {
        let tt = match track_type.as_str() {
            "Video" => TrackType::Video,
            "Audio" => TrackType::Audio,
            "Text" => TrackType::Text,
            "Effect" => TrackType::Effect,
            other => return Err(format!("Unknown track type: {}", other)),
        };
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.add_track(tt, name)
    }

    /// Add a clip to a track.
    ///
    /// - `track_id`: ID of the target track
    /// - `asset_id`: ID of the media asset
    /// - `start_ms`: Position on the timeline in milliseconds
    /// - `duration_ms`: Duration of the clip; pass 0 to use the asset's natural duration
    pub fn add_clip(
        &self,
        track_id: String,
        asset_id: String,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<ClipInfo, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.add_clip(&track_id, &asset_id, start_ms, duration_ms)
    }

    /// Trim a clip by adjusting its in/out points.
    pub fn trim_clip(
        &self,
        clip_id: String,
        trim_start_ms: u64,
        trim_end_ms: u64,
    ) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.trim_clip(&clip_id, trim_start_ms, trim_end_ms)
    }

    /// Split a clip at the given timestamp.
    ///
    /// Returns a tuple of `(left_clip, right_clip)` DTOs.
    pub fn split_clip(
        &self,
        clip_id: String,
        time_ms: u64,
    ) -> Result<(ClipInfo, ClipInfo), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.split_clip(&clip_id, time_ms)
    }

    /// Move a clip to a new position on the timeline.
    ///
    /// Optionally move it to a different track via `new_track_id`.
    pub fn move_clip(
        &self,
        clip_id: String,
        new_start_ms: u64,
        new_track_id: Option<String>,
    ) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.move_clip(&clip_id, new_start_ms, new_track_id)
    }

    /// Remove a clip from the timeline.
    pub fn remove_clip(&self, clip_id: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.remove_clip(&clip_id)
    }

    /// Render a single preview frame at the given timestamp.
    ///
    /// Returns PNG-encoded image bytes that can be displayed directly
    /// by Flutter's `Image.memory()`.
    ///
    /// The raw RGBA data from the engine is encoded to PNG on the Rust
    /// side because Flutter does not have a built-in RGBA → widget
    /// decoder.
    pub fn get_frame(&self, time_ms: u64) -> Result<Vec<u8>, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;

        // The engine returns a FrameData struct with width, height, and RGBA data.
        let frame_data = engine.get_frame(time_ms).map_err(|e| format!("{}", e))?;

        encode_rgba_to_png(&frame_data.data, frame_data.width, frame_data.height)
    }

    /// Undo the last action.
    pub fn undo(&self) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.undo()
    }

    /// Redo the last undone action.
    pub fn redo(&self) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.redo()
    }

    /// Save the current project to an `.epp` file.
    pub fn save_project(&self, path: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.save_project(&path)
    }

    /// Load a project from an `.epp` file.
    pub fn load_project(&self, path: String) -> Result<ProjectInfo, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.load_project(&path)
    }

    /// Get information about the current project.
    ///
    /// Returns `null` (in Dart: `null`) when no project is open.
    pub fn get_project_info(&self) -> Option<ProjectInfo> {
        let engine = self.inner.lock().ok()?;
        engine.get_project_info()
    }

    /// Get the total timeline duration in milliseconds.
    pub fn get_timeline_duration(&self) -> u64 {
        match self.inner.lock() {
            Ok(engine) => engine.get_timeline_duration(),
            Err(_) => 0,
        }
    }

    /// Check whether undo is available.
    pub fn can_undo(&self) -> bool {
        match self.inner.lock() {
            Ok(engine) => engine.can_undo(),
            Err(_) => false,
        }
    }

    /// Check whether redo is available.
    pub fn can_redo(&self) -> bool {
        match self.inner.lock() {
            Ok(engine) => engine.can_redo(),
            Err(_) => false,
        }
    }
}

/// Encode raw RGBA pixel data to a PNG byte buffer.
///
/// Uses the `image` crate's PngEncoder to produce compressed PNG data
/// suitable for consumption by Flutter's `Image.memory()`.
fn encode_rgba_to_png(rgba_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;

    let expected_size = (width as usize) * (height as usize) * 4;
    if rgba_data.len() < expected_size {
        return Err(format!(
            "Frame data too short: expected {} bytes ({}x{} RGBA), got {}",
            expected_size,
            width,
            height,
            rgba_data.len()
        ));
    }

    let mut png_buf = std::io::Cursor::new(Vec::with_capacity(expected_size / 4));
    image::codecs::png::PngEncoder::new(&mut png_buf)
        .write_image(
            rgba_data,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG encoding failed: {}", e))?;

    Ok(png_buf.into_inner())
}
