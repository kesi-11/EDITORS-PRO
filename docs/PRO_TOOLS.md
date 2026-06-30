# Pro Tools — Phase F (Persona-Driven Videographer Toolkit)

Phase F adds a curated professional videographer toolkit to EDITORS-PRO, inspired by the [ponytail](https://github.com/DietrichGebert/ponytail) persona framework. It combines three layers:

1. **11 new Rust engine modules** — pro features that were either missing entirely (LUT, scopes, stabilization, motion tracking, batch export, format interop, advanced trim, legalizer) or only stubs (sky replacement, color match, beat detection).
2. **11 new Flutter UI panels** — exposing the engine-only pro modules (multicam, masking, lens correction, grain, noise reduction, markers) and the new modules above.
3. **A 24-skill persona system** at `persona/` — organizing everything a videographer needs as a curated, invariants-pinned skill system, with a `video:` debt marker convention, a CI checker, and an intensity dial.

## Why a persona?

Ponytail's insight is that a "persona" is **not a chatbot personality** — it's a decision procedure. The persona is a named character ("a lazy senior developer") instantiated as a ruleset the agent climbs before writing code. The ruleset has:

- A **ladder** of escalation (YAGNI → reuse → stdlib → ... → minimum viable).
- **Explicit carve-outs** for what the persona never cuts (security, accessibility, data loss prevention).
- A **`<word>:` comment convention** that turns deliberate shortcuts into harvestable debt.
- **Intensity dial** (lite / full / ultra) for the same ruleset at different enforcement postures.
- **Invariant pinning** via a CI checker so safety-critical phrases can't be silently reworded out.

Phase F transplants this architecture to video editing. The persona is *"a professional videographer, colorist, and editor. Lazy means efficient, not careless. The best cut is the cut never made. The best node is the node never added."*

## The 7-rung ladder

Before adding any effect, plugin, LUT, node, or workaround, climb the ladder — cheapest rung first:

1. **Does this edit need to exist at all?** A hard cut is usually the professional choice. Transitions are punctuation, not decoration.
2. **Already a preset, LUT, or macro in this project?** Reuse it.
3. **Does the NLE do it natively?** See [`persona/docs/nle-native.md`](../persona/docs/nle-native.md).
4. **Native platform feature?** HW decode, OS color management, GPU-composited preview — these are free.
5. **Already-installed plugin or preset pack?** Use what's paid for.
6. **One node. One keyframe. One filter.** The minimum that achieves the look.
7. **Only then:** the minimum node graph that works. Document every shortcut with a `video:` marker.

## The never-cut list

These are pinned by [`persona/scripts/check-video-invariants.js`](../persona/scripts/check-video-invariants.js). A reword that drops one of these phrases from any skill trips CI:

- Broadcast loudness (−23 LUFS EBU R128, −24 LKFS ATSC A/85, platform targets)
- True-peak ceiling (≤ −1 dBTP streaming, ≤ −2 dBTP broadcast)
- Legal color range (Rec.709 16–235 / 64–940)
- Title-safe (90% broadcast, 80% social captions)
- Frame-rate and field-order compliance
- Color space and gamma tagging
- Delivery spec compliance
- Data loss prevention
- Stabilization ceiling (2D deshake → 3D camera solve upgrade path)

## The 24 skills

| # | Skill | Engine module | Flutter widget |
|---|---|---|---|
| 1 | lut-management | `effects/lut.rs` | `lut_browser.dart` |
| 2 | color-scopes | `analysis/scopes.rs` | `color_scopes_panel.dart` |
| 3 | color-match-shots | `effects/color_match.rs` | — |
| 4 | dialogue-cleanup | `audio/effects.rs` (extend) | — |
| 5 | loudness-target | `analysis/loudness.rs` | `audio_loudness_meter.dart` |
| 6 | beat-sync-cut | `analysis/beat_detect.rs` | — |
| 7 | narrative-pacing | — (workflow) | — |
| 8 | proxy-workflow | `proxy/` (existing) | `proxy_status_badge.dart` |
| 9 | delivery-encode-ladder | `export_engine/` (existing) | `export_screen.dart` |
| 10 | green-screen-key | `effects/chroma_key.rs` (existing) | `chroma_key_controls.dart` |
| 11 | film-grain-recipe | `effects/grain.rs` (existing) | `film_grain_picker.dart` |
| 12 | sky-replacement | `effects/sky_replace.rs` | — |
| 13 | video-stabilization | `effects/stabilization.rs` | `stabilization_panel.dart` |
| 14 | motion-tracking | `effects/motion_tracking.rs` | — |
| 15 | multicam-editing | `effects/multicam.rs` (existing) | `multicam_switcher.dart` |
| 16 | mask-animation | `effects/masking.rs` (existing) | — |
| 17 | lens-correction | `effects/lens_correction.rs` (existing) | `lens_correction_panel.dart` |
| 18 | noise-reduction | `effects/noise_reduction.rs` (existing) | `noise_reduction_panel.dart` |
| 19 | batch-export | `export_engine/batch.rs` | `batch_export_queue.dart` |
| 20 | format-interop | `project/interop.rs` | — |
| 21 | ripple-roll-trim | `timeline/advanced_trim.rs` | `advanced_trim_modes.dart` |
| 22 | keyframe-curves | `timeline/keyframe.rs` (existing) | `keyframe_graph_editor.dart` |
| 23 | hdr-delivery | `effects/color_space.rs` (existing) | — |
| 24 | broadcast-legal | `effects/legalizer.rs` | — |

Each skill lives at `persona/skills/<name>/SKILL.md` with the ponytail anatomy:
- YAML frontmatter (name, description with trigger phrases, license)
- `## The trick` — what the trick actually is
- `## When to use` / `## When NOT to use` — explicit carve-outs
- `## Examples` — amateur ❌ vs professional ✅
- `## Intensity` — lite / full / ultra enforcement
- `## Safety carve-outs` — the pinned phrases for this skill
- `## Boundaries` — explicit scope limit

## The `video:` debt convention

Every deliberate shortcut is marked with its **ceiling and upgrade path**:

```rust
// video: 8-bit timeline, grade in 10-bit if banding appears in skies
// video: proxy at 1/4 res, switch to full at picture-lock
// video: −16 LUFS for streaming, re-target to −23 LUFS for broadcast
// video: 2D deshake only, upgrade to 3D camera solve if motion is parallax-heavy
```

A shortcut without its ceiling is debt rotting in silence. Harvest with:

```bash
node persona/scripts/video-debt-ledger.js --write
```

Output: `persona/DEBT.md` — a categorized ledger of every `video:` marker in the codebase.

## Intensity dial

| Level | Use when |
|---|---|
| `/video lite` | Social cut — vertical, captions, platform LUFS, one-pass grade |
| `/video full` | Broadcast default — legal range, −23 LUFS EBU R128, true-peak ≤ −1 dBTP, title-safe |
| `/video ultra` | Feature / festival grade — 10-bit, ACES, scene-referred, full scopes, full QC |

Switch with `/video ultra`. Switch off with `stop video` or `normal mode`. The level persists in `.video-active`.

## CI integration

Add to your GitHub Actions workflow:

```yaml
- name: Verify video persona invariants
  run: node persona/scripts/check-video-invariants.js

- name: Generate video debt ledger
  run: node persona/scripts/video-debt-ledger.js --write
```

The first fails CI if any pinned safety phrase is missing. The second regenerates the debt ledger (doesn't fail CI).

## How to extend

### Add a new skill

1. Create `persona/skills/<your-skill>/SKILL.md` following the ponytail anatomy.
2. Create `persona/commands/<your-skill>.toml` as a slash-command shortcut.
3. Add the safety-critical phrases your skill pins to `persona/scripts/check-video-invariants.js`.
4. Run `node persona/scripts/check-video-invariants.js` to verify.
5. Document any engine module + Flutter widget mapping in `persona/README.md`.

### Add a new `video:` debt marker

Anywhere in the codebase:

```rust
// video: <shortcut>, <ceiling / when-to-upgrade>, <upgrade path>
```

Then run `node persona/scripts/video-debt-ledger.js --write` to refresh `persona/DEBT.md`.

### Pay down debt

When you upgrade a `video:` shortcut, **delete the marker**. The ledger shrinks. The CI check stays green. The delivery gets safer.
