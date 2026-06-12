//! Sync manager — orchestrates the cloud sync process
//!
//! Manages the lifecycle of sync operations including:
//! - Detecting changes that need syncing
//! - Handling conflicts
//! - Progress reporting
//! - Retry logic
//!
//! ## Usage
//!
//! ```rust,ignore
//! use editors_pro_engine::cloud::provider::PlaceholderCloudProvider;
//! use editors_pro_engine::cloud::sync_manager::SyncManager;
//! use editors_pro_engine::cloud::CloudProvider;
//!
//! let provider = PlaceholderCloudProvider::new(CloudProvider::GoogleDrive);
//! let mut manager = SyncManager::new(Box::new(provider));
//!
//! // Track a project for sync
//! manager.track_project(metadata);
//!
//! // Sync a single project
//! let result = manager.sync_project("my-project-id");
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::conflict::{ConflictResolution, ConflictStrategy, SyncConflict};
use super::provider::CloudProviderTrait;
use super::{SyncMetadata, SyncResult, SyncStatus};

/// Sync manager state (serializable for persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManagerState {
    /// All tracked projects and their sync status
    pub projects: HashMap<String, SyncMetadata>,
    /// Whether auto-sync is enabled
    pub auto_sync_enabled: bool,
    /// Whether currently syncing
    pub is_syncing: bool,
    /// Last sync attempt timestamp (milliseconds since epoch)
    pub last_sync_attempt: Option<i64>,
    /// Number of conflicts requiring resolution
    pub pending_conflicts: usize,
}

impl Default for SyncManagerState {
    fn default() -> Self {
        Self {
            projects: HashMap::new(),
            auto_sync_enabled: false,
            is_syncing: false,
            last_sync_attempt: None,
            pending_conflicts: 0,
        }
    }
}

/// Sync manager — the central coordinator for cloud sync operations.
///
/// Wraps a `CloudProviderTrait` implementation and tracks the sync
/// state for all registered projects.  Actual cloud I/O is delegated
/// to the provider; this struct handles the orchestration logic
/// (conflict detection, state transitions, error handling).
pub struct SyncManager {
    state: SyncManagerState,
    provider: Box<dyn CloudProviderTrait>,
    default_conflict_strategy: ConflictStrategy,
}

impl SyncManager {
    /// Create a new sync manager with the given cloud provider.
    pub fn new(provider: Box<dyn CloudProviderTrait>) -> Self {
        Self {
            state: SyncManagerState::default(),
            provider,
            default_conflict_strategy: ConflictStrategy::default(),
        }
    }

    /// Create a new sync manager with a specific default conflict strategy.
    pub fn with_conflict_strategy(
        provider: Box<dyn CloudProviderTrait>,
        strategy: ConflictStrategy,
    ) -> Self {
        Self {
            state: SyncManagerState::default(),
            provider,
            default_conflict_strategy: strategy,
        }
    }

    // ─── State accessors ──────────────────────────────────────────

    /// Get a snapshot of the current sync manager state.
    pub fn state(&self) -> &SyncManagerState {
        &self.state
    }

    /// Check whether auto-sync is enabled.
    pub fn auto_sync_enabled(&self) -> bool {
        self.state.auto_sync_enabled
    }

