---
name: lens-correction
description: >
  Lens Correction. Use when the user says "lens correction", "lens distortion", "chromatic aberration", "ca fix", "vignette", "lens profile", "brown-conrady", "de-fish", "fisheye fix",
  or whenever they describe a workflow involving lens correction.
license: MIT
---

# Lens Correction

## The trick

Lens correction fixes optical defects: distortion (barrel/pincushion/moustache), chromatic aberration (color fringing at edges), and vignetting (darkening at corners). EDITORS-PRO's `effects/lens_correction.rs` does Brown-Conrady distortion (K1/K2/K3 + tangential P1/P2), CA correction, vignette removal, and has 8 built-in lens profiles. The Flutter `lens_correction_panel.dart` widget exposes them.

The pro workflow: (1) pick the lens profile (if known), or (2) dial in K1/K2/K3 manually with a grid reference, (3) fix CA with the red/blue offset sliders (zoom in to 200% to see fringing), (4) remove vignette with the amount/midpoint/roundness sliders.

## When to use

Wide-angle footage with visible barrel distortion. Drone footage with fisheye. Cheap lenses with CA. Footage with vignetting you want to remove (or add for stylization).

## When NOT to use

When the lens is already corrected in-camera (most modern phones). When the distortion is the point (fisheye music video look).

## Examples

**Amateur** ❌: Sliding K1 to 0.5 because the frame looks "weird" with no reference grid. Result: over-corrected, wobbly straight lines.

**Professional** ✅: Pick the lens profile if known. Otherwise, enable a grid overlay and dial K1 until straight lines are straight. Fix CA at 200% zoom — match red and blue offsets. Vignette: subtle removal, or add for stylization.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Auto-profile. Skip CA. Skip vignette. |
| `full` | Auto-profile or manual K1/K2/K3 with grid. Fix CA. Subtle vignette. |
| `ultra` | Per-lens calibration. CA per-channel. Vignette calibrated to the lens. Document in QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`
- `color space`

## Boundaries

Lens correction covers optical defect correction. It does not cover color grading (color-scopes) or the creative use of distortion (out of scope).
