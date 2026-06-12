//! Bridge command definitions
//!
//! Request and response types for the Flutter-Rust bridge.
//! These types are designed to be easily serializable and
//! compatible with flutter_rust_bridge v2.

use serde::{Deserialize, Serialize};

/// Generic request wrapper for bridge commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub command: String,
    pub payload: serde_json::Value,
}

/// Generic response wrapper for bridge results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl BridgeResponse {
    pub fn ok(data: impl Serialize) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).ok(),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// Create project request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f32>,
}

/// Import media request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMediaRequest {
    pub file_path: String,
}

/// Add clip request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddClipRequest {
    pub track_id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
}

/// Trim clip request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimClipRequest {
    pub clip_id: String,
    pub trim_start_ms: u64,
    pub trim_end_ms: u64,
}

/// Split clip request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitClipRequest {
    pub clip_id: String,
    pub time_ms: u64,
}

/// Move clip request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveClipRequest {
    pub clip_id: String,
    pub new_start_ms: u64,
    pub new_track_id: Option<String>,
}

/// Remove clip request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveClipRequest {
    pub clip_id: String,
}

/// Get frame request (for preview)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFrameRequest {
    pub time_ms: u64,
}

/// Export video request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportVideoRequest {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub bitrate_kbps: u64,
    pub codec: String,
    pub format: String,
}

/// Save project request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveProjectRequest {
    pub path: String,
}

/// Load project request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProjectRequest {
    pub path: String,
}

/// Progress callback data (sent from Rust to Flutter during long operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressCallback {
    pub operation: String,
    pub progress: f32,
    pub current: u64,
    pub total: u64,
    pub message: Option<String>,
}