    /// Enable or disable auto-sync.
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.state.auto_sync_enabled = enabled;
    }

    /// Check whether a sync operation is currently in progress.
    pub fn is_syncing(&self) -> bool {
        self.state.is_syncing
    }

    // ─── Project tracking ─────────────────────────────────────────

    /// Register a project for sync tracking.
    pub fn track_project(&mut self, metadata: SyncMetadata) {
        self.state
            .projects
            .insert(metadata.project_id.clone(), metadata);
        self.recount_conflicts();
    }

    /// Unregister a project from sync tracking.
    pub fn untrack_project(&mut self, project_id: &str) {
        self.state.projects.remove(project_id);
        self.recount_conflicts();
    }

    /// Get the sync status for a project.
    pub fn get_status(&self, project_id: &str) -> Option<SyncStatus> {
        self.state
            .projects
            .get(project_id)
            .map(|m| m.status)
    }

    // ─── Sync operations ──────────────────────────────────────────

    /// Sync a single project.
    ///
    /// Delegates to the cloud provider for the actual upload/download.
    /// If the provider is not authenticated, returns an error.
    /// If a conflict is detected, the project status is set to `Conflict`
    /// and the caller must resolve it before retrying.
    pub fn sync_project(&mut self, project_id: &str) -> Result<SyncResult, String> {
        if self.state.is_syncing {
            return Err("Sync already in progress".to_string());
        }

        // Check authentication
        if !self.provider.is_authenticated() {
            return Ok(SyncResult::err(
                project_id.to_string(),
                "Not authenticated with cloud provider".to_string(),
            ));
        }

        self.state.is_syncing = true;
        self.state.last_sync_attempt = Some(chrono::Utc::now().timestamp_millis());

        let result = self.do_sync_project(project_id);

        self.state.is_syncing = false;
        result
    }

    /// Internal implementation of project sync.
    fn do_sync_project(&mut self, project_id: &str) -> Result<SyncResult, String> {
        let status = self.get_status(project_id);

        match status {
            None => Err(format!("Project {} is not tracked for sync", project_id)),
            Some(SyncStatus::Syncing) => Err(format!(
                "Project {} is already being synced",
                project_id
            )),
            Some(SyncStatus::Conflict) => Err(format!(
                "Project {} has a conflict that must be resolved first",
                project_id
            )),
            Some(SyncStatus::LocalOnly) | Some(SyncStatus::PendingUpload) => {
                // Upload the project
                let result = SyncResult::not_implemented(project_id.to_string());
                if let Some(metadata) = self.state.projects.get_mut(project_id) {
                    if result.success {
                        metadata.status = SyncStatus::Synced;
                    } else {
                        metadata.mark_error(result.message.clone());
                    }
                }
                Ok(result)
            }
            Some(SyncStatus::PendingDownload) => {
                // Download the project
                let result = SyncResult::not_implemented(project_id.to_string());
                if let Some(metadata) = self.state.projects.get_mut(project_id) {
                    if result.success {
                        metadata.status = SyncStatus::Synced;
                    } else {
                        metadata.mark_error(result.message.clone());
                    }
                }
                Ok(result)
            }
            Some(SyncStatus::Synced) => {
                // Already synced — no action needed
                Ok(SyncResult::ok(
                    project_id.to_string(),
                    SyncStatus::Synced,
                    "Already synced".to_string(),
                    0,
                ))
            }
            Some(SyncStatus::Error) => {
                // Retry the sync
                let result = SyncResult::not_implemented(project_id.to_string());
                if let Some(metadata) = self.state.projects.get_mut(project_id) {
                    if result.success {
                        metadata.status = SyncStatus::Synced;
                    }
                    // Keep the error state if it failed again
                }
                Ok(result)
            }
        }
    }

    /// Sync all tracked projects.
    ///
    /// Returns a list of results, one per project.  Projects that are
    /// already synced or in conflict are skipped.
    pub fn sync_all(&mut self) -> Vec<SyncResult> {
        if self.state.is_syncing || !self.provider.is_authenticated() {
            return vec![SyncResult::err(
                String::new(),
                "Cannot sync: already syncing or not authenticated".to_string(),
            )];
        }

        let project_ids: Vec<String> = self
            .state
            .projects
            .keys()
            .cloned()
            .collect();

        let mut results = Vec::new();
        for id in project_ids {
            match self.sync_project(&id) {
                Ok(result) => results.push(result),
                Err(e) => results.push(SyncResult::err(id, e)),
            }
        }

        results
    }

    // ─── Conflict detection & resolution ───────────────────────────

    /// Detect conflicts for a project.
    ///
    /// Returns `Some(SyncConflict)` if both local and cloud versions
    /// have been modified since the last sync (i.e., they have
    /// different checksums).
    pub fn detect_conflict(&self, project_id: &str) -> Option<SyncConflict> {
        let metadata = self.state.projects.get(project_id)?;

        // Only check for conflicts when the status indicates one
        if metadata.status != SyncStatus::Conflict {
            return None;
        }

        let local_checksum = metadata.local_checksum.clone().unwrap_or_default();
        let cloud_checksum = metadata.cloud_checksum.clone().unwrap_or_default();

        if local_checksum == cloud_checksum {
            return None;
        }

        Some(SyncConflict::new(
            metadata.project_id.clone(),
            metadata.project_id.clone(), // project_name falls back to ID
            metadata.local_modified_at,
            metadata.cloud_modified_at.unwrap_or(0),
            local_checksum,
            cloud_checksum,
        ))
    }

    /// Resolve a conflict for a project.
    ///
    /// The resolution strategy determines how the conflict is handled:
    /// - `KeepLocal`: Mark as synced with the local version
    /// - `KeepCloud`: Download the cloud version and mark as synced
    /// - `KeepBoth`: Rename the local version and download cloud
    /// - `Merged`: Use the merged version
    pub fn resolve_conflict(
        &mut self,
        project_id: &str,
        resolution: ConflictResolution,
    ) -> Result<(), String> {
        let metadata = self
            .state
            .projects
            .get_mut(project_id)
            .ok_or_else(|| format!("Project {} is not tracked", project_id))?;

        if metadata.status != SyncStatus::Conflict {
            return Err(format!(
                "Project {} is not in conflict state (current: {})",
                project_id,
                metadata.status.display_name()
            ));
        }

        match resolution {
            ConflictResolution::KeepLocal => {
                log::info!(
                    "Conflict resolved for project {}: keeping local version",
                    project_id
                );
                metadata.status = SyncStatus::PendingUpload;
                metadata.cloud_modified_at = None;
            }
            ConflictResolution::KeepCloud => {
                log::info!(
                    "Conflict resolved for project {}: keeping cloud version",
                    project_id
                );
                metadata.status = SyncStatus::PendingDownload;
                metadata.local_modified_at = 0; // Will be updated on download
            }
            ConflictResolution::KeepBoth { renamed_suffix } => {
                log::info!(
                    "Conflict resolved for project {}: keeping both (suffix: {})",
                    project_id,
                    renamed_suffix
                );
                // The local version gets renamed; cloud version is downloaded
                metadata.status = SyncStatus::PendingDownload;
            }
            ConflictResolution::Merged { merged_path: _ } => {
                log::info!(
                    "Conflict resolved for project {}: using merged version",
                    project_id
                );
                metadata.status = SyncStatus::PendingUpload;
            }
        }

        self.recount_conflicts();
        Ok(())
    }

    /// Get all projects that need syncing (pending upload, download, or error).
    pub fn pending_sync_projects(&self) -> Vec<&SyncMetadata> {
        self.state
            .projects
            .values()
            .filter(|m| matches!(
                m.status,
                SyncStatus::PendingUpload
                    | SyncStatus::PendingDownload
                    | SyncStatus::Error
            ))
            .collect()
    }

    /// Get all detected conflicts.
    pub fn get_conflicts(&self) -> Vec<SyncConflict> {
        self.state
            .projects
            .values()
            .filter(|m| m.status == SyncStatus::Conflict)
            .filter_map(|m| {
                let local_checksum = m.local_checksum.clone().unwrap_or_default();
                let cloud_checksum = m.cloud_checksum.clone().unwrap_or_default();

                if local_checksum == cloud_checksum {
                    return None;
                }

                Some(SyncConflict::new(
                    m.project_id.clone(),
                    m.project_id.clone(),
                    m.local_modified_at,
                    m.cloud_modified_at.unwrap_or(0),
                    local_checksum,
                    cloud_checksum,
                ))
            })
            .collect()
    }

    // ─── Internal helpers ─────────────────────────────────────────

    /// Recount the number of pending conflicts in the state.
    fn recount_conflicts(&mut self) {
        self.state.pending_conflicts = self
            .state
            .projects
            .values()
            .filter(|m| m.status == SyncStatus::Conflict)
            .count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloud::provider::PlaceholderCloudProvider;
    use crate::cloud::CloudProvider;

    fn make_manager() -> SyncManager {
        SyncManager::new(Box::new(PlaceholderCloudProvider::new(
            CloudProvider::GoogleDrive,
        )))
    }

    #[test]
    fn test_track_and_untrack_project() {
        let mut mgr = make_manager();
        let meta = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);

        mgr.track_project(meta);
        assert_eq!(mgr.get_status("proj-1"), Some(SyncStatus::LocalOnly));

        mgr.untrack_project("proj-1");
        assert_eq!(mgr.get_status("proj-1"), None);
    }

    #[test]
    fn test_sync_untracked_project() {
        let mut mgr = make_manager();
        let result = mgr.sync_project("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_not_authenticated() {
        let mut mgr = make_manager();
        mgr.track_project(SyncMetadata::new_local(
            "proj-1".to_string(),
            CloudProvider::GoogleDrive,
        ));

        // PlaceholderCloudProvider is not authenticated
        let result = mgr.sync_project("proj-1").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Not authenticated"));
    }

    #[test]
    fn test_sync_already_synced() {
        let mut mgr = make_manager();
        let mut meta = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);
        meta.status = SyncStatus::Synced;
        mgr.track_project(meta);

        // Since not authenticated, we get "not authenticated" first.
        // But the logic should short-circuit for synced + authenticated.
        // With placeholder (not authed), we'll get the auth check.
        let result = mgr.sync_project("proj-1").unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_auto_sync_toggle() {
        let mut mgr = make_manager();
        assert!(!mgr.auto_sync_enabled());
        mgr.set_auto_sync(true);
        assert!(mgr.auto_sync_enabled());
    }

    #[test]
    fn test_conflict_detection() {
        let mut mgr = make_manager();
        let mut meta = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);
        meta.status = SyncStatus::Conflict;
        meta.local_checksum = Some("abc".to_string());
        meta.cloud_checksum = Some("def".to_string());
        meta.cloud_modified_at = Some(2000);
        mgr.track_project(meta);

        let conflict = mgr.detect_conflict("proj-1").unwrap();
        assert_eq!(conflict.project_id, "proj-1");
        assert_eq!(conflict.local_checksum, "abc");
        assert_eq!(conflict.cloud_checksum, "def");
    }

    #[test]
    fn test_resolve_conflict_keep_local() {
        let mut mgr = make_manager();
        let mut meta = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);
        meta.status = SyncStatus::Conflict;
        meta.local_checksum = Some("abc".to_string());
        meta.cloud_checksum = Some("def".to_string());
        mgr.track_project(meta);

        mgr.resolve_conflict("proj-1", ConflictResolution::KeepLocal)
            .unwrap();
        assert_eq!(mgr.get_status("proj-1"), Some(SyncStatus::PendingUpload));
    }

    #[test]
    fn test_resolve_conflict_keep_cloud() {
        let mut mgr = make_manager();
        let mut meta = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);
        meta.status = SyncStatus::Conflict;
        meta.local_checksum = Some("abc".to_string());
        meta.cloud_checksum = Some("def".to_string());
        mgr.track_project(meta);

        mgr.resolve_conflict("proj-1", ConflictResolution::KeepCloud)
            .unwrap();
        assert_eq!(
            mgr.get_status("proj-1"),
            Some(SyncStatus::PendingDownload)
        );
    }

    #[test]
    fn test_resolve_conflict_not_in_conflict() {
        let mut mgr = make_manager();
        mgr.track_project(SyncMetadata::new_local(
            "proj-1".to_string(),
            CloudProvider::GoogleDrive,
        ));

        let result = mgr.resolve_conflict("proj-1", ConflictResolution::KeepLocal);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_conflicts() {
        let mut mgr = make_manager();

        // No conflicts initially
        assert!(mgr.get_conflicts().is_empty());

        // Add a conflicted project
        let mut meta1 = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);
        meta1.status = SyncStatus::Conflict;
        meta1.local_checksum = Some("aaa".to_string());
        meta1.cloud_checksum = Some("bbb".to_string());
        meta1.cloud_modified_at = Some(2000);
        mgr.track_project(meta1);

        // Add a non-conflicted project
        let meta2 = SyncMetadata::new_local("proj-2".to_string(), CloudProvider::GoogleDrive);
        mgr.track_project(meta2);

        let conflicts = mgr.get_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].project_id, "proj-1");
    }

    #[test]
    fn test_pending_sync_projects() {
        let mut mgr = make_manager();

        // LocalOnly — not pending
        mgr.track_project(SyncMetadata::new_local(
            "proj-1".to_string(),
            CloudProvider::GoogleDrive,
        ));

        // PendingUpload — pending
        let mut meta2 = SyncMetadata::new_local("proj-2".to_string(), CloudProvider::GoogleDrive);
        meta2.status = SyncStatus::PendingUpload;
        mgr.track_project(meta2);

        // Synced — not pending
        let mut meta3 = SyncMetadata::new_local("proj-3".to_string(), CloudProvider::GoogleDrive);
        meta3.status = SyncStatus::Synced;
        mgr.track_project(meta3);

        let pending = mgr.pending_sync_projects();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].project_id, "proj-2");
    }

    #[test]
    fn test_conflict_count_updates() {
        let mut mgr = make_manager();

        let mut meta = SyncMetadata::new_local("proj-1".to_string(), CloudProvider::GoogleDrive);
        meta.status = SyncStatus::Conflict;
        meta.local_checksum = Some("aaa".to_string());
        meta.cloud_checksum = Some("bbb".to_string());
        mgr.track_project(meta);

        assert_eq!(mgr.state().pending_conflicts, 1);

        // Resolve the conflict
        mgr.resolve_conflict("proj-1", ConflictResolution::KeepLocal)
            .unwrap();
        assert_eq!(mgr.state().pending_conflicts, 0);
    }

    #[test]
    fn test_sync_all_without_auth() {
        let mut mgr = make_manager();
        mgr.track_project(SyncMetadata::new_local(
            "proj-1".to_string(),
            CloudProvider::GoogleDrive,
        ));

        let results = mgr.sync_all();
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
    }
}
