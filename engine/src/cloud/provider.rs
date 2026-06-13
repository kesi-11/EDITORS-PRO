//! Abstract cloud storage provider interface
//!
//! Defines the trait that all cloud storage backends must implement.
//! Actual implementations (Google Drive, Dropbox) will be added later.
//! The `PlaceholderCloudProvider` is available for development and testing.

use serde::{Deserialize, Serialize};

use super::{CloudAuthState, CloudProvider, SyncResult};

/// Trait for cloud storage providers
///
/// Each provider must implement these methods for full cloud integration.
/// Placeholder implementations return `Err("Cloud sync not yet implemented")`.
pub trait CloudProviderTrait: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> CloudProvider;

    /// Check if authenticated
    fn is_authenticated(&self) -> bool;

    /// Authenticate with the provider (OAuth2 flow)
    fn authenticate(&mut self) -> Result<CloudAuthState, String>;

    /// Upload a project file to cloud storage
    fn upload(&self, project_id: &str, file_path: &str) -> Result<SyncResult, String>;

    /// Download a project file from cloud storage
    fn download(&self, project_id: &str, file_path: &str) -> Result<SyncResult, String>;

    /// List all synced projects in cloud storage
    fn list_projects(&self) -> Result<Vec<CloudProjectEntry>, String>;

    /// Delete a project from cloud storage
    fn delete(&self, project_id: &str) -> Result<(), String>;

    /// Get the auth state
    fn auth_state(&self) -> &CloudAuthState;
}

/// Entry in the cloud project listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProjectEntry {
    /// Unique project identifier
    pub project_id: String,
    /// Display name of the project
    pub name: String,
    /// Last modification timestamp (milliseconds since epoch)
    pub modified_at: i64,
    /// Size of the .epp file in bytes
    pub size_bytes: u64,
    /// Provider-specific file identifier
    pub cloud_file_id: String,
}

/// Placeholder cloud provider (for development/testing)
///
/// All operations return a "not implemented" error except for
/// `is_authenticated()` which returns `false` and `provider_type()`
/// which returns the configured provider type.
pub struct PlaceholderCloudProvider {
    auth_state: CloudAuthState,
}

impl PlaceholderCloudProvider {
    /// Create a new placeholder provider for the given cloud type.
    pub fn new(provider: CloudProvider) -> Self {
        Self {
            auth_state: CloudAuthState::unauthenticated(provider),
        }
    }
}

impl CloudProviderTrait for PlaceholderCloudProvider {
    fn provider_type(&self) -> CloudProvider {
        self.auth_state.provider
    }

    fn is_authenticated(&self) -> bool {
        false
    }

    fn authenticate(&mut self) -> Result<CloudAuthState, String> {
        Err("Cloud sync not yet implemented".to_string())
    }

    fn upload(&self, project_id: &str, _file_path: &str) -> Result<SyncResult, String> {
        Err(format!(
            "Cloud sync not yet implemented (upload for project {})",
            project_id
        ))
    }

    fn download(&self, project_id: &str, _file_path: &str) -> Result<SyncResult, String> {
        Err(format!(
            "Cloud sync not yet implemented (download for project {})",
            project_id
        ))
    }

    fn list_projects(&self) -> Result<Vec<CloudProjectEntry>, String> {
        Err("Cloud sync not yet implemented".to_string())
    }

    fn delete(&self, project_id: &str) -> Result<(), String> {
        Err(format!(
            "Cloud sync not yet implemented (delete for project {})",
            project_id
        ))
    }

    fn auth_state(&self) -> &CloudAuthState {
        &self.auth_state
    }
}
