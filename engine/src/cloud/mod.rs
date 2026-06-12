//! Cloud sync foundation for project synchronization
//!
//! Provides the architecture for syncing .epp project files across devices.
//! Source media (video files) are NOT synced — only the lightweight project
//! metadata is transmitted, keeping sync fast and bandwidth-efficient.
//!
//! ## Architecture
//!
//! 1. **Sync Provider** — Abstract interface for cloud storage backends
//!    (Google Drive, Dropbox, custom server)
//! 2. **Sync State** — Tracks what's been synced, pending changes, conflicts
//! 3. **Conflict Resolution** — Strategy for merging simultaneous edits
//! 4. **Sync Manager** — Orchestrates the sync process
//!
//! ## Design Principles
//!
//! - Only .epp project files are synced (typically < 1MB)
//! - Source media stays local — referenced by content hash
//! - Conflict resolution prefers the latest modification timestamp
//! - All sync operations are atomic (no partial states)
//! - Works offline; syncs when connectivity is restored

pub mod conflict;
pub mod provider;
pub mod sync_manager;

use serde::{Deserialize, Serialize};

/// Cloud storage provider type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CloudProvider {
    GoogleDrive,
    Dropbox,
    Custom,
}

impl CloudProvider {
    /// Human-readable name for display in the UI.
    pub fn display_name(&self) -> &str {
        match self {
            CloudProvider::GoogleDrive => "Google Drive",
            CloudProvider::Dropbox => "Dropbox",
            CloudProvider::Custom => "Custom Server",
        }
    }

    /// Parse a provider name string (case-insensitive, tolerant).
    ///
    /// Returns `None` if the string does not match any known provider.
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "google_drive" | "google drive" | "googledrive" | "gdrive" => {
                Some(CloudProvider::GoogleDrive)
            }
            "dropbox" => Some(CloudProvider::Dropbox),
            "custom" | "custom server" => Some(CloudProvider::Custom),
            _ => None,
        }
    }
}

/// Sync status of a project
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    /// Not synced — local only
    LocalOnly,
    /// Synced — local and cloud are identical
    Synced,
    /// Local changes not yet uploaded
    PendingUpload,
    /// Cloud changes not yet downloaded
    PendingDownload,
    /// Both local and cloud have changes — needs resolution
    Conflict,
    /// Sync in progress
    Syncing,
    /// Error during last sync
    Error,
}

impl SyncStatus {
    /// Human-readable name for display in the UI.
    pub fn display_name(&self) -> &str {
        match self {
            SyncStatus::LocalOnly => "Local Only",
            SyncStatus::Synced => "Synced",
            SyncStatus::PendingUpload => "Pending Upload",
            SyncStatus::PendingDownload => "Pending Download",
            SyncStatus::Conflict => "Conflict",
            SyncStatus::Syncing => "Syncing",
            SyncStatus::Error => "Error",
        }
    }

    /// Whether the user can take action on this status
    /// (e.g., trigger a sync or resolve a conflict).
    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            SyncStatus::PendingUpload
                | SyncStatus::PendingDownload
                | SyncStatus::Conflict
                | SyncStatus::Error
        )
    }

    /// Parse a status string (case-insensitive, tolerant).
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "localonly" | "local only" | "local_only" => Some(SyncStatus::LocalOnly),
            "synced" => Some(SyncStatus::Synced),
            "pendingupload" | "pending upload" | "pending_upload" => {
                Some(SyncStatus::PendingUpload)
            }
            "pendingdownload" | "pending download" | "pending_download" => {
                Some(SyncStatus::PendingDownload)
            }
            "conflict" => Some(SyncStatus::Conflict),
            "syncing" => Some(SyncStatus::Syncing),
            "error" => Some(SyncStatus::Error),
            _ => None,
        }
    }
}

