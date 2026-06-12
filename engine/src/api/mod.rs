//! Public API for the Flutter bridge
//!
//! This module defines the main entry point that Flutter calls into.
//! All methods are designed to be called via flutter_rust_bridge and
//! return serializable results.

pub mod bridge_api;
pub mod commands;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::audio::decoder::AudioDecoder;
use crate::audio::ducking::DuckingConfig;
use crate::audio::mixer::{AudioBuffer, AudioMixer, TrackAudioSource, VolumeEnvelope};
use crate::audio::waveform::WaveformData;
use crate::decoder::hardware::HardwareDecoder;
use crate::decoder::VideoInfo;
use crate::effects::filters::FilterType;
use crate::export_engine::{
    check_storage_space, ExportPipeline, ExportProgress, ExportResult, ExportSettings, ExportStage,
    VideoEncoder,
};
use crate::project::format::EppFormat;
use crate::project::{MediaAsset, MediaType, Project, ProjectSettings};
use crate::renderer::PreviewRenderer;
use crate::timeline::clip::Clip;
use crate::timeline::command::{
    AddClipCommand, AddEffectCommand, AddKeyframeCommand, AddTransitionCommand, Command,
    CommandHistory, MoveClipCommand, RemoveClipCommand, RemoveEffectCommand,
    RemoveKeyframeCommand, SetEffectParameterCommand, SetSpeedCurveCommand,
    SetTrackVolumeCommand, SplitClipCommand, ToggleTrackVisibilityCommand, TrimClipCommand,
    UpdateKeyframeCommand,
};
use crate::timeline::keyframe::{Keyframe, KEYFRAME_PROPERTIES};
use crate::timeline::speed_curve::{EasingType, SpeedCurve, SpeedSegment};
use crate::timeline::track::TrackType;
use crate::timeline::Timeline;
use crate::system::SystemMetrics;
use crate::EngineError;

/// The main engine API that Flutter interacts with
pub struct EditorsProEngine {
    project: Option<Project>,
    decoder: HardwareDecoder,
    renderer: PreviewRenderer,
    command_history: CommandHistory,
    audio_decoder: AudioDecoder,
    audio_mixer: AudioMixer,
    initialized: bool,
    /// The file path of the video currently loaded in the decoder.
    /// Used to detect clip-switching so the decoder is re-opened when
    /// the user scrubs to a different clip.
    current_file_path: Option<String>,
    /// Flag set by the Flutter side to cancel an in-progress export.
    /// The encoding loop checks this flag before each frame and will
    /// abort early if it is set.
    export_canceled: std::sync::atomic::AtomicBool,
    /// Cached audio data keyed by asset_id for quick access
    audio_cache: std::collections::HashMap<String, AudioBuffer>,
    /// Ducking configurations per track
    ducking_configs: std::collections::HashMap<String, DuckingConfig>,
}

