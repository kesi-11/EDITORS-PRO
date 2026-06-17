//! Manager-based decomposition of EditorsProEngine (Phase B.10).
//!
//! ## Background
//!
//! `EditorsProEngine` in `api/mod.rs` is a 2,250-line "God Object"
//! holding 16 fields spanning 7 distinct concerns: project, decoder,
//! renderer, command history, audio, proxy, transcription. The audit
//! (see `AUDIT_REPORT.md` §5.4) flagged this as a HIGH-severity
//! architectural issue.
//!
//! ## Target architecture
//!
//! The plan is to split `EditorsProEngine` into focused managers:
//!
//! ```text
//! EditorsProEngine (thin facade)
//! ├── ProjectManager     — project CRUD, .epp I/O, media assets
//! ├── DecodeManager      — HardwareDecoder lifecycle, frame cache
//! ├── RenderEngine       — PreviewRenderer, effects, compositing
//! ├── AudioEngine        — AudioDecoder, AudioMixer, transcription
//! ├── CommandManager     — CommandHistory, undo/redo
//! └── ProxyManager       — proxy generation, cache, auto-proxy
//! ```
//!
//! Each manager owns its subsystem's state and exposes a focused API.
//! The top-level `EditorsProEngine` becomes a thin facade that
//! delegates to the managers, preserving the existing public API.
//!
//! ## Migration strategy
//!
//! Rather than a big-bang refactor (which would touch ~500 call sites
//! in `api/mod.rs` and `api/bridge_api.rs`), we extract managers
//! incrementally:
//!
//! 1. **Phase B.10.1** (this commit): extract `CommandManager` as a
//!    thin newtype around `CommandHistory`. No behavior change — just
//!    establishes the pattern.
//! 2. Phase B.10.2: extract `ProjectManager` (project + media assets).
//! 3. Phase B.10.3: extract `DecodeManager` (decoder + frame cache).
//! 4. Phase B.10.4: extract `AudioEngine` (audio decoder + mixer + transcription).
//! 5. Phase B.10.5: extract `RenderEngine` (renderer + effects).
//! 6. Phase B.10.6: remove the corresponding fields from `EditorsProEngine`,
//!    keeping only the manager instances.
//!
//! Each step is independently testable and reviewable.

use crate::timeline::command::CommandHistory;

/// Manages undo/redo history for timeline operations.
///
/// Phase B.10.1: this is a thin newtype around `CommandHistory` that
/// establishes the manager pattern. Future phases will add methods
/// for command execution, history inspection, and persistence.
#[derive(Debug)]
pub struct CommandManager {
    history: CommandHistory,
}

impl CommandManager {
    /// Create a new empty command manager.
    pub fn new() -> Self {
        Self {
            history: CommandHistory::new(),
        }
    }

    /// Get a reference to the underlying `CommandHistory`.
    ///
    /// This is a temporary accessor for backward compatibility during
    /// the migration. Once all call sites are updated to use
    /// `CommandManager` methods directly, this will be removed.
    pub fn history(&self) -> &CommandHistory {
        &self.history
    }

    /// Get a mutable reference to the underlying `CommandHistory`.
    pub fn history_mut(&mut self) -> &mut CommandHistory {
        &mut self.history
    }

    /// Clear all undo/redo history.
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_manager_new() {
        let mgr = CommandManager::new();
        assert!(!mgr.can_undo());
        assert!(!mgr.can_redo());
    }

    #[test]
    fn test_command_manager_clear() {
        let mut mgr = CommandManager::new();
        // Clear on an empty history is a no-op.
        mgr.clear();
        assert!(!mgr.can_undo());
    }

    #[test]
    fn test_command_manager_history_accessors() {
        let mgr = CommandManager::new();
        let _ = mgr.history();
        let mut mgr = mgr;
        let _ = mgr.history_mut();
    }
}
