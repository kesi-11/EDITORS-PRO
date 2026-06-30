---
name: multicam-editing
description: >
  Multicam Editing. Use when the user says "multicam", "multi-cam", "multi angle", "camera angle", "angle switch", "live cut", "concert edit", "event edit",
  or whenever they describe a workflow involving multicam.
license: MIT
---

# Multicam Editing

## The trick

Multicam editing cuts between multiple camera angles of the same event in real time. EDITORS-PRO's `effects/multicam.rs` has angle grouping, audio cross-correlation sync, angle switching, and transitions. The Flutter `multicam_switcher.dart` widget exposes an angle grid for real-time switching.

Workflow: (1) group angles into a multicam clip, (2) sync via timecode or audio (cross-correlation), (3) play back in real time, switching angles by tapping, (4) refine the cuts after the live pass, (5) add transitions where wanted.

## When to use

Concert footage, event coverage, talk shows, interviews with multiple cameras, sports.

## When NOT to use

Single-camera shoots. When the angles aren't synced (audio or timecode).

## Examples

**Amateur** ❌: Cutting between angles with no rhythm, every cut visible, no audio sync reference.

**Professional** ✅: Sync via audio cross-correlation. Live-switch with the beat of the event. Refine cuts. Hard cuts for energy, dissolves for soft transitions. Verify audio is from the best angle (usually the board feed).

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Auto-sync via audio. Live-switch. Hard cuts only. |
| `full` | Auto-sync via audio. Live-switch. Refine cuts. Add transitions where wanted. Verify audio source. |
| `ultra` | Timecode sync. Per-cut transition choice. Audio from the board feed. Document the angle choices in the QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`
- `frame-rate`

## Boundaries

Multicam covers multi-angle cutting. It does not cover the audio mix (loudness-target) or the per-angle color (color-match-shots).
