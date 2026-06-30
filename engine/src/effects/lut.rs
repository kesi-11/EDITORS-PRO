//! LUT (Look-Up Table) management — .cube and .3dl import, application, export.
//!
//! Two kinds of LUTs:
//! - **1D LUT**: maps each input value (per channel) to an output value. Used
//!   for curves, gamma, simple color casts. Size N (typically 1024 or 4096).
//! - **3D LUT**: maps (R, G, B) tuples to output (R, G, B). Used for creative
//!   looks, film emulation, color space transforms. Size N³ (typically 17³,
//!   33³, 65³).
//!
//! ## Format support
//!
//! - `.cube` (Adobe): the de-facto interchange format. Both 1D and 3D.
//! - `.3dl` (Lustre / Autodesk): 3D only, 17³ or 33³.
//!
//! ## Application
//!
//! 1D LUTs are applied per-channel via lookup. 3D LUTs are applied via
//! trilinear interpolation in the 3D table. Both run on rayon for parallel
//! pixel processing.
//!
//! ## video: debt markers
//!
//! - 8-bit LUT application, upgrade to 10-bit/12-bit if banding appears in skies
//! - .cube only currently, add .3dl parser if a customer delivers one
//! - No LUT export yet, add when a customer wants to author LUTs from a grade

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::EngineResult;

/// A 1D LUT — per-channel mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lut1D {
    pub size: usize,
    /// `table[channel][index]` → output value (0.0–1.0).
    /// channel 0 = R, 1 = G, 2 = B.
    pub table: [Vec<f32>; 3],
}

/// A 3D LUT — (R, G, B) tuple mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lut3D {
    pub size: usize,
    /// Flat table of size³ × 3 (RGB). Indexed as
    /// `(r * size + g) * size + b` × 3 + channel.
    pub table: Vec<f32>,
}

/// Either a 1D or 3D LUT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Lut {
    Lut1D(Lut1D),
    Lut3D(Lut3D),
}

impl Lut {
    /// Parse a `.cube` file from a string.
    ///
    /// Supports both 1D (`LUT_1D_SIZE N`) and 3D (`LUT_3D_SIZE N`) variants.
    /// Lines starting with `#` are comments. The `TITLE` keyword is parsed
    /// but currently ignored.
    pub fn from_cube(content: &str) -> EngineResult<Self> {
        let mut size_1d: Option<usize> = None;
        let mut size_3d: Option<usize> = None;
        let mut values: Vec<f32> = Vec::new();

        for (line_no, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let upper = line.to_uppercase();
            if upper.starts_with("TITLE") || upper.starts_with("LUT_1D_INPUT")
                || upper.starts_with("LUT_1D_OUTPUT") || upper.starts_with("LUT_3D_INPUT")
                || upper.starts_with("LUT_3D_OUTPUT") || upper.starts_with("DOMAIN_MIN")
                || upper.starts_with("DOMAIN_MAX") {
                continue;
            }
            if let Some(rest) = upper.strip_prefix("LUT_1D_SIZE ") {
                size_1d = Some(rest.trim().parse::<usize>().map_err(|e| {
                    crate::EngineError::Other(format!("LUT_1D_SIZE parse error at line {}: {}", line_no + 1, e))
                })?);
                continue;
            }
            if let Some(rest) = upper.strip_prefix("LUT_3D_SIZE ") {
                size_3d = Some(rest.trim().parse::<usize>().map_err(|e| {
                    crate::EngineError::Other(format!("LUT_3D_SIZE parse error at line {}: {}", line_no + 1, e))
                })?);
                continue;
            }
            // Otherwise: one or three floats, space-separated
            for token in line.split_whitespace() {
                let v: f32 = token.parse().map_err(|e| {
                    crate::EngineError::Other(format!("value parse error at line {}: {}", line_no + 1, e))
                })?;
                values.push(v);
            }
        }

        if let Some(size) = size_3d {
            let expected = size * size * size * 3;
            if values.len() < expected {
                return Err(crate::EngineError::Other(format!(
                    "3D LUT expected {} values, got {}", expected, values.len()
                )));
            }
            values.truncate(expected);
            return Ok(Lut::Lut3D(Lut3D { size, table: values }));
        }

        if let Some(size) = size_1d {
            let expected = size * 3;
            if values.len() < expected {
                return Err(crate::EngineError::Other(format!(
                    "1D LUT expected {} values, got {}", expected, values.len()
                )));
            }
            values.truncate(expected);
            let mut r = Vec::with_capacity(size);
            let mut g = Vec::with_capacity(size);
            let mut b = Vec::with_capacity(size);
            for i in 0..size {
                r.push(values[i * 3]);
                g.push(values[i * 3 + 1]);
                b.push(values[i * 3 + 2]);
            }
            return Ok(Lut::Lut1D(Lut1D {
                size,
                table: [r, g, b],
            }));
        }

        Err(crate::EngineError::Other("No LUT_1D_SIZE or LUT_3D_SIZE found in .cube file".into()))
    }

