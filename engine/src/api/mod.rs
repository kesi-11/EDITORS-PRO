//! Public API for the Flutter bridge
//!
//! This module defines the main entry point that Flutter calls into.
//! All methods are designed to be called via flutter_rust_bridge and
//! return serializable results.

pub mod commands;
pub mod bridge_api;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::decoder::hardware::HardwareDecoder;
use crate::decoder::VideoInfo;
use crate::effects::filters::FilterType;
use crate::export_engine::{
    check_storage_space, ExportPipeline, ExportProgress, ExportResult, ExportSettings,
    ExportStage, VideoEncoder,
};
use crate::project::format::EppFormat;
use crate::project::{MediaAsset, MediaType, Project, ProjectSettings};
use crate::renderer::PreviewRenderer;
use crate::timeline::clip::Clip;
use crate::timeline::command::{
    AddClipCommand, Command, CommandHistory, MoveClipCommand, RemoveClipCommand,
    SplitClipCommand, TrimClipCommand,
};
use crate::timeline::track::TrackType;
use crate::timeline::Timeline;
use crate::EngineError;

/// The main engine API that Flutter interacts with
pub struct EditorsProEngine {
    project: Option<Project>,
    decoder: HardwareDecoder,
    renderer: PreviewRenderer,
    command_history: CommandHistory,
    initialized: bool,
    /// The file path of the video currently loaded in the decoder.
    /// Used to detect clip-switching so the decoder is re-opened when
    /// the user scrubs to a different clip.
    current_file_path: Option<String>,
    /// Flag set by the Flutter side to cancel an in-progress export.
    /// The encoding loop checks this flag before each frame and will
    /// abort early if it is set.
    export_canceled: std::sync::atomic::AtomicBool,
}

impl EditorsProEngine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self {
            project: None,
            decoder: HardwareDecoder::new(),
            renderer: PreviewRenderer::new(1920, 1080),
            command_history: CommandHistory::new(),
            initialized: false,
            current_file_path: None,
            export_canceled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Initialize the engine (must be called before any other operations)
    pub fn initialize(&mut self) -> Result<(), EngineError> {
        crate::init_engine()?;
        self.initialized = true;
        log::info!("EditorsProEngine initialized");
        Ok(())
    }

    /// Create a new project
    pub fn create_project(&mut self, name: &str, settings: Option<ProjectSettings>) -> Result<ProjectInfo, EngineError> {
        if !self.initialized {
            return Err(EngineError::InvalidState("Engine not initialized".to_string()));
        }

        let mut project = match settings {
            Some(s) => Project::with_settings(name, s),
            None => Project::new(name),
        };

        // Add default tracks
        project.timeline_mut().add_track(TrackType::Video, Some("Video 1".to_string()));
        project.timeline_mut().add_track(TrackType::Audio, Some("Audio 1".to_string()));
        project.timeline_mut().add_track(TrackType::Text, Some("Text".to_string()));

        let info = ProjectInfo::from_project(&project);
        self.project = Some(project);
        self.command_history.clear();

        log::info!("Created project: {}", name);
        Ok(info)
    }

    /// Import a media file into the current project
    pub fn import_media(&mut self, file_path: &str) -> Result<MediaAssetInfo, String> {
        let project = self.project.as_mut().ok_or("No project open")?;

        // Determine media type from file extension
        let media_type = Self::detect_media_type(file_path);
        let mut asset = MediaAsset::new(file_path, media_type);

        // If it's a video, extract metadata
        if media_type == MediaType::Video {
            let mut decoder = HardwareDecoder::new();
            if let Ok(()) = decoder.open(file_path) {
                if let Some(info) = decoder.get_video_info() {
                    asset.width = Some(info.width);
                    asset.height = Some(info.height);
                    asset.duration_ms = Some(info.duration_ms);
                    asset.codec = Some(info.codec_name.clone());
                    asset.bitrate = Some(info.bitrate);
                }
                decoder.close();
            }
        }

        let asset_info = MediaAssetInfo::from_asset(&asset);
        project.add_media_asset(asset);

        log::info!("Imported media: {}", file_path);
        Ok(asset_info)
    }

    /// Add a track to the timeline
    pub fn add_track(&mut self, track_type: TrackType, name: Option<String>) -> Result<TrackInfo, String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        project.timeline_mut().add_track(track_type, name);

