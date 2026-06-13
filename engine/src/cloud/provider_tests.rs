//! Tests for the cloud module — CloudProvider, SyncStatus, SyncMetadata,
//! CloudAuthState, SyncResult, and PlaceholderCloudProvider
//!
//! These tests cover all cloud data structures and the placeholder provider
//! without requiring any network access.

use crate::cloud::provider::{CloudProjectEntry, CloudProviderTrait, PlaceholderCloudProvider};
use crate::cloud::{CloudAuthState, CloudProvider, SyncResult, SyncStatus};

// ─── CloudProvider ───────────────────────────────────────────

#[test]
fn cloud_provider_display_names() {
    assert_eq!(CloudProvider::GoogleDrive.display_name(), "Google Drive");
    assert_eq!(CloudProvider::Dropbox.display_name(), "Dropbox");
    assert_eq!(CloudProvider::Custom.display_name(), "Custom Server");
}

#[test]
fn cloud_provider_from_str_lossy() {
    assert_eq!(CloudProvider::from_str_lossy("google_drive"), Some(CloudProvider::GoogleDrive));
    assert_eq!(CloudProvider::from_str_lossy("Google Drive"), Some(CloudProvider::GoogleDrive));
    assert_eq!(CloudProvider::from_str_lossy("googledrive"), Some(CloudProvider::GoogleDrive));
    assert_eq!(CloudProvider::from_str_lossy("gdrive"), Some(CloudProvider::GoogleDrive));
    assert_eq!(CloudProvider::from_str_lossy("dropbox"), Some(CloudProvider::Dropbox));
    assert_eq!(CloudProvider::from_str_lossy("custom"), Some(CloudProvider::Custom));
    assert_eq!(CloudProvider::from_str_lossy("custom server"), Some(CloudProvider::Custom));
    assert_eq!(CloudProvider::from_str_lossy("icloud"), None);
    assert_eq!(CloudProvider::from_str_lossy(""), None);
}

#[test]
fn cloud_provider_serde_roundtrip() {
    for provider in [CloudProvider::GoogleDrive, CloudProvider::Dropbox, CloudProvider::Custom] {
        let json = serde_json::to_string(&provider).unwrap();
        let parsed: CloudProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(provider, parsed);
    }
}

// ─── SyncStatus ──────────────────────────────────────────────

#[test]
fn sync_status_display_names() {
    assert_eq!(SyncStatus::LocalOnly.display_name(), "Local Only");
    assert_eq!(SyncStatus::Synced.display_name(), "Synced");
    assert_eq!(SyncStatus::PendingUpload.display_name(), "Pending Upload");
    assert_eq!(SyncStatus::PendingDownload.display_name(), "Pending Download");
    assert_eq!(SyncStatus::Conflict.display_name(), "Conflict");
    assert_eq!(SyncStatus::Syncing.display_name(), "Syncing");
    assert_eq!(SyncStatus::Error.display_name(), "Error");
}

#[test]
fn sync_status_is_actionable() {
    assert!(SyncStatus::PendingUpload.is_actionable());
    assert!(SyncStatus::PendingDownload.is_actionable());
    assert!(SyncStatus::Conflict.is_actionable());
    assert!(SyncStatus::Error.is_actionable());
    assert!(!SyncStatus::LocalOnly.is_actionable());
    assert!(!SyncStatus::Synced.is_actionable());
    assert!(!SyncStatus::Syncing.is_actionable());
}

#[test]
fn sync_status_from_str_lossy() {
    assert_eq!(SyncStatus::from_str_lossy("localonly"), Some(SyncStatus::LocalOnly));
    assert_eq!(SyncStatus::from_str_lossy("local only"), Some(SyncStatus::LocalOnly));
    assert_eq!(SyncStatus::from_str_lossy("synced"), Some(SyncStatus::Synced));
    assert_eq!(SyncStatus::from_str_lossy("pendingupload"), Some(SyncStatus::PendingUpload));
    assert_eq!(SyncStatus::from_str_lossy("pending download"), Some(SyncStatus::PendingDownload));
    assert_eq!(SyncStatus::from_str_lossy("conflict"), Some(SyncStatus::Conflict));
    assert_eq!(SyncStatus::from_str_lossy("syncing"), Some(SyncStatus::Syncing));
    assert_eq!(SyncStatus::from_str_lossy("error"), Some(SyncStatus::Error));
    assert_eq!(SyncStatus::from_str_lossy("unknown"), None);
}

#[test]
fn sync_status_serde_roundtrip() {
    for status in [
        SyncStatus::LocalOnly, SyncStatus::Synced, SyncStatus::PendingUpload,
        SyncStatus::PendingDownload, SyncStatus::Conflict, SyncStatus::Syncing,
        SyncStatus::Error,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let parsed: SyncStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }
}