impl EditorsProEngine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self {
            project: None,
            decoder: HardwareDecoder::new(),
            renderer: PreviewRenderer::new(1920, 1080),
            command_history: CommandHistory::new(),
            audio_decoder: AudioDecoder::new(),
            audio_mixer: AudioMixer::new(44100, 2),
            initialized: false,
            current_file_path: None,
            export_canceled: std::sync::atomic::AtomicBool::new(false),
            audio_cache: std::collections::HashMap::new(),
            ducking_configs: std::collections::HashMap::new(),
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
    pub fn create_project(
        &mut self,
        name: &str,
        settings: Option<ProjectSettings>,
    ) -> Result<ProjectInfo, EngineError> {
        if !self.initialized {
            return Err(EngineError::InvalidState(
                "Engine not initialized".to_string(),
            ));
        }

        let mut project = match settings {
            Some(s) => Project::with_settings(name, s),
            None => Project::new(name),
        };

        // Add default tracks
        project
            .timeline_mut()
            .add_track(TrackType::Video, Some("Video 1".to_string()));
        project
            .timeline_mut()
            .add_track(TrackType::Audio, Some("Audio 1".to_string()));
        project
            .timeline_mut()
            .add_track(TrackType::Text, Some("Text".to_string()));

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
    pub fn add_track(
        &mut self,
        track_type: TrackType,
        name: Option<String>,
    ) -> Result<TrackInfo, String> {
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
            let asset = project
                .find_media_asset(asset_id)
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
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;

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
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Split a clip at the given timestamp
    pub fn split_clip(
        &mut self,
        clip_id: &str,
        time_ms: u64,
    ) -> Result<(ClipInfo, ClipInfo), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = SplitClipCommand::new(clip_id.to_string(), time_ms);
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;

        // Get the resulting clips
        let clips = project.timeline().get_clips_at_time(time_ms);
        let results: Vec<ClipInfo> = clips
            .iter()
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
    pub fn move_clip(
        &mut self,
        clip_id: &str,
        new_start_ms: u64,
        new_track_id: Option<String>,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = MoveClipCommand::new(clip_id.to_string(), new_start_ms, new_track_id);
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Remove a clip
    pub fn remove_clip(&mut self, clip_id: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = RemoveClipCommand::new(clip_id.to_string());
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Get a rendered frame at the specified timestamp for preview.
    ///
    /// Returns the full [`FrameData`] including width, height, and raw
    /// RGBA pixel data. Callers that only need the bytes can destructure
    /// the result.
    pub fn get_frame(&mut self, time_ms: u64) -> Result<crate::decoder::FrameData, EngineError> {
        if !self.initialized {
            return Err(EngineError::InvalidState(
                "Engine not initialized".to_string(),
            ));
        }

        // First, gather the information we need from the project (immutable borrow)
        let frame_request = {
            let project = self
                .project
                .as_ref()
                .ok_or_else(|| EngineError::InvalidState("No project open".to_string()))?;

            // Find the active video clip at this time
            let video_clips = project.timeline().tracks_of_type(TrackType::Video);
            let active_clip = video_clips
                .iter()
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
            let source_time = clip.trim_start_ms + (relative_time as f32 * clip.speed_at(relative_time)) as u64;

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
                self.decoder
                    .open(&file_path)
                    .map_err(|e| EngineError::DecoderError(e))?;
                self.current_file_path = Some(file_path);
            }
            Some(
                self.decoder
                    .decode_frame_at(source_time)
                    .map_err(|e| EngineError::DecoderError(e))?,
            )
        } else {
            None
        };

        // Compose the frame with all layers
        let project = self
            .project
            .as_ref()
            .ok_or_else(|| EngineError::InvalidState("No project open".to_string()))?;
        let composed = self
            .renderer
            .compose_frame(project.timeline(), time_ms, video_frame);

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
        self.export_canceled
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let mut encoder = VideoEncoder::new(&settings)?;
        encoder.open(output_path)?;

        let start_time = std::time::Instant::now();

        for frame_num in 0..total_frames {
            // Check for cancellation
            if self
                .export_canceled
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                log::info!(
                    "Export canceled by user at frame {}/{}",
                    frame_num,
                    total_frames
                );
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
            let resized = self
                .renderer
                .resize_frame(&rgba_frame, settings.width, settings.height);

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
        self.export_canceled
            .store(true, std::sync::atomic::Ordering::Relaxed);
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
            let active_clip = video_clips
                .iter()
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
            let source_time = clip.trim_start_ms + (relative_time as f32 * clip.speed_at(relative_time)) as u64;

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
            Some(
                self.decoder
                    .decode_frame_at(source_time)
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };

        // Compose the frame
        let project = self.project.as_ref().ok_or("No project open")?;
        let composed = self
            .renderer
            .compose_frame(project.timeline(), time_ms, video_frame);

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
        self.project
            .as_ref()
            .map(|p| p.timeline().duration_ms)
            .unwrap_or(0)
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

    // ─── Effect Operations ──────────────────────────────────────────────

    /// Add a filter effect to a clip
    pub fn add_effect(
        &mut self,
        clip_id: &str,
        filter_type_name: &str,
    ) -> Result<EffectInfo, String> {
        let project = self.project.as_mut().ok_or("No project open")?;

        // Parse the filter type
        let filter_type = crate::effects::filters::FilterType::all_filters()
            .iter()
            .find(|ft| ft.display_name().eq_ignore_ascii_case(filter_type_name))
            .ok_or_else(|| format!("Unknown filter type: {}", filter_type_name))?
            .clone();

        // Create the effect
        let mut effect = filter_type.to_effect();
        // Assign the next order number
        let (_, clip) = project
            .timeline()
            .find_clip(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;
        effect.order = clip.effects.len() as u32;

        let effect_info = EffectInfo::from_effect(&effect);

        let command = AddEffectCommand::new(clip_id.to_string(), effect);
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;

        log::info!("Added {} effect to clip {}", filter_type_name, clip_id);
        Ok(effect_info)
    }

    /// Remove an effect from a clip
    pub fn remove_effect(&mut self, clip_id: &str, effect_id: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = RemoveEffectCommand::new(clip_id.to_string(), effect_id.to_string());
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        log::info!("Removed effect {} from clip {}", effect_id, clip_id);
        Ok(())
    }

    /// Set a parameter value for an effect on a clip
    pub fn set_effect_parameter(
        &mut self,
        clip_id: &str,
        effect_id: &str,
        param_name: &str,
        value: f32,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = SetEffectParameterCommand::new(
            clip_id.to_string(),
            effect_id.to_string(),
            param_name.to_string(),
            value,
        );
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Get the effects applied to a clip
    pub fn get_clip_effects(&self, clip_id: &str) -> Result<Vec<EffectInfo>, String> {
        let project = self.project.as_ref().ok_or("No project open")?;
        let (_, clip) = project
            .timeline()
            .find_clip(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;
        Ok(clip.effects.iter().map(EffectInfo::from_effect).collect())
    }

    /// Toggle the enabled state of an effect on a clip
    pub fn toggle_effect(&mut self, clip_id: &str, effect_id: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let clip = project
            .timeline_mut()
            .find_clip_mut(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;
        let effect = clip
            .effects
            .iter_mut()
            .find(|e| e.id == effect_id)
            .ok_or_else(|| format!("Effect {} not found", effect_id))?;
        effect.toggle_enabled();
        log::info!("Toggled effect {} on clip {}", effect_id, clip_id);
        Ok(())
    }

    /// Get the available filter catalog
    pub fn get_filter_catalog(&self) -> Vec<FilterTypeInfo> {
        crate::effects::filters::FilterType::all_filters()
            .iter()
            .map(|ft| FilterTypeInfo {
                name: ft.display_name().to_string(),
                icon: ft.icon().to_string(),
                parameters: ft
                    .default_parameters()
                    .iter()
                    .map(|p| EffectParameterInfo {
                        name: p.name.clone(),
                        display_name: p.display_name.clone(),
                        value: p.value,
                        min_value: p.min_value,
                        max_value: p.max_value,
                        default_value: p.default_value,
                        step: p.step,
                    })
                    .collect(),
            })
            .collect()
    }

    /// Get the available filter presets
    pub fn get_filter_presets(&self) -> Vec<FilterPresetInfo> {
        crate::effects::filters::FilterPreset::built_in_presets()
            .iter()
            .map(|p| FilterPresetInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                description: p.description.clone(),
            })
            .collect()
    }

    /// Apply a filter preset to a clip (replaces all existing effects)
    pub fn apply_filter_preset(&mut self, clip_id: &str, preset_id: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;

        let preset = crate::effects::filters::FilterPreset::built_in_presets()
            .iter()
            .find(|p| p.id == preset_id)
            .ok_or_else(|| format!("Preset {} not found", preset_id))?;

        let effects = preset.to_effects();

        // Remove all existing effects first
        let clip = project
            .timeline_mut()
            .find_clip_mut(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;
        clip.effects.clear();

        // Add the preset effects
        for effect in effects {
            let command = AddEffectCommand::new(clip_id.to_string(), effect);
            self.command_history
                .execute(Box::new(command), project.timeline_mut())?;
        }

        log::info!("Applied preset {} to clip {}", preset_id, clip_id);
        Ok(())
    }

    // ─── Transition Operations ─────────────────────────────────────────

    /// Add a transition between two clips
    ///
    /// `transition_type` must be one of: "Cut", "Fade", "Dissolve",
    /// "WipeLeft", "WipeRight", "WipeUp", "WipeDown", "SlideLeft",
    /// "SlideRight", "ZoomIn", "ZoomOut", "Spin"
    pub fn add_transition(
        &mut self,
        clip_id: &str,
        transition_type: &str,
        duration_ms: u64,
        direction: &str, // "in" or "out"
    ) -> Result<TransitionInfo, String> {
        let project = self.project.as_mut().ok_or("No project open")?;

        let tt = crate::effects::transitions::TransitionType::from_str_lossy(transition_type)
            .ok_or_else(|| format!("Unknown transition type: {}", transition_type))?;

        // Find the neighboring clip for the transition
        let (_, clip) = project
            .timeline()
            .find_clip(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;

        let neighbor_id = if direction == "in" {
            // Find the clip that ends before this clip starts
            project
                .timeline()
                .tracks
                .iter()
                .flat_map(|t| t.clips.iter())
                .filter(|c| c.end_ms() <= clip.start_ms)
                .max_by_key(|c| c.end_ms())
                .map(|c| c.id.clone())
                .unwrap_or_default()
        } else {
            // Find the clip that starts after this clip ends
            project
                .timeline()
                .tracks
                .iter()
                .flat_map(|t| t.clips.iter())
                .filter(|c| c.start_ms >= clip.end_ms())
                .min_by_key(|c| c.start_ms)
                .map(|c| c.id.clone())
                .unwrap_or_default()
        };

        let transition = crate::effects::Transition::new(tt, duration_ms, clip_id, &neighbor_id);
        let info = TransitionInfo::from_transition(&transition);

        let command = if direction == "in" {
            AddTransitionCommand::new_in(clip_id.to_string(), transition)
        } else {
            AddTransitionCommand::new_out(clip_id.to_string(), transition)
        };
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;

        log::info!(
            "Added {} transition to clip {} ({})",
            transition_type,
            clip_id,
            direction
        );
        Ok(info)
    }

    /// Get the transition on a clip (in-point or out-point)
    pub fn get_clip_transition(&self, clip_id: &str, direction: &str) -> Option<TransitionInfo> {
        let project = self.project.as_ref()?;
        let (_, clip) = project.timeline().find_clip(clip_id)?;
        let transition = if direction == "in" {
            clip.transition_in.as_ref()
        } else {
            clip.transition_out.as_ref()
        };
        transition.map(TransitionInfo::from_transition)
    }

    /// Remove a transition from a clip
    pub fn remove_transition(&mut self, clip_id: &str, direction: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let clip = project
            .timeline_mut()
            .find_clip_mut(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;

        if direction == "in" {
            clip.transition_in = None;
        } else {
            clip.transition_out = None;
        }

        log::info!("Removed {} transition from clip {}", direction, clip_id);
        Ok(())
    }

    /// Get the available transition types
    pub fn get_transition_catalog(&self) -> Vec<TransitionTypeInfo> {
        crate::effects::transitions::TransitionType::all_transitions()
            .iter()
            .map(|tt| TransitionTypeInfo {
                name: tt.display_name().to_string(),
                icon: tt.icon().to_string(),
                default_duration_ms: tt.default_duration_ms(),
            })
            .collect()
    }

    // ─── Text Clip Operations ──────────────────────────────────────────

    /// Add a text clip to a text track
    pub fn add_text_clip(
        &mut self,
        track_id: &str,
        text: &str,
        font_family: &str,
        font_size: f32,
        color_hex: &str,
        position_x: f32,
        position_y: f32,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<ClipInfo, String> {
        let project = self.project.as_mut().ok_or("No project open")?;

        // Verify the track exists and is a text track
        let track = project
            .timeline()
            .find_track(track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;
        if track.track_type != TrackType::Text {
            return Err(format!("Track {} is not a text track", track_id));
        }

        // Create a new clip with text overlay data stored in properties
        let mut clip = Clip::new("__text__", start_ms, duration_ms);

        // Store text properties in the clip's custom properties map
        clip.set_property(
            "text_type".into(),
            serde_json::Value::String("text_overlay".into()),
        );
        clip.set_property("content".into(), serde_json::Value::String(text.into()));
        clip.set_property(
            "font_family".into(),
            serde_json::Value::String(font_family.into()),
        );
        clip.set_property("font_size".into(), serde_json::json!(font_size));
        clip.set_property(
            "color_hex".into(),
            serde_json::Value::String(color_hex.into()),
        );
        clip.set_property("position_x".into(), serde_json::json!(position_x));
        clip.set_property("position_y".into(), serde_json::json!(position_y));

        let clip_info = ClipInfo::from_clip(&clip);
        let command = AddClipCommand::new(track_id.to_string(), clip);
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;

        log::info!(
            "Added text clip '{}' to track {} at {}ms",
            text,
            track_id,
            start_ms
        );
        Ok(clip_info)
    }

    /// Set text position on a text clip
    pub fn set_text_position(
        &mut self,
        clip_id: &str,
        position_x: f32,
        position_y: f32,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let clip = project
            .timeline_mut()
            .find_clip_mut(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;

        // Verify this is a text clip
        let is_text = clip
            .get_property("text_type")
            .map(|v| v.as_str() == Some("text_overlay"))
            .unwrap_or(false);
        if !is_text {
            return Err(format!("Clip {} is not a text clip", clip_id));
        }

        clip.set_property("position_x".into(), serde_json::json!(position_x));
        clip.set_property("position_y".into(), serde_json::json!(position_y));

        log::info!(
            "Set text position on clip {} to ({:.2}, {:.2})",
            clip_id,
            position_x,
            position_y
        );
        Ok(())
    }

    /// Set text style on a text clip
    pub fn set_text_style(
        &mut self,
        clip_id: &str,
        font_family: &str,
        font_size: f32,
        color_hex: &str,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let clip = project
            .timeline_mut()
            .find_clip_mut(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;

        // Verify this is a text clip
        let is_text = clip
            .get_property("text_type")
            .map(|v| v.as_str() == Some("text_overlay"))
            .unwrap_or(false);
        if !is_text {
            return Err(format!("Clip {} is not a text clip", clip_id));
        }

        clip.set_property(
            "font_family".into(),
            serde_json::Value::String(font_family.into()),
        );
        clip.set_property("font_size".into(), serde_json::json!(font_size));
        clip.set_property(
            "color_hex".into(),
            serde_json::Value::String(color_hex.into()),
        );

        log::info!(
            "Set text style on clip {}: family={}, size={:.1}, color={}",
            clip_id,
            font_family,
            font_size,
            color_hex
        );
        Ok(())
    }

    /// Get available fonts
    pub fn get_available_fonts(&self) -> Vec<FontInfo> {
        vec![
            FontInfo {
                name: "DejaVu Sans".into(),
                family: "sans-serif".into(),
                style: "Regular".into(),
                is_builtin: true,
            },
            FontInfo {
                name: "DejaVu Sans Bold".into(),
                family: "sans-serif".into(),
                style: "Bold".into(),
                is_builtin: true,
            },
            FontInfo {
                name: "DejaVu Serif".into(),
                family: "serif".into(),
                style: "Regular".into(),
                is_builtin: true,
            },
            FontInfo {
                name: "DejaVu Serif Bold".into(),
                family: "serif".into(),
                style: "Bold".into(),
                is_builtin: true,
            },
            FontInfo {
                name: "DejaVu Sans Mono".into(),
                family: "monospace".into(),
                style: "Regular".into(),
                is_builtin: true,
            },
        ]
    }

    /// Import subtitles from an SRT file
    pub fn import_subtitles(&self, file_path: &str) -> Result<Vec<SubtitleEntry>, String> {
        crate::subtitle::parser::parse_srt_file(file_path)
    }

    // ─── Audio Operations ─────────────────────────────────────────────

    /// Set the volume level for a track
    pub fn set_track_volume(&mut self, track_id: &str, volume: f32) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = SetTrackVolumeCommand::new(track_id.to_string(), volume);
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        log::info!("Set track {} volume to {:.2}", track_id, volume);
        Ok(())
    }

    /// Toggle track visibility (mute/unmute for audio tracks)
    pub fn toggle_track_visibility(&mut self, track_id: &str) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = ToggleTrackVisibilityCommand::new(track_id.to_string());
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        Ok(())
    }

    /// Decode audio samples from a media asset
    ///
    /// Returns the decoded audio as interleaved f32 samples at the
    /// project's sample rate (44100Hz stereo by default). Results
    /// are cached for quick access.
    pub fn get_audio_samples(&mut self, asset_id: &str) -> Result<AudioBuffer, String> {
        // Check cache first
        if let Some(cached) = self.audio_cache.get(asset_id).cloned() {
            return Ok(cached);
        }

        // Find the asset's file path
        let file_path = {
            let project = self.project.as_ref().ok_or("No project open")?;
            let asset = project
                .find_media_asset(asset_id)
                .ok_or_else(|| format!("Asset {} not found", asset_id))?;
            asset.file_path.clone()
        };

        // Open and decode the audio
        self.audio_decoder.open(&file_path)?;
        let project = self.project.as_ref().ok_or("No project open")?;
        let sample_rate = project.settings.sample_rate;

        let decoded = self.audio_decoder.decode_all(sample_rate, 2)?;
        self.audio_decoder.close();

        // Convert DecodedAudio → AudioBuffer via the From impl
        let audio: AudioBuffer = decoded.into();

        // Cache the result
        self.audio_cache.insert(asset_id.to_string(), audio.clone());

        log::info!(
            "Decoded audio for asset {}: {} samples ({}ms)",
            asset_id,
            audio.samples.len(),
            audio.duration_ms
        );

        Ok(audio)
    }

    /// Get audio samples for a specific time range
    pub fn get_audio_samples_range(
        &mut self,
        asset_id: &str,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<Vec<f32>, String> {
        let audio = self.get_audio_samples(asset_id)?;
        let segment = audio.segment(start_ms, start_ms + duration_ms);
        Ok(segment.samples)
    }

    /// Mix all audio tracks at a given timeline position
    ///
    /// Returns mixed audio samples for the given time range, taking
    /// into account each track's volume, visibility, and ducking settings.
    pub fn mix_audio_at_time(
        &mut self,
        start_ms: u64,
        duration_ms: u64,
    ) -> Result<AudioBuffer, String> {
        let project = self.project.as_ref().ok_or("No project open")?;

        // Collect audio tracks and their clips
        let audio_tracks: Vec<_> = project
            .timeline()
            .tracks_of_type(TrackType::Audio)
            .into_iter()
            .filter(|t| t.visible)
            .collect();

        let mut sources: Vec<TrackAudioSource> = Vec::new();

        for track in audio_tracks {
            for clip in &track.clips {
                // Check if this clip overlaps with our requested range
                let clip_start = clip.start_ms;
                let clip_end = clip.end_ms();
                if clip_start >= start_ms + duration_ms || clip_end <= start_ms {
                    continue;
                }

                // Calculate the actual time range we need from this clip
                let overlap_start = clip_start.max(start_ms);
                let overlap_end = clip_end.min(start_ms + duration_ms);
                let overlap_duration = overlap_end - overlap_start;

                // Get the source audio
                let audio = self.get_audio_samples(&clip.asset_id)?;

                // Calculate the offset within the source
                let source_start = clip.trim_start_ms + (overlap_start - clip_start);
                let segment = audio.segment(source_start, source_start + overlap_duration);

                sources.push(TrackAudioSource {
                    buffer: segment,
                    volume: track.volume,
                    offset_ms: overlap_start - start_ms,
                    envelope: VolumeEnvelope::default(),
                    muted: !track.visible,
                });
            }
        }

        // Also include audio from video tracks
        let video_tracks: Vec<_> = project
            .timeline()
            .tracks_of_type(TrackType::Video)
            .into_iter()
            .filter(|t| t.visible)
            .collect();

        for track in video_tracks {
            for clip in &track.clips {
                let clip_start = clip.start_ms;
                let clip_end = clip.end_ms();
                if clip_start >= start_ms + duration_ms || clip_end <= start_ms {
                    continue;
                }

                let overlap_start = clip_start.max(start_ms);
                let overlap_end = clip_end.min(start_ms + duration_ms);
                let overlap_duration = overlap_end - overlap_start;

                // Video clips may have embedded audio
                if let Ok(audio) = self.get_audio_samples(&clip.asset_id) {
                    let source_start = clip.trim_start_ms + (overlap_start - clip_start);
                    let segment = audio.segment(source_start, source_start + overlap_duration);

                    sources.push(TrackAudioSource {
                        buffer: segment,
                        volume: track.volume,
                        offset_ms: overlap_start - start_ms,
                        envelope: VolumeEnvelope::default(),
                        muted: !track.visible,
                    });
                }
            }
        }

        // Mix all sources
        let mixed = self.audio_mixer.mix_sources(&sources, duration_ms);

        // Apply ducking if configured
        for (track_id, config) in &self.ducking_configs {
            if config.enabled {
                // Find the trigger track (voiceover)
                let trigger_track = project.timeline().tracks.iter().find(|t| &t.id == track_id);
                if let Some(trigger) = trigger_track {
                    // Collect trigger audio
                    for clip in &trigger.clips {
                        let clip_start = clip.start_ms;
                        let clip_end = clip.end_ms();
                        if clip_start >= start_ms + duration_ms || clip_end <= start_ms {
                            continue;
                        }
                        if let Ok(trigger_audio) = self.get_audio_samples(&clip.asset_id) {
                            let source_start =
                                clip.trim_start_ms + (clip_start.max(start_ms) - clip_start);
                            let segment =
                                trigger_audio.segment(source_start, source_start + duration_ms);
                            let mut mixed_mut = mixed.clone();
                            crate::audio::ducking::apply_ducking(&mut mixed_mut, &segment, config);
                        }
                    }
                }
            }
        }

        Ok(mixed)
    }

    /// Get waveform data for an audio asset
    ///
    /// Returns peak values for visualization. The `num_bins` parameter
    /// controls how many data points are returned (typically matches
    /// the pixel width of the waveform display).
    pub fn get_waveform(&mut self, asset_id: &str, num_bins: u32) -> Result<WaveformData, String> {
        let audio = self.get_audio_samples(asset_id)?;
        let waveform =
            WaveformData::from_samples(&audio.samples, audio.sample_rate, audio.channels, num_bins);
        Ok(waveform)
    }

    /// Configure ducking for a track
    ///
    /// When ducking is enabled, other audio tracks will have their
    /// volume reduced when this track's audio is active.
    pub fn set_ducking(
        &mut self,
        track_id: String,
        enabled: bool,
        duck_level: f32,
    ) -> Result<(), String> {
        let project = self.project.as_ref().ok_or("No project open")?;

        // Verify track exists
        project
            .timeline()
            .find_track(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        let config = self
            .ducking_configs
            .entry(track_id.clone())
            .or_insert_with(DuckingConfig::default);
        config.enabled = enabled;
        config.duck_level = duck_level.clamp(0.0, 1.0);

        log::info!(
            "Set ducking for track {}: enabled={}, level={:.2}",
            track_id,
            enabled,
            duck_level
        );
        Ok(())
    }

    /// Get the ducking configuration for a track
    pub fn get_ducking_config(&self, track_id: &str) -> DuckingConfig {
        self.ducking_configs
            .get(track_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get the full timeline state as a DTO for Flutter
    ///
    /// Returns all tracks and clips in a serializable format that
    /// Flutter can use to render the timeline.
    pub fn get_timeline_state(&self) -> Option<TimelineState> {
        let project = self.project.as_ref()?;

        let tracks: Vec<TrackStateDto> = project
            .timeline()
            .tracks
            .iter()
            .map(|t| {
                let clips: Vec<ClipStateDto> = t
                    .clips
                    .iter()
                    .map(|c| ClipStateDto {
                        id: c.id.clone(),
                        asset_id: c.asset_id.clone(),
                        start_ms: c.start_ms,
                        duration_ms: c.effective_duration(),
                        trim_start_ms: c.trim_start_ms,
                        trim_end_ms: c.trim_end_ms,
                        speed: c.speed(),
                        opacity: c.opacity,
                        effects: c.effects.iter().map(EffectInfo::from_effect).collect(),
                        transition_in: c
                            .transition_in
                            .as_ref()
                            .map(TransitionInfo::from_transition),
                        transition_out: c
                            .transition_out
                            .as_ref()
                            .map(TransitionInfo::from_transition),
                    })
                    .collect();

                TrackStateDto {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    track_type: format!("{}", t.track_type),
                    clips,
                    locked: t.locked,
                    visible: t.visible,
                    volume: t.volume,
                }
            })
            .collect();

        Some(TimelineState {
            tracks,
            duration_ms: project.timeline().duration_ms,
        })
    }

    // ─── Speed Curve & Keyframe Operations ────────────────────────────

    /// Set the speed curve for a clip
    pub fn set_clip_speed_curve(&mut self, clip_id: &str, curve: SpeedCurve) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = SetSpeedCurveCommand::new(clip_id.to_string(), curve);
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        log::info!("Set speed curve for clip {}", clip_id);
        Ok(())
    }

    /// Add a keyframe to a clip's property track
    ///
    /// Returns the keyframe ID on success.
    pub fn add_keyframe(
        &mut self,
        clip_id: &str,
        property: &str,
        time_ms: u64,
        value: f32,
        easing: EasingType,
    ) -> Result<String, String> {
        // Validate property name
        if !KEYFRAME_PROPERTIES.contains(&property) {
            return Err(format!(
                "Invalid keyframe property '{}'. Must be one of: {:?}",
                property, KEYFRAME_PROPERTIES
            ));
        }

        let keyframe = Keyframe::new(time_ms, value, easing);
        let keyframe_id = keyframe.id.clone();

        let project = self.project.as_mut().ok_or("No project open")?;
        let command = AddKeyframeCommand::new(
            clip_id.to_string(),
            property.to_string(),
            keyframe,
        );
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;

        log::info!(
            "Added keyframe to clip {} property {} at {}ms",
            clip_id,
            property,
            time_ms
        );
        Ok(keyframe_id)
    }

    /// Remove a keyframe from a clip's property track
    pub fn remove_keyframe(
        &mut self,
        clip_id: &str,
        property: &str,
        keyframe_id: &str,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = RemoveKeyframeCommand::new(
            clip_id.to_string(),
            property.to_string(),
            keyframe_id.to_string(),
        );
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        log::info!(
            "Removed keyframe {} from clip {} property {}",
            keyframe_id,
            clip_id,
            property
        );
        Ok(())
    }

    /// Update a keyframe's value and/or easing
    pub fn update_keyframe(
        &mut self,
        clip_id: &str,
        property: &str,
        keyframe_id: &str,
        value: Option<f32>,
        easing: Option<EasingType>,
    ) -> Result<(), String> {
        let project = self.project.as_mut().ok_or("No project open")?;
        let command = UpdateKeyframeCommand::new(
            clip_id.to_string(),
            property.to_string(),
            keyframe_id.to_string(),
            value,
            easing,
        );
        self.command_history
            .execute(Box::new(command), project.timeline_mut())?;
        log::info!(
            "Updated keyframe {} on clip {} property {}",
            keyframe_id,
            clip_id,
            property
        );
        Ok(())
    }

    /// Get all keyframes for a clip's property track
    pub fn get_keyframes(
        &self,
        clip_id: &str,
        property: &str,
    ) -> Result<Vec<KeyframeInfo>, String> {
        let project = self.project.as_ref().ok_or("No project open")?;
        let (_, clip) = project
            .timeline()
            .find_clip(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;

        let track = clip.keyframe_track(property)
            .ok_or_else(|| format!("Unknown keyframe property: {}", property))?;

        Ok(track.keyframes.iter().map(|kf| KeyframeInfo {
            id: kf.id.clone(),
            time_ms: kf.time_ms,
            value: kf.value,
            easing_name: kf.easing.display_name().to_string(),
        }).collect())
    }

    /// Get the speed curve for a clip
    pub fn get_clip_speed_curve(&self, clip_id: &str) -> Result<SpeedCurveInfo, String> {
        let project = self.project.as_ref().ok_or("No project open")?;
        let (_, clip) = project
            .timeline()
            .find_clip(clip_id)
            .ok_or_else(|| format!("Clip {} not found", clip_id))?;

        Ok(SpeedCurveInfo::from_curve(&clip.speed_curve))
    }

    /// Get current system metrics for memory monitoring
    pub fn get_system_metrics(&self) -> SystemMetrics {
        use crate::system::memory::MemoryMonitor;
        let mut monitor = MemoryMonitor::new();
        monitor.collect_metrics(0, self.audio_cache.len())
    }

    /// Check if the engine should reduce quality due to memory pressure
    pub fn is_memory_pressure(&self) -> bool {
        use crate::system::memory::MemoryMonitor;
        let monitor = MemoryMonitor::new();
        monitor.should_release_caches()
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
    pub effects_count: usize,
    pub has_transition_in: bool,
    pub has_transition_out: bool,
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
            speed: clip.speed(),
            opacity: clip.opacity,
            effects_count: clip.effects.len(),
            has_transition_in: clip.transition_in.is_some(),
            has_transition_out: clip.transition_out.is_some(),
        }
    }
}

/// Full timeline state DTO for Flutter synchronization
///
/// This DTO carries the complete timeline state (all tracks and clips)
/// from the Rust engine to Flutter, establishing Rust as the single
/// source of truth. Flutter reads this after every mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineState {
    pub tracks: Vec<TrackStateDto>,
    pub duration_ms: u64,
}

/// Track state DTO for timeline synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackStateDto {
    pub id: String,
    pub name: String,
    pub track_type: String,
    pub clips: Vec<ClipStateDto>,
    pub locked: bool,
    pub visible: bool,
    pub volume: f32,
}

/// Clip state DTO for timeline synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipStateDto {
    pub id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub trim_start_ms: u64,
    pub trim_end_ms: u64,
    pub speed: f32,
    pub opacity: f32,
    pub effects: Vec<EffectInfo>,
    pub transition_in: Option<TransitionInfo>,
    pub transition_out: Option<TransitionInfo>,
}

/// Effect info DTO for bridge transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInfo {
    pub id: String,
    pub name: String,
    pub effect_type: String,
    pub enabled: bool,
    pub order: u32,
    pub parameters: Vec<EffectParameterInfo>,
}

impl EffectInfo {
    fn from_effect(effect: &crate::effects::Effect) -> Self {
        Self {
            id: effect.id.clone(),
            name: effect.name.clone(),
            effect_type: format!("{:?}", effect.effect_type),
            enabled: effect.enabled,
            order: effect.order,
            parameters: effect
                .parameters
                .iter()
                .map(|p| EffectParameterInfo {
                    name: p.name.clone(),
                    display_name: p.display_name.clone(),
                    value: p.value,
                    min_value: p.min_value,
                    max_value: p.max_value,
                    default_value: p.default_value,
                    step: p.step,
                })
                .collect(),
        }
    }
}

/// Effect parameter info DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectParameterInfo {
    pub name: String,
    pub display_name: String,
    pub value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub step: f32,
}

/// Filter type info DTO for the filter catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTypeInfo {
    pub name: String,
    pub icon: String,
    pub parameters: Vec<EffectParameterInfo>,
}

/// Filter preset info DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Transition info DTO for bridge transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionInfo {
    pub id: String,
    pub transition_type: String,
    pub duration_ms: u64,
    pub from_clip_id: String,
    pub to_clip_id: String,
}

impl TransitionInfo {
    fn from_transition(transition: &crate::effects::Transition) -> Self {
        Self {
            id: transition.id.clone(),
            transition_type: format!("{:?}", transition.transition_type),
            duration_ms: transition.duration_ms,
            from_clip_id: transition.from_clip_id.clone(),
            to_clip_id: transition.to_clip_id.clone(),
        }
    }
}

/// Transition type info DTO for the transition catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionTypeInfo {
    pub name: String,
    pub icon: String,
    pub default_duration_ms: u64,
}

/// Font info DTO for text overlay support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub name: String,
    pub family: String,
    pub style: String,
    pub is_builtin: bool,
}

/// Subtitle entry DTO for importing subtitle files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    pub index: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// Speed segment info DTO for bridge transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSegmentInfo {
    pub start_ms: u64,
    pub end_ms: u64,
    pub start_speed: f32,
    pub end_speed: f32,
    pub easing_name: String,
}

impl SpeedSegmentInfo {
    fn from_segment(seg: &SpeedSegment) -> Self {
        Self {
            start_ms: seg.start_ms,
            end_ms: seg.end_ms,
            start_speed: seg.start_speed,
            end_speed: seg.end_speed,
            easing_name: seg.easing.display_name().to_string(),
        }
    }
}

/// Speed curve info DTO for bridge transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedCurveInfo {
    pub segments: Vec<SpeedSegmentInfo>,
}

impl SpeedCurveInfo {
    /// Create from a SpeedCurve
    pub fn from_curve(curve: &SpeedCurve) -> Self {
        Self {
            segments: curve.segments.iter().map(SpeedSegmentInfo::from_segment).collect(),
        }
    }
}

/// Keyframe info DTO for bridge transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeInfo {
    pub id: String,
    pub time_ms: u64,
    pub value: f32,
    pub easing_name: String,
}