        let tracks = &project.timeline().tracks;
        let track = tracks.last().unwrap();
        Ok(TrackInfo::from_track(track))
    }

    /// Add a clip to a track
    pub fn add_clip(
        &mut self,
        track_id: &str,
        asset_id: &str,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<ClipInfo, String> {
        // Verify asset exists and determine clip duration
        let clip_duration = {
            let project = self.project.as_ref().ok_or("No project open")?;
            let asset = project.find_media_asset(asset_id)
                .ok_or_else(|| format!("Asset {} not found", asset_id))?;
            if duration_ms > 0 {
                duration_ms
            } else {
                asset.duration_ms.unwrap_or(5000)
            }
        };

        let clip = Clip::new(asset_id, start_ms, clip_duration);
        let clip_info = ClipInfo::from_clip(&clip);

        let project = self.project.as_mut().ok_or("No project open")?;
        let command = AddClipCommand::new(track_id.to_string(), clip);
        self.command_history.execute(Box::new(command), project.timeline_mut())?;

        log::info!("Added clip on track {} at {}ms", track_id, start_ms);
        Ok(clip_info)
    }

    /// Trim a clip
    pub fn trim_clip(
        &mut self,
        clip_id: &str,
        trim_start_ms: u64,
        trim_end_ms: u64,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = TrimClipCommand::new(clip_id.to_string(), trim_start_ms, trim_end_ms);
        self.command_history.execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Split a clip at the given timestamp
    pub fn split_clip(&mut self, clip_id: &str, time_ms: u64) -> Result<(ClipInfo, ClipInfo), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = SplitClipCommand::new(clip_id.to_string(), time_ms);
        self.command_history.execute(Box::new(command), project.timeline_mut())?;

        // Get the resulting clips
        let clips = project.timeline().get_clips_at_time(time_ms);
        let results: Vec<ClipInfo> = clips.iter()
            .filter(|(_, c)| c.start_ms == time_ms || c.end_ms() > time_ms)
            .map(|(_, c)| ClipInfo::from_clip(c))
            .collect();

        if results.len() >= 2 {
            Ok((results[0].clone(), results[1].clone()))
        } else {
            Err("Split did not produce two clips".to_string())
        }
    }

    /// Move a clip to a new position
    pub fn move_clip(&mut self, clip_id: &str, new_start_ms: u64, new_track_id: Option<String>) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = MoveClipCommand::new(clip_id.to_string(), new_start_ms, new_track_id);
        self.command_history.execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Remove a clip
    pub fn remove_clip(&mut self, clip_id: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = RemoveClipCommand::new(clip_id.to_string());
        self.command_history.execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Get a rendered frame at the specified timestamp for preview.
    ///
    /// Returns the full [`FrameData`] including width, height, and raw
    /// RGBA pixel data. Callers that only need the bytes can destructure
    /// the result.
    pub fn get_frame(&mut self, time_ms: u64) -> Result<crate::decoder::FrameData, EngineError> {
        if !self.initialized {
            return Err(EngineError::InvalidState("Engine not initialized".to_string()));
        }

        // First, gather the information we need from the project (immutable borrow)
        let frame_request = {
            let project = self.project.as_ref()
                .ok_or_else(|| EngineError::InvalidState("No project open".to_string()))?;

            // Find the active video clip at this time
            let video_clips = project.timeline().tracks_of_type(TrackType::Video);
            let active_clip = video_clips.iter()
                .flat_map(|t| t.clips.iter())
                .find(|c| c.contains_time(time_ms));

            active_clip.map(|clip| {
                let asset = project.find_media_asset(&clip.asset_id);
                (clip.clone(), asset.map(|a| a.file_path.clone()))
            })
        };

        // Now decode the frame (mutable borrow of self.decoder)
        let video_frame = if let Some((clip, Some(file_path))) = frame_request {
            let relative_time = time_ms - clip.start_ms;
            let source_time = clip.trim_start_ms + (relative_time as f32 * clip.speed) as u64;

            // Check whether the decoder already has the correct file open.
            // If the file path differs from the currently-open file, close
            // the decoder and re-open the new file. This fixes the bug where
            // scrubbing between different clips would not re-open the decoder.
            let needs_reopen = match &self.current_file_path {
                Some(current) => current != &file_path,
                None => true,
            };

            if needs_reopen {
                // Close the previous decoder to release FFmpeg resources.
                self.decoder.close();
                self.current_file_path = None;

                // Open the new file.
                self.decoder.open(&file_path).map_err(|e| EngineError::DecoderError(e))?;
                self.current_file_path = Some(file_path);
            }
            Some(self.decoder.decode_frame_at(source_time).map_err(|e| EngineError::DecoderError(e))?)
        } else {
            None
        };

        // Compose the frame with all layers
        let project = self.project.as_ref()
            .ok_or_else(|| EngineError::InvalidState("No project open".to_string()))?;
        let composed = self.renderer.compose_frame(project.timeline(), time_ms, video_frame);

        Ok(composed)
    }

    /// Export the project as a video file with real FFmpeg encoding.
    ///
    /// This method renders each frame from the timeline through the
    /// decoder → renderer → encoder pipeline and produces a valid MP4
    /// (or other container) file at `output_path`.
    ///
    /// Progress is reported via the `progress_callback`, which is called
    /// for every frame encoded. The callback receives an `ExportProgress`
    /// struct with the current state of the export.
    pub fn export_video(
        &mut self,
        output_path: &str,
        settings: ExportSettings,
        progress_callback: &dyn Fn(ExportProgress),
    ) -> Result<ExportResult, String> {
        let project = self.project.as_ref().ok_or("No project open")?;
        let duration_ms = project.timeline().duration_ms;

        if duration_ms == 0 {
            return Err("Timeline is empty — nothing to export".to_string());
        }

        let pipeline = ExportPipeline::new(settings.clone());
        let total_frames = pipeline.total_frames(duration_ms);

        log::info!(
            "Starting export: {} frames ({}ms @ {}fps) → {}",
            total_frames,
            duration_ms,
            settings.fps,
            output_path
        );

        // ── Pre-flight checks ──────────────────────────────────────
        progress_callback(ExportProgress::preparing());

        // Check storage space
        let estimated_size = settings.estimated_file_size(duration_ms);
        if let Err(e) = check_storage_space(output_path, estimated_size) {
            return Err(format!("Storage check failed: {}", e));
        }

        // Ensure output directory exists
        if let Some(parent) = std::path::Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        // ── Encoding ───────────────────────────────────────────────
        self.export_canceled.store(false, std::sync::atomic::Ordering::Relaxed);

        let mut encoder = VideoEncoder::new(&settings)?;
        encoder.open(output_path)?;

        let start_time = std::time::Instant::now();

        for frame_num in 0..total_frames {
            // Check for cancellation
            if self.export_canceled.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("Export canceled by user at frame {}/{}", frame_num, total_frames);
                encoder.cancel();
                return Err("Export canceled".to_string());
            }

            // Calculate the timestamp for this frame
            let time_ms = pipeline.time_at_frame(frame_num);

            // Render the frame using the engine's get_frame logic
            // (We need to do this without calling self.get_frame() because
            //  the encoder already borrows self mutably through the settings.)
            let rgba_frame = self.render_export_frame(time_ms)?;

            // Resize to output dimensions if needed
            let resized = self.renderer.resize_frame(&rgba_frame, settings.width, settings.height);

            // Encode the frame
            if let Err(e) = encoder.encode_rgba_frame(&resized.data, frame_num as i64) {
                encoder.cancel();
                return Err(format!("Encoding error at frame {}: {}", frame_num, e));
            }

            // Report progress (throttled to every 10 frames to avoid overhead)
            if frame_num % 10 == 0 || frame_num == total_frames - 1 {
                progress_callback(ExportProgress::encoding(
                    frame_num + 1,
                    total_frames,
                    start_time,
                ));
            }
        }

        // ── Finalize ───────────────────────────────────────────────
        progress_callback(ExportProgress::finalizing());

        let result = encoder.finish(duration_ms)?;

        progress_callback(ExportProgress::complete());

        log::info!(
            "Export complete: {} → {} ({} bytes)",
            output_path,
            result.file_size_human(),
            result.file_size_bytes
        );

        Ok(result)
    }

    /// Request cancellation of an in-progress export.
    ///
    /// The encoding loop checks this flag before each frame and will
    /// abort early if set. This is safe to call from any thread.
    pub fn cancel_export(&self) {
        self.export_canceled.store(true, std::sync::atomic::Ordering::Relaxed);
        log::info!("Export cancellation requested");
    }

    /// Render a single frame for export at the given timestamp.
    ///
    /// This is the same logic as `get_frame()` but returns only the
    /// RGBA data without PNG encoding, which is what the encoder needs.
    fn render_export_frame(&mut self, time_ms: u64) -> Result<crate::decoder::FrameData, String> {
        // Gather clip info from the project
        let frame_request = {
            let project = self.project.as_ref().ok_or("No project open")?;

            let video_clips = project.timeline().tracks_of_type(TrackType::Video);
            let active_clip = video_clips.iter()
                .flat_map(|t| t.clips.iter())
                .find(|c| c.contains_time(time_ms));

            active_clip.map(|clip| {
                let asset = project.find_media_asset(&clip.asset_id);
                (clip.clone(), asset.map(|a| a.file_path.clone()))
            })
        };

        // Decode the video frame
        let video_frame = if let Some((clip, Some(file_path))) = frame_request {
            let relative_time = time_ms - clip.start_ms;
            let source_time = clip.trim_start_ms + (relative_time as f32 * clip.speed) as u64;

            let needs_reopen = match &self.current_file_path {
                Some(current) => current != &file_path,
                None => true,
            };

            if needs_reopen {
                self.decoder.close();
                self.current_file_path = None;
                self.decoder.open(&file_path).map_err(|e| e.to_string())?;
                self.current_file_path = Some(file_path);
            }
            Some(self.decoder.decode_frame_at(source_time).map_err(|e| e.to_string())?)
        } else {
            None
        };

        // Compose the frame
        let project = self.project.as_ref().ok_or("No project open")?;
        let composed = self.renderer.compose_frame(project.timeline(), time_ms, video_frame);

        Ok(composed)
    }

    /// Undo the last action
    pub fn undo(&mut self) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        self.command_history.undo(project.timeline_mut())?;
        Ok(())
    }

    /// Redo the last undone action
    pub fn redo(&mut self) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        self.command_history.redo(project.timeline_mut())?;
        Ok(())
    }

    /// Save the current project
    pub fn save_project(&mut self, path: &str) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("No project open")?;
        let file_path = PathBuf::from(path);
        project.save_as_epp(&file_path)?;
        Ok(())
    }

    /// Load a project from file
    pub fn load_project(&mut self, path: &str) -> Result<ProjectInfo, String> {
        let file_path = PathBuf::from(path);
        let project = Project::load_from_epp(&file_path)?;
        let info = ProjectInfo::from_project(&project);
        self.project = Some(project);
        self.command_history.clear();
        Ok(info)
    }

    /// Get information about the current project
    pub fn get_project_info(&self) -> Option<ProjectInfo> {
        self.project.as_ref().map(ProjectInfo::from_project)
    }

    /// Get the timeline duration
    pub fn get_timeline_duration(&self) -> u64 {
        self.project.as_ref().map(|p| p.timeline().duration_ms).unwrap_or(0)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.command_history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.command_history.can_redo()
    }

    /// Detect media type from file extension
    fn detect_media_type(file_path: &str) -> MediaType {
        let ext = PathBuf::from(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp4" | "avi" | "mov" | "mkv" | "webm" | "flv" | "wmv" | "3gp" => MediaType::Video,
            "mp3" | "wav" | "aac" | "flac" | "ogg" | "m4a" | "wma" => MediaType::Audio,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => MediaType::Image,
            _ => MediaType::Video, // Default to video
        }
    }
}