// ─── CloudAuthState ──────────────────────────────────────────

#[test]
fn auth_state_unauthenticated() {
    let auth = CloudAuthState::unauthenticated(CloudProvider::GoogleDrive);
    assert_eq!(auth.provider, CloudProvider::GoogleDrive);
    assert!(!auth.is_authenticated);
    assert!(auth.account_name.is_none());
    assert!(auth.access_token.is_none());
    assert!(auth.refresh_token.is_none());
    assert!(auth.expires_at.is_none());
}

#[test]
fn auth_state_is_expired_no_token() {
    let auth = CloudAuthState::unauthenticated(CloudProvider::Dropbox);
    assert!(auth.is_expired()); // No token → expired
}

#[test]
fn auth_state_is_expired_past_timestamp() {
    let auth = CloudAuthState {
        provider: CloudProvider::GoogleDrive,
        is_authenticated: true,
        account_name: Some("user@example.com".into()),
        access_token: Some("abc123".into()),
        refresh_token: Some("refresh123".into()),
        expires_at: Some(1000), // Past timestamp
    };
    assert!(auth.is_expired());
}

#[test]
fn auth_state_is_not_expired_future_timestamp() {
    let future = chrono::Utc::now().timestamp_millis() + 3600_000; // 1 hour from now
    let auth = CloudAuthState {
        provider: CloudProvider::GoogleDrive,
        is_authenticated: true,
        account_name: Some("user@example.com".into()),
        access_token: Some("abc123".into()),
        refresh_token: Some("refresh123".into()),
        expires_at: Some(future),
    };
    assert!(!auth.is_expired());
}

#[test]
fn auth_state_serialization() {
    let auth = CloudAuthState::unauthenticated(CloudProvider::Dropbox);
    let json = serde_json::to_string(&auth).unwrap();
    let parsed: CloudAuthState = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.provider, CloudProvider::Dropbox);
    assert!(!parsed.is_authenticated);
}

// ─── SyncResult ──────────────────────────────────────────────

#[test]
fn sync_result_ok() {
    let result = SyncResult::ok(
        "proj-1".into(),
        SyncStatus::Synced,
        "Upload complete".into(),
        1024,
    );
    assert!(result.success);
    assert_eq!(result.status, SyncStatus::Synced);
    assert_eq!(result.project_id, "proj-1");
    assert_eq!(result.bytes_transferred, 1024);
}

#[test]
fn sync_result_err() {
    let result = SyncResult::err("proj-1".into(), "Network error".into());
    assert!(!result.success);
    assert_eq!(result.status, SyncStatus::Error);
    assert_eq!(result.bytes_transferred, 0);
}

#[test]
fn sync_result_not_implemented() {
    let result = SyncResult::not_implemented("proj-1".into());
    assert!(!result.success);
    assert_eq!(result.status, SyncStatus::Error);
    assert!(result.message.contains("not yet implemented"));
}

// ─── PlaceholderCloudProvider ────────────────────────────────

#[test]
fn placeholder_provider_type() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
    assert_eq!(provider.provider_type(), CloudProvider::GoogleDrive);
}

#[test]
fn placeholder_provider_not_authenticated() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::Dropbox);
    assert!(!provider.is_authenticated());
}

#[test]
fn placeholder_provider_authenticate_fails() {
    let mut provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
    let result = provider.authenticate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not yet implemented"));
}

#[test]
fn placeholder_provider_upload_fails() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
    let result = provider.upload("proj-1", "/path/to/file.epp");
    assert!(result.is_err());
}

#[test]
fn placeholder_provider_download_fails() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
    let result = provider.download("proj-1", "/path/to/file.epp");
    assert!(result.is_err());
}

#[test]
fn placeholder_provider_list_projects_fails() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
    let result = provider.list_projects();
    assert!(result.is_err());
}

#[test]
fn placeholder_provider_delete_fails() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
    let result = provider.delete("proj-1");
    assert!(result.is_err());
}

#[test]
fn placeholder_provider_auth_state() {
    let provider = PlaceholderCloudProvider::new(CloudProvider::Custom);
    let state = provider.auth_state();
    assert_eq!(state.provider, CloudProvider::Custom);
    assert!(!state.is_authenticated);
}

// ─── CloudProjectEntry ───────────────────────────────────────

#[test]
fn cloud_project_entry_serialization() {
    let entry = CloudProjectEntry {
        project_id: "proj-1".into(),
        name: "My Project".into(),
        modified_at: 1700000000000,
        size_bytes: 524288,
        cloud_file_id: "file-abc123".into(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    let parsed: CloudProjectEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.project_id, "proj-1");
    assert_eq!(parsed.name, "My Project");
    assert_eq!(parsed.size_bytes, 524288);
}
