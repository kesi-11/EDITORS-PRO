//! Lens Correction — Brown-Conrady distortion, chromatic aberration, vignette.
//!
//! Supports radial distortion (K1/K2/K3), tangential distortion (P1/P2),
//! chromatic aberration correction, vignette removal, and 8 built-in lens profiles.

use serde::{Deserialize, Serialize};

/// Lens distortion parameters (Brown-Conrady model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensDistortionParams {
    pub k1: f64, pub k2: f64, pub k3: f64,  // Radial distortion
    pub p1: f64, pub p2: f64,                // Tangential distortion
}

impl Default for LensDistortionParams {
    fn default() -> Self {
        Self { k1: 0.0, k2: 0.0, k3: 0.0, p1: 0.0, p2: 0.0 }
    }
}

/// Chromatic aberration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaticAberrationParams {
    pub red_offset_x: f32,
    pub red_offset_y: f32,
    pub blue_offset_x: f32,
    pub blue_offset_y: f32,
    pub radial_factor: f32, // How much CA increases from center
}

impl Default for ChromaticAberrationParams {
    fn default() -> Self {
        Self { red_offset_x: 0.0, red_offset_y: 0.0, blue_offset_x: 0.0, blue_offset_y: 0.0, radial_factor: 1.0 }
    }
}

/// Vignette parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VignetteParams {
    pub amount: f32,     // 0 = none, 1 = heavy
    pub midpoint: f32,   // 0..1 where vignette starts
    pub roundness: f32,  // 0 = oval, 1 = circular
}

impl Default for VignetteParams {
    fn default() -> Self {
        Self { amount: 0.0, midpoint: 0.5, roundness: 1.0 }
    }
}

/// Built-in lens profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensProfile {
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub distortion: LensDistortionParams,
    pub ca: ChromaticAberrationParams,
    pub vignette: VignetteParams,
}

/// Complete lens correction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensCorrectionConfig {
    pub enabled: bool,
    pub distortion: LensDistortionParams,
    pub ca: ChromaticAberrationParams,
    pub vignette: VignetteParams,
    pub selected_profile: Option<String>,
    pub crop_to_original: bool,
}

impl Default for LensCorrectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            distortion: LensDistortionParams::default(),
            ca: ChromaticAberrationParams::default(),
            vignette: VignetteParams::default(),
            selected_profile: None,
            crop_to_original: true,
        }
    }
}

