# The `video:` Debt Convention

Inspired by ponytail's `ponytail:` marker convention. Every deliberate shortcut is marked with its **ceiling and upgrade path**, in a comment next to the shortcut itself.

## Why

"Later" is a lie. "Later" becomes "never" becomes "the deliverable fails QC at 2am." A `video:` marker turns "later" into a tracked artifact — a debt that can be harvested, listed, prioritized, and paid down before it rots.

## Syntax

```
// video: <shortcut description>, <ceiling / when-to-upgrade>
# video: <shortcut description>, <ceiling / when-to-upgrade>
```

Use `//` for Rust/Dart/JS/TS/C++. Use `#` for Python/YAML/Shell/TOML.

## Examples

```rust
// video: 8-bit timeline, grade in 10-bit if banding appears in skies
let lut = lut::Lut::from_cube_8bit(&cube_data)?;
```

```rust
// video: proxy at 1/4 res, switch to full at picture-lock
let preview_frame = renderer.render_proxy_frame(time_ms)?;
```

```rust
// video: −16 LUFS for streaming, re-target to −23 LUFS for broadcast
let target_lufs = -16.0;
```

```rust
// video: 2D deshake only, upgrade to 3D camera solve if motion is parallax-heavy
let stabilized = stabilization::deshake_2d(&frames, smoothing=0.8)?;
```

```dart
// video: single-pass grade, upgrade to node-based if more than 3 corrections needed
colorGrade.apply(preview);
```

```rust
// video: stub motion tracker (centroid only), upgrade to KLT or planar tracker for production
let track = motion_tracking::track_point(&frame, start_xy)?;
```

## The three required parts

1. **What the shortcut is** — "8-bit timeline", "2D deshake", "centroid tracker".
2. **The ceiling** — when this shortcut stops being acceptable. "If banding appears", "if motion is parallax-heavy", "if more than 3 corrections needed".
3. **The upgrade path** — what to upgrade to. "Grade in 10-bit", "3D camera solve", "node-based grade", "KLT or planar tracker".

A shortcut without its ceiling is debt rotting in silence.

## Harvesting

Run [`scripts/video-debt-ledger.js`](../scripts/video-debt-ledger.js) to scan the codebase and produce a ledger:

```bash
node persona/scripts/video-debt-ledger.js
```

Output (to stdout, also saved to `persona/DEBT.md` if `--write` is passed):

```
# EDITORS-PRO video: Debt Ledger

Generated: 2026-06-23T14:07:00Z
Total markers: 12
Files affected: 7

## engine/src/effects/stabilization.rs
- L23: 2D deshake only, upgrade to 3D camera solve if motion is parallax-heavy
- L41: per-frame block matching, upgrade to multi-pass with pyramidal refinement if motion is fast

## engine/src/audio/effects.rs
- L107: low-pass only, upgrade to full EQ rack if dialogue cleanup is needed
- L112: no de-esser, upgrade to MB-style de-esser if sibilance is audible
...
```

## When NOT to add a `video:` marker

- For shortcuts that are intentional forever (e.g., "we will never support interlaced export" — that's a product decision, not a debt).
- For trivial choices that have no upgrade path (e.g., "use sRGB for icons").
- For comments that are just explanations, not deferred work.

If there's no ceiling, there's no debt. Don't pollute the ledger with non-debt comments.

## Paying down debt

When you upgrade a `video:` shortcut, delete the marker. The ledger shrinks. The CI check stays green. The delivery gets safer.
