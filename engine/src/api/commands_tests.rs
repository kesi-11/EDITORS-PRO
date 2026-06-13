//! Tests for the api/commands module — BridgeRequest, BridgeResponse,
//! and all request DTOs
//!
//! Verifies serialization, deserialization, and construction of all
//! bridge command types used for Flutter↔Rust communication.

use crate::api::commands::{
    AddClipRequest, BridgeRequest, BridgeResponse, CreateProjectRequest, ExportVideoRequest,
    GetFrameRequest, ImportMediaRequest, LoadProjectRequest, MoveClipRequest, ProgressCallback,
    RemoveClipRequest, SaveProjectRequest, SplitClipRequest, TrimClipRequest,
};

// ─── BridgeRequest ───────────────────────────────────────────

#[test]
fn bridge_request_serialization() {
    let req = BridgeRequest {
        command: "create_project".into(),
        payload: serde_json::json!({"name": "Test"}),
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: BridgeRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.command, "create_project");
    assert_eq!(parsed.payload["name"], "Test");
}

#[test]
fn bridge_request_empty_payload() {
    let req = BridgeRequest {
        command: "ping".into(),
        payload: serde_json::Value::Null,
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: BridgeRequest = serde_json::from_str(&json).unwrap();
    assert!(parsed.payload.is_null());
}

// ─── BridgeResponse ──────────────────────────────────────────

#[test]
fn bridge_response_ok() {
    let resp = BridgeResponse::ok(serde_json::json!({"id": "123"}));
    assert!(resp.success);
    assert!(resp.data.is_some());
    assert!(resp.error.is_none());
    assert_eq!(resp.data.unwrap()["id"], "123");
}

#[test]
fn bridge_response_error() {
    let resp = BridgeResponse::error("Something went wrong");
    assert!(!resp.success);
    assert!(resp.data.is_none());
    assert_eq!(resp.error.as_deref(), Some("Something went wrong"));
}

#[test]
fn bridge_response_ok_with_string() {
    let resp = BridgeResponse::ok("hello");
    assert!(resp.success);
    assert_eq!(resp.data.unwrap(), serde_json::json!("hello"));
}

#[test]
fn bridge_response_ok_with_number() {
    let resp = BridgeResponse::ok(42u64);
    assert!(resp.success);
    assert_eq!(resp.data.unwrap(), serde_json::json!(42));
}

#[test]
fn bridge_response_serialization_roundtrip() {
    let resp = BridgeResponse::ok(serde_json::json!({"count": 5}));
    let json = serde_json::to_string(&resp).unwrap();
    let parsed: BridgeResponse = serde_json::from_str(&json).unwrap();
    assert!(parsed.success);
    assert_eq!(parsed.data.unwrap()["count"], 5);
}

// ─── CreateProjectRequest ────────────────────────────────────

#[test]
fn create_project_request_all_fields() {
    let req = CreateProjectRequest {
        name: "My Project".into(),
        width: Some(1920),
        height: Some(1080),
        fps: Some(30.0),
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: CreateProjectRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "My Project");
    assert_eq!(parsed.width, Some(1920));
    assert_eq!(parsed.height, Some(1080));
    assert!((parsed.fps.unwrap() - 30.0).abs() < f32::EPSILON);
}

#[test]
fn create_project_request_minimal() {
    let req = CreateProjectRequest {
        name: "Quick".into(),
        width: None,
        height: None,
        fps: None,
    };
    let parsed: CreateProjectRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.name, "Quick");
    assert!(parsed.width.is_none());
}

// ─── ImportMediaRequest ──────────────────────────────────────

#[test]
fn import_media_request() {
    let req = ImportMediaRequest {
        file_path: "/storage/emulated/0/DCIM/video.mp4".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: ImportMediaRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.file_path, "/storage/emulated/0/DCIM/video.mp4");
}

// ─── AddClipRequest ──────────────────────────────────────────

