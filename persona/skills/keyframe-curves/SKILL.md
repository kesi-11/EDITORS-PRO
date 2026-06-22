---
name: keyframe-curves
description: >
  Keyframe Curves (Bezier). Use when the user says "keyframe curve", "bezier keyframe", "easing", "ease in", "ease out", "animation curve", "smooth keyframes", "keyframe interpolation", "graph editor",
  or whenever they describe a workflow involving keyframe curve.
license: MIT
---

# Keyframe Curves (Bezier)

## The trick

Keyframe curves control the interpolation between keyframes. Linear keyframes (default in amateur tools) produce robotic, mechanical motion. Bezier keyframes with the right easing produce natural, organic motion. EDITORS-PRO's `timeline/keyframe.rs` supports linear, ease-in, ease-out, ease-in-out, and bezier with adjustable tangent handles. The Flutter `keyframe_graph_editor.dart` widget exposes them.

The pro workflow: (1) set keyframes for the property (position, scale, opacity, etc.), (2) open the graph editor, (3) adjust the bezier tangent handles to shape the curve, (4) use ease-in for things entering, ease-out for things leaving, ease-in-out for things settling, (5) verify with a real-time preview.

## When to use

Any animated property — position, scale, rotation, opacity, effect parameters. Title animations. Zoom/pan on stills (Ken Burns).

## When NOT to use

When linear is the right look (mechanical, robotic UI animations). When there are only two keyframes and the easing doesn't matter.

## Examples

**Amateur** ❌: Linear keyframes everywhere. Title slides in at constant speed, stops dead. Result: looks like a PowerPoint animation.

**Professional** ✅: Bezier keyframes with shaped curves. Ease-in for entering, ease-out for leaving. Smooth, organic motion. Verify with real-time preview. Per-property lanes in the graph editor.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Ease-in-out on key transitions. Don't touch the graph editor. |
| `full` | Bezier with shaped tangents. Per-property lanes. Verify with preview. |
| `ultra` | Per-property bezier. Frame-precise timing. Document the curve choices in the editor's notes. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Keyframe curves cover animation interpolation. They do not cover the property being animated (the relevant effect skill) or the speed curves (which are a separate concept — see speed-curve-editor.dart).
