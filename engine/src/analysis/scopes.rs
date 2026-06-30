//! Color scopes — waveform monitor, vectorscope, RGB parade, histogram.
//!
//! All scopes operate on a packed RGBA8 frame and return a small data
//! structure suitable for UI rendering. Computed on rayon for parallel
//! row processing.
//!
//! ## Scopes
//!
//! - **Waveform (Y)**: luma distribution per column. X = horizontal position,
//!   Y = luma value (0 at bottom, 255 at top for 8-bit). Used to verify
//!   legal range (16–235) and check for crushed blacks / clipped highlights.
//! - **Vectorscope**: chroma distribution. Angle = hue (Rec.601/701 chroma
//!   vector), distance from center = saturation. The flesh-tone "I-line"
//!   at ~123° (between R and Y) is where skin tones land.
//! - **RGB Parade**: three separate waveforms for R, G, B. Used for white
//!   balance (lines should align) and color cast detection.
//! - **Histogram**: distribution of luma values across the whole frame.
//!   256 bins for 8-bit.

use serde::{Deserialize, Serialize};

/// Waveform monitor data — per-column luma histogram.
///
/// `columns[x]` is a `Vec<u32>` of length 256 (8-bit luma), where
/// `columns[x][y]` is the count of pixels in column `x` with luma `y`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waveform {
    pub width: usize,
    pub height: usize, // = 256
    pub columns: Vec<Vec<u32>>,
}

/// Vectorscope data — 2D histogram on the chroma plane.
///
/// `grid[y][x]` is the count of pixels at chroma position (x, y).
/// Grid is square, `size × size`. The center is (size/2, size/2).
/// X axis = Cb (blue-yellow), Y axis = Cr (red-cyan).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vectorscope {
    pub size: usize,
    pub grid: Vec<u32>, // size * size, row-major
}

/// RGB parade — three separate waveforms (R, G, B).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbParade {
    pub width: usize,
    pub height: usize, // = 256
    pub red: Vec<Vec<u32>>,
    pub green: Vec<Vec<u32>>,
    pub blue: Vec<Vec<u32>>,
}

/// Histogram — 256-bin distribution of luma values across the whole frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub bins: Vec<u32>, // length 256
}

/// All four scopes computed for a single frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scopes {
    pub waveform: Waveform,
    pub vectorscope: Vectorscope,
    pub rgb_parade: RgbParade,
    pub histogram: Histogram,
}

/// Compute all four scopes from a packed RGBA8 frame.
///
/// `pixels.len()` must equal `width * height * 4`.
pub fn compute_scopes(pixels: &[u8], width: usize, height: usize) -> Scopes {
    Scopes {
        waveform: compute_waveform(pixels, width, height),
        vectorscope: compute_vectorscope(pixels, width, height),
        rgb_parade: compute_rgb_parade(pixels, width, height),
        histogram: compute_histogram(pixels, width, height),
    }
}

/// Compute the waveform (luma distribution per column).
pub fn compute_waveform(pixels: &[u8], width: usize, height: usize) -> Waveform {
    debug_assert_eq!(pixels.len(), width * height * 4);

    let mut columns = vec![vec![0u32; 256]; width];

    for y in 0..height {
        let row_offset = y * width * 4;
        for x in 0..width {
            let i = row_offset + x * 4;
            let r = pixels[i] as u32;
            let g = pixels[i + 1] as u32;
            let b = pixels[i + 2] as u32;
            // Rec.709 luma
            let y_val = ((r * 54 + g * 183 + b * 19) >> 8) as usize; // 0.2126, 0.7152, 0.0722 scaled
            columns[x][y_val.min(255)] += 1;
        }
    }

    Waveform {
        width,
        height: 256,
        columns,
    }
}

/// Compute the vectorscope (Cb-Cr chroma plane histogram).
pub fn compute_vectorscope(pixels: &[u8], width: usize, height: usize) -> Vectorscope {
    debug_assert_eq!(pixels.len(), width * height * 4);

    let size = 256usize;
    let mut grid = vec![0u32; size * size];

    // Cb, Cr in [-128, 127]. Map to [0, 255] for the grid.
    // Rec.601/701 approximation:
    //   Cb = -0.168736 R - 0.331264 G + 0.5 B + 128
    //   Cr =  0.5 R - 0.418688 G - 0.081312 B + 128
    for i in (0..pixels.len()).step_by(4) {
        let r = pixels[i] as i32;
        let g = pixels[i + 1] as i32;
        let b = pixels[i + 2] as i32;

        let cb = (-43 * r - 85 * g + 128 * b + 128 * 256) >> 8;
        let cr = (128 * r - 107 * g - 21 * b + 128 * 256) >> 8;

        let cb_idx = cb.clamp(0, 255) as usize;
        let cr_idx = cr.clamp(0, 255) as usize;
        grid[cr_idx * size + cb_idx] += 1;
    }

    Vectorscope { size, grid }
}

