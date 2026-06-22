---
name: motion-tracking
description: >
  Motion Tracking. Use when the user says "motion track", "track point", "planar track", "track mask", "attach to track", "blur face", "track text", "pin to subject", "camera track", "track matte",
  or whenever they describe a workflow involving motion track.
license: MIT
---

# Motion Tracking

## The trick

Motion tracking follows a feature in the frame over time, so you can attach something to it (text, a blur, a mask, an effect). Three kinds: **point track** (single feature — fast, fragile), **planar track** (a region — robust, used for masks and screen replacements), **camera track** (3D camera solve — used for compositing 3D elements). EDITORS-PRO has a `motion_tracking.rs` stub with point-track (centroid) — planar and camera are the upgrade paths (mark with `video:`).

The amateur move is to track once and hope. The pro move is to track, verify the track, fix drift with manual keyframes, and only then attach.

## When to use

Blur a face. Attach text to a moving subject. Replace a screen. Drive a mask with tracking data.

## When NOT to use

When the subject is static — just position the effect, don't track. When the motion is too fast or too blurry — the tracker will fail.

## Examples

**Amateur** ❌: Track, attach text, text drifts off the subject halfway through. Or: blur a face, the blur doesn't follow the face.

**Professional** ✅: Pick a high-contrast feature. Track forward. Verify the track frame-by-frame. Fix drift with manual keyframes. Smooth the track. Attach the effect. Verify the attachment frame-by-frame.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Point track. Auto. Acceptable for face blur on social. |
| `full` | Point track with manual drift fixes. Verify frame-by-frame. Planar track for masks. |
| `ultra` | Planar track for screen replacements. Camera solve for 3D composites. Full QC on the track. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Motion tracking covers point, planar, and camera tracking. It does not cover stabilization (video-stabilization) or the effect attached to the track (the relevant effect skill).
