---
name: format-interop
description: >
  Format Interoperability (EDL / FCPXML / OpenTimelineIO). Use when the user says "edl", "fcpxml", "opentimelineio", "otio", "aaf", "xml export", "premiere round trip", "resolve round trip", "final cut round trip", "project interchange",
  or whenever they describe a workflow involving edl.
license: MIT
---

# Format Interoperability (EDL / FCPXML / OpenTimelineIO)

## The trick

Format interoperability moves a timeline between NLEs. EDITORS-PRO's `project/interop.rs` exports EDL (Edit Decision List — the oldest, simplest format, supported everywhere), FCPXML (Final Cut Pro XML, also imported by Premiere and Resolve), and OpenTimelineIO (the modern open standard, from Pixar).

The pro workflow: (1) export the timeline to the format the next tool supports (FCPXML for FCP/Premiere/Resolve, EDL for older systems, OTIO for modern pipelines), (2) open in the target tool, (3) verify clips and cuts survived the round trip, (4) note that effects, color, and audio don't always translate — those need to be re-done in the target tool.

## When to use

Round-tripping between EDITORS-PRO and DaVinci/Premiere/FCP. Handing off to a colorist. Moving to a different NLE for a specific tool.

## When NOT to use

When you can finish in EDITORS-PRO — don't round-trip unnecessarily. When the target NLE doesn't support the format.

## Examples

**Amateur** ❌: Exporting a project as a single video file and re-importing, losing all edit decisions. Then asking the colorist to "just match it."

**Professional** ✅: Export to FCPXML or OTIO. Open in the target NLE. Verify clips and cuts. Re-do effects, color, and audio in the target — they don't translate. Document what translated and what didn't.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Skip — finish in EDITORS-PRO. |
| `full` | Export FCPXML for round trip to Premiere/Resolve. Verify clips and cuts survived. |
| `ultra` | Export OTIO for pipeline handoff. Document what translated. Verify frame-rate and color space metadata survived. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `ProRes`
- `frame-rate`
- `color space`
- `delivery spec`

## Boundaries

Format interop covers timeline interchange. It does not cover the encode (delivery-encode-ladder) or the source media relinking (project's job).
