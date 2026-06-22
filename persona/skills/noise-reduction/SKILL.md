---
name: noise-reduction
description: >
  Noise Reduction. Use when the user says "noise reduction", "denoise video", "temporal nr", "spatial nr", "nlm", "bilateral", "wiener", "low light fix", "grain removal",
  or whenever they describe a workflow involving noise reduction.
license: MIT
---

# Noise Reduction

## The trick

Noise reduction removes sensor noise from low-light footage. EDITORS-PRO's `effects/noise_reduction.rs` has 4 methods: **Bilateral** (edge-preserving, fast), **Wiener** (frequency-domain, good for fine noise), **NLM** (non-local means, best quality, slow), **Temporal** (uses frame-to-frame coherence, excellent for static shots). The Flutter `noise_reduction_panel.dart` widget exposes them.

The tradeoff: more reduction = more detail loss. The amateur move is to crank NR to 100% and get a plastic, wax-figure look. The pro move is to NR just enough to clean the noise, then add a tiny bit of grain back to break up the plasticity.

## When to use

Low-light footage with visible noise. Underexposed footage that's been pushed up. High-ISO footage.

## When NOT to use

On clean footage — NR on clean footage just softens detail. On footage where the noise is the aesthetic (e.g., Super 8 emulation).

## Examples

**Amateur** ❌: NR at 100% with NLM. Result: subject looks like a wax figure, no skin texture, no detail.

**Professional** ✅: Method by shot type: Temporal for static, Bilateral for motion. Strength at 30-50%. Add a tiny bit of grain back to break up the plasticity. Verify with a skin close-up.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Bilateral at 40%. Fast. Acceptable for social. |
| `full` | Temporal for static shots, Bilateral for motion. Strength 30-50%. Add grain back. Verify on skin close-up. |
| `ultra` | Per-shot method choice. NLM for the worst shots. Luma/chroma separation. Document in QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Noise reduction covers video NR. It does not cover audio NR (dialogue-cleanup) or the grain addition (film-grain-recipe).
