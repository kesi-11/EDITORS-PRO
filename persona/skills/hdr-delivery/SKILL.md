---
name: hdr-delivery
description: >
  HDR Delivery (PQ / HLG / HDR10 / Dolby Vision). Use when the user says "hdr", "hdr10", "dolby vision", "hlg", "pq", "smpte st 2084", "bt.2020", "wide gamut", "rec 2020", "tone map", "10-bit delivery",
  or whenever they describe a workflow involving hdr.
license: MIT
---

# HDR Delivery (PQ / HLG / HDR10 / Dolby Vision)

## The trick

HDR delivery uses a wider color gamut (Rec.2020) and a wider dynamic range (PQ or HLG transfer function). PQ (Perceptual Quantizer, SMPTE ST 2084) is the standard for HDR10 and Dolby Vision. HLG (Hybrid Log-Gamma) is the standard for broadcast HDR. EDITORS-PRO's `effects/color_space.rs` does HDR PQ/HLG tone mapping on the input side; embedding HDR metadata (MaxFALL, MaxCLL, master display) in the encode is the gap (mark with `video:`).

The pro workflow: (1) confirm the delivery spec (HDR10, HDR10+, Dolby Vision, HLG), (2) grade in 10-bit Rec.2020 PQ, (3) verify with HDR scopes (Dolby L1/L2/L3 analyzer if Dolby Vision), (4) encode in HEVC main-10 or AV1, (5) embed the static metadata (MaxFALL, MaxCLL, master display) for HDR10, or the dynamic metadata for HDR10+/Dolby Vision.

## When to use

When the delivery spec calls for HDR. When the source is HDR (log footage with HDR intent). When delivering to HDR-capable platforms (Netflix, Apple TV+, Disney+, YouTube HDR).

## When NOT to use

When the delivery is SDR — don't ship HDR to SDR platforms. When the source is SDR — upconverting to HDR doesn't add dynamic range. When you don't have an HDR reference monitor — grading HDR on an SDR monitor is dangerous.

## Examples

**Amateur** ❌: Upconverting SDR footage to HDR by stretching the values, result looks washed out and over-saturated on HDR displays.

**Professional** ✅: Source is HDR. Grade in 10-bit Rec.2020 PQ. Verify with HDR scopes. Encode HEVC main-10. Embed static metadata (MaxFALL, MaxCLL, master display) for HDR10. Per-spec for HDR10+/Dolby Vision. Verify on an HDR reference monitor.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Skip HDR — deliver SDR. |
| `full` | HDR10 with static metadata. Verify MaxFALL/MaxCLL. Encode HEVC main-10. |
| `ultra` | Per-spec HDR (HDR10/HDR10+/Dolby Vision/HLG). Dynamic metadata if applicable. Full HDR QC with Dolby analyzer. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `color space`
- `legal range`
- `delivery spec`

## Boundaries

HDR delivery covers HDR grading, encoding, and metadata. It does not cover SDR delivery (delivery-encode-ladder) or the color space conversion (broadcast-legal).
