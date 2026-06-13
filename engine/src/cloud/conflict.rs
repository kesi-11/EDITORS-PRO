//! Conflict resolution for cloud sync
//!
//! When a project has been modified both locally and in the cloud,
//! a conflict must be resolved before sync can proceed.
//!
//! ## Resolution Strategies
//!
//! - **KeepLocal** — Discard cloud changes, keep the local version.
//! - **KeepCloud** — Discard local changes, keep the cloud version.
//! - **KeepBoth** — Rename the local version and download the cloud version
//!   so the user has both copies.
//! - **AutoMerge** — Attempt automatic merge; falls back to KeepBoth
//!   if the versions cannot be merged automatically.
//!
//! ## Timestamp-Based Suggestions
//!
//! The `SyncConflict::suggest_strategy()` method examines the modification
//! timestamps and checksums to recommend a strategy:
//!
//! | Condition                | Suggested strategy |
//! |--------------------------|--------------------|
//! | Cloud is newer           | KeepCloud          |
//! | Local is newer           | KeepLocal          |
//! | Same timestamp           | KeepBoth           |
//! | Checksums already match  | Synced (no action) |

use serde::{Deserialize, Serialize};

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConflictStrategy {
    /// Keep the local version (discard cloud changes)
    KeepLocal,
    /// Keep the cloud version (discard local changes)
    KeepCloud,
    /// Keep both — rename the local version and download cloud version
    KeepBoth,
    /// Automatically merge if possible, otherwise KeepBoth
    AutoMerge,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        Self::AutoMerge
    }
}

impl ConflictStrategy {
    /// Parse a strategy name string (case-insensitive, tolerant).
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "keeplocal" | "keep_local" | "keep local" => Some(ConflictStrategy::KeepLocal),
            "keepcloud" | "keep_cloud" | "keep cloud" => Some(ConflictStrategy::KeepCloud),
            "keepboth" | "keep_both" | "keep both" => Some(ConflictStrategy::KeepBoth),
            "automerge" | "auto_merge" | "auto merge" => Some(ConflictStrategy::AutoMerge),
            _ => None,
        }
    }

    /// Human-readable name for display.
    pub fn display_name(&self) -> &str {
        match self {
            ConflictStrategy::KeepLocal => "Keep Local",
            ConflictStrategy::KeepCloud => "Keep Cloud",
            ConflictStrategy::KeepBoth => "Keep Both",
            ConflictStrategy::AutoMerge => "Auto Merge",
        }
    }
}

/// A detected conflict between local and cloud versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// The project ID that has a conflict
    pub project_id: String,
    /// Display name of the project
    pub project_name: String,
    /// Local modification timestamp (milliseconds since epoch)
    pub local_modified_at: i64,
    /// Cloud modification timestamp (milliseconds since epoch)
    pub cloud_modified_at: i64,
    /// CRC32 checksum of the local .epp file
    pub local_checksum: String,
    /// CRC32 checksum of the cloud .epp file
    pub cloud_checksum: String,
    /// The strategy suggested by `suggest_strategy()`
    pub suggested_strategy: ConflictStrategy,
}

impl SyncConflict {
    /// Create a new conflict descriptor.
    ///
    /// The `suggested_strategy` is automatically determined by
    /// `suggest_strategy()`.
    pub fn new(
        project_id: String,
        project_name: String,
        local_modified_at: i64,
        cloud_modified_at: i64,
        local_checksum: String,
        cloud_checksum: String,
    ) -> Self {
        let mut conflict = Self {
            project_id,
            project_name,
            local_modified_at,
            cloud_modified_at,
            local_checksum,
            cloud_checksum,
            suggested_strategy: ConflictStrategy::AutoMerge, // placeholder
        };
        conflict.suggested_strategy = conflict.suggest_strategy();
        conflict
    }

    /// Determine the suggested resolution strategy based on timestamps
    /// and checksums.
    ///
    /// - If checksums are identical, no conflict exists (returns KeepLocal
    ///   as a no-op signal — the caller should check first).
    /// - If cloud is newer: suggest KeepCloud
    /// - If local is newer: suggest KeepLocal
    /// - If same timestamp: suggest KeepBoth
    pub fn suggest_strategy(&self) -> ConflictStrategy {
        // If checksums match, there's no real conflict.
        if self.local_checksum == self.cloud_checksum {
            return ConflictStrategy::KeepLocal; // no-op, versions are identical
        }

        if self.cloud_modified_at > self.local_modified_at {
            ConflictStrategy::KeepCloud
        } else if self.local_modified_at > self.cloud_modified_at {
            ConflictStrategy::KeepLocal
        } else {
            // Same timestamp but different checksums — safest to keep both
            ConflictStrategy::KeepBoth
        }
    }

    /// Resolve the conflict using the given strategy.
    pub fn resolve(&self, strategy: ConflictStrategy) -> ConflictResolution {
        match strategy {
            ConflictStrategy::KeepLocal => ConflictResolution::KeepLocal,
            ConflictStrategy::KeepCloud => ConflictResolution::KeepCloud,
            ConflictStrategy::KeepBoth => ConflictResolution::KeepBoth {
                renamed_suffix: "_local".to_string(),
            },
            ConflictStrategy::AutoMerge => self.auto_merge(),
        }
    }

