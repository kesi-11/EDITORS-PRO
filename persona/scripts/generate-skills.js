#!/usr/bin/env node
/**
 * generate-skills.js
 *
 * Generates the 24 SKILL.md files in persona/skills/ and the matching
 * 24 commands/*.toml slash-command shortcuts. Each skill follows the
 * ponytail anatomy: YAML frontmatter (name, description, license) →
 *   # Title
 *   ## The trick
 *   ## When to use
 *   ## When NOT to use
 *   ## Examples (amateur vs pro)
 *   ## Intensity (lite/full/ultra)
 *   ## Safety carve-outs (never cut)
 *   ## Boundaries
 *
 * The descriptions are trigger-rich (they list the natural-language
 * phrases that should fire the skill) — that's how a host's skill-picker
 * matches.
 *
 * Run:  node scripts/generate-skills.js
 */

const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const SKILLS_DIR = path.join(ROOT, 'skills');
const COMMANDS_DIR = path.join(ROOT, 'commands');

/**
 * Skill definitions. Each entry produces one SKILL.md and one .toml.
 *
 * Fields:
 *   name        — directory name (kebab-case)
 *   title       — human title
 *   triggers    — array of phrases that should fire the skill
 *   trick       — what the trick actually is (1-2 paragraphs)
 *   whenToUse   — when to apply
 *   whenNotToUse — when NOT to apply (the carve-out)
 *   amateur     — what an amateur does (the ❌)
 *   pro         — what a pro does (the ✅)
 *   intensity   — { lite, full, ultra } enforcement posture
 *   safety      — array of pinned safety phrases (must appear verbatim;
 *                  these are checked by check-video-invariants.js)
 *   boundaries  — explicit scope limit
 *   command     — the prompt body for the .toml slash command (compressed)
 */
