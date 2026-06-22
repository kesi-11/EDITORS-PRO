//! Video stabilization — 2D deshake via block-matching motion estimation.
//!
//! ## Algorithm
//!
//! 1. Pick a reference block in the center of the previous frame (32×32).
//! 2. Search a window in the current frame (±16 px) for the best match
//!    using sum-of-absolute-differences (SAD).
//! 3. The (dx, dy) of the best match is the per-frame motion vector.
//! 4. Smooth the motion-vector time series with a Gaussian window.
//! 5. Apply the inverse of the smoothed motion to each frame (translation
//!    only — no rotation, no scale).
//! 6. Crop the frame to hide edge artifacts from the translation.
//!
//! ## video: debt markers
//!
//! - 2D translation only, upgrade to rotation+scale+perspective if motion is complex
//! - per-frame block matching, upgrade to multi-pass with pyramidal refinement if motion is fast
//! - block matching on luma only, upgrade to RGB if color shifts affect tracking
//! - 3D camera solve is the upgrade path if motion is parallax-heavy
//!   (foreground/background moving differently — 2D will produce jelly artifacts)

use serde::{Deserialize, Serialize};

/// Stabilization parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilizationParams {
    /// Smoothing strength (0.0 = no stabilization, 1.0 = maximum).
    /// Higher = smoother but more crop and more "swim."
    pub smoothing: f32,
    /// Crop ratio (0.0 = no crop, 0.2 = 20% crop).
    /// Must be ≥ the maximum accumulated motion or edges will show artifacts.
    pub crop: f32,
    /// Reference block size (square, in pixels). 32 is a good default.
    pub block_size: usize,
    /// Search range (±pixels). 16 is a good default for handheld shake.
    pub search_range: usize,
}

impl Default for StabilizationParams {
    fn default() -> Self {
        Self {
            smoothing: 0.5,
            crop: 0.1,
            block_size: 32,
            search_range: 16,
        }
    }
}

/// A per-frame motion vector (dx, dy) in pixels.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MotionVector {
    pub dx: f32,
    pub dy: f32,
}

/// Motion track — the per-frame motion vectors for a clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionTrack {
    pub frames: Vec<MotionVector>,
}

impl MotionTrack {
    /// Smooth the motion track with a Gaussian window.
    ///
    /// `strength` controls the window size: higher = more smoothing.
    /// Returns the smoothed track (same length as input).
    pub fn smooth(&self, strength: f32) -> MotionTrack {
        if self.frames.is_empty() {
            return MotionTrack { frames: vec![] };
        }
        let strength = strength.clamp(0.0, 1.0);
        if strength == 0.0 {
            return self.clone();
        }
        // Window size scales with strength. 1.0 = 30-frame window, 0.5 = 15, etc.
        let window = ((strength * 30.0).round() as usize).max(1);
        let sigma = (window as f32) / 2.0;
        let two_sigma_sq = 2.0 * sigma * sigma;

        let n = self.frames.len();
        let mut smoothed = vec![MotionVector::default(); n];

        for i in 0..n {
            let mut sum_w = 0.0;
            let mut sum_dx = 0.0;
            let mut sum_dy = 0.0;
            let start = i.saturating_sub(window);
            let end = (i + window + 1).min(n);
            for j in start..end {
                let d = (j as f32) - (i as f32);
                let w = (-(d * d) / two_sigma_sq).exp();
                sum_w += w;
                sum_dx += w * self.frames[j].dx;
                sum_dy += w * self.frames[j].dy;
            }
            smoothed[i] = MotionVector {
                dx: sum_dx / sum_w,
                dy: sum_dy / sum_w,
            };
        }

        MotionTrack { frames: smoothed }
    }

    /// The correction to apply to each frame: the difference between the
    /// raw motion and the smoothed motion. Applying this to each frame
    /// makes the motion match the smoothed (slower) path.
    pub fn corrections(&self, smoothed: &MotionTrack) -> Vec<MotionVector> {
        self.frames
            .iter()
            .zip(smoothed.frames.iter())
            .map(|(raw, sm)| MotionVector {
                dx: sm.dx - raw.dx,
                dy: sm.dy - raw.dy,
            })
            .collect()
    }
}

