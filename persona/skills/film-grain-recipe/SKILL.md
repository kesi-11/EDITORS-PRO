---
name: film-grain-recipe
description: >
  Film Grain Recipe. Use when the user says "film grain", "add grain", "grain recipe", "film stock", "vhs grain", "halation", "film emulation", "super 8", "16mm", "35mm",
  or whenever they describe a workflow involving film grain.
license: MIT
---

# Film Grain Recipe

## The trick

Film grain adds organic texture to digital footage. Two parts: the **grain** (the actual noise pattern, varies by stock — Kodak 5219 500T, Fuji Eterna, etc.) and the **halation** (the red glow around bright highlights, caused by light bouncing off the film base). EDITORS-PRO's `effects/grain.rs` has 17 stock presets plus VHS and halation. The Flutter `film_grain_picker.dart` widget exposes them.

The amateur move is to crank grain to 100% and call it "cinematic." The pro move is to apply grain at 15-30% to break up digital cleanliness, with the right stock for the look (Kodak Vision3 for warm tones, Fuji for cool tones, Ilford for B&W).

## When to use

When footage looks too clean/digital. When matching film footage. When emulating a film stock. When adding texture for a stylized look.

## When NOT to use

On footage that's already noisy — grain on top of noise looks worse. On content where the digital look is the point (UI demos, screencasts). For broadcast unless spec'd — grain eats bitrate.

## Examples

**Amateur** ❌: Grain at 100%, result looks like a 90s TV with bad reception.

**Professional** ✅: Pick the right stock. Grain at 15-30%. Add halation on highlights for film emulation. Match the grain to the color temperature of the footage. Verify the grain doesn't push the encode bitrate over budget.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Grain at 20%, generic 35mm stock. Acceptable for social stylization. |
| `full` | Pick stock by look. Grain at 15-25%. Halation on highlights. Verify bitrate. |
| `ultra` | Per-shot grain matching. Halation calibrated to highlight EV. Grain plate generated at full res, downsampled. Document the recipe in the QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Film grain covers grain addition and film emulation. It does not cover color grading (color-scopes), LUT management (lut-management), or VHS-style glitch effects (out of scope).
