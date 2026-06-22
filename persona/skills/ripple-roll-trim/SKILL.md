---
name: ripple-roll-trim
description: >
  Ripple / Roll / Slip / Slide Trim. Use when the user says "ripple trim", "roll trim", "slip trim", "slide trim", "ripple delete", "trim mode", "advanced trim", "close gap", "overwrite trim",
  or whenever they describe a workflow involving ripple trim.
license: MIT
---

# Ripple / Roll / Slip / Slide Trim

## The trick

Four trim modes pros use constantly: **Ripple** — trims a clip and shifts everything after it to close the gap. **Roll** — trims two adjacent clips simultaneously (one gets shorter, the other gets longer, total duration unchanged). **Slip** — changes the in/out of a clip without changing its duration or position (you see different frames of the same shot). **Slide** — moves a clip left/right between its neighbors, trimming the neighbors to make room.

EDITORS-PRO's `timeline/advanced_trim.rs` implements all four. The Flutter `advanced_trim_modes.dart` widget exposes them as toolbar buttons.

The amateur move is to do everything with regular trim + delete + ripple-delete. The pro move is to use the right trim mode for the job — ripple to close gaps, roll to adjust a cut point, slip to fix a framing issue, slide to nudge a clip without affecting duration.

## When to use

Ripple: when closing a gap after a delete. Roll: when fine-tuning a cut between two clips. Slip: when a clip's framing is off but the duration is right. Slide: when a clip needs to move but the total duration can't change.

## When NOT to use

For simple trims — regular trim is fine. When you don't understand which mode does what — learn them first.

## Examples

**Amateur** ❌: Doing everything with regular trim + ripple-delete. Result: lots of fiddly manual re-positioning.

**Professional** ✅: Ripple to close gaps. Roll for cut-point refinement. Slip for reframing. Slide for nudging. The right mode for the job.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Ripple + ripple-delete. Skip roll/slip/slide. |
| `full` | All four modes. Use roll for cut refinement. |
| `ultra` | All four modes with J/K/L shuttle scrubbing. Trim with frame accuracy. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `frame-rate`
- `data loss`

## Boundaries

Trim modes cover timeline trimming. They do not cover the clip effects, color, or audio — those are unaffected by trim mode.
