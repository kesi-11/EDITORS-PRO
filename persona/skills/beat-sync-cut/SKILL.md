---
name: beat-sync-cut
description: >
  Beat-Synced Cutting. Use when the user says "beat sync", "cut on beat", "music sync", "beat detect", "cut to music", "rhythmic editing", "music marker", "tempo",
  or whenever they describe a workflow involving beat sync.
license: MIT
---

# Beat-Synced Cutting

## The trick

Cutting on the beat makes a music-driven edit feel tight. The amateur move is to manually tap-M to add markers and hope you kept time. The pro move is to detect beats programmatically (onset detection — peaks in the spectral flux), drop markers on the timeline, then snap cuts to markers. EDITORS-PRO's `analysis/beat_detect.rs` provides onset detection; the markers can then drive magnetic snapping.

Beat detection is not perfect — it finds transients, which include kicks, snares, and other percussive events. For music with a strong kick, it works great. For ambient music, you may need to set the BPM manually and generate markers from that.

## When to use

Music-driven edits: montages, social clips with a beat, dance videos, music videos, ad spots with a music bed.

## When NOT to use

Dialogue-driven scenes — cutting on the beat fights the natural rhythm of speech. Documentary. Narrative scenes where pacing is emotional, not musical.

## Examples

**Amateur** ❌: Cutting randomly and hoping it lines up. Or cutting exactly on every beat, making the edit feel mechanical and exhausting.

**Professional** ✅: Detect beats. Snap cuts to beat markers with magnetic snapping. But cut on the **accented** beats (1 and 3 in 4/4), not every beat. Leave some shots longer for breathing room. The edit should feel tight, not frantic.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Auto-detect, snap to every beat. Quick and acceptable for social. |
| `full` | Auto-detect, snap to accented beats. Verify sync by ear at the end. |
| `ultra` | Manual sync to the score. Tempo map documented. Per-frame offset for emotional timing. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Beat-sync covers aligning cuts to musical beats. It does not cover the music selection, music licensing, or the broader pacing of the edit (narrative-pacing).
