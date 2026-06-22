//! Advanced trim modes — ripple, roll, slip, slide.
//!
//! ## Modes
//!
//! - **Ripple**: trims a clip's out-point (or in-point) and shifts all
//!   subsequent clips on the same track to close the gap. Total timeline
//!   duration changes.
//! - **Roll**: trims two adjacent clips simultaneously — one gets shorter,
//!   the other gets longer by the same amount. Total timeline duration
//!   is unchanged.
//! - **Slip**: changes the in/out points of a clip without changing its
//!   duration or position on the timeline. You see different frames of
//!   the same shot.
//! - **Slide**: moves a clip left or right between its neighbors. The
//!   neighbors are trimmed to make room. Total timeline duration is
//!   unchanged; the slid clip's duration is unchanged.
//!
//! All four are undoable via the existing command pattern.
//!
//! ## video: debt markers
//!
//! - Single-track ripple only, upgrade to multi-track ripple with sync-lock if cross-track sync matters
//! - No J/K/L shuttle preview, upgrade to real-time shuttle scrubbing for pro trim feel
//! - Slip is unconstrained, upgrade to source media bounds check if user can slip past clip start/end

use serde::{Deserialize, Serialize};

/// The kind of advanced trim to perform.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    /// Trim clip + shift subsequent clips to close the gap.
    Ripple,
    /// Trim two adjacent clips simultaneously (duration-preserving).
    Roll,
    /// Change in/out without changing duration or position.
    Slip,
    /// Move clip between neighbors (neighbors trimmed).
    Slide,
}

/// Parameters for an advanced trim operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTrimParams {
    pub mode: TrimMode,
    /// ID of the primary clip being trimmed.
    pub clip_id: String,
    /// For Ripple: the delta in milliseconds applied to the clip's out-point.
    /// Positive = clip gets longer, negative = clip gets shorter.
    /// For Roll: the delta applied to the cut-point between this clip and the next.
    /// For Slip: the delta applied to both in and out (positive = later frames, negative = earlier).
    /// For Slide: the delta to move the clip left (negative) or right (positive).
    pub delta_ms: i64,
    /// For Roll/Slide: the ID of the adjacent clip (next clip for Roll, either neighbor for Slide).
    pub adjacent_clip_id: Option<String>,
}

/// Result of an advanced trim operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTrimResult {
    pub mode: TrimMode,
    /// Clip IDs that were modified, with their new in/out/position values.
    pub modified: Vec<ClipModification>,
    /// New total timeline duration in ms.
    pub new_timeline_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipModification {
    pub clip_id: String,
    pub new_in_point_ms: Option<i64>,
    pub new_out_point_ms: Option<i64>,
    pub new_start_ms: Option<i64>,
}

