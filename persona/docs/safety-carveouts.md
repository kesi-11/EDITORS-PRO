# Safety Carve-outs — The Never-Cut List

These are enumerated things the persona refuses to skip regardless of intensity level. They are pinned by [`scripts/check-video-invariants.js`](../scripts/check-video-invariants.js) — a reword that drops one of these phrases from any skill file trips CI.

The discipline: **safety-critical phrases must appear verbatim across the canonical ruleset and every skill that touches them.** A helpful-seeming edit to one file cannot silently delete a delivery-spec requirement from another.

## The list

### 1. Broadcast loudness

- **EBU R128:** −23 LUFS ±0.5 integrated, −18 LUFS short-term max, −1 dBTP true-peak max. EU broadcast.
- **ATSC A/85:** −24 LKFS integrated, ±2 dB tolerance. US broadcast (CALM Act).
- **AGCOM 219/09:** −24 LKFS, Italy.
- **Streaming general:** YouTube −14 LUFS, TikTok −18 LUFS, Spotify −14 LUFS, Apple Music −14 LUFS, Tidal −14 LUFS, Amazon Music −14 LUFS.
- **Podcast:** −16 LUFS (Apple Podcasts recommendation), −1 dBTP.

Never ship a deliverable without measuring. The number is the contract, not your ear.

**Pinned phrase:** `−23 LUFS` (and platform targets as listed above).

### 2. True-peak ceiling

- ≤ −1 dBTP for streaming (iTunes, Spotify, YouTube, TikTok).
- ≤ −2 dBTP for broadcast (EBU R128, ATSC A/85).
- Intersample peaks kill encoders. A peak sample at −0.1 dBFS can produce an inter-sample peak above 0 dBTP after DAC reconstruction.

**Pinned phrase:** `dBTP`.

### 3. Legal color range

- Rec.709 SDI/broadcast: luma 16–235 (8-bit), 64–940 (10-bit). Chroma 16–240 (8-bit), 64–960 (10-bit).
- Full-range RGB 0–255 is for web/JPEG/computer-graphics only. Never for broadcast.
- Run a legalizer pass (clamps to legal range with optional soft-clip) before encoding for broadcast.

**Pinned phrase:** `legal range`.

### 4. Title-safe and action-safe

- **Broadcast:** 90% title-safe (text inside), 80% action-safe (essential content inside). Per SMPTE/EBU.
- **Social:** 80% safe area for captions and essential text — phones clip the edges, and the platform UI overlays the bottom and right.
- **Vertical social:** Keep all essential content in the center 9:16 80% safe area. Bottom 20% is covered by platform UI.

**Pinned phrase:** `title-safe`.

### 5. Frame-rate and field-order compliance

- **23.976p** vs **24p**: not interchangeable. 23.976 is the broadcast-friendly NTSC pull-down of 24. Festivals often require 24p native.
- **25p** for PAL territories.
- **29.97i** / **59.94i**: interlaced with field-order dominance. Preserve BFF (bottom-field-first) or TFF (top-field-first) per spec. Wrong field order = juddering motion.
- **30p** vs **29.97p**: 30p is rare; 29.97p is the NTSC progressive.
- **50i** / **60i**: PAL/NTSC interlaced broadcast.
- A wrong frame rate is a failed delivery, not a stylistic choice.

**Pinned phrase:** `frame-rate` and `field-order`.

### 6. Color space and gamma tagging

- Tag the encode with the right color primaries (Rec.709, Rec.2020, P3-D65), transfer function (BT.1886, sRGB, PQ, HLG), and matrix (BT.709, BT.2020 NCL/CL).
- Untagged sRGB on a Rec.709 deliverable is a defect — the player will misinterpret the color.
- For HDR: master display, MaxFALL, MaxCLL must be embedded in the bitstream (HEVC SEI).

**Pinned phrase:** `color space`.

### 7. Delivery contract

Anything explicitly specified in the delivery contract is non-negotiable. If the spec says ProRes 422 HQ, do not ship ProRes 422 LT to save space. If it says 48 kHz audio, do not ship 44.1. If it says 16:9, do not ship 2:1. The spec is the contract.

**Pinned phrase:** `delivery spec`.

### 8. Data loss prevention

- Project backups before destructive ops (delete, ripple-delete, replace).
- Source media preserved — never overwrite originals.
- Snapshot before color or audio changes that affect the whole timeline.
- Version history retained per the team's policy.

**Pinned phrase:** `data loss`.

## How the invariant checker works

`scripts/check-video-invariants.js` scans every `skills/**/SKILL.md` and `AGENTS.md` for the pinned phrases. If a skill claims to touch loudness but the phrase `−23 LUFS` doesn't appear in either that skill or AGENTS.md, the check fails. This is the analog of ponytail's `check-rule-copies.js` — a reword can't silently drop a carve-out.

Run locally:

```bash
node persona/scripts/check-video-invariants.js
```

Exits 0 if all invariants hold, 1 otherwise. Output is human-readable.
