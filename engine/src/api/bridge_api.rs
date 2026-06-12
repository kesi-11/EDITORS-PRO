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
    ClipInfo, EditorsProEngine, EffectInfo, EffectParameterInfo, FilterPresetInfo,
    FilterTypeInfo, MediaAssetInfo, ProjectInfo, TrackInfo, TimelineState,
    TransitionInfo, TransitionTypeInfo,
};
use crate::audio::ducking::DuckingConfig;
use crate::export_engine::{ExportProgress, ExportResult, ExportSettings, ExportStage, OutputFormat, VideoCodec};
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

/// Bridge-compatible export settings
///
/// This is the Flutter-facing version of `ExportSettings` that is
/// constructed from the export screen UI and passed across the bridge.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BridgeExportSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub bitrate_kbps: u64,
    pub codec: String,
    pub format: String,
    pub audio_bitrate_kbps: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: u32,
    pub include_audio: bool,
    pub two_pass: bool,
}

impl From<BridgeExportSettings> for ExportSettings {
    fn from(s: BridgeExportSettings) -> Self {
        Self {
            width: s.width,
            height: s.height,
            fps: s.fps,
            bitrate_kbps: s.bitrate_kbps,
            codec: VideoCodec::from_str_lossy(&s.codec).unwrap_or(VideoCodec::H264),
            format: OutputFormat::from_str_lossy(&s.format).unwrap_or(OutputFormat::Mp4),
            audio_bitrate_kbps: s.audio_bitrate_kbps,
            audio_sample_rate: s.audio_sample_rate,
            audio_channels: s.audio_channels,
            include_audio: s.include_audio,
            two_pass: s.two_pass,
        }
    }
}

/// Bridge-compatible export progress
///
/// Simplified version of `ExportProgress` that uses a String for the
/// stage instead of the enum, which is easier to handle in Dart.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BridgeExportProgress {
    /// Percentage complete (0.0 to 1.0)
    pub progress: f32,
    /// Current frame being processed
    pub current_frame: u64,
    /// Total frames to process
    pub total_frames: u64,
    /// Estimated time remaining in seconds
    pub estimated_seconds_remaining: u64,
    /// Current processing stage name
    pub stage_name: String,
}

impl From<ExportProgress> for BridgeExportProgress {
    fn from(p: ExportProgress) -> Self {
        Self {
            progress: p.progress,
            current_frame: p.current_frame,
            total_frames: p.total_frames,
            estimated_seconds_remaining: p.estimated_seconds_remaining,
            stage_name: p.stage.display_name().to_string(),
        }
    }
}

/// Bridge-compatible export result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BridgeExportResult {
    pub success: bool,
    pub output_path: String,
    pub file_size_bytes: u64,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub file_size_human: String,
}

impl From<ExportResult> for BridgeExportResult {
    fn from(r: ExportResult) -> Self {
        Self {
            file_size_human: r.file_size_human(),
            success: r.success,
            output_path: r.output_path,
            file_size_bytes: r.file_size_bytes,
            duration_ms: r.duration_ms,
            error_message: r.error_message,
        }
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

    /// Export the project as a video file with real FFmpeg encoding.
    ///
    /// This is a **synchronous** export that blocks the calling thread
    /// until complete. For progress reporting, use
    /// `export_video_with_progress()` instead, or call this from a
    /// background isolate.
    ///
    /// Returns a `BridgeExportResult` with details about the exported file.
    pub fn export_video(
        &self,
        output_path: String,
        settings: BridgeExportSettings,
    ) -> Result<BridgeExportResult, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        let export_settings = ExportSettings::from(settings);

        let result = engine.export_video(&output_path, export_settings, &|_progress| {
            // No-op for synchronous export — progress is not reported
        })?;

        Ok(BridgeExportResult::from(result))
    }

    /// Export the project with progress reporting via a callback.
    ///
    /// This method runs the encoding loop on the calling thread but
    /// invokes the `progress_callback` for each frame, allowing
    /// Flutter to update its UI. The callback is invoked inside the
    /// Mutex lock, so it should be lightweight (e.g., just store the
    /// progress in a shared variable or send it through a channel).
    ///
    /// For the StreamSink-based version (recommended for flutter_rust_bridge),
    /// use `export_video_streaming()` which returns progress items as a Stream.
    pub fn export_video_with_callback(
        &self,
        output_path: String,
        settings: BridgeExportSettings,
        progress_callback: impl Fn(BridgeExportProgress),
    ) -> Result<BridgeExportResult, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        let export_settings = ExportSettings::from(settings);

        let result = engine.export_video(&output_path, export_settings, &|progress| {
            progress_callback(BridgeExportProgress::from(progress));
        })?;