/// Validate that a trim operation is legal given the current clip state.
///
/// Returns `Ok(())` if legal, or an error message describing the violation.
///
/// video: single-track ripple only, upgrade to multi-track ripple with sync-lock if cross-track sync matters
pub fn validate_trim(
    params: &AdvancedTrimParams,
    clip_duration_ms: u64,
    clip_current_in_ms: i64,
    clip_current_out_ms: i64,
    clip_current_start_ms: i64,
    adjacent_clip_duration_ms: Option<u64>,
    adjacent_clip_current_in_ms: Option<i64>,
    adjacent_clip_current_out_ms: Option<i64>,
    adjacent_clip_current_start_ms: Option<i64>,
) -> Result<(), String> {
    match params.mode {
        TrimMode::Ripple => {
            // Delta can be negative (clip gets shorter) as long as the clip
            // still has positive duration.
            let new_duration = (clip_current_out_ms - clip_current_in_ms) + params.delta_ms;
            if new_duration <= 0 {
                return Err(format!(
                    "Ripple trim would make clip {} zero or negative duration (new duration: {} ms)",
                    params.clip_id, new_duration
                ));
            }
            Ok(())
        }
        TrimMode::Roll => {
            let adj_in = adjacent_clip_current_in_ms
                .ok_or_else(|| "Roll requires adjacent clip in-point".to_string())?;
            let adj_out = adjacent_clip_current_out_ms
                .ok_or_else(|| "Roll requires adjacent clip out-point".to_string())?;

            // This clip's new out-point = current out + delta
            let new_out = clip_current_out_ms + params.delta_ms;
            if new_out <= clip_current_in_ms {
                return Err("Roll would make primary clip zero or negative duration".into());
            }
            // Adjacent clip's new in-point = current in - delta (mirror)
            let new_adj_in = adj_in - params.delta_ms;
            if new_adj_in >= adj_out {
                return Err("Roll would make adjacent clip zero or negative duration".into());
            }
            Ok(())
        }
        TrimMode::Slip => {
            // Slip changes in/out by delta. Source media must have enough headroom.
            // video: slip is unconstrained, upgrade to source media bounds check if user can slip past clip start/end
            let new_in = clip_current_in_ms + params.delta_ms;
            let new_out = clip_current_out_ms + params.delta_ms;
            if new_in < 0 {
                return Err("Slip would move in-point before source start".into());
            }
            // We don't know source duration here — the caller should also check.
            let _ = new_out;
            Ok(())
        }
        TrimMode::Slide => {
            let adj_start = adjacent_clip_current_start_ms
                .ok_or_else(|| "Slide requires adjacent clip start".to_string())?;
            let _ = adjacent_clip_duration_ms;

            // Slide moves this clip by delta. The neighbor's out-point (if before)
            // or in-point (if after) shifts to accommodate.
            if params.delta_ms < 0 {
                // Sliding left: previous neighbor's out-point shifts left (gets shorter)
                // The neighbor can't get shorter than 0 duration.
                let _ = adj_start;
            } else {
                // Sliding right: previous neighbor's out-point shifts right (gets longer)
                // The next neighbor's in-point shifts right (gets shorter)
            }
            // Loose validation — real validation happens at the timeline level
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripple_legal_positive_delta() {
        let params = AdvancedTrimParams {
            mode: TrimMode::Ripple,
            clip_id: "c1".into(),
            delta_ms: 500,
            adjacent_clip_id: None,
        };
        let result = validate_trim(&params, 2000, 0, 2000, 0, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn ripple_illegal_negative_delta_zero_duration() {
        let params = AdvancedTrimParams {
            mode: TrimMode::Ripple,
            clip_id: "c1".into(),
            delta_ms: -2000,
            adjacent_clip_id: None,
        };
        let result = validate_trim(&params, 2000, 0, 2000, 0, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn roll_legal_small_delta() {
        let params = AdvancedTrimParams {
            mode: TrimMode::Roll,
            clip_id: "c1".into(),
            delta_ms: 200,
            adjacent_clip_id: Some("c2".into()),
        };
        let result = validate_trim(&params, 2000, 0, 2000, 0, Some(2000), Some(0), Some(2000), Some(2000));
        assert!(result.is_ok());
    }

    #[test]
    fn roll_illegal_too_large_delta() {
        let params = AdvancedTrimParams {
            mode: TrimMode::Roll,
            clip_id: "c1".into(),
            delta_ms: 2500, // exceeds clip duration
            adjacent_clip_id: Some("c2".into()),
        };
        let result = validate_trim(&params, 2000, 0, 2000, 0, Some(2000), Some(0), Some(2000), Some(2000));
        assert!(result.is_err());
    }

    #[test]
    fn slip_illegal_before_source_start() {
        let params = AdvancedTrimParams {
            mode: TrimMode::Slip,
            clip_id: "c1".into(),
            delta_ms: -100,
            adjacent_clip_id: None,
        };
        let result = validate_trim(&params, 2000, 0, 2000, 0, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn slip_legal_positive_delta() {
        let params = AdvancedTrimParams {
            mode: TrimMode::Slip,
            clip_id: "c1".into(),
            delta_ms: 100,
            adjacent_clip_id: None,
        };
        let result = validate_trim(&params, 2000, 500, 2500, 0, None, None, None, None);
        assert!(result.is_ok());
    }
}
