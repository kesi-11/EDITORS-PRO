//! Comprehensive tests for the command system (undo/redo)
//!
//! Covers: AddClip, RemoveClip, MoveClip, SplitClip, TrimClip,
//! CommandHistory execution and undo/redo stacks.

use super::clip::Clip;
use super::command::{
    AddClipCommand, Command, CommandHistory, MoveClipCommand, RemoveClipCommand,
    SplitClipCommand, TrimClipCommand,
};
use super::track::TrackType;
use super::Timeline;

fn make_clip(start_ms: u64, duration_ms: u64) -> Clip {
    Clip::new("asset-1", start_ms, duration_ms)
}

fn make_timeline_with_track() -> Timeline {
    let mut tl = Timeline::new();
    tl.add_track(TrackType::Video, Some("V1".into()));
    tl
}

// ─── AddClipCommand ──────────────────────────────────────────

#[test]
fn add_clip_command_executes_and_undo() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);
    let clip_id = clip.id.clone();

    let mut cmd = AddClipCommand::new(track_id.clone(), clip);

    // Execute
    let result = cmd.execute(&mut tl).unwrap();
    assert!(result.success);
    assert!(tl.find_clip(&clip_id).is_some());

    // Undo
    let undo_result = cmd.undo(&mut tl).unwrap();
    assert!(undo_result.success);
    assert!(tl.find_clip(&clip_id).is_none());
}

#[test]
fn add_clip_command_wrong_track_fails() {
    let mut tl = make_timeline_with_track();
    let clip = make_clip(0, 1000);
    let mut cmd = AddClipCommand::new("nonexistent-track".into(), clip);

    let result = cmd.execute(&mut tl);
    assert!(result.is_err());
}

#[test]
fn add_clip_command_double_execute_fails() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);
    let mut cmd = AddClipCommand::new(track_id, clip);

    cmd.execute(&mut tl).unwrap();
    let second = cmd.execute(&mut tl);
    assert!(second.is_err()); // clip was taken
}

// ─── RemoveClipCommand ──────────────────────────────────────

#[test]
fn remove_clip_command_executes_and_undo() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);
    let clip_id = clip.id.clone();

    tl.add_clip_to_track(&track_id, clip).unwrap();

    let mut cmd = RemoveClipCommand {
        track_id: track_id.clone(),
        clip_id: clip_id.clone(),
        removed_clip: None,
    };

    let result = cmd.execute(&mut tl).unwrap();
    assert!(result.success);
    assert!(tl.find_clip(&clip_id).is_none());

    // Undo restores the clip
    let undo_result = cmd.undo(&mut tl).unwrap();
    assert!(undo_result.success);
    assert!(tl.find_clip(&clip_id).is_some());
}

#[test]
fn remove_clip_command_nonexistent_clip_fails() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();

    let mut cmd = RemoveClipCommand {
        track_id,
        clip_id: "nonexistent".into(),
        removed_clip: None,
    };

    let result = cmd.execute(&mut tl);
    assert!(result.is_err());
}

// ─── MoveClipCommand ─────────────────────────────────────────

#[test]
fn move_clip_command_changes_start_ms() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);
    let clip_id = clip.id.clone();

    tl.add_clip_to_track(&track_id, clip).unwrap();

    let mut cmd = MoveClipCommand {
        clip_id: clip_id.clone(),
        new_start_ms: 5000,
        old_start_ms: 0,
        new_track_id: None,
        old_track_id: track_id,
    };

    let result = cmd.execute(&mut tl).unwrap();
    assert!(result.success);
    assert_eq!(tl.find_clip(&clip_id).unwrap().1.start_ms, 5000);

    // Undo
    cmd.undo(&mut tl).unwrap();
    assert_eq!(tl.find_clip(&clip_id).unwrap().1.start_ms, 0);
}

#[test]
fn move_clip_command_nonexistent_clip_fails() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();

    let mut cmd = MoveClipCommand {
        clip_id: "nonexistent".into(),
        new_start_ms: 5000,
        old_start_ms: 0,
        new_track_id: None,
        old_track_id: track_id,
    };

    let result = cmd.execute(&mut tl);
    assert!(result.is_err());
}

// ─── SplitClipCommand ────────────────────────────────────────

#[test]
fn split_clip_command_splits_and_undo() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 5000);
    let clip_id = clip.id.clone();

    tl.add_clip_to_track(&track_id, clip).unwrap();

    let mut cmd = SplitClipCommand {
        clip_id: clip_id.clone(),
        time_ms: 2500,
        original_clip: None,
        left_clip: None,
        right_clip: None,
    };

    let result = cmd.execute(&mut tl).unwrap();
    assert!(result.success);
    // Original clip should be gone
    assert!(tl.find_clip(&clip_id).is_none());
    // Should have two clips now
    assert_eq!(tl.tracks[0].clips.len(), 2);

    // Undo should restore the original
    cmd.undo(&mut tl).unwrap();
    assert_eq!(tl.tracks[0].clips.len(), 1);
}

#[test]
fn split_clip_command_invalid_time_fails() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 5000);
    let clip_id = clip.id.clone();

    tl.add_clip_to_track(&track_id, clip).unwrap();

    let mut cmd = SplitClipCommand {
        clip_id,
        time_ms: 0, // at start — invalid
        original_clip: None,
        left_clip: None,
        right_clip: None,
    };

    let result = cmd.execute(&mut tl);
    assert!(result.is_err());
}

// ─── TrimClipCommand ─────────────────────────────────────────

