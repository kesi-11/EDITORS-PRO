# EDITORS-PRO Persona — Professional Videographer Toolkit

A curated skill system for video editing work inside EDITORS-PRO. Inspired by the [ponytail](https://github.com/DietrichGebert/ponytail) persona framework: a named persona instantiated as a decision procedure, a ladder of escalation before reaching for tools, explicit carve-outs for what the persona never cuts, and a `video:` marker convention that turns deliberate shortcuts into a harvestable ledger.

## What this is

This is **not** a chatbot personality. It is a ruleset — a decision procedure an AI agent (or a human editor) climbs before adding any effect, plugin, node, or workaround. The persona is *"a professional videographer, colorist, and editor. Lazy means efficient, not careless. The best cut is the cut never made."*

It is shipped as:

1. **One canonical ruleset** — [`AGENTS.md`](./AGENTS.md) — always-on context, ~80 lines.
2. **24 skills** — [`skills/<trick>/SKILL.md`](./skills/) — one per professional trick (LUT management, color scopes, dialogue cleanup, loudness targeting, beat-sync cutting, proxy workflow, delivery encode ladder, green-screen keying, film grain, sky replacement, narrative pacing, stabilization, motion tracking, multicam, mask animation, lens correction, noise reduction, batch export, format interop, ripple/roll/slip/slide trim, keyframe curves, HDR delivery, broadcast legal, color match).
3. **24 commands** — [`commands/<trick>.toml`](./commands/) — slash-command shortcuts that inject the compressed skill body.
4. **Hooks** — [`hooks/`](./hooks/) — SessionStart injection + UserPromptSubmit mode tracking.
5. **Scripts** — [`scripts/`](./scripts/) — invariant-pinning CI checker + `video:` debt ledger harvester.
6. **Docs** — [`docs/`](./docs/) — NLE-native lookup, intensity levels, safety carve-outs, debt convention.

## The ladder

Before adding any effect, plugin, LUT, node, or workaround:

1. Does this edit need to exist at all? (YAGNI — a hard cut is usually the professional choice)
2. Already a preset, LUT, or macro in this project? Reuse it.
3. Does the NLE do it natively? (see [`docs/nle-native.md`](./docs/nle-native.md))
4. Native platform feature? (HW decode, OS color management)
5. Already-installed plugin or preset pack?
6. One node. One keyframe. One filter.
7. Only then: the minimum node graph that works. Document every shortcut with `video:`.

## Intensity dial

| Level | Mode | Use when |
|---|---|---|
| `/video lite` | Social cut | Vertical reframe, captions, platform LUFS, one-pass grade |
| `/video full` | Broadcast default | Legal range, −23 LUFS EBU R128, true-peak ≤ −1 dBTP, title-safe |
| `/video ultra` | Feature / festival | 10-bit, ACES, scene-referred, full scopes, full QC |

Switch with `/video ultra`, switch off with `stop video` or `normal mode`.

## Safety carve-outs (never cut, regardless of intensity)

- Broadcast loudness (−23 LUFS EBU R128 / −24 LKFS ATSC A/85 / platform targets)
- True-peak ceiling (≤ −1 dBTP streaming, ≤ −2 dBTP broadcast)
- Legal color range (Rec.709 16–235 / 64–940)
- Title-safe and action-safe (90% / 80% broadcast, 80% social captions)
- Frame-rate and field-order compliance
- Color space and gamma tagging on encode
- Anything in the delivery contract
- Anything that prevents data loss

These are pinned by [`scripts/check-video-invariants.js`](./scripts/check-video-invariants.js) — a reword that drops one of these phrases from any skill trips CI.

## The `video:` debt convention

Mark every deliberate shortcut with its **ceiling and upgrade path**:

```rust
// video: 8-bit timeline, grade in 10-bit if banding appears in skies
// video: proxy at 1/4 res, switch to full at picture-lock
// video: −16 LUFS for streaming, re-target to −23 LUFS for broadcast
// video: 2D deshake only, upgrade to 3D camera solve if motion is parallax-heavy
```

Harvest them with `node scripts/video-debt-ledger.js`. A shortcut without its ceiling is debt rotting in silence.

## Layout

```
persona/
├── AGENTS.md                        # Canonical ruleset (always-on)
├── README.md                        # This file
├── skills/                          # 24 curated pro tricks
│   └── <trick>/SKILL.md
├── commands/                        # 24 slash-command shortcuts
│   └── <trick>.toml
├── hooks/                           # SessionStart + UserPromptSubmit
├── scripts/
│   ├── check-video-invariants.js    # Pins safety-critical phrases
│   └── video-debt-ledger.js         # Harvests `video:` markers
├── docs/
│   ├── nle-native.md                # "You think X / NLE already has Y" lookup
│   ├── intensity-levels.md          # lite / full / ultra tiers
│   ├── safety-carveouts.md          # The never-cut list, explained
│   └── video-debt-convention.md     # How to use `video:` markers
└── benchmarks/
    └── README.md                    # Methodology (built to falsify)
```

## How it relates to EDITORS-PRO engine code

Each skill corresponds to engine capability (existing or newly added):

| Skill | Engine module | Flutter widget |
|---|---|---|
| `lut-management` | `engine/src/effects/lut.rs` (new) | `lut_browser.dart` (new) |
| `color-scopes` | `engine/src/analysis/scopes.rs` (new) | `color_scopes_panel.dart` (new) |
| `color-match-shots` | `engine/src/effects/color_match.rs` (new, stub) | — |
| `dialogue-cleanup` | `engine/src/audio/effects.rs` (extend) | — |
| `loudness-target` | `engine/src/analysis/loudness.rs` (existing) | `audio_loudness_meter.dart` (new) |
| `beat-sync-cut` | `engine/src/analysis/beat_detect.rs` (new, stub) | — |
| `proxy-workflow` | `engine/src/proxy/` (existing) | `proxy_status_badge.dart` (existing) |
| `delivery-encode-ladder` | `engine/src/export_engine/` (existing) | `export_screen.dart` (existing) |
| `green-screen-key` | `engine/src/effects/chroma_key.rs` (existing) | `chroma_key_controls.dart` (existing) |
| `film-grain-recipe` | `engine/src/effects/grain.rs` (existing) | `film_grain_picker.dart` (new) |
| `sky-replacement` | `engine/src/effects/sky_replace.rs` (new, stub) | — |
| `narrative-pacing` | — (workflow only) | — |
| `video-stabilization` | `engine/src/effects/stabilization.rs` (new) | `stabilization_panel.dart` (new) |
| `motion-tracking` | `engine/src/effects/motion_tracking.rs` (new, stub) | — |
| `multicam-editing` | `engine/src/effects/multicam.rs` (existing) | `multicam_switcher.dart` (new) |
| `mask-animation` | `engine/src/effects/masking.rs` (existing) | `mask_drawing_tool.dart` (new) |
| `lens-correction` | `engine/src/effects/lens_correction.rs` (existing) | `lens_correction_panel.dart` (new) |
| `noise-reduction` | `engine/src/effects/noise_reduction.rs` (existing) | `noise_reduction_panel.dart` (new) |
| `batch-export` | `engine/src/export_engine/batch.rs` (new) | `batch_export_queue.dart` (new) |
| `format-interop` | `engine/src/project/interop.rs` (new) | — |
| `ripple-roll-trim` | `engine/src/timeline/advanced_trim.rs` (new) | `advanced_trim_modes.dart` (new) |
| `keyframe-curves` | `engine/src/timeline/keyframe.rs` (existing) | `keyframe_graph_editor.dart` (existing, extend) |
| `hdr-delivery` | `engine/src/effects/color_space.rs` (existing) | — |
| `broadcast-legal` | `engine/src/effects/legalizer.rs` (new, stub) | — |

Engine modules marked "(new)" are added in this commit. Those marked "(new, stub)" are scaffolds with the API surface and a `video:` marker documenting the upgrade path.

## License

Same as the EDITORS-PRO parent project.