        Ok(BridgeExportResult::from(result))
    }

    /// Request cancellation of an in-progress export.
    ///
    /// The encoding loop checks a cancellation flag before each frame
    /// and will abort early if set. This is safe to call from any
    /// thread (including the Flutter UI thread).
    pub fn cancel_export(&self) -> Result<(), String> {
        let engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.cancel_export();
        Ok(())
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

    /// Get the list of available export presets.
    ///
    /// Returns a list of preset names that can be used with
    /// `export_video()` or `export_video_with_callback()`.
    pub fn get_export_presets(&self) -> Vec<String> {
        vec![
            "720p".to_string(),
            "1080p".to_string(),
            "4K".to_string(),
            "Social Vertical".to_string(),
            "Social Square".to_string(),
        ]
    }

    /// Get the export settings for a named preset.
    ///
    /// Returns `None` if the preset name is not recognized.
    pub fn get_export_preset(&self, name: String) -> Option<BridgeExportSettings> {
        let settings = ExportSettings::preset_by_name(&name)?;
        Some(BridgeExportSettings::from(settings))
    }

    // ─── Audio Operations ─────────────────────────────────────────────

    /// Set the volume level for a track.
    ///
    /// Volume is clamped to 0.0–2.0 (0.0 = mute, 1.0 = normal, 2.0 = double).
    pub fn set_track_volume(&self, track_id: String, volume: f32) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.set_track_volume(&track_id, volume)
    }

    /// Toggle track visibility (mute/unmute for audio).
    pub fn toggle_track_visibility(&self, track_id: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.toggle_track_visibility(&track_id)
    }

    /// Get audio samples for an asset at a specific time range.
    ///
    /// Returns interleaved f32 PCM samples at the project's sample rate.
    /// The `start_ms` and `duration_ms` define the time range to extract.
    pub fn get_audio_samples(
        &self,
        asset_id: String,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<Vec<f32>, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.get_audio_samples_range(&asset_id, start_ms, duration_ms)
    }

    /// Mix all audio tracks at the given timeline position.
    ///
    /// Returns interleaved f32 PCM samples for the mixed output.
    /// Respects each track's volume, visibility, and ducking settings.
    pub fn mix_audio_at_time(
        &self,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<Vec<f32>, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        let mixed = engine.mix_audio_at_time(start_ms, duration_ms)?;
        Ok(mixed.samples)
    }

    /// Get waveform peak data for an audio asset.
    ///
    /// Returns a list of peak values (0.0 to 1.0) suitable for
    /// rendering a waveform visualization. The `num_bins` parameter
    /// controls the resolution (typically matches the pixel width).
    pub fn get_waveform(&self, asset_id: String, num_bins: u32) -> Result<Vec<f32>, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        let waveform = engine.get_waveform(&asset_id, num_bins)?;
        Ok(waveform.peaks)
    }

    /// Configure audio ducking for a track.
    ///
    /// When ducking is enabled, other audio tracks will have their
    /// volume reduced when this track's audio is active (e.g., voiceover).
    pub fn set_ducking(
        &self,
        track_id: String,
        enabled: bool,
        duck_level: f32,
    ) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.set_ducking(track_id, enabled, duck_level)
    }

    /// Get the ducking configuration for a track.
    ///
    /// Returns the duck level (0.0 to 1.0) and whether ducking is enabled.
    pub fn get_ducking_config(&self, track_id: String) -> BridgeDuckingConfig {
        let engine = self.inner.lock().ok();
        match engine {
            Some(e) => {
                let config = e.get_ducking_config(&track_id);
                BridgeDuckingConfig {
                    enabled: config.enabled,
                    duck_level: config.duck_level,
                    attack_ms: config.attack_ms,
                    release_ms: config.release_ms,
                    threshold: config.threshold,
                }
            }
            None => BridgeDuckingConfig::default(),
        }
    }

    /// Get the full timeline state from the engine.
    ///
    /// Returns all tracks and clips so Flutter can render the timeline
    /// using Rust as the single source of truth.
    pub fn get_timeline_state(&self) -> Option<TimelineState> {
        let engine = self.inner.lock().ok()?;
        engine.get_timeline_state()
    }

    /// Get audio information for a media file.
    ///
    /// Returns sample rate, channels, duration, and codec name
    /// for the audio stream in the given file.
    pub fn get_audio_info(&self, file_path: String) -> Result<BridgeAudioInfo, String> {
        let mut decoder = crate::audio::decoder::AudioDecoder::new();
        decoder.open(&file_path)?;
        let info = decoder.audio_info();
        decoder.close();
        Ok(BridgeAudioInfo {
            sample_rate: info.sample_rate,
            channels: info.channels,
            duration_ms: info.duration_ms,
            codec_name: info.codec_name,
        })
    }

    // ─── Effect Operations ─────────────────────────────────────────────

    /// Add a filter effect to a clip.
    ///
    /// `filter_type_name` must match one of the display names from
    /// `get_filter_catalog()` (e.g., "Brightness", "Contrast", etc.).
    /// Returns an `EffectInfo` DTO describing the newly added effect.
    pub fn add_effect(&self, clip_id: String, filter_type_name: String) -> Result<EffectInfo, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.add_effect(&clip_id, &filter_type_name)
    }

    /// Remove an effect from a clip by its effect ID.
    pub fn remove_effect(&self, clip_id: String, effect_id: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.remove_effect(&clip_id, &effect_id)
    }

    /// Set a parameter value for an effect on a clip.
    ///
    /// The `param_name` must match the `name` field of the `EffectParameterInfo`
    /// returned in the effect's parameter list (e.g., "brightness", "contrast").
    pub fn set_effect_parameter(
        &self,
        clip_id: String,
        effect_id: String,
        param_name: String,
        value: f32,
    ) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.set_effect_parameter(&clip_id, &effect_id, &param_name, value)
    }

    /// Get the effects applied to a clip.
    pub fn get_clip_effects(&self, clip_id: String) -> Result<Vec<EffectInfo>, String> {
        let engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.get_clip_effects(&clip_id)
    }

    /// Toggle the enabled/disabled state of an effect on a clip.
    pub fn toggle_effect(&self, clip_id: String, effect_id: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.toggle_effect(&clip_id, &effect_id)
    }

    /// Get the catalog of all available filter types.
    ///
    /// Returns a list of `FilterTypeInfo` objects describing each filter
    /// and its default parameters. Use this to populate the effects panel UI.
    pub fn get_filter_catalog(&self) -> Vec<FilterTypeInfo> {
        match self.inner.lock() {
            Ok(engine) => engine.get_filter_catalog(),
            Err(_) => vec![],
        }
    }

    /// Get the list of available filter presets.
    pub fn get_filter_presets(&self) -> Vec<FilterPresetInfo> {
        match self.inner.lock() {
            Ok(engine) => engine.get_filter_presets(),
            Err(_) => vec![],
        }
    }

    /// Apply a filter preset to a clip (replaces all existing effects).
    pub fn apply_filter_preset(&self, clip_id: String, preset_id: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.apply_filter_preset(&clip_id, &preset_id)
    }

    // ─── Transition Operations ─────────────────────────────────────────

    /// Add a transition to a clip.
    ///
    /// `transition_type` must be one of: "Cut", "Fade", "Dissolve",
    /// "WipeLeft", "WipeRight", "WipeUp", "WipeDown", "SlideLeft",
    /// "SlideRight", "ZoomIn", "ZoomOut", "Spin".
    /// `direction` must be "in" (start of clip) or "out" (end of clip).
    pub fn add_transition(
        &self,
        clip_id: String,
        transition_type: String,
        duration_ms: u64,
        direction: String,
    ) -> Result<TransitionInfo, String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.add_transition(&clip_id, &transition_type, duration_ms, &direction)
    }

    /// Get the transition on a clip (in-point or out-point).
    pub fn get_clip_transition(&self, clip_id: String, direction: String) -> Option<TransitionInfo> {
        let engine = self.inner.lock().ok()?;
        engine.get_clip_transition(&clip_id, &direction)
    }

    /// Remove a transition from a clip.
    pub fn remove_transition(&self, clip_id: String, direction: String) -> Result<(), String> {
        let mut engine = self.inner.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        engine.remove_transition(&clip_id, &direction)
    }

    /// Get the catalog of all available transition types.
    pub fn get_transition_catalog(&self) -> Vec<TransitionTypeInfo> {
        match self.inner.lock() {
            Ok(engine) => engine.get_transition_catalog(),
            Err(_) => vec![],
        }
    }
}

