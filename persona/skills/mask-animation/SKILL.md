---
name: mask-animation
description: >
  Mask Drawing and Animation. Use when the user says "mask", "draw mask", "bezier mask", "animate mask", "rotoscope", "roto", "mask path", "feather mask", "mask shape",
  or whenever they describe a workflow involving mask.
license: MIT
---

# Mask Drawing and Animation

## The trick

Masks isolate regions of the frame for targeted effects. EDITORS-PRO's `effects/masking.rs` has Rectangle, Ellipse, Bezier, Luminance, Chroma, and Depth masks with feather, expansion, and 4 composite modes. The Flutter `mask_drawing_tool.dart` widget provides Bezier drawing on the canvas.

The pro workflow: (1) draw the mask roughly, (2) refine the shape, (3) feather the edges, (4) animate the mask path if the subject moves (rotoscope), (5) invert if needed, (6) apply the effect inside or outside the mask.

## When to use

Targeted effects (color just on the face, blur just on the background). Rotoscoping (subject isolation for compositing). Creative transitions (iris wipes, shape reveals).

## When NOT to use

When the effect should apply to the whole frame — don't mask unnecessarily. When a luminance or chroma key would do the job automatically — don't rotoscope what can be keyed.

## Examples

**Amateur** ❌: Drawing a mask, not animating it, then the subject walks out of the mask and the effect breaks.

**Professional** ✅: Draw the mask. Refine. Feather. Animate the path with the subject (rotoscope). Verify frame-by-frame. Use the right mask type (Bezier for shapes, Luminance for sky, Chroma for green screen, Depth for AR footage).

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Rectangle/ellipse mask. No animation. Acceptable for static shots. |
| `full` | Bezier mask with feather. Animate the path if subject moves. Verify frame-by-frame. |
| `ultra` | Per-frame rotoscope. Planar-tracked mask. Edge-aware feather. Full QC on the mask. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Mask covers shape drawing, feathering, and path animation. It does not cover the effect inside the mask (the relevant effect skill) or chroma keying (green-screen-key).