/// Metadata about a synced project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    /// Project ID
    pub project_id: String,
    /// Cloud provider
    pub provider: CloudProvider,
    /// Cloud file ID (provider-specific identifier)
    pub cloud_file_id: Option<String>,
    /// Current sync status
    pub status: SyncStatus,
    /// Last local modification timestamp (milliseconds since epoch)
    pub local_modified_at: i64,
    /// Last cloud modification timestamp (milliseconds since epoch)
    pub cloud_modified_at: Option<i64>,
    /// Last successful sync timestamp (milliseconds since epoch)
    pub last_synced_at: Option<i64>,
    /// Local .epp file CRC32 at last sync
    pub local_checksum: Option<String>,
    /// Cloud .epp file CRC32 at last sync
    pub cloud_checksum: Option<String>,
    /// Error message if status is Error
    pub error_message: Option<String>,
}

impl SyncMetadata {
    /// Create metadata for a local-only project (not yet synced).
    pub fn new_local(project_id: String, provider: CloudProvider) -> Self {
        Self {
            project_id,
            provider,
            cloud_file_id: None,
            status: SyncStatus::LocalOnly,
            local_modified_at: chrono::Utc::now().timestamp_millis(),
            cloud_modified_at: None,
            last_synced_at: None,
            local_checksum: None,
            cloud_checksum: None,
            error_message: None,
        }
    }

    /// Update the local modification timestamp to now.
    pub fn touch_local(&mut self) {
        self.local_modified_at = chrono::Utc::now().timestamp_millis();
    }

    /// Mark the project as synced with the given cloud file ID and checksums.
    pub fn mark_synced(
        &mut self,
        cloud_file_id: String,
        local_checksum: String,
        cloud_checksum: String,
    ) {
        let now = chrono::Utc::now().timestamp_millis();
        self.cloud_file_id = Some(cloud_file_id);
        self.status = SyncStatus::Synced;
        self.last_synced_at = Some(now);
        self.local_checksum = Some(local_checksum);
        self.cloud_checksum = Some(cloud_checksum);
        self.error_message = None;
    }

    /// Mark the project as having a sync error.
    pub fn mark_error(&mut self, message: String) {
        self.status = SyncStatus::Error;
        self.error_message = Some(message);
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Whether the sync operation succeeded
    pub success: bool,
    /// The resulting sync status
    pub status: SyncStatus,
    /// Human-readable message describing the outcome
    pub message: String,
    /// The project ID that was synced
    pub project_id: String,
    /// Number of bytes transferred during this operation
    pub bytes_transferred: u64,
}

impl SyncResult {
    /// Create a successful sync result.
    pub fn ok(project_id: String, status: SyncStatus, message: String, bytes_transferred: u64) -> Self {
        Self {
            success: true,
            status,
            message,
            project_id,
            bytes_transferred,
        }
    }

    /// Create a failed sync result.
    pub fn err(project_id: String, message: String) -> Self {
        Self {
            success: false,
            status: SyncStatus::Error,
            message,
            project_id,
            bytes_transferred: 0,
        }
    }

    /// Create a placeholder "not implemented" result.
    pub fn not_implemented(project_id: String) -> Self {
        Self {
            success: false,
            status: SyncStatus::Error,
            message: "Cloud sync not yet implemented".to_string(),
            project_id,
            bytes_transferred: 0,
        }
    }
}

/// Cloud authentication state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAuthState {
    /// The cloud provider this auth state belongs to
    pub provider: CloudProvider,
    /// Whether the user is currently authenticated
    pub is_authenticated: bool,
    /// The account name / email of the authenticated user
    pub account_name: Option<String>,
    /// OAuth2 access token
    pub access_token: Option<String>,
    /// OAuth2 refresh token
    pub refresh_token: Option<String>,
    /// Token expiration timestamp (milliseconds since epoch)
    pub expires_at: Option<i64>,
}

impl CloudAuthState {
    /// Create an unauthenticated state for a given provider.
    pub fn unauthenticated(provider: CloudProvider) -> Self {
        Self {
            provider,
            is_authenticated: false,
            account_name: None,
            access_token: None,
            refresh_token: None,
            expires_at: None,
        }
    }

    /// Check if the access token has expired.
    ///
    /// Returns `true` if there is no token or the expiration timestamp
    /// is in the past.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => {
                let now = chrono::Utc::now().timestamp_millis();
                now >= expires_at
            }
            None => true,
        }
    }
}
