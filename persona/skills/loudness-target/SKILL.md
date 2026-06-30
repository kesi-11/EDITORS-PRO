---
name: loudness-target
description: >
  Loudness Targeting (EBU R128 / ATSC A/85 / Platform). Use when the user says "loudness", "lufs", "ebu r128", "atsc a/85", "integrated loudness", "true peak", "dbtp", "normalize audio", "audio normalization", "calm act", "platform loudness", "youtube loudness", "tiktok loudness",
  or whenever they describe a workflow involving loudness.
license: MIT
---

# Loudness Targeting (EBU R128 / ATSC A/85 / Platform)

## The trick

Loudness is measured in LUFS (Loudness Units Full Scale, K-weighted). Integrated loudness is the average over the whole program; short-term is over 3-second windows; momentary is over 0.4-second windows. True-peak (dBTP) measures inter-sample peaks — a sample at −0.1 dBFS can produce a true-peak above 0 dBTP after DAC reconstruction, killing encoders.

Targets:
- EBU R128 (EU broadcast): −23 LUFS ±0.5 integrated, −18 LUFS short-term max, −1 dBTP max.
- ATSC A/85 (US broadcast, CALM Act): −24 LKFS ±2 integrated.
- YouTube: −14 LUFS (they normalize to this).
- TikTok: −18 LUFS.
- Spotify/Apple Music/Amazon Music: −14 LUFS.
- Apple Podcasts: −16 LUFS.

EDITORS-PRO's `analysis/loudness.rs` computes R128 integrated. The Flutter `audio_loudness_meter.dart` widget renders it live. Never ship without measuring.

## When to use

Always, before sign-off. Every deliverable has a loudness target — find it, hit it, verify it.

## When NOT to use

Never. There is no scenario where you skip loudness measurement on a deliverable.

## Examples

**Amateur** ❌: Normalizing to 0 dBFS peak and calling it done. The result is dynamic range crushed flat, and the platform normalizes it down anyway, making it quieter than competitors.

**Professional** ✅: Mix to the target integrated LUFS. Verify with a true-peak meter — keep ≤ −1 dBTP for streaming, ≤ −2 dBTP for broadcast. Don't crush dynamics to hit the target — adjust the mix, not just the limiter.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Platform target (YouTube −14, TikTok −18). True-peak ≤ −1 dBTP. One-pass. |
| `full` | −23 LUFS EBU R128 or −24 LKFS ATSC A/85. True-peak ≤ −2 dBTP. Documented in QC report. |
| `ultra` | Per-spec loudness. Dolby Atmos loudness measurement if applicable. True-peak ≤ −2 dBTP. Full QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `−23 LUFS`
- `−24 LKFS`
- `dBTP`
- `delivery spec`

## Boundaries

Loudness targeting covers the measurement and normalization of the master. It does not cover the mix itself (dialogue-cleanup, music mixing) or the encode (delivery-encode-ladder).