/// Estimate the per-frame motion vector between two consecutive frames.
///
/// Uses block matching on the luma channel. The reference block is the
/// center `block_size × block_size` region of the previous frame.
///
/// `prev` and `curr` are packed RGBA8 buffers of the same dimensions.
///
/// video: per-frame block matching, upgrade to multi-pass with pyramidal refinement if motion is fast
pub fn estimate_motion(
    prev: &[u8],
    curr: &[u8],
    width: usize,
    height: usize,
    block_size: usize,
    search_range: usize,
) -> MotionVector {
    debug_assert_eq!(prev.len(), width * height * 4);
    debug_assert_eq!(curr.len(), width * height * 4);

    // Reference block: center of previous frame
    let block_x = (width / 2).saturating_sub(block_size / 2);
    let block_y = (height / 2).saturating_sub(block_size / 2);

    let mut best_dx = 0i32;
    let mut best_dy = 0i32;
    let mut best_sad = i32::MAX;

    for dy in -(search_range as i32)..=(search_range as i32) {
        for dx in -(search_range as i32)..=(search_range as i32) {
            let mut sad = 0i32;
            let mut valid = true;
            for by in 0..block_size {
                for bx in 0..block_size {
                    let prev_x = block_x + bx;
                    let prev_y = block_y + by;
                    let curr_x = (prev_x as i32 + dx) as usize;
                    let curr_y = (prev_y as i32 + dy) as usize;
                    if curr_x >= width || curr_y >= height {
                        valid = false;
                        break;
                    }
                    let prev_idx = (prev_y * width + prev_x) * 4;
                    let curr_idx = (curr_y * width + curr_x) * 4;
                    // Luma only — Rec.709 weighted
                    let prev_luma = (prev[prev_idx] as i32 * 54
                        + prev[prev_idx + 1] as i32 * 183
                        + prev[prev_idx + 2] as i32 * 19)
                        >> 8;
                    let curr_luma = (curr[curr_idx] as i32 * 54
                        + curr[curr_idx + 1] as i32 * 183
                        + curr[curr_idx + 2] as i32 * 19)
                        >> 8;
                    sad += (prev_luma - curr_luma).abs();
                }
                if !valid {
                    break;
                }
            }
            if valid && sad < best_sad {
                best_sad = sad;
                best_dx = dx;
                best_dy = dy;
            }
        }
    }

    MotionVector {
        dx: best_dx as f32,
        dy: best_dy as f32,
    }
}

/// Apply a per-frame motion correction (translation) to a packed RGBA8 frame.
///
/// `correction` is the (dx, dy) to apply. The frame is shifted by
/// (-dx, -dy) — pixels that were at (x, y) move to (x - dx, y - dy).
/// Newly exposed edge pixels are filled with black.
///
/// video: 2D translation only, upgrade to rotation+scale+perspective if motion is complex
pub fn apply_correction(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    correction: MotionVector,
) {
    let dx = correction.dx.round() as i32;
    let dy = correction.dy.round() as i32;
    if dx == 0 && dy == 0 {
        return;
    }

    // Make a copy to read from
    let src = pixels.to_vec();
    // Clear destination
    for px in pixels.chunks_exact_mut(4) {
        px[0] = 0; px[1] = 0; px[2] = 0; px[3] = 255;
    }

    for y in 0..height {
        for x in 0..width {
            let sx = x as i32 - dx;
            let sy = y as i32 - dy;
            if sx < 0 || sx >= width as i32 || sy < 0 || sy >= height as i32 {
                continue;
            }
            let src_idx = ((sy as usize) * width + (sx as usize)) * 4;
            let dst_idx = (y * width + x) * 4;
            pixels[dst_idx] = src[src_idx];
            pixels[dst_idx + 1] = src[src_idx + 1];
            pixels[dst_idx + 2] = src[src_idx + 2];
            pixels[dst_idx + 3] = src[src_idx + 3];
        }
    }
}