// Helper trait to access timeline on Project
trait ProjectTimeline {
    fn timeline(&self) -> &Timeline;
    fn timeline_mut(&mut self) -> &mut Timeline;
}

impl ProjectTimeline for Project {
    fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }
}

// ============ Bridge-friendly data transfer objects ============

/// Serializable project info for the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration_ms: u64,
    pub track_count: usize,
    pub clip_count: usize,
    pub asset_count: usize,
}

impl ProjectInfo {
    fn from_project(project: &Project) -> Self {
        let clip_count: usize = project.timeline.tracks.iter().map(|t| t.clips.len()).sum();
        Self {
            id: project.id.clone(),
            name: project.name.clone(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            width: project.settings.width,
            height: project.settings.height,
            fps: project.settings.fps,
            duration_ms: project.timeline.duration_ms,
            track_count: project.timeline.tracks.len(),
            clip_count,
            asset_count: project.media_assets.len(),
        }
    }
}

/// Serializable media asset info for the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAssetInfo {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub media_type: String,
    pub duration_ms: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size_bytes: u64,
}

impl MediaAssetInfo {
    fn from_asset(asset: &MediaAsset) -> Self {
        Self {
            id: asset.id.clone(),
            file_path: asset.file_path.clone(),
            file_name: asset.file_name.clone(),
            media_type: format!("{:?}", asset.media_type),
            duration_ms: asset.duration_ms,
            width: asset.width,
            height: asset.height,
            file_size_bytes: asset.file_size_bytes,
        }
    }
}

/// Serializable track info for the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: String,
    pub name: String,
    pub track_type: String,
    pub clip_count: usize,
    pub locked: bool,
    pub visible: bool,
    pub volume: f32,
}

impl TrackInfo {
    fn from_track(track: &crate::timeline::track::Track) -> Self {
        Self {
            id: track.id.clone(),
            name: track.name.clone(),
            track_type: format!("{}", track.track_type),
            clip_count: track.clips.len(),
            locked: track.locked,
            visible: track.visible,
            volume: track.volume,
        }
    }
}

/// Serializable clip info for the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipInfo {
    pub id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub trim_start_ms: u64,
    pub trim_end_ms: u64,
    pub speed: f32,
    pub opacity: f32,
}

impl ClipInfo {
    fn from_clip(clip: &Clip) -> Self {
        Self {
            id: clip.id.clone(),
            asset_id: clip.asset_id.clone(),
            start_ms: clip.start_ms,
            duration_ms: clip.duration_ms,
            trim_start_ms: clip.trim_start_ms,
            trim_end_ms: clip.trim_end_ms,
            speed: clip.speed,
            opacity: clip.opacity,
        }
    }
}