#[test]
fn trim_clip_command_changes_trim_points() {
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 5000);
    let clip_id = clip.id.clone();

    tl.add_clip_to_track(&track_id, clip).unwrap();

    let mut cmd = TrimClipCommand {
        clip_id: clip_id.clone(),
        new_trim_start_ms: 500,
        new_trim_end_ms: 500,
        old_trim_start_ms: 0,
        old_trim_end_ms: 0,
    };

    let result = cmd.execute(&mut tl).unwrap();
    assert!(result.success);

    let found = tl.find_clip(&clip_id).unwrap().1;
    assert_eq!(found.trim_start_ms, 500);
    assert_eq!(found.trim_end_ms, 500);

    // Undo
    cmd.undo(&mut tl).unwrap();
    let found = tl.find_clip(&clip_id).unwrap().1;
    assert_eq!(found.trim_start_ms, 0);
    assert_eq!(found.trim_end_ms, 0);
}

#[test]
fn trim_clip_command_nonexistent_clip_fails() {
    let mut tl = make_timeline_with_track();

    let mut cmd = TrimClipCommand {
        clip_id: "nonexistent".into(),
        new_trim_start_ms: 500,
        new_trim_end_ms: 0,
        old_trim_start_ms: 0,
        old_trim_end_ms: 0,
    };

    let result = cmd.execute(&mut tl);
    assert!(result.is_err());
}

// ─── CommandHistory ──────────────────────────────────────────

#[test]
fn command_history_new_is_empty() {
    let history = CommandHistory::new();
    assert_eq!(history.undo_stack_len(), 0);
    assert_eq!(history.redo_stack_len(), 0);
}

#[test]
fn command_history_execute_pushes_to_undo_stack() {
    let mut history = CommandHistory::new();
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);

    let cmd = AddClipCommand::new(track_id, clip);
    history.execute(cmd, &mut tl).unwrap();

    assert_eq!(history.undo_stack_len(), 1);
    assert_eq!(history.redo_stack_len(), 0);
}

#[test]
fn command_history_undo_moves_to_redo_stack() {
    let mut history = CommandHistory::new();
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);

    let cmd = AddClipCommand::new(track_id, clip);
    history.execute(cmd, &mut tl).unwrap();

    history.undo(&mut tl).unwrap();
    assert_eq!(history.undo_stack_len(), 0);
    assert_eq!(history.redo_stack_len(), 1);
}

#[test]
fn command_history_redo_restores_command() {
    let mut history = CommandHistory::new();
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);
    let clip_id = clip.id.clone();

    let cmd = AddClipCommand::new(track_id, clip);
    history.execute(cmd, &mut tl).unwrap();
    history.undo(&mut tl).unwrap();

    assert!(tl.find_clip(&clip_id).is_none());

    history.redo(&mut tl).unwrap();
    assert!(tl.find_clip(&clip_id).is_some());
    assert_eq!(history.undo_stack_len(), 1);
    assert_eq!(history.redo_stack_len(), 0);
}

#[test]
fn command_history_undo_on_empty_does_nothing() {
    let mut history = CommandHistory::new();
    let mut tl = Timeline::new();
    let result = history.undo(&mut tl);
    assert!(result.is_none());
}

#[test]
fn command_history_redo_on_empty_does_nothing() {
    let mut history = CommandHistory::new();
    let mut tl = Timeline::new();
    let result = history.redo(&mut tl);
    assert!(result.is_none());
}

#[test]
fn command_history_new_command_clears_redo_stack() {
    let mut history = CommandHistory::new();
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();

    // First command
    let clip1 = make_clip(0, 1000);
    let cmd1 = AddClipCommand::new(track_id.clone(), clip1);
    history.execute(cmd1, &mut tl).unwrap();

    // Undo
    history.undo(&mut tl).unwrap();
    assert_eq!(history.redo_stack_len(), 1);

    // New command should clear redo stack
    let clip2 = make_clip(2000, 1000);
    let cmd2 = AddClipCommand::new(track_id, clip2);
    history.execute(cmd2, &mut tl).unwrap();

    assert_eq!(history.redo_stack_len(), 0);
    assert_eq!(history.undo_stack_len(), 1);
}

#[test]
fn command_history_description() {
    let history = CommandHistory::new();
    assert_eq!(history.description(), "Command History (0 undo, 0 redo)");
}

#[test]
fn command_history_clear() {
    let mut history = CommandHistory::new();
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();
    let clip = make_clip(0, 1000);
    let cmd = AddClipCommand::new(track_id, clip);
    history.execute(cmd, &mut tl).unwrap();

    history.clear();
    assert_eq!(history.undo_stack_len(), 0);
    assert_eq!(history.redo_stack_len(), 0);
}

#[test]
fn command_history_multiple_undo_redo() {
    let mut history = CommandHistory::new();
    let mut tl = make_timeline_with_track();
    let track_id = tl.tracks[0].id.clone();

    for i in 0..5 {
        let clip = make_clip(i * 1000, 1000);
        let cmd = AddClipCommand::new(track_id.clone(), clip);
        history.execute(cmd, &mut tl).unwrap();
    }

    assert_eq!(history.undo_stack_len(), 5);

    // Undo all
    for _ in 0..5 {
        history.undo(&mut tl).unwrap();
    }
    assert_eq!(history.undo_stack_len(), 0);
    assert_eq!(history.redo_stack_len(), 5);

    // Redo all
    for _ in 0..5 {
        history.redo(&mut tl).unwrap();
    }
    assert_eq!(history.undo_stack_len(), 5);
    assert_eq!(history.redo_stack_len(), 0);
}
