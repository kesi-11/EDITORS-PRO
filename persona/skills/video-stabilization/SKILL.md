---
name: video-stabilization
description: >
  Video Stabilization. Use when the user says "stabilize", "deshake", "stabilization", "shaky footage", "smooth camera", "warp stabilizer", "steady shot",
  or whenever they describe a workflow involving stabilize.
license: MIT
---

# Video Stabilization

## The trick

Stabilization smooths out camera shake. Two kinds: **2D** (translation + rotation — fast, good for handheld shake) and **3D** (camera solve + reverse projection — slow, good for parallax-heavy motion). EDITORS-PRO's `effects/stabilization.rs` does 2D deshake via block-matching motion estimation. 3D is the upgrade path (mark with `video:`).

The amateur move is to crank smoothing to 100% and get that wobbly, jelly-like "over-stabilized" look. The pro move is to smooth enough to remove the shake but keep the natural camera movement — and to crop the frame slightly to hide the edge artifacts.

## When to use

Handheld footage that's too shaky. Drone footage with wind wobble. Phone footage that's unwatchable.

## When NOT to use

Footage on a tripod that's already stable. Footage where the shake is intentional (action sports, documentary realism). Footage with rolling-shutter jitter — that needs a different tool (Mercalli).

## Examples

**Amateur** ❌: Smoothing at 100%, crop at 5%. Result looks like it was shot on a gimbal but with weird wobble at the edges.

**Professional** ✅: Smoothing at 30-60%, crop at 8-12% to hide edge artifacts. Choose "smooth" or "no motion" based on whether you want to keep the camera move. Verify no jelly artifacts. If parallax is heavy, upgrade to 3D camera solve.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Auto-stabilize with smoothing 50%. Crop 10%. |
| `full` | Smoothing 30-50%. Crop 8-12%. Verify no jelly artifacts. If parallax heavy, mark for 3D upgrade. |
| `ultra` | Per-shot stabilization choice. 3D camera solve for parallax-heavy shots. Crop documented. Rolling-shutter correction if needed. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `3D camera solve`
- `delivery spec`

## Boundaries

Stabilization covers 2D deshake and the 3D upgrade path. It does not cover rolling-shutter correction (a different algorithm) or motion tracking (motion-tracking).