const SKILLS = [
  // ─── COLOR ──────────────────────────────────────────────────────────────
  {
    name: 'lut-management',
    title: 'LUT Management',
    triggers: ['apply lut', 'cube file', '3dl', 'creative lut', 'technical lut', 'conversion lut', 'look lut', 'color preset', 'film emulation lut'],
    trick: `A LUT (Look-Up Table) is a precomputed color transformation. Two kinds: **technical** (color space conversion — e.g., Log→Rec.709) and **creative** (a stylized look — e.g., film emulation). EDITORS-PRO's \`effects/lut.rs\` supports the .cube format (1D and 3D) and .3dl. Apply a technical LUT first to put log footage into a working color space, then grade on top, then optionally apply a creative LUT last for delivery.

A LUT is a frozen grade. Don't use a LUT to do what a node graph should do — LUTs are for transformations you want to apply identically across many clips, or for transformations authored externally (e.g., a camera manufacturer's Log→Rec.709 LUT). For per-shot work, grade directly.`,
    whenToUse: 'You have log footage and need to convert to Rec.709. You have a creative look you want to apply across the whole timeline. You received a delivery LUT from a colorist. You want to emulate a film stock.',
    whenNotToUse: 'For per-shot grading — use the color wheels instead. For transformations that depend on clip content (e.g., "make this shot match that shot") — use shot match instead. For anything that needs to be tweaked per-clip — a LUT is immutable.',
    amateur: 'Stacking 3 creative LUTs on top of each other, each at 50% opacity, hoping one of them looks right. Or applying a "cinematic" LUT you downloaded from YouTube to log footage without first converting to Rec.709 — the result is washed-out shadows and crushed highlights.',
    pro: 'Node graph: (1) Color Space Transform from camera Log to Rec.709 (technical LUT or CST node), (2) primary grade (lift/gamma/gain), (3) secondary grade (qualifier + power window), (4) optional creative LUT at 80-100% mix for stylization. Export the LUT from the grade if you want to reuse it across clips.',
    intensity: {
      lite: 'One creative LUT at 100% over a one-pass grade. Acceptable for social.',
      full: 'Technical LUT first (Log→Rec.709), then grade, then creative LUT last. LUT intensity ≤ 100%. Verify legal range after LUT.',
      ultra: 'LUT pipeline is 32-bit float, applied in scene-referred linear. Creative LUTs authored in the DI suite, not downloaded. Every LUT applied must pass legal range check before encode.',
    },
    safety: ['legal range', 'color space', 'delivery spec'],
    boundaries: 'LUT management covers the application, authoring, and import/export of .cube and .3dl files. It does not cover color grading itself (that\'s the color-scopes and color-match-shots skills) or color space conversion (that\'s the broadcast-legal and hdr-delivery skills).',
    command: `Apply LUT management. Load the .cube/.3dl file via effects/lut.rs. Technical LUT first (Log→Rec.709), then grade, then creative LUT last. Verify legal range after LUT. Never stack multiple creative LUTs — grade in the node graph instead. Mark any LUT shortcut with a \`video:\` comment and its ceiling.`,
  },
  {
    name: 'color-scopes',
    title: 'Color Scopes (Waveform, Vectorscope, RGB Parade, Histogram)',
    triggers: ['scopes', 'waveform', 'vectorscope', 'rgb parade', 'histogram', 'luma scope', 'color analyzer', 'read the scope', 'crushed blacks', 'clipped highlights'],
    trick: `Scopes tell you what's actually in the image — your eyes lie, your monitor lies, the room lighting lies. The waveform shows luma distribution across the frame (X = horizontal position, Y = luma value). The vectorscope shows chroma distribution (angle = hue, distance from center = saturation). The RGB parade shows three separate waveforms for R, G, B — used for white balance and color cast. The histogram shows the distribution of luma values across the whole frame.

EDITORS-PRO's \`analysis/scopes.rs\` computes all four from any frame. The Flutter \`color_scopes_panel.dart\` widget renders them live in the editor. Use scopes to verify: blacks aren't crushed (waveform bottom ≥ 16), highlights aren't clipped (waveform top ≤ 235), white balance is neutral (RGB parade lines up), skin tones are on the I-line (vectorscope).`,
    whenToUse: 'Always, in full and ultra mode. Before signing off any deliverable. When matching shots. When grading log footage. When the room lighting is unreliable.',
    whenNotToUse: 'In lite mode for a one-pass social grade — scopes are optional. When your eyes are calibrated to a reference monitor in a graded room and you trust them — but verify with scopes before sign-off anyway.',
    amateur: 'Grading by eye on an uncalibrated laptop screen in a bright room, then being surprised when the deliverable looks wrong on the client\'s monitor.',
    pro: 'Scopes open on a second monitor. Waveform set to Y (luma). Vectorscope with 75% target. RGB parade for white balance. Grade to the scopes, verify with eyes, sign off with scopes. Skin tones land on the vectorscope I-line (flesh tone line, ~123° from R).',
    intensity: {
      lite: 'Optional. A waveform check before export is enough.',
      full: 'Required. Waveform + vectorscope before sign-off. Verify legal range (16–235 luma) on the waveform.',
      ultra: 'Required at every stage. Scopes pinned on a dedicated monitor. RGB parade for white balance, vectorscope for skin tone I-line, waveform for legal range, histogram for distribution. Sign-off documented in the QC report.',
    },
    safety: ['legal range', 'color space', 'vectorscope', 'waveform'],
    boundaries: 'Color scopes cover reading and interpreting the scopes. They do not cover grading decisions (that\'s color-match-shots and lut-management) or legal range enforcement (that\'s broadcast-legal).',
    command: `Open color scopes. Waveform (Y) for luma distribution, vectorscope for chroma, RGB parade for white balance, histogram for distribution. Verify: blacks ≥ 16, highlights ≤ 235 (8-bit) or 64/940 (10-bit), skin tones on I-line. In full/ultra: scopes required before sign-off. Don't grade by eye alone.`,
  },
  {
    name: 'color-match-shots',
    title: 'Shot Matching (Color Match)',
    triggers: ['match shots', 'shot match', 'color match', 'match this shot', 'consistency', 'continuity of color', 'auto-match', 'reference frame'],
    trick: `Shot matching makes two clips shot in different conditions look like they were shot in the same conditions. Three layers: (1) **white balance match** — neutralize each shot, (2) **exposure match** — align luma midpoints, (3) **color match** — align hue/saturation in shadows/mids/highlights. EDITORS-PRO's \`effects/color_match.rs\` provides histogram-based matching to a reference frame.

The amateur move is to slap a LUT on and hope. The pro move is to match each shot to a reference frame (usually a clean middle-shot of the scene), then apply a creative LUT last across the whole timeline for stylization.`,
    whenToUse: 'When two clips cut together and the color/exposure doesn\'t match. When establishing a scene. When footage came from multiple cameras or multiple days.',
    whenNotToUse: 'When the shots are intentionally different (e.g., day-for-night vs day scene). When the mismatch is in the white balance and you can fix it with one temperature slider — don\'t over-engineer.',
    amateur: 'Trying to fix mismatches by stacking creative LUTs on each shot. The result is six different "cinematic" looks cutting back and forth.',
    pro: 'Pick a reference frame. Match each shot to it: white balance first, exposure second, color third. Apply a creative LUT last, uniformly across the timeline. Verify with scopes.',
    intensity: {
      lite: 'Auto-match to a reference frame is fine. Don\'t spend more than 30 seconds per shot.',
      full: 'Match each shot manually to the reference. Verify with vectorscope (skin tones on I-line) and waveform (luma midpoint).',
      ultra: 'Scene-referred match in ACES. Per-channel match. Document the reference frame in the QC report.',
    },
    safety: ['color space', 'legal range', '.cube'],
    boundaries: 'Shot matching covers making clips look consistent. It does not cover grading a single shot (color-scopes), LUT management (lut-management), or creative stylization (film-grain-recipe).',
    command: `Match shots. Pick reference frame. Match white balance, then exposure, then color. Apply creative LUT last, uniformly. Verify with scopes. In ultra: scene-referred in ACES, per-channel match.`,
  },
  // ─── AUDIO ──────────────────────────────────────────────────────────────
  {
    name: 'dialogue-cleanup',
    title: 'Dialogue Cleanup',
    triggers: ['clean dialogue', 'remove noise', 'denoise voice', 'de-ess', 'de-reverb', 'remove echo', 'remove room sound', 'clean up audio', 'fix audio', 'remove hiss', 'remove hum'],
    trick: `Dialogue cleanup is the difference between "professional" and "amateur." Stages: (1) **noise reduction** — broadband hiss, AC, room tone, (2) **de-reverb** — remove room echo, (3) **de-essing** — tame sibilance, (4) **EQ** — remove low-end rumble, brighten for intelligibility, (5) **compression** — even out dynamics, (6) **limiter** — catch peaks. EDITORS-PRO's \`audio/effects.rs\` currently has a low-pass filter; the full chain requires external processing (DaVinci Voice Isolation, iZotope RX) and re-linking. This is documented as a \`video:\` debt marker.

Workflow: clean in a dedicated audio app, re-link in EDITORS-PRO. Don\'t try to do spectral repair in an NLE.`,
    whenToUse: 'Anytime you have dialogue recorded in less-than-ideal conditions. Which is almost always.',
    whenNotToUse: 'On music or sound design — these need different processing. On "clean" studio dialogue that\'s already been processed.',
    amateur: 'Slapping a heavy noise reduction on the whole track, getting that underwater robotic sound, then publishing.',
    pro: 'Subtle noise reduction (5-15 dB). De-reverb only if needed. De-esser on sibilance. High-pass filter at 80 Hz for rumble. Gentle EQ boost around 3 kHz for intelligibility. Compressor at 3:1, slow attack. Limiter at −1 dBTP. Result: dialogue that sounds natural and clear.',
    intensity: {
      lite: 'High-pass + light NR + limiter. Enough to make dialogue intelligible.',
      full: 'Full chain: NR → de-reverb → de-esser → EQ → compressor → limiter. Verify loudness hits −23 LUFS (broadcast) or platform target.',
      ultra: 'Per-channel processing. A/B against reference. Document the chain in the QC report. True-peak ≤ −2 dBTP.',
    },
    safety: ['−23 LUFS', 'dBTP', 'delivery spec'],
    boundaries: 'Dialogue cleanup covers spoken-word processing. It does not cover music mixing (loudness-target handles platform loudness), sound design, or foley.',
    command: `Clean dialogue: NR (subtle, 5-15 dB) → de-reverb → de-esser → high-pass 80 Hz → EQ boost 3 kHz → compressor 3:1 → limiter. Don't over-denoise (avoid the underwater sound). True-peak ≤ −1 dBTP streaming, −2 dBTP broadcast. If EDITORS-PRO engine lacks the tool, clean externally and re-link — mark with \`video:\` debt.`,
  },
  {
    name: 'loudness-target',
    title: 'Loudness Targeting (EBU R128 / ATSC A/85 / Platform)',
    triggers: ['loudness', 'lufs', 'ebu r128', 'atsc a/85', 'integrated loudness', 'true peak', 'dbtp', 'normalize audio', 'audio normalization', 'calm act', 'platform loudness', 'youtube loudness', 'tiktok loudness'],
    trick: `Loudness is measured in LUFS (Loudness Units Full Scale, K-weighted). Integrated loudness is the average over the whole program; short-term is over 3-second windows; momentary is over 0.4-second windows. True-peak (dBTP) measures inter-sample peaks — a sample at −0.1 dBFS can produce a true-peak above 0 dBTP after DAC reconstruction, killing encoders.

Targets:
- EBU R128 (EU broadcast): −23 LUFS ±0.5 integrated, −18 LUFS short-term max, −1 dBTP max.
- ATSC A/85 (US broadcast, CALM Act): −24 LKFS ±2 integrated.
- YouTube: −14 LUFS (they normalize to this).
- TikTok: −18 LUFS.
- Spotify/Apple Music/Amazon Music: −14 LUFS.
- Apple Podcasts: −16 LUFS.

EDITORS-PRO's \`analysis/loudness.rs\` computes R128 integrated. The Flutter \`audio_loudness_meter.dart\` widget renders it live. Never ship without measuring.`,
    whenToUse: 'Always, before sign-off. Every deliverable has a loudness target — find it, hit it, verify it.',
    whenNotToUse: 'Never. There is no scenario where you skip loudness measurement on a deliverable.',
    amateur: 'Normalizing to 0 dBFS peak and calling it done. The result is dynamic range crushed flat, and the platform normalizes it down anyway, making it quieter than competitors.',
    pro: 'Mix to the target integrated LUFS. Verify with a true-peak meter — keep ≤ −1 dBTP for streaming, ≤ −2 dBTP for broadcast. Don\'t crush dynamics to hit the target — adjust the mix, not just the limiter.',
    intensity: {
      lite: 'Platform target (YouTube −14, TikTok −18). True-peak ≤ −1 dBTP. One-pass.',
      full: '−23 LUFS EBU R128 or −24 LKFS ATSC A/85. True-peak ≤ −2 dBTP. Documented in QC report.',
      ultra: 'Per-spec loudness. Dolby Atmos loudness measurement if applicable. True-peak ≤ −2 dBTP. Full QC report.',
    },
    safety: ['−23 LUFS', '−24 LKFS', 'dBTP', 'delivery spec'],
    boundaries: 'Loudness targeting covers the measurement and normalization of the master. It does not cover the mix itself (dialogue-cleanup, music mixing) or the encode (delivery-encode-ladder).',
    command: `Target loudness. Measure integrated LUFS and true-peak. EBU R128: −23 LUFS, ≤ −1 dBTP. ATSC A/85: −24 LKFS. YouTube: −14 LUFS. TikTok: −18 LUFS. ≤ −1 dBTP streaming, ≤ −2 dBTP broadcast. Never ship without measuring. Use audio_loudness_meter.dart widget to verify live.`,
  },
  // ─── PACING & CUTTING ───────────────────────────────────────────────────
  {
    name: 'beat-sync-cut',
    title: 'Beat-Synced Cutting',
    triggers: ['beat sync', 'cut on beat', 'music sync', 'beat detect', 'cut to music', 'rhythmic editing', 'music marker', 'tempo'],
    trick: `Cutting on the beat makes a music-driven edit feel tight. The amateur move is to manually tap-M to add markers and hope you kept time. The pro move is to detect beats programmatically (onset detection — peaks in the spectral flux), drop markers on the timeline, then snap cuts to markers. EDITORS-PRO's \`analysis/beat_detect.rs\` provides onset detection; the markers can then drive magnetic snapping.

Beat detection is not perfect — it finds transients, which include kicks, snares, and other percussive events. For music with a strong kick, it works great. For ambient music, you may need to set the BPM manually and generate markers from that.`,
    whenToUse: 'Music-driven edits: montages, social clips with a beat, dance videos, music videos, ad spots with a music bed.',
    whenNotToUse: 'Dialogue-driven scenes — cutting on the beat fights the natural rhythm of speech. Documentary. Narrative scenes where pacing is emotional, not musical.',
    amateur: 'Cutting randomly and hoping it lines up. Or cutting exactly on every beat, making the edit feel mechanical and exhausting.',
    pro: 'Detect beats. Snap cuts to beat markers with magnetic snapping. But cut on the **accented** beats (1 and 3 in 4/4), not every beat. Leave some shots longer for breathing room. The edit should feel tight, not frantic.',
    intensity: {
      lite: 'Auto-detect, snap to every beat. Quick and acceptable for social.',
      full: 'Auto-detect, snap to accented beats. Verify sync by ear at the end.',
      ultra: 'Manual sync to the score. Tempo map documented. Per-frame offset for emotional timing.',
    },
    safety: ['delivery spec'],
    boundaries: 'Beat-sync covers aligning cuts to musical beats. It does not cover the music selection, music licensing, or the broader pacing of the edit (narrative-pacing).',
    command: `Beat-sync cuts. Detect beats (analysis/beat_detect.rs). Snap cuts to beat markers with magnetic snapping. Cut on accented beats (1 and 3 in 4/4), not every beat. Leave breathing room. Verify sync by ear. Don't cut on every beat — feels mechanical.`,
  },
  {
    name: 'narrative-pacing',
    title: 'Narrative Pacing',
    triggers: ['pacing', 'rhythm of the edit', 'tighten the cut', 'breathing room', 'hold on the shot', 'let it breathe', 'draggy', 'rushed', 'rhythm'],
    trick: `Pacing is the rhythm of the cut — when to hold, when to cut, when to leave silence. The ladder: (1) cut anything that doesn\'t serve the story, (2) hold on reactions, not actions, (3) trust silence — don\'t fill every gap with B-roll, (4) cut on motion (entering/leaving frame, gesture completion), (5) match emotional beats, not just visual beats.

The amateur move is to cut to keep the viewer\'s attention with rapid-fire shots. The pro move is to cut to serve the moment — sometimes that means a 6-second hold on a face, sometimes that means a hard cut on a gesture mid-motion. Pace serves story, not the other way around.`,
    whenToUse: 'Always. Pacing is the editor\'s primary storytelling tool.',
    whenNotToUse: 'Never. Pacing applies to every edit.',
    amateur: 'Cutting every 1-2 seconds to "keep it engaging." The result is exhausting and meaningless — the viewer can\'t absorb anything.',
    pro: 'Cut to serve the moment. Reaction shots held longer than expected. Hard cuts on motion. Silence left in. The edit breathes. Pacing matches the emotional arc, not the music.',
    intensity: {
      lite: 'Tight cuts for social attention span. 1-3 second average shot length.',
      full: 'Pacing serves the story. Variable shot length. Reaction holds.',
      ultra: 'Frame-precise pacing. Emotional arc mapped. Pacing documented in the editor\'s notes.',
    },
    safety: ['delivery spec'],
    boundaries: 'Narrative pacing covers the rhythm of the cut. It does not cover shot selection (the director/DP\'s job), color (color-scopes), or audio (loudness-target).',
    command: `Pace the edit. Cut to serve the moment. Hold on reactions. Cut on motion. Trust silence. Don't cut every 1-2 seconds to "keep it engaging" — that's exhausting. Pacing serves story, not attention span.`,
  },
  // ─── PERFORMANCE / WORKFLOW ────────────────────────────────────────────
  {
    name: 'proxy-workflow',
    title: 'Proxy Workflow',
    triggers: ['proxy', 'offline edit', 'low-res preview', 'proxy generation', 'swap to full', '4k lag', 'timeline lag', 'preview lag'],
    trick: `Proxies are low-res transcodes of your source media used for editing. You cut with proxies (fast, smooth scrubbing), then swap to full-res at picture-lock for color and export. EDITORS-PRO auto-generates proxies when source res exceeds the threshold; the \`proxy_status_badge.dart\` widget shows proxy status.

Proxies are 1/4 or 1/8 resolution, often in a fast codec (ProRes Proxy, DNxHR LB). They keep the timeline responsive on phones and older hardware. The full-res swap is automatic at export time.`,
    whenToUse: 'Always when editing 4K+ on a phone or older hardware. Whenever the timeline lags. When the source codec is heavy (HEVC, AV1) and the hardware can\'t decode in real time.',
    whenNotToUse: 'When editing 1080p on a fast desktop with hardware decode — you may not need proxies. When the source codec is already light (ProRes Proxy).',
    amateur: 'Editing 4K HEVC directly on a phone, timeline lagging, scrubbing one frame at a time, getting frustrated and giving up.',
    pro: 'Proxies on. Cut smoothly. Swap to full at picture-lock. Verify color on full-res before export. The proxy is a tool, not the deliverable.',
    intensity: {
      lite: 'Proxies at 1/4 res. Always on for 4K+ sources.',
      full: 'Proxies at 1/4 res. Swap to full at picture-lock. Verify color on full-res.',
      ultra: 'Proxies at 1/8 res for cutting. Full-res for color. Original RAW for DI. Document the proxy generation settings.',
    },
    safety: ['data loss', 'delivery spec'],
    boundaries: 'Proxy workflow covers the generation, use, and swapping of low-res transcodes. It does not cover the source media management (project\'s job) or the export (delivery-encode-ladder).',
    command: `Use proxies. Auto-generate at 1/4 res when source > threshold. Cut with proxies (fast scrubbing). Swap to full at picture-lock. Verify color on full-res before export. Mark with \`video:\` if you skip the full-res verification.`,
  },
  {
    name: 'delivery-encode-ladder',
    title: 'Delivery Encode Ladder',
    triggers: ['export', 'encode', 'delivery', 'prores', 'h.264', 'h.265', 'hevc', 'av1', 'crf', 'bitrate', 'master', 'mezzanine', 'streaming encode'],
    trick: `The encode ladder is the chain from master to delivery. **Master** (lossless or mezzanine — ProRes 4444 XQ, DNxHR 444, DPX, EXR) → **Mezzanine** (ProRes 422 HQ, DNxHR HQX — for post-production handoff) → **Streaming** (H.264 high-profile, HEVC main-10, AV1 — for delivery). At every rung, verify the loudness hits the target (−23 LUFS EBU R128 for broadcast, platform target for streaming), the true-peak is ≤ −2 dBTP broadcast / ≤ −1 dBTP streaming, and the frame rate matches the delivery spec exactly.

Bitrate is content-dependent. Use CRF (constant rate factor) instead of a fixed bitrate for variable-content delivery: CRF 18 visually lossless, CRF 20 high quality, CRF 23 standard. For streaming, use 2-pass VBR with a bitrate ceiling set by the platform.

EDITORS-PRO's \`export_engine/encoder.rs\` does H.264/H.265/VP9 with 2-pass and AAC muxing. ProRes is on the roadmap (mark with \`video:\`).`,
    whenToUse: 'Always, at delivery. Pick the rung based on the delivery spec — never ship a master when a streaming encode is asked for, and never ship a streaming encode when a master is asked for.',
    whenNotToUse: 'Never. Every delivery needs an encode.',
    amateur: 'Exporting H.264 at default settings, getting a 200 MB file for a 30-second clip, then wondering why it buffers on mobile.',
    pro: 'Master in ProRes 422 HQ (or ProRes 4444 XQ for HDR). Mezzanine for post handoff. Streaming encode at CRF 18-23 with the right profile/level. Bitrate ceiling per platform. AAC audio at 256 kbps.',
    intensity: {
      lite: 'H.264 CRF 23, AAC 128 kbps, yuv420p. Single-pass. Good enough for social.',
      full: 'ProRes 422 HQ master → H.264 high-profile CRF 18-20 streaming encode. 2-pass VBR. AAC 256 kbps. Verify frame rate, color space, and loudness on the encode.',
      ultra: 'ProRes 4444 XQ or DPX/EXR master. Per-spec delivery encode. HEVC 10-bit for HDR. Verify MaxFALL/MaxCLL for HDR10. Full QC report.',
    },
    safety: ['ProRes', 'frame-rate', 'color space', 'dBTP', 'delivery spec'],
    boundaries: 'The encode ladder covers the codec/bitrate choice and the encode pass. It does not cover the loudness (loudness-target), the legal range (broadcast-legal), or the format interoperability (format-interop).',
    command: `Delivery encode. Master in ProRes 422 HQ (or 4444 XQ for HDR). Streaming in H.264 high-profile CRF 18-23, 2-pass VBR, AAC 256 kbps. Verify frame-rate, color space, and true-peak on the encode. Match the delivery spec exactly. Don't ship the wrong rung.`,
  },
  // ─── EFFECTS ────────────────────────────────────────────────────────────
  {
    name: 'green-screen-key',
    title: 'Green/Blue Screen Keying',
    triggers: ['green screen', 'chroma key', 'blue screen', 'key out background', 'green backdrop', 'spill', 'color key', 'matte'],
    trick: `Keying removes a colored background (green or blue) to composite a subject over a new background. EDITORS-PRO's \`effects/chroma_key.rs\` does HSV-based keying with an eyedropper. The full pro chain: (1) **sample the screen color** with the eyedropper, (2) **adjust the key range** (tolerance), (3) **despill** — remove green spill on edges of the subject, (4) **edge refinement** — matte choke/expand, feather, (5) **lighting match** — match the subject\'s lighting to the new background.

The amateur move is to crank the tolerance until the green is gone, leaving hard edges and green spill. The pro move is to key narrowly, despill, and refine edges.`,
    whenToUse: 'When you have footage shot on a green or blue screen and need to composite over a new background.',
    whenNotToUse: 'When the green screen was lit unevenly — fix the lighting in reshoot, not in post. When the green screen has shadows or wrinkles — same. When the subject has green in it (clothing, eyes) — use blue or rotoscope.',
    amateur: 'Tolerance at 100, green gone but subject has a halo of green spill and jagged edges.',
    pro: 'Sample the screen color. Tolerance just enough to remove the screen. Despill to kill green on subject edges. Matte choke to remove the halo. Feather for soft edges. Match subject lighting to the new background (direction, color, intensity).',
    intensity: {
      lite: 'Auto-key with eyedropper. Acceptable for talking-head social composites.',
      full: 'Full chain: key → despill → edge refine → lighting match. Verify with the final background in place.',
      ultra: 'Per-pixel key with spill suppression. Edge-aware matte. Match grain between subject and background. Full QC on the composite.',
    },
    safety: ['color space', 'delivery spec'],
    boundaries: 'Green-screen key covers chroma keying. It does not cover masking (mask-animation), rotoscoping, or the composite (compositing in effects/compositing.rs).',
    command: `Green/blue screen key. Sample screen color with eyedropper. Tolerance just enough. Despill to kill color spill on subject. Matte choke + feather for edges. Match subject lighting to new background (direction, color, intensity). Don't crank tolerance — fix edges with despill + choke.`,
  },
  {
    name: 'film-grain-recipe',
    title: 'Film Grain Recipe',
    triggers: ['film grain', 'add grain', 'grain recipe', 'film stock', 'vhs grain', 'halation', 'film emulation', 'super 8', '16mm', '35mm'],
    trick: `Film grain adds organic texture to digital footage. Two parts: the **grain** (the actual noise pattern, varies by stock — Kodak 5219 500T, Fuji Eterna, etc.) and the **halation** (the red glow around bright highlights, caused by light bouncing off the film base). EDITORS-PRO's \`effects/grain.rs\` has 17 stock presets plus VHS and halation. The Flutter \`film_grain_picker.dart\` widget exposes them.

The amateur move is to crank grain to 100% and call it "cinematic." The pro move is to apply grain at 15-30% to break up digital cleanliness, with the right stock for the look (Kodak Vision3 for warm tones, Fuji for cool tones, Ilford for B&W).`,
    whenToUse: 'When footage looks too clean/digital. When matching film footage. When emulating a film stock. When adding texture for a stylized look.',
    whenNotToUse: 'On footage that\'s already noisy — grain on top of noise looks worse. On content where the digital look is the point (UI demos, screencasts). For broadcast unless spec\'d — grain eats bitrate.',
    amateur: 'Grain at 100%, result looks like a 90s TV with bad reception.',
    pro: 'Pick the right stock. Grain at 15-30%. Add halation on highlights for film emulation. Match the grain to the color temperature of the footage. Verify the grain doesn\'t push the encode bitrate over budget.',
    intensity: {
      lite: 'Grain at 20%, generic 35mm stock. Acceptable for social stylization.',
      full: 'Pick stock by look. Grain at 15-25%. Halation on highlights. Verify bitrate.',
      ultra: 'Per-shot grain matching. Halation calibrated to highlight EV. Grain plate generated at full res, downsampled. Document the recipe in the QC report.',
    },
    safety: ['delivery spec'],
    boundaries: 'Film grain covers grain addition and film emulation. It does not cover color grading (color-scopes), LUT management (lut-management), or VHS-style glitch effects (out of scope).',
    command: `Film grain. Pick the right stock (Kodak Vision3 warm, Fuji cool, Ilford B&W). Grain at 15-30%. Halation on highlights for film emulation. Match grain to footage color temperature. Verify grain doesn\'t push encode bitrate over budget. Use film_grain_picker.dart widget.`,
  },
  {
    name: 'sky-replacement',
    title: 'Sky Replacement',
    triggers: ['sky replacement', 'replace sky', 'sky swap', 'gradient sky', 'overcast fix', 'boring sky', 'sky gradient'],
    trick: `Sky replacement swaps a blown-out or boring sky for a more interesting one. EDITORS-PRO has a \`sky_replace.rs\` stub — the workflow is: (1) qualify the sky (luminance key on the bright sky region), (2) refine the mask (edge softness, hole-filling for trees/buildings), (3) composite the new sky, (4) match lighting (color, direction, intensity to the foreground), (5) add interaction (reflections, shadows).

The amateur move is to drop a sunset gradient behind every scene, ignoring whether the lighting matches. The pro move is to use a sky that matches the scene\'s lighting direction, color temperature, and time of day.`,
    whenToUse: 'When the sky is blown out and unrecoverable. When the sky is boring (overcast gray) and the scene calls for something better. When matching plate photography.',
    whenNotToUse: 'When the sky has detail that\'s recoverable with a grad filter or highlight recovery. When the scene\'s mood depends on the actual sky (e.g., ominous clouds). When the foreground lighting won\'t match any sky you swap in.',
    amateur: 'Sunset gradient on every scene. Foreground lit from the side, sky sun setting straight ahead. Result looks like a bad Photoshop job.',
    pro: 'Qualify the sky with a luminance mask. Refine the edges (especially through trees). Pick a sky that matches the scene\'s lighting (direction, color temperature, time of day). Match exposure. Add reflections and shadows for ground interaction. Color-match the composite.',
    intensity: {
      lite: 'Luminance key + gradient sky. Acceptable for quick fixes.',
      full: 'Luminance key + real sky plate. Edge refinement. Lighting match.',
      ultra: 'Per-pixel qualification. Planar track for camera movement. Atmospheric perspective (sky color shifts with depth). Full QC on the composite.',
    },
    safety: ['color space', 'delivery spec'],
    boundaries: 'Sky replacement covers the sky swap. It does not cover the qualification technique (mask-animation) or the composite (compositing).',
    command: `Sky replacement. Qualify sky with luminance mask. Refine edges (trees, buildings). Pick sky matching scene lighting (direction, color, time of day). Match exposure. Add reflections + shadows for ground interaction. Color-match the composite. Don't use a sunset gradient that doesn't match foreground lighting.`,
  },
  {
    name: 'video-stabilization',
    title: 'Video Stabilization',
    triggers: ['stabilize', 'deshake', 'stabilization', 'shaky footage', 'smooth camera', 'warp stabilizer', 'steady shot'],
    trick: `Stabilization smooths out camera shake. Two kinds: **2D** (translation + rotation — fast, good for handheld shake) and **3D** (camera solve + reverse projection — slow, good for parallax-heavy motion). EDITORS-PRO's \`effects/stabilization.rs\` does 2D deshake via block-matching motion estimation. 3D is the upgrade path (mark with \`video:\`).

The amateur move is to crank smoothing to 100% and get that wobbly, jelly-like "over-stabilized" look. The pro move is to smooth enough to remove the shake but keep the natural camera movement — and to crop the frame slightly to hide the edge artifacts.`,
    whenToUse: 'Handheld footage that\'s too shaky. Drone footage with wind wobble. Phone footage that\'s unwatchable.',
    whenNotToUse: 'Footage on a tripod that\'s already stable. Footage where the shake is intentional (action sports, documentary realism). Footage with rolling-shutter jitter — that needs a different tool (Mercalli).',
    amateur: 'Smoothing at 100%, crop at 5%. Result looks like it was shot on a gimbal but with weird wobble at the edges.',
    pro: 'Smoothing at 30-60%, crop at 8-12% to hide edge artifacts. Choose "smooth" or "no motion" based on whether you want to keep the camera move. Verify no jelly artifacts. If parallax is heavy, upgrade to 3D camera solve.',
    intensity: {
      lite: 'Auto-stabilize with smoothing 50%. Crop 10%.',
      full: 'Smoothing 30-50%. Crop 8-12%. Verify no jelly artifacts. If parallax heavy, mark for 3D upgrade.',
      ultra: 'Per-shot stabilization choice. 3D camera solve for parallax-heavy shots. Crop documented. Rolling-shutter correction if needed.',
    },
    safety: ['3D camera solve', 'delivery spec'],
    boundaries: 'Stabilization covers 2D deshake and the 3D upgrade path. It does not cover rolling-shutter correction (a different algorithm) or motion tracking (motion-tracking).',
    command: `Stabilize. 2D deshake via effects/stabilization.rs. Smoothing 30-60%, crop 8-12% to hide edge artifacts. Don't crank smoothing to 100% — jelly artifacts. If motion is parallax-heavy, upgrade to 3D camera solve (mark with \`video:\` debt). Verify no wobble.`,
  },
  {
    name: 'motion-tracking',
    title: 'Motion Tracking',
    triggers: ['motion track', 'track point', 'planar track', 'track mask', 'attach to track', 'blur face', 'track text', 'pin to subject', 'camera track', 'track matte'],
    trick: `Motion tracking follows a feature in the frame over time, so you can attach something to it (text, a blur, a mask, an effect). Three kinds: **point track** (single feature — fast, fragile), **planar track** (a region — robust, used for masks and screen replacements), **camera track** (3D camera solve — used for compositing 3D elements). EDITORS-PRO has a \`motion_tracking.rs\` stub with point-track (centroid) — planar and camera are the upgrade paths (mark with \`video:\`).

The amateur move is to track once and hope. The pro move is to track, verify the track, fix drift with manual keyframes, and only then attach.`,
    whenToUse: 'Blur a face. Attach text to a moving subject. Replace a screen. Drive a mask with tracking data.',
    whenNotToUse: 'When the subject is static — just position the effect, don\'t track. When the motion is too fast or too blurry — the tracker will fail.',
    amateur: 'Track, attach text, text drifts off the subject halfway through. Or: blur a face, the blur doesn\'t follow the face.',
    pro: 'Pick a high-contrast feature. Track forward. Verify the track frame-by-frame. Fix drift with manual keyframes. Smooth the track. Attach the effect. Verify the attachment frame-by-frame.',
    intensity: {
      lite: 'Point track. Auto. Acceptable for face blur on social.',
      full: 'Point track with manual drift fixes. Verify frame-by-frame. Planar track for masks.',
      ultra: 'Planar track for screen replacements. Camera solve for 3D composites. Full QC on the track.',
    },
    safety: ['delivery spec'],
    boundaries: 'Motion tracking covers point, planar, and camera tracking. It does not cover stabilization (video-stabilization) or the effect attached to the track (the relevant effect skill).',
    command: `Motion track. Pick high-contrast feature. Track forward. Verify frame-by-frame. Fix drift with manual keyframes. Smooth the track. Attach the effect. Verify attachment. EDITORS-PRO has point-track (centroid) — mark with \`video:\` for planar/camera upgrade if needed.`,
  },
  {
    name: 'multicam-editing',
    title: 'Multicam Editing',
    triggers: ['multicam', 'multi-cam', 'multi angle', 'camera angle', 'angle switch', 'live cut', 'concert edit', 'event edit'],
    trick: `Multicam editing cuts between multiple camera angles of the same event in real time. EDITORS-PRO's \`effects/multicam.rs\` has angle grouping, audio cross-correlation sync, angle switching, and transitions. The Flutter \`multicam_switcher.dart\` widget exposes an angle grid for real-time switching.

Workflow: (1) group angles into a multicam clip, (2) sync via timecode or audio (cross-correlation), (3) play back in real time, switching angles by tapping, (4) refine the cuts after the live pass, (5) add transitions where wanted.`,
    whenToUse: 'Concert footage, event coverage, talk shows, interviews with multiple cameras, sports.',
    whenNotToUse: 'Single-camera shoots. When the angles aren\'t synced (audio or timecode).',
    amateur: 'Cutting between angles with no rhythm, every cut visible, no audio sync reference.',
    pro: 'Sync via audio cross-correlation. Live-switch with the beat of the event. Refine cuts. Hard cuts for energy, dissolves for soft transitions. Verify audio is from the best angle (usually the board feed).',
    intensity: {
      lite: 'Auto-sync via audio. Live-switch. Hard cuts only.',
      full: 'Auto-sync via audio. Live-switch. Refine cuts. Add transitions where wanted. Verify audio source.',
      ultra: 'Timecode sync. Per-cut transition choice. Audio from the board feed. Document the angle choices in the QC report.',
    },
    safety: ['delivery spec', 'frame-rate'],
    boundaries: 'Multicam covers multi-angle cutting. It does not cover the audio mix (loudness-target) or the per-angle color (color-match-shots).',
    command: `Multicam. Group angles. Sync via audio cross-correlation or timecode. Live-switch by tapping. Refine cuts. Hard cuts for energy, dissolves for soft transitions. Audio from the best angle (usually board feed). Use multicam_switcher.dart widget.`,
  },
  {
    name: 'mask-animation',
    title: 'Mask Drawing and Animation',
    triggers: ['mask', 'draw mask', 'bezier mask', 'animate mask', 'rotoscope', 'roto', 'mask path', 'feather mask', 'mask shape'],
    trick: `Masks isolate regions of the frame for targeted effects. EDITORS-PRO's \`effects/masking.rs\` has Rectangle, Ellipse, Bezier, Luminance, Chroma, and Depth masks with feather, expansion, and 4 composite modes. The Flutter \`mask_drawing_tool.dart\` widget provides Bezier drawing on the canvas.

The pro workflow: (1) draw the mask roughly, (2) refine the shape, (3) feather the edges, (4) animate the mask path if the subject moves (rotoscope), (5) invert if needed, (6) apply the effect inside or outside the mask.`,
    whenToUse: 'Targeted effects (color just on the face, blur just on the background). Rotoscoping (subject isolation for compositing). Creative transitions (iris wipes, shape reveals).',
    whenNotToUse: 'When the effect should apply to the whole frame — don\'t mask unnecessarily. When a luminance or chroma key would do the job automatically — don\'t rotoscope what can be keyed.',
    amateur: 'Drawing a mask, not animating it, then the subject walks out of the mask and the effect breaks.',
    pro: 'Draw the mask. Refine. Feather. Animate the path with the subject (rotoscope). Verify frame-by-frame. Use the right mask type (Bezier for shapes, Luminance for sky, Chroma for green screen, Depth for AR footage).',
    intensity: {
      lite: 'Rectangle/ellipse mask. No animation. Acceptable for static shots.',
      full: 'Bezier mask with feather. Animate the path if subject moves. Verify frame-by-frame.',
      ultra: 'Per-frame rotoscope. Planar-tracked mask. Edge-aware feather. Full QC on the mask.',
    },
    safety: ['delivery spec'],
    boundaries: 'Mask covers shape drawing, feathering, and path animation. It does not cover the effect inside the mask (the relevant effect skill) or chroma keying (green-screen-key).',
    command: `Draw mask. Pick type: Rectangle/Ellipse for shapes, Bezier for organic, Luminance for sky, Chroma for green screen, Depth for AR. Refine shape. Feather edges. Animate path if subject moves (rotoscope). Verify frame-by-frame. Use mask_drawing_tool.dart widget.`,
  },
  {
    name: 'lens-correction',
    title: 'Lens Correction',
    triggers: ['lens correction', 'lens distortion', 'chromatic aberration', 'ca fix', 'vignette', 'lens profile', 'brown-conrady', 'de-fish', 'fisheye fix'],
    trick: `Lens correction fixes optical defects: distortion (barrel/pincushion/moustache), chromatic aberration (color fringing at edges), and vignetting (darkening at corners). EDITORS-PRO's \`effects/lens_correction.rs\` does Brown-Conrady distortion (K1/K2/K3 + tangential P1/P2), CA correction, vignette removal, and has 8 built-in lens profiles. The Flutter \`lens_correction_panel.dart\` widget exposes them.

The pro workflow: (1) pick the lens profile (if known), or (2) dial in K1/K2/K3 manually with a grid reference, (3) fix CA with the red/blue offset sliders (zoom in to 200% to see fringing), (4) remove vignette with the amount/midpoint/roundness sliders.`,
    whenToUse: 'Wide-angle footage with visible barrel distortion. Drone footage with fisheye. Cheap lenses with CA. Footage with vignetting you want to remove (or add for stylization).',
    whenNotToUse: 'When the lens is already corrected in-camera (most modern phones). When the distortion is the point (fisheye music video look).',
    amateur: 'Sliding K1 to 0.5 because the frame looks "weird" with no reference grid. Result: over-corrected, wobbly straight lines.',
    pro: 'Pick the lens profile if known. Otherwise, enable a grid overlay and dial K1 until straight lines are straight. Fix CA at 200% zoom — match red and blue offsets. Vignette: subtle removal, or add for stylization.',
    intensity: {
      lite: 'Auto-profile. Skip CA. Skip vignette.',
      full: 'Auto-profile or manual K1/K2/K3 with grid. Fix CA. Subtle vignette.',
      ultra: 'Per-lens calibration. CA per-channel. Vignette calibrated to the lens. Document in QC report.',
    },
    safety: ['delivery spec', 'color space'],
    boundaries: 'Lens correction covers optical defect correction. It does not cover color grading (color-scopes) or the creative use of distortion (out of scope).',
    command: `Lens correction. Pick profile if known, else dial K1/K2/K3 with grid overlay. Fix CA at 200% zoom (red/blue offsets). Vignette: subtle removal or stylized addition. Use lens_correction_panel.dart widget. Don't over-correct — straight lines should be straight, not wobbly.`,
  },
  {
    name: 'noise-reduction',
    title: 'Noise Reduction',
    triggers: ['noise reduction', 'denoise video', 'temporal nr', 'spatial nr', 'nlm', 'bilateral', 'wiener', 'low light fix', 'grain removal'],
    trick: `Noise reduction removes sensor noise from low-light footage. EDITORS-PRO's \`effects/noise_reduction.rs\` has 4 methods: **Bilateral** (edge-preserving, fast), **Wiener** (frequency-domain, good for fine noise), **NLM** (non-local means, best quality, slow), **Temporal** (uses frame-to-frame coherence, excellent for static shots). The Flutter \`noise_reduction_panel.dart\` widget exposes them.

The tradeoff: more reduction = more detail loss. The amateur move is to crank NR to 100% and get a plastic, wax-figure look. The pro move is to NR just enough to clean the noise, then add a tiny bit of grain back to break up the plasticity.`,
    whenToUse: 'Low-light footage with visible noise. Underexposed footage that\'s been pushed up. High-ISO footage.',
    whenNotToUse: 'On clean footage — NR on clean footage just softens detail. On footage where the noise is the aesthetic (e.g., Super 8 emulation).',
    amateur: 'NR at 100% with NLM. Result: subject looks like a wax figure, no skin texture, no detail.',
    pro: 'Method by shot type: Temporal for static, Bilateral for motion. Strength at 30-50%. Add a tiny bit of grain back to break up the plasticity. Verify with a skin close-up.',
    intensity: {
      lite: 'Bilateral at 40%. Fast. Acceptable for social.',
      full: 'Temporal for static shots, Bilateral for motion. Strength 30-50%. Add grain back. Verify on skin close-up.',
      ultra: 'Per-shot method choice. NLM for the worst shots. Luma/chroma separation. Document in QC report.',
    },
    safety: ['delivery spec'],
    boundaries: 'Noise reduction covers video NR. It does not cover audio NR (dialogue-cleanup) or the grain addition (film-grain-recipe).',
    command: `Noise reduction. Method by shot: Temporal for static, Bilateral for motion, NLM for worst case. Strength 30-50%. Don't crank to 100% — wax figure. Add a tiny bit of grain back to break up plasticity. Verify on skin close-up. Use noise_reduction_panel.dart widget.`,
  },
  // ─── EXPORT / DELIVERY ─────────────────────────────────────────────────
  {
    name: 'batch-export',
    title: 'Batch Export Queue',
    triggers: ['batch export', 'queue export', 'multiple exports', 'export queue', 'render queue', 'parallel export', 'batch encode'],
    trick: `Batch export queues multiple export jobs (different resolutions, codecs, or platform targets) and runs them in sequence (or in parallel if the hardware supports it). EDITORS-PRO's \`export_engine/batch.rs\` provides the queue; the Flutter \`batch_export_queue.dart\` widget exposes it.

Workflow: (1) add jobs to the queue (e.g., 1080p H.264 for YouTube, 1080×1920 H.264 for TikTok, 4K ProRes master), (2) set the order (master first, then platform encodes), (3) start the queue, (4) the foreground \`ExportService.kt\` handles one job at a time, with notifications between.`,
    whenToUse: 'Delivering to multiple platforms. Mastering + delivery in one pass. Generating vertical + horizontal versions of the same edit.',
    whenNotToUse: 'For a single export — just use the regular export. When the platforms have different edits (not just different encodes) — those are separate projects.',
    amateur: 'Exporting one version, then exporting another, manually, repeating for every platform. Hours of waiting.',
    pro: 'Queue the master, then the platform encodes. Start the queue. Walk away. Come back to all the deliverables.',
    intensity: {
      lite: 'Queue 2-3 social encodes. Single-pass.',
      full: 'Queue master + platform encodes. 2-pass VBR for streaming. Verify each encode.',
      ultra: 'Per-spec encode ladder for each platform. Full QC on each output. Document in QC report.',
    },
    safety: ['data loss', 'ProRes', 'delivery spec'],
    boundaries: 'Batch export covers queueing multiple export jobs. It does not cover the encode settings per job (delivery-encode-ladder) or the loudness per platform (loudness-target).',
    command: `Batch export. Queue master + platform encodes (1080p H.264 YouTube, 1080×1920 TikTok, 4K ProRes master). Order: master first. Start queue. Use batch_export_queue.dart widget. Verify each encode meets its platform spec.`,
  },
  {
    name: 'format-interop',
    title: 'Format Interoperability (EDL / FCPXML / OpenTimelineIO)',
    triggers: ['edl', 'fcpxml', 'opentimelineio', 'otio', 'aaf', 'xml export', 'premiere round trip', 'resolve round trip', 'final cut round trip', 'project interchange'],
    trick: `Format interoperability moves a timeline between NLEs. EDITORS-PRO's \`project/interop.rs\` exports EDL (Edit Decision List — the oldest, simplest format, supported everywhere), FCPXML (Final Cut Pro XML, also imported by Premiere and Resolve), and OpenTimelineIO (the modern open standard, from Pixar).

The pro workflow: (1) export the timeline to the format the next tool supports (FCPXML for FCP/Premiere/Resolve, EDL for older systems, OTIO for modern pipelines), (2) open in the target tool, (3) verify clips and cuts survived the round trip, (4) note that effects, color, and audio don\'t always translate — those need to be re-done in the target tool.`,
    whenToUse: 'Round-tripping between EDITORS-PRO and DaVinci/Premiere/FCP. Handing off to a colorist. Moving to a different NLE for a specific tool.',
    whenNotToUse: 'When you can finish in EDITORS-PRO — don\'t round-trip unnecessarily. When the target NLE doesn\'t support the format.',
    amateur: 'Exporting a project as a single video file and re-importing, losing all edit decisions. Then asking the colorist to "just match it."',
    pro: 'Export to FCPXML or OTIO. Open in the target NLE. Verify clips and cuts. Re-do effects, color, and audio in the target — they don\'t translate. Document what translated and what didn\'t.',
    intensity: {
      lite: 'Skip — finish in EDITORS-PRO.',
      full: 'Export FCPXML for round trip to Premiere/Resolve. Verify clips and cuts survived.',
      ultra: 'Export OTIO for pipeline handoff. Document what translated. Verify frame-rate and color space metadata survived.',
    },
    safety: ['ProRes', 'frame-rate', 'color space', 'delivery spec'],
    boundaries: 'Format interop covers timeline interchange. It does not cover the encode (delivery-encode-ladder) or the source media relinking (project\'s job).',
    command: `Format interop. Export to FCPXML for Premiere/Resolve/FCP, EDL for older systems, OTIO for modern pipelines. Verify clips and cuts survived the round trip. Effects, color, audio don\'t translate — re-do in the target. Use project/interop.rs module.`,
  },
  // ─── TIMELINE ──────────────────────────────────────────────────────────
  {
    name: 'ripple-roll-trim',
    title: 'Ripple / Roll / Slip / Slide Trim',
    triggers: ['ripple trim', 'roll trim', 'slip trim', 'slide trim', 'ripple delete', 'trim mode', 'advanced trim', 'close gap', 'overwrite trim'],
    trick: `Four trim modes pros use constantly: **Ripple** — trims a clip and shifts everything after it to close the gap. **Roll** — trims two adjacent clips simultaneously (one gets shorter, the other gets longer, total duration unchanged). **Slip** — changes the in/out of a clip without changing its duration or position (you see different frames of the same shot). **Slide** — moves a clip left/right between its neighbors, trimming the neighbors to make room.

EDITORS-PRO's \`timeline/advanced_trim.rs\` implements all four. The Flutter \`advanced_trim_modes.dart\` widget exposes them as toolbar buttons.

The amateur move is to do everything with regular trim + delete + ripple-delete. The pro move is to use the right trim mode for the job — ripple to close gaps, roll to adjust a cut point, slip to fix a framing issue, slide to nudge a clip without affecting duration.`,
    whenToUse: 'Ripple: when closing a gap after a delete. Roll: when fine-tuning a cut between two clips. Slip: when a clip\'s framing is off but the duration is right. Slide: when a clip needs to move but the total duration can\'t change.',
    whenNotToUse: 'For simple trims — regular trim is fine. When you don\'t understand which mode does what — learn them first.',
    amateur: 'Doing everything with regular trim + ripple-delete. Result: lots of fiddly manual re-positioning.',
    pro: 'Ripple to close gaps. Roll for cut-point refinement. Slip for reframing. Slide for nudging. The right mode for the job.',
    intensity: {
      lite: 'Ripple + ripple-delete. Skip roll/slip/slide.',
      full: 'All four modes. Use roll for cut refinement.',
      ultra: 'All four modes with J/K/L shuttle scrubbing. Trim with frame accuracy.',
    },
    safety: ['frame-rate', 'data loss'],
    boundaries: 'Trim modes cover timeline trimming. They do not cover the clip effects, color, or audio — those are unaffected by trim mode.',
    command: `Use the right trim mode. Ripple: trim + close gap. Roll: trim two adjacent clips (one shorter, one longer). Slip: change in/out without changing duration/position. Slide: move clip between neighbors (trims neighbors). Don't do everything with regular trim + ripple-delete — use the right mode. Use advanced_trim_modes.dart widget.`,
  },
  {
    name: 'keyframe-curves',
    title: 'Keyframe Curves (Bezier)',
    triggers: ['keyframe curve', 'bezier keyframe', 'easing', 'ease in', 'ease out', 'animation curve', 'smooth keyframes', 'keyframe interpolation', 'graph editor'],
    trick: `Keyframe curves control the interpolation between keyframes. Linear keyframes (default in amateur tools) produce robotic, mechanical motion. Bezier keyframes with the right easing produce natural, organic motion. EDITORS-PRO's \`timeline/keyframe.rs\` supports linear, ease-in, ease-out, ease-in-out, and bezier with adjustable tangent handles. The Flutter \`keyframe_graph_editor.dart\` widget exposes them.

The pro workflow: (1) set keyframes for the property (position, scale, opacity, etc.), (2) open the graph editor, (3) adjust the bezier tangent handles to shape the curve, (4) use ease-in for things entering, ease-out for things leaving, ease-in-out for things settling, (5) verify with a real-time preview.`,
    whenToUse: 'Any animated property — position, scale, rotation, opacity, effect parameters. Title animations. Zoom/pan on stills (Ken Burns).',
    whenNotToUse: 'When linear is the right look (mechanical, robotic UI animations). When there are only two keyframes and the easing doesn\'t matter.',
    amateur: 'Linear keyframes everywhere. Title slides in at constant speed, stops dead. Result: looks like a PowerPoint animation.',
    pro: 'Bezier keyframes with shaped curves. Ease-in for entering, ease-out for leaving. Smooth, organic motion. Verify with real-time preview. Per-property lanes in the graph editor.',
    intensity: {
      lite: 'Ease-in-out on key transitions. Don\'t touch the graph editor.',
      full: 'Bezier with shaped tangents. Per-property lanes. Verify with preview.',
      ultra: 'Per-property bezier. Frame-precise timing. Document the curve choices in the editor\'s notes.',
    },
    safety: ['delivery spec'],
    boundaries: 'Keyframe curves cover animation interpolation. They do not cover the property being animated (the relevant effect skill) or the speed curves (which are a separate concept — see speed-curve-editor.dart).',
    command: `Keyframe curves. Use bezier, not linear. Ease-in for entering, ease-out for leaving, ease-in-out for settling. Shape tangents in the graph editor. Per-property lanes. Verify with real-time preview. Use keyframe_graph_editor.dart widget.`,
  },
  // ─── HDR & LEGAL ───────────────────────────────────────────────────────
  {
    name: 'hdr-delivery',
    title: 'HDR Delivery (PQ / HLG / HDR10 / Dolby Vision)',
    triggers: ['hdr', 'hdr10', 'dolby vision', 'hlg', 'pq', 'smpte st 2084', 'bt.2020', 'wide gamut', 'rec 2020', 'tone map', '10-bit delivery'],
    trick: `HDR delivery uses a wider color gamut (Rec.2020) and a wider dynamic range (PQ or HLG transfer function). PQ (Perceptual Quantizer, SMPTE ST 2084) is the standard for HDR10 and Dolby Vision. HLG (Hybrid Log-Gamma) is the standard for broadcast HDR. EDITORS-PRO's \`effects/color_space.rs\` does HDR PQ/HLG tone mapping on the input side; embedding HDR metadata (MaxFALL, MaxCLL, master display) in the encode is the gap (mark with \`video:\`).

The pro workflow: (1) confirm the delivery spec (HDR10, HDR10+, Dolby Vision, HLG), (2) grade in 10-bit Rec.2020 PQ, (3) verify with HDR scopes (Dolby L1/L2/L3 analyzer if Dolby Vision), (4) encode in HEVC main-10 or AV1, (5) embed the static metadata (MaxFALL, MaxCLL, master display) for HDR10, or the dynamic metadata for HDR10+/Dolby Vision.`,
    whenToUse: 'When the delivery spec calls for HDR. When the source is HDR (log footage with HDR intent). When delivering to HDR-capable platforms (Netflix, Apple TV+, Disney+, YouTube HDR).',
    whenNotToUse: 'When the delivery is SDR — don\'t ship HDR to SDR platforms. When the source is SDR — upconverting to HDR doesn\'t add dynamic range. When you don\'t have an HDR reference monitor — grading HDR on an SDR monitor is dangerous.',
    amateur: 'Upconverting SDR footage to HDR by stretching the values, result looks washed out and over-saturated on HDR displays.',
    pro: 'Source is HDR. Grade in 10-bit Rec.2020 PQ. Verify with HDR scopes. Encode HEVC main-10. Embed static metadata (MaxFALL, MaxCLL, master display) for HDR10. Per-spec for HDR10+/Dolby Vision. Verify on an HDR reference monitor.',
    intensity: {
      lite: 'Skip HDR — deliver SDR.',
      full: 'HDR10 with static metadata. Verify MaxFALL/MaxCLL. Encode HEVC main-10.',
      ultra: 'Per-spec HDR (HDR10/HDR10+/Dolby Vision/HLG). Dynamic metadata if applicable. Full HDR QC with Dolby analyzer.',
    },
    safety: ['color space', 'legal range', 'delivery spec'],
    boundaries: 'HDR delivery covers HDR grading, encoding, and metadata. It does not cover SDR delivery (delivery-encode-ladder) or the color space conversion (broadcast-legal).',
    command: `HDR delivery. Confirm spec (HDR10/HDR10+/Dolby Vision/HLG). Grade in 10-bit Rec.2020 PQ. Verify with HDR scopes. Encode HEVC main-10. Embed MaxFALL/MaxCLL/master display for HDR10. Verify on HDR reference monitor. Don't upconvert SDR to HDR. Mark metadata-embedding gaps with \`video:\` debt.`,
  },
  {
    name: 'broadcast-legal',
    title: 'Broadcast Legal (Loudness, Color Range, Title-Safe)',
    triggers: ['broadcast legal', 'legal range', 'legalize', 'title safe', 'action safe', 'broadcast safe', 'qc', 'quality control', 'broadcast compliance', 'ebu', 'atsc', 'r128'],
    trick: `Broadcast legal is the bundle of compliance checks: loudness (EBU R128 −23 LUFS or ATSC A/85 −24 LKFS), true-peak (≤ −1 dBTP streaming, ≤ −2 dBTP broadcast), legal color range (Rec.709 16–235 / 64–940), title-safe (90% broadcast) and action-safe (80%) areas, frame-rate compliance, color space tagging. EDITORS-PRO has a \`legalizer.rs\` stub — the workflow is: (1) run a legalizer pass before encode (clamps color to legal range with optional soft-clip), (2) verify loudness with the meter, (3) verify graphics are inside title-safe, (4) verify frame rate matches spec, (5) verify color space tags in the encode, (6) verify legal range on the **waveform** monitor and skin tones on the **vectorscope** I-line.

The amateur move is to ship without QC. The pro move is to run a full QC pass and document it.`,
    whenToUse: 'Always, in full and ultra mode. Any deliverable with a spec. Before sign-off.',
    whenNotToUse: 'Never in full/ultra. In lite mode, the platform loudness check is the minimum.',
    amateur: 'Shipping without QC. Client finds the loudness violation at 2am the day before air.',
    pro: 'Legalizer pass. Loudness meter. Title-safe overlay. Frame-rate check. Color space tags. Full QC report.',
    intensity: {
      lite: 'Platform loudness check only. Skip the rest.',
      full: 'Legalizer + loudness + title-safe + frame-rate + color space tags. Full QC report.',
      ultra: 'Per-frame legalizer. Per-spec loudness. Per-pixel safe-area verification. Full QC report with sign-off.',
    },
    safety: ['−23 LUFS', '−24 LKFS', 'dBTP', 'legal range', 'title-safe', 'frame-rate', 'field-order', 'color space', 'delivery spec', 'data loss'],
    boundaries: 'Broadcast legal covers compliance checks. It does not cover the encode (delivery-encode-ladder) or the loudness targeting (loudness-target — broadcast-legal verifies, loudness-target targets).',
    command: `Broadcast legal. Legalizer pass (clamp to legal range). Loudness meter (−23 LUFS EBU R128 or −24 LKFS ATSC A/85, ≤ −2 dBTP broadcast). Title-safe 90% / action-safe 80%. Frame-rate matches spec. Color space tagged. Field-order preserved if interlaced. Full QC report. Never ship without QC in full/ultra mode.`,
  },
];

