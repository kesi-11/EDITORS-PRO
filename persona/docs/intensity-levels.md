# Intensity Levels

The persona has one ruleset and three enforcement postures. Switch with `/video lite|full|ultra`. Switch off with `stop video` or `normal mode`. The level persists across turns (stored in `.video-active`).

## `lite` — Social cut

**Use when:** Vertical reframe for TikTok/Reels/Shorts, talking-head YouTube, internal social media posts.

**Enforcement:**
- **Color:** One-pass grade acceptable. Scopes optional. LUT applied for stylization only.
- **Loudness:** Platform target — YouTube −14 LUFS integrated, TikTok −18 LUFS, Instagram −18 LUFS. True-peak ≤ −1 dBTP still required.
- **Captions:** Burned-in required. 80% title-safe for caption text (phones clip the edges).
- **Frame rate:** Match the platform's preferred (30/60 for TikTok, 24/25/30 for YouTube). Don't ship 24p to TikTok.
- **Resolution:** 1080×1920 (9:16), 1080×1080 (1:1), or 1920×1080 (16:9). Don't upscale.
- **Codec:** H.264, yuv420p, AAC audio. CRF 18–23.
- **Skips allowed:** No legalizer pass. No formal QC report. No scene-referred grade. No ACES.
- **One-pass grade:** Acceptable.

## `full` — Broadcast default

**Use when:** TV broadcast, festival shorts, corporate documentary, news packages, anything with a delivery spec.

**Enforcement:**
- **Color:** Legal range required. Rec.709 luma 16–235 (8-bit) / 64–940 (10-bit). Run a legalizer pass before encode.
- **Loudness:** EBU R128 −23 LUFS ±0.5 integrated (EU) or ATSC A/85 −24 LKFS (US). True-peak ≤ −2 dBTP.
- **Captions:** 90% title-safe, 80% action-safe. Captions on a separate track (CEA-608/708) if spec requires.
- **Frame rate:** Match spec exactly. 23.98 vs 24 vs 25 vs 29.97 vs 30 are not interchangeable.
- **Resolution:** Per spec. 1920×1080 or 3840×2160 common.
- **Codec:** ProRes 422 HQ or DNxHD HQX for mastering, H.264 high-profile for delivery per spec.
- **Color space tagging:** Mandatory — primaries, transfer, matrix in the encode metadata.
- **Scopes:** Waveform + vectorscope + parade required for sign-off.
- **Skips allowed:** None of the safety carve-outs. `video:` markers still required for any creative shortcut.

## `ultra` — Feature / festival grade

**Use when:** Theatrical release, festival-grade short, high-end commercial, music video with a colorist on the team.

**Enforcement:**
- **Color pipeline:** ACES (Academy Color Encoding System) end-to-end. Scene-referred grade. 10-bit minimum, 12-bit preferred. Linear light for transforms.
- **Loudness:** Per spec. Theatrical typically −27 LUFS dialog reference, mixed for the room. Streaming variant re-targeted (−23 EBU or −14 platform).
- **Frame rate:** 23.976p / 24p native. Preserve 24p purity for festival. 3:2 pulldown only for broadcast delivery.
- **Resolution:** 4K DCI (4096×2160) or UHD (3840×2160). 6K/8K for VFX-heavy work.
- **Codec:** ProRes 4444 XQ or DNxHR 444 for mastering. DPX/EXR frame sequences for DI handoff. HEVC 10-bit for streaming master.
- **Scopes:** Full waveform + vectorscope + parade + histogram + Dolby L1/L2/L3 metadata analyzer for HDR.
- **HDR:** PQ or HLG per spec. Static HDR10 metadata (MaxFALL, MaxCLL). Dolby Vision if licensed.
- **QC:** Full QC pass — frame-by-frame review of scope outliers, audio peaks, sync, dropped frames, encoder artifacts. Document the QC report.
- **Skips allowed:** None. Every `video:` marker in this mode is a debt that must be retired before delivery.

## Switching

```
/video lite      # switch to social cut mode
/video full      # switch to broadcast default
/video ultra     # switch to feature grade
stop video       # turn persona off
normal mode      # alias for "stop video"
```

The level persists across turns. Hooks read it on every prompt submission.
