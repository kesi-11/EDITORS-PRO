---
name: green-screen-key
description: >
  Green/Blue Screen Keying. Use when the user says "green screen", "chroma key", "blue screen", "key out background", "green backdrop", "spill", "color key", "matte",
  or whenever they describe a workflow involving green screen.
license: MIT
---

# Green/Blue Screen Keying

## The trick

Keying removes a colored background (green or blue) to composite a subject over a new background. EDITORS-PRO's `effects/chroma_key.rs` does HSV-based keying with an eyedropper. The full pro chain: (1) **sample the screen color** with the eyedropper, (2) **adjust the key range** (tolerance), (3) **despill** — remove green spill on edges of the subject, (4) **edge refinement** — matte choke/expand, feather, (5) **lighting match** — match the subject's lighting to the new background.

The amateur move is to crank the tolerance until the green is gone, leaving hard edges and green spill. The pro move is to key narrowly, despill, and refine edges.

## When to use

When you have footage shot on a green or blue screen and need to composite over a new background.

## When NOT to use

When the green screen was lit unevenly — fix the lighting in reshoot, not in post. When the green screen has shadows or wrinkles — same. When the subject has green in it (clothing, eyes) — use blue or rotoscope.

## Examples

**Amateur** ❌: Tolerance at 100, green gone but subject has a halo of green spill and jagged edges.

**Professional** ✅: Sample the screen color. Tolerance just enough to remove the screen. Despill to kill green on subject edges. Matte choke to remove the halo. Feather for soft edges. Match subject lighting to the new background (direction, color, intensity).

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Auto-key with eyedropper. Acceptable for talking-head social composites. |
| `full` | Full chain: key → despill → edge refine → lighting match. Verify with the final background in place. |
| `ultra` | Per-pixel key with spill suppression. Edge-aware matte. Match grain between subject and background. Full QC on the composite. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `color space`
- `delivery spec`

## Boundaries

Green-screen key covers chroma keying. It does not cover masking (mask-animation), rotoscoping, or the composite (compositing in effects/compositing.rs).
