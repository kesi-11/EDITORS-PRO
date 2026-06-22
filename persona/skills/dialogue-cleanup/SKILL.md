---
name: dialogue-cleanup
description: >
  Dialogue Cleanup. Use when the user says "clean dialogue", "remove noise", "denoise voice", "de-ess", "de-reverb", "remove echo", "remove room sound", "clean up audio", "fix audio", "remove hiss", "remove hum",
  or whenever they describe a workflow involving clean dialogue.
license: MIT
---

# Dialogue Cleanup

## The trick

Dialogue cleanup is the difference between "professional" and "amateur." Stages: (1) **noise reduction** — broadband hiss, AC, room tone, (2) **de-reverb** — remove room echo, (3) **de-essing** — tame sibilance, (4) **EQ** — remove low-end rumble, brighten for intelligibility, (5) **compression** — even out dynamics, (6) **limiter** — catch peaks. EDITORS-PRO's `audio/effects.rs` currently has a low-pass filter; the full chain requires external processing (DaVinci Voice Isolation, iZotope RX) and re-linking. This is documented as a `video:` debt marker.

Workflow: clean in a dedicated audio app, re-link in EDITORS-PRO. Don't try to do spectral repair in an NLE.

## When to use

Anytime you have dialogue recorded in less-than-ideal conditions. Which is almost always.

## When NOT to use

On music or sound design — these need different processing. On "clean" studio dialogue that's already been processed.

## Examples

**Amateur** ❌: Slapping a heavy noise reduction on the whole track, getting that underwater robotic sound, then publishing.

**Professional** ✅: Subtle noise reduction (5-15 dB). De-reverb only if needed. De-esser on sibilance. High-pass filter at 80 Hz for rumble. Gentle EQ boost around 3 kHz for intelligibility. Compressor at 3:1, slow attack. Limiter at −1 dBTP. Result: dialogue that sounds natural and clear.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | High-pass + light NR + limiter. Enough to make dialogue intelligible. |
| `full` | Full chain: NR → de-reverb → de-esser → EQ → compressor → limiter. Verify loudness hits −23 LUFS (broadcast) or platform target. |
| `ultra` | Per-channel processing. A/B against reference. Document the chain in the QC report. True-peak ≤ −2 dBTP. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `−23 LUFS`
- `dBTP`
- `delivery spec`

## Boundaries

Dialogue cleanup covers spoken-word processing. It does not cover music mixing (loudness-target handles platform loudness), sound design, or foley.