#[test]
fn add_clip_request() {
    let req = AddClipRequest {
        track_id: "track-1".into(),
        asset_id: "asset-1".into(),
        start_ms: 1000,
        duration_ms: 5000,
    };
    let parsed: AddClipRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.track_id, "track-1");
    assert_eq!(parsed.start_ms, 1000);
}

// ─── TrimClipRequest ─────────────────────────────────────────

#[test]
fn trim_clip_request() {
    let req = TrimClipRequest {
        clip_id: "clip-1".into(),
        trim_start_ms: 500,
        trim_end_ms: 300,
    };
    let parsed: TrimClipRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.trim_start_ms, 500);
    assert_eq!(parsed.trim_end_ms, 300);
}

// ─── SplitClipRequest ────────────────────────────────────────

#[test]
fn split_clip_request() {
    let req = SplitClipRequest {
        clip_id: "clip-1".into(),
        time_ms: 2500,
    };
    let parsed: SplitClipRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.time_ms, 2500);
}

// ─── MoveClipRequest ─────────────────────────────────────────

#[test]
fn move_clip_request_same_track() {
    let req = MoveClipRequest {
        clip_id: "clip-1".into(),
        new_start_ms: 5000,
        new_track_id: None,
    };
    let parsed: MoveClipRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert!(parsed.new_track_id.is_none());
}

#[test]
fn move_clip_request_cross_track() {
    let req = MoveClipRequest {
        clip_id: "clip-1".into(),
        new_start_ms: 3000,
        new_track_id: Some("track-2".into()),
    };
    let parsed: MoveClipRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.new_track_id.as_deref(), Some("track-2"));
}

// ─── RemoveClipRequest ───────────────────────────────────────

#[test]
fn remove_clip_request() {
    let req = RemoveClipRequest {
        clip_id: "clip-1".into(),
    };
    let parsed: RemoveClipRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.clip_id, "clip-1");
}

// ─── GetFrameRequest ─────────────────────────────────────────

#[test]
fn get_frame_request() {
    let req = GetFrameRequest { time_ms: 15000 };
    let parsed: GetFrameRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.time_ms, 15000);
}

// ─── ExportVideoRequest ──────────────────────────────────────

#[test]
fn export_video_request() {
    let req = ExportVideoRequest {
        output_path: "/output/video.mp4".into(),
        width: 1920,
        height: 1080,
        fps: 30.0,
        bitrate_kbps: 5000,
        codec: "h264".into(),
        format: "mp4".into(),
    };
    let parsed: ExportVideoRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.codec, "h264");
    assert_eq!(parsed.bitrate_kbps, 5000);
}

// ─── SaveProjectRequest / LoadProjectRequest ─────────────────

#[test]
fn save_project_request() {
    let req = SaveProjectRequest { path: "/projects/my.epp".into() };
    let parsed: SaveProjectRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.path, "/projects/my.epp");
}

#[test]
fn load_project_request() {
    let req = LoadProjectRequest { path: "/projects/my.epp".into() };
    let parsed: LoadProjectRequest = serde_json::from_str(
        &serde_json::to_string(&req).unwrap()
    ).unwrap();
    assert_eq!(parsed.path, "/projects/my.epp");
}

// ─── ProgressCallback ────────────────────────────────────────

#[test]
fn progress_callback() {
    let cb = ProgressCallback {
        operation: "export".into(),
        progress: 0.75,
        current: 750,
        total: 1000,
        message: Some("Rendering frame 750/1000".into()),
    };
    let json = serde_json::to_string(&cb).unwrap();
    let parsed: ProgressCallback = serde_json::from_str(&json).unwrap();
    assert!((parsed.progress - 0.75).abs() < f32::EPSILON);
    assert_eq!(parsed.current, 750);
    assert!(parsed.message.is_some());
}

#[test]
fn progress_callback_no_message() {
    let cb = ProgressCallback {
        operation: "render".into(),
        progress: 0.5,
        current: 50,
        total: 100,
        message: None,
    };
    let parsed: ProgressCallback = serde_json::from_str(
        &serde_json::to_string(&cb).unwrap()
    ).unwrap();
    assert!(parsed.message.is_none());
}
