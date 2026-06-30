---
name: sky-replacement
description: >
  Sky Replacement. Use when the user says "sky replacement", "replace sky", "sky swap", "gradient sky", "overcast fix", "boring sky", "sky gradient",
  or whenever they describe a workflow involving sky replacement.
license: MIT
---

# Sky Replacement

## The trick

Sky replacement swaps a blown-out or boring sky for a more interesting one. EDITORS-PRO has a `sky_replace.rs` stub — the workflow is: (1) qualify the sky (luminance key on the bright sky region), (2) refine the mask (edge softness, hole-filling for trees/buildings), (3) composite the new sky, (4) match lighting (color, direction, intensity to the foreground), (5) add interaction (reflections, shadows).

The amateur move is to drop a sunset gradient behind every scene, ignoring whether the lighting matches. The pro move is to use a sky that matches the scene's lighting direction, color temperature, and time of day.

## When to use

When the sky is blown out and unrecoverable. When the sky is boring (overcast gray) and the scene calls for something better. When matching plate photography.

## When NOT to use

When the sky has detail that's recoverable with a grad filter or highlight recovery. When the scene's mood depends on the actual sky (e.g., ominous clouds). When the foreground lighting won't match any sky you swap in.

## Examples

**Amateur** ❌: Sunset gradient on every scene. Foreground lit from the side, sky sun setting straight ahead. Result looks like a bad Photoshop job.

**Professional** ✅: Qualify the sky with a luminance mask. Refine the edges (especially through trees). Pick a sky that matches the scene's lighting (direction, color temperature, time of day). Match exposure. Add reflections and shadows for ground interaction. Color-match the composite.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Luminance key + gradient sky. Acceptable for quick fixes. |
| `full` | Luminance key + real sky plate. Edge refinement. Lighting match. |
| `ultra` | Per-pixel qualification. Planar track for camera movement. Atmospheric perspective (sky color shifts with depth). Full QC on the composite. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `color space`
- `delivery spec`

## Boundaries

Sky replacement covers the sky swap. It does not cover the qualification technique (mask-animation) or the composite (compositing).
