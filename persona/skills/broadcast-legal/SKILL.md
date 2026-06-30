---
name: broadcast-legal
description: >
  Broadcast Legal (Loudness, Color Range, Title-Safe). Use when the user says "broadcast legal", "legal range", "legalize", "title safe", "action safe", "broadcast safe", "qc", "quality control", "broadcast compliance", "ebu", "atsc", "r128",
  or whenever they describe a workflow involving broadcast legal.
license: MIT
---

# Broadcast Legal (Loudness, Color Range, Title-Safe)

## The trick

Broadcast legal is the bundle of compliance checks: loudness (EBU R128 −23 LUFS or ATSC A/85 −24 LKFS), true-peak (≤ −1 dBTP streaming, ≤ −2 dBTP broadcast), legal color range (Rec.709 16–235 / 64–940), title-safe (90% broadcast) and action-safe (80%) areas, frame-rate compliance, color space tagging. EDITORS-PRO has a `legalizer.rs` stub — the workflow is: (1) run a legalizer pass before encode (clamps color to legal range with optional soft-clip), (2) verify loudness with the meter, (3) verify graphics are inside title-safe, (4) verify frame rate matches spec, (5) verify color space tags in the encode, (6) verify legal range on the **waveform** monitor and skin tones on the **vectorscope** I-line.

The amateur move is to ship without QC. The pro move is to run a full QC pass and document it.

## When to use

Always, in full and ultra mode. Any deliverable with a spec. Before sign-off.

## When NOT to use

Never in full/ultra. In lite mode, the platform loudness check is the minimum.

## Examples

**Amateur** ❌: Shipping without QC. Client finds the loudness violation at 2am the day before air.

**Professional** ✅: Legalizer pass. Loudness meter. Title-safe overlay. Frame-rate check. Color space tags. Full QC report.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Platform loudness check only. Skip the rest. |
| `full` | Legalizer + loudness + title-safe + frame-rate + color space tags. Full QC report. |
| `ultra` | Per-frame legalizer. Per-spec loudness. Per-pixel safe-area verification. Full QC report with sign-off. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `−23 LUFS`
- `−24 LKFS`
- `dBTP`
- `legal range`
- `title-safe`
- `frame-rate`
- `field-order`
- `color space`
- `delivery spec`
- `data loss`

## Boundaries

Broadcast legal covers compliance checks. It does not cover the encode (delivery-encode-ladder) or the loudness targeting (loudness-target — broadcast-legal verifies, loudness-target targets).