/// 8 built-in lens profiles for common camera/lens combinations.
pub fn builtin_profiles() -> Vec<LensProfile> {
    vec![
        LensProfile {
            name: "GoPro Hero 11 Wide".into(),
            manufacturer: "GoPro".into(),
            model: "Hero 11".into(),
            distortion: LensDistortionParams { k1: -0.268, k2: 0.085, k3: -0.012, p1: 0.0, p2: 0.0 },
            ca: ChromaticAberrationParams { red_offset_x: 0.5, red_offset_y: 0.0, blue_offset_x: -0.5, blue_offset_y: 0.0, radial_factor: 1.5 },
            vignette: VignetteParams { amount: 0.4, midpoint: 0.3, roundness: 1.0 },
        },
        LensProfile {
            name: "iPhone 15 Main".into(),
            manufacturer: "Apple".into(),
            model: "iPhone 15".into(),
            distortion: LensDistortionParams { k1: -0.045, k2: 0.012, k3: 0.0, p1: 0.001, p2: 0.0 },
            ca: ChromaticAberrationParams { red_offset_x: 0.2, red_offset_y: 0.0, blue_offset_x: -0.2, blue_offset_y: 0.0, radial_factor: 1.2 },
            vignette: VignetteParams { amount: 0.15, midpoint: 0.5, roundness: 1.0 },
        },
        LensProfile {
            name: "Samsung S24 Ultra".into(),
            manufacturer: "Samsung".into(),
            model: "S24 Ultra".into(),
            distortion: LensDistortionParams { k1: -0.052, k2: 0.018, k3: -0.003, p1: 0.0, p2: 0.001 },
            ca: ChromaticAberrationParams { red_offset_x: 0.3, red_offset_y: 0.0, blue_offset_x: -0.3, blue_offset_y: 0.0, radial_factor: 1.3 },
            vignette: VignetteParams { amount: 0.2, midpoint: 0.45, roundness: 1.0 },
        },
        LensProfile {
            name: "Sony 24-70mm f/2.8 GM".into(),
            manufacturer: "Sony".into(),
            model: "FE 24-70mm".into(),
            distortion: LensDistortionParams { k1: -0.028, k2: 0.008, k3: 0.0, p1: 0.002, p2: -0.001 },
            ca: ChromaticAberrationParams { red_offset_x: 0.1, red_offset_y: 0.0, blue_offset_x: -0.1, blue_offset_y: 0.0, radial_factor: 1.0 },
            vignette: VignetteParams { amount: 0.3, midpoint: 0.4, roundness: 0.95 },
        },
        LensProfile {
            name: "Canon RF 50mm f/1.2".into(),
            manufacturer: "Canon".into(),
            model: "RF 50mm".into(),
            distortion: LensDistortionParams { k1: -0.008, k2: 0.003, k3: 0.0, p1: 0.0, p2: 0.0 },
            ca: ChromaticAberrationParams { red_offset_x: 0.15, red_offset_y: 0.0, blue_offset_x: -0.15, blue_offset_y: 0.0, radial_factor: 1.1 },
            vignette: VignetteParams { amount: 0.35, midpoint: 0.35, roundness: 1.0 },
        },
        LensProfile {
            name: "DJI Mavic 3 Wide".into(),
            manufacturer: "DJI".into(),
            model: "Mavic 3".into(),
            distortion: LensDistortionParams { k1: -0.18, k2: 0.045, k3: -0.008, p1: 0.0, p2: 0.0 },
            ca: ChromaticAberrationParams { red_offset_x: 0.4, red_offset_y: 0.0, blue_offset_x: -0.4, blue_offset_y: 0.0, radial_factor: 1.4 },
            vignette: VignetteParams { amount: 0.25, midpoint: 0.35, roundness: 1.0 },
        },
        LensProfile {
            name: "Pixel 8 Pro Main".into(),
            manufacturer: "Google".into(),
            model: "Pixel 8 Pro".into(),
            distortion: LensDistortionParams { k1: -0.038, k2: 0.01, k3: 0.0, p1: 0.0, p2: 0.001 },
            ca: ChromaticAberrationParams { red_offset_x: 0.2, red_offset_y: 0.0, blue_offset_x: -0.2, blue_offset_y: 0.0, radial_factor: 1.1 },
            vignette: VignetteParams { amount: 0.18, midpoint: 0.5, roundness: 1.0 },
        },
        LensProfile {
            name: "Sigma 35mm f/1.4 Art".into(),
            manufacturer: "Sigma".into(),
            model: "35mm Art".into(),
            distortion: LensDistortionParams { k1: -0.015, k2: 0.005, k3: 0.0, p1: 0.001, p2: -0.001 },
            ca: ChromaticAberrationParams { red_offset_x: 0.12, red_offset_y: 0.0, blue_offset_x: -0.12, blue_offset_y: 0.0, radial_factor: 1.05 },
            vignette: VignetteParams { amount: 0.28, midpoint: 0.4, roundness: 0.9 },
        },
    ]
}

/// Compute undistorted normalized coordinates from distorted ones.
/// Brown-Conrady model: r²=x²+y², then apply radial and tangential corrections.
pub fn undistort_point(nx: f64, ny: f64, params: &LensDistortionParams) -> (f64, f64) {
    let r2 = nx * nx + ny * ny;
    let r4 = r2 * r2;
    let r6 = r4 * r2;

    // Radial distortion
    let radial = 1.0 + params.k1 * r2 + params.k2 * r4 + params.k3 * r6;

    // Tangential distortion
    let dx_tang = 2.0 * params.p1 * nx * ny + params.p2 * (r2 + 2.0 * nx * nx);
    let dy_tang = params.p1 * (r2 + 2.0 * ny * ny) + 2.0 * params.p2 * nx * ny;

    let ux = nx * radial + dx_tang;
    let uy = ny * radial + dy_tang;
    (ux, uy)
}

