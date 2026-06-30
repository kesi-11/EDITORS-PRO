---
name: color-scopes
description: >
  Color Scopes (Waveform, Vectorscope, RGB Parade, Histogram). Use when the user says "scopes", "waveform", "vectorscope", "rgb parade", "histogram", "luma scope", "color analyzer", "read the scope", "crushed blacks", "clipped highlights",
  or whenever they describe a workflow involving scopes.
license: MIT
---

# Color Scopes (Waveform, Vectorscope, RGB Parade, Histogram)

## The trick

Scopes tell you what's actually in the image — your eyes lie, your monitor lies, the room lighting lies. The waveform shows luma distribution across the frame (X = horizontal position, Y = luma value). The vectorscope shows chroma distribution (angle = hue, distance from center = saturation). The RGB parade shows three separate waveforms for R, G, B — used for white balance and color cast. The histogram shows the distribution of luma values across the whole frame.

EDITORS-PRO's `analysis/scopes.rs` computes all four from any frame. The Flutter `color_scopes_panel.dart` widget renders them live in the editor. Use scopes to verify: blacks aren't crushed (waveform bottom ≥ 16), highlights aren't clipped (waveform top ≤ 235), white balance is neutral (RGB parade lines up), skin tones are on the I-line (vectorscope).

## When to use

Always, in full and ultra mode. Before signing off any deliverable. When matching shots. When grading log footage. When the room lighting is unreliable.

## When NOT to use

In lite mode for a one-pass social grade — scopes are optional. When your eyes are calibrated to a reference monitor in a graded room and you trust them — but verify with scopes before sign-off anyway.

## Examples

**Amateur** ❌: Grading by eye on an uncalibrated laptop screen in a bright room, then being surprised when the deliverable looks wrong on the client's monitor.

**Professional** ✅: Scopes open on a second monitor. Waveform set to Y (luma). Vectorscope with 75% target. RGB parade for white balance. Grade to the scopes, verify with eyes, sign off with scopes. Skin tones land on the vectorscope I-line (flesh tone line, ~123° from R).

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Optional. A waveform check before export is enough. |
| `full` | Required. Waveform + vectorscope before sign-off. Verify legal range (16–235 luma) on the waveform. |
| `ultra` | Required at every stage. Scopes pinned on a dedicated monitor. RGB parade for white balance, vectorscope for skin tone I-line, waveform for legal range, histogram for distribution. Sign-off documented in the QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `legal range`
- `color space`
- `vectorscope`
- `waveform`

## Boundaries

Color scopes cover reading and interpreting the scopes. They do not cover grading decisions (that's color-match-shots and lut-management) or legal range enforcement (that's broadcast-legal).