impl From<ExportSettings> for BridgeExportSettings {
    fn from(s: ExportSettings) -> Self {
        Self {
            width: s.width,
            height: s.height,
            fps: s.fps,
            bitrate_kbps: s.bitrate_kbps,
            codec: match s.codec {
                VideoCodec::H264 => "H.264".to_string(),
                VideoCodec::H265 => "H.265".to_string(),
                VideoCodec::Vp9 => "VP9".to_string(),
                VideoCodec::Av1 => "AV1".to_string(),
            },
            format: match s.format {
                OutputFormat::Mp4 => "MP4".to_string(),
                OutputFormat::WebM => "WebM".to_string(),
                OutputFormat::Mov => "MOV".to_string(),
                OutputFormat::Avi => "AVI".to_string(),
                OutputFormat::Gif => "GIF".to_string(),
            },
            audio_bitrate_kbps: s.audio_bitrate_kbps,
            audio_sample_rate: s.audio_sample_rate,
            audio_channels: s.audio_channels,
            include_audio: s.include_audio,
            two_pass: s.two_pass,
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

/// Bridge-compatible ducking configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BridgeDuckingConfig {
    pub enabled: bool,
    pub duck_level: f32,
    pub attack_ms: u64,
    pub release_ms: u64,
    pub threshold: f32,
}

impl Default for BridgeDuckingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            duck_level: 0.3,
            attack_ms: 50,
            release_ms: 300,
            threshold: 0.05,
        }
    }
}

impl From<DuckingConfig> for BridgeDuckingConfig {
    fn from(c: DuckingConfig) -> Self {
        Self {
            enabled: c.enabled,
            duck_level: c.duck_level,
            attack_ms: c.attack_ms,
            release_ms: c.release_ms,
            threshold: c.threshold,
        }
    }
}

/// Bridge-compatible audio info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BridgeAudioInfo {
    pub sample_rate: u32,
    pub channels: u32,
    pub duration_ms: u64,
    pub codec_name: String,
}