    /// Attempt automatic merge.
    ///
    /// For .epp files, auto-merge is complex (JSON diff + merge of the
    /// project structure).  For now, fall back to `KeepBoth`.
    /// Full auto-merge support will be added in a future phase.
    fn auto_merge(&self) -> ConflictResolution {
        log::info!(
            "Auto-merge requested for project '{}' — falling back to KeepBoth (not yet implemented)",
            self.project_id
        );
        ConflictResolution::KeepBoth {
            renamed_suffix: "_local".to_string(),
        }
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep the local version; discard cloud changes.
    KeepLocal,
    /// Keep the cloud version; discard local changes.
    KeepCloud,
    /// Keep both versions; the local copy is renamed with the given suffix.
    KeepBoth { renamed_suffix: String },
    /// The versions were automatically merged; the merged file is at the
    /// given path.
    Merged { merged_path: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a conflict with given timestamps and checksums.
    fn make_conflict(
        local_ts: i64,
        cloud_ts: i64,
        local_crc: &str,
        cloud_crc: &str,
    ) -> SyncConflict {
        SyncConflict::new(
            "proj-1".to_string(),
            "Test Project".to_string(),
            local_ts,
            cloud_ts,
            local_crc.to_string(),
            cloud_crc.to_string(),
        )
    }

    #[test]
    fn test_suggest_strategy_cloud_newer() {
        let conflict = make_conflict(1000, 2000, "aaa", "bbb");
        assert_eq!(conflict.suggested_strategy, ConflictStrategy::KeepCloud);
    }

    #[test]
    fn test_suggest_strategy_local_newer() {
        let conflict = make_conflict(2000, 1000, "aaa", "bbb");
        assert_eq!(conflict.suggested_strategy, ConflictStrategy::KeepLocal);
    }

    #[test]
    fn test_suggest_strategy_same_timestamp() {
        let conflict = make_conflict(1000, 1000, "aaa", "bbb");
        assert_eq!(conflict.suggested_strategy, ConflictStrategy::KeepBoth);
    }

    #[test]
    fn test_suggest_strategy_same_checksum() {
        // If checksums match there is no real conflict
        let conflict = make_conflict(2000, 1000, "same", "same");
        assert_eq!(conflict.suggested_strategy, ConflictStrategy::KeepLocal);
    }

    #[test]
    fn test_resolve_keep_local() {
        let conflict = make_conflict(2000, 1000, "aaa", "bbb");
        let resolution = conflict.resolve(ConflictStrategy::KeepLocal);
        assert!(matches!(resolution, ConflictResolution::KeepLocal));
    }

    #[test]
    fn test_resolve_keep_cloud() {
        let conflict = make_conflict(2000, 1000, "aaa", "bbb");
        let resolution = conflict.resolve(ConflictStrategy::KeepCloud);
        assert!(matches!(resolution, ConflictResolution::KeepCloud));
    }

    #[test]
    fn test_resolve_keep_both() {
        let conflict = make_conflict(2000, 1000, "aaa", "bbb");
        let resolution = conflict.resolve(ConflictStrategy::KeepBoth);
        match resolution {
            ConflictResolution::KeepBoth { renamed_suffix } => {
                assert_eq!(renamed_suffix, "_local");
            }
            _ => panic!("Expected KeepBoth resolution"),
        }
    }

    #[test]
    fn test_resolve_auto_merge_falls_back_to_keep_both() {
        let conflict = make_conflict(1000, 1000, "aaa", "bbb");
        let resolution = conflict.resolve(ConflictStrategy::AutoMerge);
        assert!(matches!(
            resolution,
            ConflictResolution::KeepBoth { .. }
        ));
    }

    #[test]
    fn test_strategy_from_str_lossy() {
        assert_eq!(
            ConflictStrategy::from_str_lossy("keep_local"),
            Some(ConflictStrategy::KeepLocal)
        );
        assert_eq!(
            ConflictStrategy::from_str_lossy("KEEPCLOUD"),
            Some(ConflictStrategy::KeepCloud)
        );
        assert_eq!(
            ConflictStrategy::from_str_lossy("keep both"),
            Some(ConflictStrategy::KeepBoth)
        );
        assert_eq!(
            ConflictStrategy::from_str_lossy("auto_merge"),
            Some(ConflictStrategy::AutoMerge)
        );
        assert_eq!(ConflictStrategy::from_str_lossy("unknown"), None);
    }

    #[test]
    fn test_strategy_display_name() {
        assert_eq!(ConflictStrategy::KeepLocal.display_name(), "Keep Local");
        assert_eq!(ConflictStrategy::KeepCloud.display_name(), "Keep Cloud");
        assert_eq!(ConflictStrategy::KeepBoth.display_name(), "Keep Both");
        assert_eq!(ConflictStrategy::AutoMerge.display_name(), "Auto Merge");
    }

    #[test]
    fn test_default_strategy_is_auto_merge() {
        assert_eq!(ConflictStrategy::default(), ConflictStrategy::AutoMerge);
    }
}