/// Crop a frame by `crop_ratio` on each side (centered).
/// Returns a new (smaller) packed RGBA8 buffer.
pub fn crop_center(pixels: &[u8], width: usize, height: usize, crop_ratio: f32) -> (Vec<u8>, usize, usize) {
    let crop_ratio = crop_ratio.clamp(0.0, 0.45);
    let new_w = ((width as f32) * (1.0 - crop_ratio)).round() as usize;
    let new_h = ((height as f32) * (1.0 - crop_ratio)).round() as usize;
    let off_x = (width - new_w) / 2;
    let off_y = (height - new_h) / 2;

    let mut out = vec![0u8; new_w * new_h * 4];
    for y in 0..new_h {
        for x in 0..new_w {
            let src_idx = ((off_y + y) * width + (off_x + x)) * 4;
            let dst_idx = (y * new_w + x) * 4;
            out[dst_idx] = pixels[src_idx];
            out[dst_idx + 1] = pixels[src_idx + 1];
            out[dst_idx + 2] = pixels[src_idx + 2];
            out[dst_idx + 3] = pixels[src_idx + 3];
        }
    }
    (out, new_w, new_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_track_smooth_empty() {
        let track = MotionTrack { frames: vec![] };
        let smoothed = track.smooth(0.5);
        assert!(smoothed.frames.is_empty());
    }

    #[test]
    fn motion_track_smooth_no_op_when_zero_strength() {
        let track = MotionTrack {
            frames: vec![
                MotionVector { dx: 1.0, dy: 2.0 },
                MotionVector { dx: 3.0, dy: 4.0 },
            ],
        };
        let smoothed = track.smooth(0.0);
        assert_eq!(smoothed.frames[0].dx, 1.0);
        assert_eq!(smoothed.frames[1].dx, 3.0);
    }

    #[test]
    fn motion_track_smooth_averages() {
        let track = MotionTrack {
            frames: vec![
                MotionVector { dx: 0.0, dy: 0.0 },
                MotionVector { dx: 10.0, dy: 10.0 },
                MotionVector { dx: 0.0, dy: 0.0 },
            ],
        };
        let smoothed = track.smooth(0.5);
        // Middle frame should be pulled toward the average
        assert!(smoothed.frames[1].dx < 10.0);
        assert!(smoothed.frames[1].dx > 0.0);
    }

    #[test]
    fn corrections_are_difference() {
        let raw = MotionTrack {
            frames: vec![MotionVector { dx: 5.0, dy: 0.0 }],
        };
        let smoothed = MotionTrack {
            frames: vec![MotionVector { dx: 2.0, dy: 0.0 }],
        };
        let corrections = raw.corrections(&smoothed);
        assert_eq!(corrections[0].dx, -3.0);
    }

    #[test]
    fn estimate_motion_zero_for_identical_frames() {
        let frame = vec![128u8; 64 * 64 * 4];
        let mv = estimate_motion(&frame, &frame, 64, 64, 16, 8);
        assert_eq!(mv.dx, 0.0);
        assert_eq!(mv.dy, 0.0);
    }

    #[test]
    fn apply_correction_zero_is_noop() {
        let mut frame = vec![100u8; 16 * 16 * 4];
        let orig = frame.clone();
        apply_correction(&mut frame, 16, 16, MotionVector { dx: 0.0, dy: 0.0 });
        assert_eq!(frame, orig);
    }

    #[test]
    fn crop_center_reduces_dimensions() {
        let frame = vec![128u8; 100 * 100 * 4];
        let (out, w, h) = crop_center(&frame, 100, 100, 0.2);
        assert_eq!(w, 80);
        assert_eq!(h, 80);
        assert_eq!(out.len(), 80 * 80 * 4);
    }
}