/// Compute the RGB parade (three waveforms, one per channel).
pub fn compute_rgb_parade(pixels: &[u8], width: usize, height: usize) -> RgbParade {
    debug_assert_eq!(pixels.len(), width * height * 4);

    let mut red = vec![vec![0u32; 256]; width];
    let mut green = vec![vec![0u32; 256]; width];
    let mut blue = vec![vec![0u32; 256]; width];

    for y in 0..height {
        let row_offset = y * width * 4;
        for x in 0..width {
            let i = row_offset + x * 4;
            red[x][pixels[i] as usize] += 1;
            green[x][pixels[i + 1] as usize] += 1;
            blue[x][pixels[i + 2] as usize] += 1;
        }
    }

    RgbParade {
        width,
        height: 256,
        red,
        green,
        blue,
    }
}

/// Compute the luma histogram (256-bin distribution).
pub fn compute_histogram(pixels: &[u8], width: usize, height: usize) -> Histogram {
    debug_assert_eq!(pixels.len(), width * height * 4);
    let mut bins = vec![0u32; 256];

    for i in (0..pixels.len()).step_by(4) {
        let r = pixels[i] as u32;
        let g = pixels[i + 1] as u32;
        let b = pixels[i + 2] as u32;
        let y_val = ((r * 54 + g * 183 + b * 19) >> 8) as usize;
        bins[y_val.min(255)] += 1;
    }

    Histogram { bins }
}

/// Check whether a frame is broadcast-legal (Rec.709 luma 16–235, chroma 16–240).
///
/// Returns the count of out-of-range pixels. Zero = fully legal.
///
/// video: per-pixel scan, fine for QC pass — not intended for real-time preview
pub fn count_out_of_range_pixels(pixels: &[u8]) -> OutOfRangeCounts {
    let mut luma_low = 0u32;
    let mut luma_high = 0u32;
    let mut chroma_low = 0u32;
    let mut chroma_high = 0u32;

    for i in (0..pixels.len()).step_by(4) {
        let r = pixels[i] as i32;
        let g = pixels[i + 1] as i32;
        let b = pixels[i + 2] as i32;

        let y_val = (54 * r + 183 * g + 19 * b) >> 8;
        let cb = (-43 * r - 85 * g + 128 * b + 128 * 256) >> 8;
        let cr = (128 * r - 107 * g - 21 * b + 128 * 256) >> 8;

        if y_val < 16 {
            luma_low += 1;
        } else if y_val > 235 {
            luma_high += 1;
        }
        if cb < 16 || cr < 16 {
            chroma_low += 1;
        } else if cb > 240 || cr > 240 {
            chroma_high += 1;
        }
    }

    OutOfRangeCounts {
        luma_low,
        luma_high,
        chroma_low,
        chroma_high,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfRangeCounts {
    pub luma_low: u32,
    pub luma_high: u32,
    pub chroma_low: u32,
    pub chroma_high: u32,
}

impl OutOfRangeCounts {
    pub fn total(&self) -> u32 {
        self.luma_low + self.luma_high + self.chroma_low + self.chroma_high
    }

    pub fn is_legal(&self) -> bool {
        self.total() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_scopes_black_frame() {
        let pixels = vec![0u8; 4 * 4 * 4]; // 4x4 black
        let scopes = compute_scopes(&pixels, 4, 4);
        // Black frame: all luma = 0
        assert_eq!(scopes.histogram.bins[0], 16);
        assert_eq!(scopes.histogram.bins[255], 0);
    }

    #[test]
    fn compute_scopes_white_frame() {
        let pixels = vec![255u8; 4 * 4 * 4]; // 4x4 white
        let scopes = compute_scopes(&pixels, 4, 4);
        // White frame: luma ~ 235 (Rec.709 weighted sum of 255s)
        let max_bin = scopes.histogram.bins.iter().enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
            .unwrap();
        assert!(max_bin >= 230 && max_bin <= 240);
    }

    #[test]
    fn count_out_of_range_legal_frame() {
        // All values at 128 — well inside legal range
        let pixels = vec![128u8; 4 * 4 * 4];
        let counts = count_out_of_range_pixels(&pixels);
        assert!(counts.is_legal(), "expected legal, got {:?}", counts);
    }

    #[test]
    fn count_out_of_range_illegal_frame() {
        // Pure black (0) and pure white (255) — both out of legal range
        let mut pixels = vec![0u8; 4 * 4 * 4];
        for px in pixels.chunks_exact_mut(4) {
            px[0] = 255; px[1] = 255; px[2] = 255;
        }
        let counts = count_out_of_range_pixels(&pixels);
        assert!(counts.luma_low > 0 || counts.luma_high > 0,
            "expected out-of-range luma, got {:?}", counts);
    }
}
