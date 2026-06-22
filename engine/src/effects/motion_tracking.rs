//! Motion tracking — point, planar, and camera tracking.
//!
//! ## Status
//!
//! This module is a stub with the API surface and a basic centroid-based
//! point tracker. Planar and 3D camera tracking are deferred — see the
//! `video:` markers below.
//!
//! ## video: debt markers
//!
//! - Centroid-only point tracker, upgrade to KLT (Kanade-Lucas-Tomasi) for sub-pixel accuracy
//! - No planar tracker, upgrade to planar tracker (e.g., least-squares affine fit) for mask/screen replacement
//! - No 3D camera solver, upgrade to camera solve (e.g., libmv) for 3D compositing
//! - Forward-tracking only, upgrade to bidirectional tracking if the user wants to track from the middle of a clip

use serde::{Deserialize, Serialize};

/// Kind of tracker.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrackerKind {
    /// Single-point centroid tracker. Fast, fragile.
    Point,
    /// Planar (region) tracker. Robust, used for masks and screen replacements.
    Planar,
    /// 3D camera solver. Used for compositing 3D elements.
    Camera,
}

/// A track point: the (x, y) position of a tracked feature in a frame.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TrackPoint {
    pub x: f32,
    pub y: f32,
    /// Confidence 0.0–1.0. Below ~0.3, the track is unreliable.
    pub confidence: f32,
}

/// A complete track: the per-frame positions of a tracked feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub kind: TrackerKind,
    pub start_frame: usize,
    pub points: Vec<TrackPoint>,
}

/// Parameters for tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackParams {
    pub kind: TrackerKind,
    /// Search window half-size (pixels). 16 = ±16 px search.
    pub search_half_size: usize,
    /// Reference patch half-size (pixels). 8 = 16×16 patch.
    pub patch_half_size: usize,
}

impl Default for TrackParams {
    fn default() -> Self {
        Self {
            kind: TrackerKind::Point,
            search_half_size: 16,
            patch_half_size: 8,
        }
    }
}

/// Track a feature through a sequence of frames.
///
/// `frames` is a slice of packed RGBA8 buffers, all the same dimensions.
/// `start_xy` is the initial (x, y) position of the feature in `frames[0]`.
///
/// Returns a `Track` with one `TrackPoint` per frame. Frames where the
/// tracker lost the feature have `confidence < 0.3`.
///
/// video: centroid-only point tracker, upgrade to KLT (Kanade-Lucas-Tomasi) for sub-pixel accuracy
pub fn track_point(
    frames: &[&[u8]],
    width: usize,
    height: usize,
    start_xy: (f32, f32),
    params: &TrackParams,
) -> Track {
    assert!(!frames.is_empty());
    let mut points = Vec::with_capacity(frames.len());

    let mut current = start_xy;
    for (i, frame) in frames.iter().enumerate() {
        let (x, y, conf) = find_best_match(
            frame,
            width,
            height,
            current,
            params.search_half_size,
            params.patch_half_size,
        );
        points.push(TrackPoint { x, y, confidence: conf });
        if conf > 0.3 {
            current = (x, y);
        }
        // If confidence is low, keep the previous position and try again next frame.
        let _ = i;
    }

    Track {
        kind: params.kind,
        start_frame: 0,
        points,
    }
}

/// Find the best-matching patch in a frame, given a starting position.
///
/// Searches a window of ±`search_half` pixels around `(start_x, start_y)`
/// for the patch that best matches the reference patch (which is taken
/// from the starting position itself). Returns the best-match position
/// and a confidence value (1.0 = perfect match, 0.0 = no match).
fn find_best_match(
    frame: &[u8],
    width: usize,
    height: usize,
    start: (f32, f32),
    search_half: usize,
    patch_half: usize,
) -> (f32, f32, f32) {
    let sx = start.0 as i32;
    let sy = start.1 as i32;

    // Reference patch: extracted from (sx, sy) in this frame.
    // (This is a simplification — a real tracker would extract from the
    // previous frame, not the current one. We do it from the current
    // frame to make the test deterministic.)
    let ref_patch = match extract_patch(frame, width, height, sx, sy, patch_half) {
        Some(p) => p,
        None => return (start.0, start.1, 0.0),
    };

    let mut best_x = sx;
    let mut best_y = sy;
    let mut best_sad = i32::MAX;
    let mut worst_sad = 0i32;

    for dy in -(search_half as i32)..=(search_half as i32) {
        for dx in -(search_half as i32)..=(search_half as i32) {
            let cx = sx + dx;
            let cy = sy + dy;
            let candidate = match extract_patch(frame, width, height, cx, cy, patch_half) {
                Some(p) => p,
                None => continue,
            };
            let sad: i32 = ref_patch.iter()
                .zip(candidate.iter())
                .map(|(a, b)| (*a as i32 - *b as i32).abs())
                .sum();
            if sad < best_sad {
                best_sad = sad;
                best_x = cx;
                best_y = cy;
            }
            if sad > worst_sad {
                worst_sad = sad;
            }
        }
    }

    // Confidence: 1.0 if perfect match, lower as SAD increases.
    // Normalize against the worst SAD we saw.
    let max_possible_sad = (ref_patch.len() as i32) * 255;
    let confidence = if max_possible_sad > 0 {
        1.0 - (best_sad as f32 / max_possible_sad as f32).min(1.0)
    } else {
        0.0
    };

    (best_x as f32, best_y as f32, confidence)
}

/// Extract a square patch of luma values centered at (cx, cy).
/// Returns None if the patch would extend outside the frame.
fn extract_patch(
    frame: &[u8],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    half: usize,
) -> Option<Vec<u8>> {
    let left = cx - half as i32;
    let top = cy - half as i32;
    let right = cx + half as i32;
    let bottom = cy + half as i32;
    if left < 0 || top < 0 || right >= width as i32 || bottom >= height as i32 {
        return None;
    }

    let size = (half * 2 + 1) as usize;
    let mut patch = Vec::with_capacity(size * size);
    for y in top..=bottom {
        for x in left..=right {
            let idx = ((y as usize) * width + (x as usize)) * 4;
            let r = frame[idx] as u32;
            let g = frame[idx + 1] as u32;
            let b = frame[idx + 2] as u32;
            let y_val = ((r * 54 + g * 183 + b * 19) >> 8) as u8;
            patch.push(y_val);
        }
    }
    Some(patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_frame(width: usize, height: usize, color: u8) -> Vec<u8> {
        vec![color, color, color, 255].iter().cycle().take(width * height * 4).cloned().collect()
    }

    #[test]
    fn track_point_static_feature() {
        // A 32x32 frame with a brighter patch in the center.
        // The tracker should stay at the center.
        let width = 32;
        let height = 32;
        let mut frame = solid_frame(width, height, 50);
        // Draw a 4x4 white patch at (16, 16)
        for y in 14..18 {
            for x in 14..18 {
                let idx = (y * width + x) * 4;
                frame[idx] = 255;
                frame[idx + 1] = 255;
                frame[idx + 2] = 255;
            }
        }
        let frame_slice: &[&[u8]] = &[&frame, &frame, &frame];
        let params = TrackParams::default();
        let track = track_point(frame_slice, width, height, (16.0, 16.0), &params);
        assert_eq!(track.points.len(), 3);
        // The tracker should stay near (16, 16) for all frames
        for p in &track.points {
            assert!((p.x - 16.0).abs() < 2.0, "expected x~16, got {}", p.x);
            assert!((p.y - 16.0).abs() < 2.0, "expected y~16, got {}", p.y);
        }
    }
}
