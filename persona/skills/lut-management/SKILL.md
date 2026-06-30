---
name: lut-management
description: >
  LUT Management. Use when the user says "apply lut", "cube file", "3dl", "creative lut", "technical lut", "conversion lut", "look lut", "color preset", "film emulation lut",
  or whenever they describe a workflow involving apply lut.
license: MIT
---

# LUT Management

## The trick

A LUT (Look-Up Table) is a precomputed color transformation. Two kinds: **technical** (color space conversion — e.g., Log→Rec.709) and **creative** (a stylized look — e.g., film emulation). EDITORS-PRO's `effects/lut.rs` supports the .cube format (1D and 3D) and .3dl. Apply a technical LUT first to put log footage into a working color space, then grade on top, then optionally apply a creative LUT last for delivery.

A LUT is a frozen grade. Don't use a LUT to do what a node graph should do — LUTs are for transformations you want to apply identically across many clips, or for transformations authored externally (e.g., a camera manufacturer's Log→Rec.709 LUT). For per-shot work, grade directly.

## When to use

You have log footage and need to convert to Rec.709. You have a creative look you want to apply across the whole timeline. You received a delivery LUT from a colorist. You want to emulate a film stock.

## When NOT to use

For per-shot grading — use the color wheels instead. For transformations that depend on clip content (e.g., "make this shot match that shot") — use shot match instead. For anything that needs to be tweaked per-clip — a LUT is immutable.

## Examples

**Amateur** ❌: Stacking 3 creative LUTs on top of each other, each at 50% opacity, hoping one of them looks right. Or applying a "cinematic" LUT you downloaded from YouTube to log footage without first converting to Rec.709 — the result is washed-out shadows and crushed highlights.

**Professional** ✅: Node graph: (1) Color Space Transform from camera Log to Rec.709 (technical LUT or CST node), (2) primary grade (lift/gamma/gain), (3) secondary grade (qualifier + power window), (4) optional creative LUT at 80-100% mix for stylization. Export the LUT from the grade if you want to reuse it across clips.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | One creative LUT at 100% over a one-pass grade. Acceptable for social. |
| `full` | Technical LUT first (Log→Rec.709), then grade, then creative LUT last. LUT intensity ≤ 100%. Verify legal range after LUT. |
| `ultra` | LUT pipeline is 32-bit float, applied in scene-referred linear. Creative LUTs authored in the DI suite, not downloaded. Every LUT applied must pass legal range check before encode. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `legal range`
- `color space`
- `delivery spec`

## Boundaries

LUT management covers the application, authoring, and import/export of .cube and .3dl files. It does not cover color grading itself (that's the color-scopes and color-match-shots skills) or color space conversion (that's the broadcast-legal and hdr-delivery skills).
