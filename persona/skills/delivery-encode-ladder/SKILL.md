---
name: delivery-encode-ladder
description: >
  Delivery Encode Ladder. Use when the user says "export", "encode", "delivery", "prores", "h.264", "h.265", "hevc", "av1", "crf", "bitrate", "master", "mezzanine", "streaming encode",
  or whenever they describe a workflow involving export.
license: MIT
---

# Delivery Encode Ladder

## The trick

The encode ladder is the chain from master to delivery. **Master** (lossless or mezzanine — ProRes 4444 XQ, DNxHR 444, DPX, EXR) → **Mezzanine** (ProRes 422 HQ, DNxHR HQX — for post-production handoff) → **Streaming** (H.264 high-profile, HEVC main-10, AV1 — for delivery). At every rung, verify the loudness hits the target (−23 LUFS EBU R128 for broadcast, platform target for streaming), the true-peak is ≤ −2 dBTP broadcast / ≤ −1 dBTP streaming, and the frame rate matches the delivery spec exactly.

Bitrate is content-dependent. Use CRF (constant rate factor) instead of a fixed bitrate for variable-content delivery: CRF 18 visually lossless, CRF 20 high quality, CRF 23 standard. For streaming, use 2-pass VBR with a bitrate ceiling set by the platform.

EDITORS-PRO's `export_engine/encoder.rs` does H.264/H.265/VP9 with 2-pass and AAC muxing. ProRes is on the roadmap (mark with `video:`).

## When to use

Always, at delivery. Pick the rung based on the delivery spec — never ship a master when a streaming encode is asked for, and never ship a streaming encode when a master is asked for.

## When NOT to use

Never. Every delivery needs an encode.

## Examples

**Amateur** ❌: Exporting H.264 at default settings, getting a 200 MB file for a 30-second clip, then wondering why it buffers on mobile.

**Professional** ✅: Master in ProRes 422 HQ (or ProRes 4444 XQ for HDR). Mezzanine for post handoff. Streaming encode at CRF 18-23 with the right profile/level. Bitrate ceiling per platform. AAC audio at 256 kbps.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | H.264 CRF 23, AAC 128 kbps, yuv420p. Single-pass. Good enough for social. |
| `full` | ProRes 422 HQ master → H.264 high-profile CRF 18-20 streaming encode. 2-pass VBR. AAC 256 kbps. Verify frame rate, color space, and loudness on the encode. |
| `ultra` | ProRes 4444 XQ or DPX/EXR master. Per-spec delivery encode. HEVC 10-bit for HDR. Verify MaxFALL/MaxCLL for HDR10. Full QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `ProRes`
- `frame-rate`
- `color space`
- `dBTP`
- `delivery spec`

## Boundaries

The encode ladder covers the codec/bitrate choice and the encode pass. It does not cover the loudness (loudness-target), the legal range (broadcast-legal), or the format interoperability (format-interop).