// ─── Write skill files ───────────────────────────────────────────────────

function writeSkill(skill) {
  const dir = path.join(SKILLS_DIR, skill.name);
  fs.mkdirSync(dir, { recursive: true });
  const filePath = path.join(dir, 'SKILL.md');

  const triggersList = skill.triggers.map(t => `"${t}"`).join(', ');

  let content = '';
  content += `---\n`;
  content += `name: ${skill.name}\n`;
  content += `description: >\n`;
  content += `  ${skill.title}. Use when the user says ${triggersList},\n`;
  content += `  or whenever they describe a workflow involving ${skill.triggers[0]}.\n`;
  content += `license: MIT\n`;
  content += `---\n\n`;
  content += `# ${skill.title}\n\n`;
  content += `## The trick\n\n${skill.trick}\n\n`;
  content += `## When to use\n\n${skill.whenToUse}\n\n`;
  content += `## When NOT to use\n\n${skill.whenNotToUse}\n\n`;
  content += `## Examples\n\n`;
  content += `**Amateur** ❌: ${skill.amateur}\n\n`;
  content += `**Professional** ✅: ${skill.pro}\n\n`;
  content += `## Intensity\n\n`;
  content += `| Level | Enforcement |\n|---|---|\n`;
  content += `| \`lite\` | ${skill.intensity.lite} |\n`;
  content += `| \`full\` | ${skill.intensity.full} |\n`;
  content += `| \`ultra\` | ${skill.intensity.ultra} |\n\n`;
  content += `## Safety carve-outs (never cut)\n\n`;
  content += `This skill must enforce the following pinned safety phrases. ` +
    `A reword that drops one of these from this file trips CI ` +
    `(see \`scripts/check-video-invariants.js\`):\n\n`;
  for (const s of skill.safety) {
    content += `- \`${s}\`\n`;
  }
  content += `\n## Boundaries\n\n${skill.boundaries}\n`;

  fs.writeFileSync(filePath, content);
  return filePath;
}

function writeCommand(skill) {
  const filePath = path.join(COMMANDS_DIR, `${skill.name}.toml`);

  let content = '';
  content += `description = "${skill.title} — ${skill.triggers.slice(0, 3).join(', ')}"\n`;
  content += `prompt = """\n`;
  content += `${skill.command}\n`;
  content += `"""\n`;

  fs.writeFileSync(filePath, content);
  return filePath;
}

// ─── Main ────────────────────────────────────────────────────────────────

function main() {
  fs.mkdirSync(SKILLS_DIR, { recursive: true });
  fs.mkdirSync(COMMANDS_DIR, { recursive: true });

  console.log(`Generating ${SKILLS.length} skills + ${SKILLS.length} commands...\n`);

  for (const skill of SKILLS) {
    const skillPath = writeSkill(skill);
    const cmdPath = writeCommand(skill);
    console.log(`  ✓ ${skill.name}`);
  }

  console.log(`\nDone. ${SKILLS.length} SKILL.md files + ${SKILLS.length} command .toml files generated.`);
  console.log(`Run \`node scripts/check-video-invariants.js\` to verify the safety invariants.`);
}

main();