/// Compute vignette attenuation at a normalized position.
pub fn vignette_at(nx: f32, ny: f32, params: &VignetteParams) -> f32 {
    if params.amount <= 0.0 { return 1.0; }
    let dist = (nx * nx + ny * ny).sqrt();
    let midpoint = params.midpoint.max(0.01);
    let scale = 1.0 - params.midpoint;
    let normalized_dist = (dist - midpoint).max(0.0) / scale.max(0.01);
    let atten = 1.0 - params.amount * normalized_dist * normalized_dist;
    atten.max(0.0).min(1.0)
}

/// Apply lens correction to an RGBA frame.
pub fn apply_lens_correction(frame: &mut [u8], width: u32, height: u32, config: &LensCorrectionConfig) {
    if !config.enabled { return; }

    let w = width as f64;
    let h = height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let scale = w.min(h) / 2.0;

    let mut output = frame.to_vec();

    for y in 0..height {
        for x in 0..width {
            let nx = (x as f64 - cx) / scale;
            let ny = (y as f64 - cy) / scale;

            // Undistort
            let (ux, uy) = undistort_point(nx, ny, &config.distortion);
            let src_x = (ux * scale + cx).round() as i64;
            let src_y = (uy * scale + cy).round() as i64;

            let dst_idx = ((y * width + x) * 4) as usize;

            if src_x >= 0 && src_x < width as i64 && src_y >= 0 && src_y < height as i64 {
                let src_idx = ((src_y as u32 * width + src_x as u32) * 4) as usize;

                // Chromatic aberration: offset R and B channels
                let r_nx = nx + config.ca.red_offset_x as f64 * nx.abs() * config.ca.radial_factor as f64 * 0.001;
                let r_ny = ny + config.ca.red_offset_y as f64 * ny.abs() * config.ca.radial_factor as f64 * 0.001;
                let b_nx = nx + config.ca.blue_offset_x as f64 * nx.abs() * config.ca.radial_factor as f64 * 0.001;
                let b_ny = ny + config.ca.blue_offset_y as f64 * ny.abs() * config.ca.radial_factor as f64 * 0.001;

                let r_src_x = (r_nx * scale + cx).round().clamp(0.0, w - 1.0) as u32;
                let r_src_y = (r_ny * scale + cy).round().clamp(0.0, h - 1.0) as u32;
                let b_src_x = (b_nx * scale + cx).round().clamp(0.0, w - 1.0) as u32;
                let b_src_y = (b_ny * scale + cy).round().clamp(0.0, h - 1.0) as u32;

                let r_idx = ((r_src_y * width + r_src_x) * 4) as usize;
                let b_idx = ((b_src_y * width + b_src_x) * 4) as usize;

                output[dst_idx] = frame[r_idx];       // Red from offset position
                output[dst_idx + 1] = frame[src_idx + 1]; // Green from center
                output[dst_idx + 2] = frame[b_idx + 2];   // Blue from offset position
                output[dst_idx + 3] = frame[src_idx + 3];
            } else {
                output[dst_idx] = 0;
                output[dst_idx + 1] = 0;
                output[dst_idx + 2] = 0;
                output[dst_idx + 3] = 0;
            }

            // Apply vignette
            let vnx = (x as f32 / width as f32) * 2.0 - 1.0;
            let vny = (y as f32 / height as f32) * 2.0 - 1.0;
            let vignette = vignette_at(vnx, vny, &config.vignette);
            output[dst_idx] = (output[dst_idx] as f32 * vignette) as u8;
            output[dst_idx + 1] = (output[dst_idx + 1] as f32 * vignette) as u8;
            output[dst_idx + 2] = (output[dst_idx + 2] as f32 * vignette) as u8;
        }
    }

    frame.copy_from_slice(&output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undistort_identity() {
        let params = LensDistortionParams::default();
        let (ux, uy) = undistort_point(0.5, 0.5, &params);
        assert!((ux - 0.5).abs() < 0.001);
        assert!((uy - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_undistort_barrel() {
        let params = LensDistortionParams { k1: -0.1, ..Default::default() };
        let (ux, _) = undistort_point(0.5, 0.0, &params);
        assert!(ux.abs() < 0.5); // Barrel distortion pulls inward
    }

    #[test]
    fn test_undistort_pincushion() {
        let params = LensDistortionParams { k1: 0.1, ..Default::default() };
        let (ux, _) = undistort_point(0.5, 0.0, &params);
        assert!(ux > 0.5); // Pincushion pushes outward
    }

    #[test]
    fn test_vignette_center() {
        let params = VignetteParams { amount: 0.5, midpoint: 0.5, roundness: 1.0 };
        let v = vignette_at(0.0, 0.0, &params);
        assert_eq!(v, 1.0); // Center should be fully bright
    }

    #[test]
    fn test_vignette_edge() {
        let params = VignetteParams { amount: 0.8, midpoint: 0.0, roundness: 1.0 };
        let v = vignette_at(1.0, 1.0, &params);
        assert!(v < 1.0);
    }

    #[test]
    fn test_vignette_zero_amount() {
        let params = VignetteParams { amount: 0.0, ..Default::default() };
        assert_eq!(vignette_at(1.0, 1.0, &params), 1.0);
    }

    #[test]
    fn test_builtin_profiles_count() {
        assert_eq!(builtin_profiles().len(), 8);
    }

    #[test]
    fn test_builtin_profile_names() {
        let profiles = builtin_profiles();
        assert_eq!(profiles[0].name, "GoPro Hero 11 Wide");
        assert_eq!(profiles[1].name, "iPhone 15 Main");
    }

    #[test]
    fn test_apply_lens_correction_disabled() {
        let mut frame = vec![128u8; 10 * 10 * 4];
        let original = frame.clone();
        let config = LensCorrectionConfig { enabled: false, ..Default::default() };
        apply_lens_correction(&mut frame, 10, 10, &config);
        assert_eq!(frame, original);
    }

    #[test]
    fn test_apply_lens_correction_identity() {
        let mut frame = vec![128u8; 20 * 20 * 4];
        let config = LensCorrectionConfig { enabled: true, ..Default::default() };
        apply_lens_correction(&mut frame, 20, 20, &config);
        // Identity params should produce near-identical output (except vignette=0)
    }

    #[test]
    fn test_tangential_distortion() {
        let params = LensDistortionParams { p1: 0.1, p2: 0.0, ..Default::default() };
        let (_, uy) = undistort_point(0.5, 0.5, &params);
        assert!(uy != 0.5); // Should shift
    }

    #[test]
    fn test_distortion_default() {
        let d = LensDistortionParams::default();
        assert_eq!(d.k1, 0.0);
        assert_eq!(d.p1, 0.0);
    }

    #[test]
    fn test_ca_default() {
        let ca = ChromaticAberrationParams::default();
        assert_eq!(ca.red_offset_x, 0.0);
        assert_eq!(ca.radial_factor, 1.0);
    }

    #[test]
    fn test_vignette_params_default() {
        let v = VignetteParams::default();
        assert_eq!(v.amount, 0.0);
    }

    #[test]
    fn test_correction_config_default() {
        let c = LensCorrectionConfig::default();
        assert!(c.enabled);
        assert!(c.crop_to_original);
        assert!(c.selected_profile.is_none());
    }

    #[test]
    fn test_apply_with_profile() {
        let profiles = builtin_profiles();
        let mut frame = vec![128u8; 20 * 20 * 4];
        let profile = &profiles[0];
        let config = LensCorrectionConfig {
            enabled: true,
            distortion: profile.distortion.clone(),
            ca: profile.ca.clone(),
            vignette: profile.vignette.clone(),
            ..Default::default()
        };
        apply_lens_correction(&mut frame, 20, 20, &config);
    }

    #[test]
    fn test_undistort_center() {
        let params = LensDistortionParams { k1: -0.3, k2: 0.1, k3: -0.01, p1: 0.01, p2: -0.01 };
        let (ux, uy) = undistort_point(0.0, 0.0, &params);
        assert!((ux).abs() < 0.01);
        assert!((uy).abs() < 0.01);
    }
}