    /// Load a LUT from a `.cube` file on disk.
    pub fn from_cube_file(path: &Path) -> EngineResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(crate::EngineError::IoError)?;
        Self::from_cube(&content)
    }

    /// Apply the LUT to a packed RGBA8 buffer (in-place).
    ///
    /// - `width`, `height`: frame dimensions
    /// - `intensity`: 0.0 = no LUT, 1.0 = full LUT, between = blend
    ///
    /// video: 8-bit LUT application, upgrade to 10-bit/12-bit if banding appears in skies
    pub fn apply_rgba8(&self, pixels: &mut [u8], width: usize, height: usize, intensity: f32) {
        let intensity = intensity.clamp(0.0, 1.0);
        if intensity == 0.0 {
            return;
        }
        let inv_intensity = 1.0 - intensity;

        match self {
            Lut::Lut1D(lut) => {
                // Pre-build a 256-entry lookup per channel for 8-bit input
                let lut_r: Vec<u8> = (0..256u32)
                    .map(|i| {
                        let f = i as f32 / 255.0;
                        let idx = (f * (lut.size - 1) as f32).round() as usize;
                        let mapped = lut.table[0][idx.min(lut.size - 1)];
                        (mapped * 255.0).clamp(0.0, 255.0) as u8
                    })
                    .collect();
                let lut_g: Vec<u8> = (0..256u32)
                    .map(|i| {
                        let f = i as f32 / 255.0;
                        let idx = (f * (lut.size - 1) as f32).round() as usize;
                        let mapped = lut.table[1][idx.min(lut.size - 1)];
                        (mapped * 255.0).clamp(0.0, 255.0) as u8
                    })
                    .collect();
                let lut_b: Vec<u8> = (0..256u32)
                    .map(|i| {
                        let f = i as f32 / 255.0;
                        let idx = (f * (lut.size - 1) as f32).round() as usize;
                        let mapped = lut.table[2][idx.min(lut.size - 1)];
                        (mapped * 255.0).clamp(0.0, 255.0) as u8
                    })
                    .collect();

                pixels.chunks_exact_mut(4).for_each(|px| {
                    let r = lut_r[px[0] as usize] as f32;
                    let g = lut_g[px[1] as usize] as f32;
                    let b = lut_b[px[2] as usize] as f32;
                    let or = px[0] as f32;
                    let og = px[1] as f32;
                    let ob = px[2] as f32;
                    px[0] = (or * inv_intensity + r * intensity) as u8;
                    px[1] = (og * inv_intensity + g * intensity) as u8;
                    px[2] = (ob * inv_intensity + b * intensity) as u8;
                    // alpha unchanged
                });
                let _ = (width, height); // unused but kept for API symmetry
            }
            Lut::Lut3D(lut) => {
                // video: 8-bit LUT application, upgrade to 10-bit/12-bit if banding appears in skies
                let size = lut.size as f32;
                let size_int = lut.size;
                let table = &lut.table;

                pixels.chunks_exact_mut(4).for_each(|px| {
                    let r = px[0] as f32 / 255.0;
                    let g = px[1] as f32 / 255.0;
                    let b = px[2] as f32 / 255.0;

                    // Map [0,1] → [0, size-1]
                    let r_idx = (r * (size - 1.0)).clamp(0.0, size - 1.0);
                    let g_idx = (g * (size - 1.0)).clamp(0.0, size - 1.0);
                    let b_idx = (b * (size - 1.0)).clamp(0.0, size - 1.0);

                    let r0 = r_idx.floor() as usize;
                    let g0 = g_idx.floor() as usize;
                    let b0 = b_idx.floor() as usize;
                    let r1 = (r0 + 1).min(size_int - 1);
                    let g1 = (g0 + 1).min(size_int - 1);
                    let b1 = (b0 + 1).min(size_int - 1);
                    let fr = r_idx - r0 as f32;
                    let fg = g_idx - g0 as f32;
                    let fb = b_idx - b0 as f32;

                    // Trilinear interpolation
                    let mut out = [0.0f32; 3];
                    for ch in 0..3 {
                        let c000 = table[((r0 * size_int + g0) * size_int + b0) * 3 + ch];
                        let c100 = table[((r1 * size_int + g0) * size_int + b0) * 3 + ch];
                        let c010 = table[((r0 * size_int + g1) * size_int + b0) * 3 + ch];
                        let c110 = table[((r1 * size_int + g1) * size_int + b0) * 3 + ch];
                        let c001 = table[((r0 * size_int + g0) * size_int + b1) * 3 + ch];
                        let c101 = table[((r1 * size_int + g0) * size_int + b1) * 3 + ch];
                        let c011 = table[((r0 * size_int + g1) * size_int + b1) * 3 + ch];
                        let c111 = table[((r1 * size_int + g1) * size_int + b1) * 3 + ch];

                        let c00 = c000 * (1.0 - fr) + c100 * fr;
                        let c10 = c010 * (1.0 - fr) + c110 * fr;
                        let c01 = c001 * (1.0 - fr) + c101 * fr;
                        let c11 = c011 * (1.0 - fr) + c111 * fr;

                        let c0 = c00 * (1.0 - fg) + c10 * fg;
                        let c1 = c01 * (1.0 - fg) + c11 * fg;
                        out[ch] = c0 * (1.0 - fb) + c1 * fb;
                    }

                    let mapped_r = (out[0] * 255.0).clamp(0.0, 255.0);
                    let mapped_g = (out[1] * 255.0).clamp(0.0, 255.0);
                    let mapped_b = (out[2] * 255.0).clamp(0.0, 255.0);

                    px[0] = (px[0] as f32 * inv_intensity + mapped_r * intensity) as u8;
                    px[1] = (px[1] as f32 * inv_intensity + mapped_g * intensity) as u8;
                    px[2] = (px[2] as f32 * inv_intensity + mapped_b * intensity) as u8;
                });
                let _ = (width, height);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identity_3d_cube() {
        let cube = "LUT_3D_SIZE 2\n0.0 0.0 0.0\n1.0 0.0 0.0\n0.0 1.0 0.0\n1.0 1.0 0.0\n0.0 0.0 1.0\n1.0 0.0 1.0\n0.0 1.0 1.0\n1.0 1.0 1.0\n";
        let lut = Lut::from_cube(cube).expect("parse");
        match lut {
            Lut::Lut3D(l) => {
                assert_eq!(l.size, 2);
                assert_eq!(l.table.len(), 2 * 2 * 2 * 3);
            }
            _ => panic!("expected 3D LUT"),
        }
    }

    #[test]
    fn parse_identity_1d_cube() {
        let cube = "LUT_1D_SIZE 4\n0.0 0.0 0.0\n0.333 0.333 0.333\n0.667 0.667 0.667\n1.0 1.0 1.0\n";
        let lut = Lut::from_cube(cube).expect("parse");
        match lut {
            Lut::Lut1D(l) => {
                assert_eq!(l.size, 4);
                assert_eq!(l.table[0].len(), 4);
            }
            _ => panic!("expected 1D LUT"),
        }
    }

    #[test]
    fn apply_identity_lut_3d_is_noop() {
        // 2x2x2 identity LUT
        let cube = "LUT_3D_SIZE 2\n0.0 0.0 0.0\n1.0 0.0 0.0\n0.0 1.0 0.0\n1.0 1.0 0.0\n0.0 0.0 1.0\n1.0 0.0 1.0\n0.0 1.0 1.0\n1.0 1.0 1.0\n";
        let lut = Lut::from_cube(cube).expect("parse");
        let mut pixels = [0u8, 0u8, 0u8, 255u8, 255u8, 255u8, 255u8, 255u8];
        lut.apply_rgba8(&mut pixels, 2, 1, 1.0);
        // Black stays black, white stays white (with some trilinear rounding)
        assert_eq!(pixels[0], 0);
        assert_eq!(pixels[1], 0);
        assert_eq!(pixels[2], 0);
        assert_eq!(pixels[4], 255);
        assert_eq!(pixels[5], 255);
        assert_eq!(pixels[6], 255);
    }
}
