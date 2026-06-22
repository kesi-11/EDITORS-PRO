---
name: color-match-shots
description: >
  Shot Matching (Color Match). Use when the user says "match shots", "shot match", "color match", "match this shot", "consistency", "continuity of color", "auto-match", "reference frame",
  or whenever they describe a workflow involving match shots.
license: MIT
---

# Shot Matching (Color Match)

## The trick

Shot matching makes two clips shot in different conditions look like they were shot in the same conditions. Three layers: (1) **white balance match** — neutralize each shot, (2) **exposure match** — align luma midpoints, (3) **color match** — align hue/saturation in shadows/mids/highlights. EDITORS-PRO's `effects/color_match.rs` provides histogram-based matching to a reference frame.

The amateur move is to slap a LUT on and hope. The pro move is to match each shot to a reference frame (usually a clean middle-shot of the scene), then apply a creative LUT last across the whole timeline for stylization.

## When to use

When two clips cut together and the color/exposure doesn't match. When establishing a scene. When footage came from multiple cameras or multiple days.

## When NOT to use

When the shots are intentionally different (e.g., day-for-night vs day scene). When the mismatch is in the white balance and you can fix it with one temperature slider — don't over-engineer.

## Examples

**Amateur** ❌: Trying to fix mismatches by stacking creative LUTs on each shot. The result is six different "cinematic" looks cutting back and forth.

**Professional** ✅: Pick a reference frame. Match each shot to it: white balance first, exposure second, color third. Apply a creative LUT last, uniformly across the timeline. Verify with scopes.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Auto-match to a reference frame is fine. Don't spend more than 30 seconds per shot. |
| `full` | Match each shot manually to the reference. Verify with vectorscope (skin tones on I-line) and waveform (luma midpoint). |
| `ultra` | Scene-referred match in ACES. Per-channel match. Document the reference frame in the QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `color space`
- `legal range`
- `.cube`

## Boundaries

Shot matching covers making clips look consistent. It does not cover grading a single shot (color-scopes), LUT management (lut-management), or creative stylization (film-grain-recipe).
