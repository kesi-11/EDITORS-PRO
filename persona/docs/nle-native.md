# NLE-Native Lookup Table

The ladder's rung 3 says *"Does the NLE do it natively?"* — this is the lookup. Before reaching for a plugin, LUT pack, or third-party tool, check here. The platform team spent years solving the problem; the plugin author wrapped it; the wrapper goes unmaintained. Skip the wrapper.

This list covers DaVinci Resolve, Premiere Pro, Final Cut Pro, and the EDITORS-PRO engine itself. When EDITORS-PRO has a native implementation, use it before adding any effect parameter that duplicates it.

## Audio

| You think you need... | The NLE already has... | Notes |
|---|---|---|
| Third-party denoiser (iZotope RX, CrumplePop) | DaVinci **Voice Isolation** (Neural Engine), Premiere **Essential Sound → Reduce Noise** | Voice Isolation is genuinely competitive with RX for dialogue. Essential Sound is decent for broadband hiss. |
| Third-party de-esser | DaVinci **De-Esser** (Fairlight), Premiere **DeEsser** | Both are fine. |
| EQ plugin (FabFilter, Pro-Q) | DaVinci **Fairlight EQ** (6-band parametric), Premiere **Parametric EQ** | Native is enough for dialogue. Reach for FabFilter only when mastering music. |
| Compressor plugin | DaVinci **Fairlight Compressor**, Premiere **Dynamics** | Native dialogue compressor is fine. |
| Loudness meter plugin (Youlean, Waves WLM) | DaVinci **Loudness meter** (EBU R128 + ATSC A/85), Premiere **Loudness Radar** | Both ship R128-compliant measurement. Don't pay for Youlean for broadcast work. |
| Reverb plugin | DaVinci **Fairlight Reverb** | Fine for room tone matching. |
| Limiter plugin | DaVinci **Fairlight Limiter**, Premiere **Hard Limiter** | Use on the master. Set ceiling to −1 dBTP for streaming, −2 dBTP for broadcast. |
| Dialogue leveling plugin | DaVinci **Dialogue Leveler**, Premiere **Essential Sound → Auto-Match Loudness** | Both target EBU R128 by default. |

## Color

| You think you need... | The NLE already has... | Notes |
|---|---|---|
| LUT utility (Lattice, DaVinci Resolve LUT builder) | DaVinci **Color Space Transform**, Premiere **Lumetri → Color Wheels** | CST is a 1D+3D LUT applied as a node. Don't reach for Lattice unless authoring LUTs for distribution. |
| Scopes plugin | DaVinci **Scopes** (Waveform, Vectorscope, Parade, Histogram), Premiere **Lumetri Scopes** | Both are broadcast-grade. Don't buy a scope plugin. |
| Denoise plugin (Neat Video) | DaVinci **Temporal NR** + **Spatial NR** | Neat Video is still better for extreme noise, but native handles 80% of cases. |
| Stabilizer plugin (Mercalli Pro) | DaVinci **Stabilizer**, Premiere **Warp Stabilizer** | Warp is excellent for 2D. DaVinci's stabilizer is similar. Reach for Mercalli only with rolling-shower jitter or parallax-heavy motion. |
| Film grain plugin (FilmConvert) | DaVinci **Film Grain Creator**, EDITORS-PRO `effects/grain.rs` (17 stocks) | Native grain is fine for stylization. FilmConvert for stock-specific film emulation. |
| Lens correction plugin | DaVinci **Lens Distortion** (with .lens profile), EDITORS-PRO `effects/lens_correction.rs` (8 profiles, Brown-Conrady) | Native handles most mirrorless/cinema profiles. |
| Sky replacement plugin | DaVinci **Magic Mask** + qualifier | Two-node setup, no plugin needed. EDITORS-PRO has a `sky_replace.rs` stub for the same workflow. |
| Color match plugin | DaVinci **Shot Match**, Premiere **Auto-Match Colors** | Shot Match to a reference frame is one click. Don't buy a match plugin. |
| HSL qualifier plugin | DaVinci **Qualifier** (HSL + RGB + Luminance) | Built-in. EDITORS-PRO masking has chroma/luminance/depth variants. |

## Editing

| You think you need... | The NLE already has... | Notes |
|---|---|---|
| Multicam plugin | DaVinci **Multicam Clip**, Premiere **Multi-Camera Source Sequence**, EDITORS-PRO `effects/multicam.rs` | All native. EDITORS-PRO has the engine; UI in `multicam_switcher.dart`. |
| Speed ramp plugin | DaVinci **Retime Controls**, Premiere **Time Remapping**, EDITORS-PRO `timeline/speed_curve.rs` + `effects/retiming.rs` | All native. EDITORS-PRO has 5 CapCut velocity presets + 7 interpolation types + optical flow. |
| Motion tracking plugin | DaVinci **Tracker** (point, planar, camera), Premiere **Track Matte** + Mask Path | DaVinci's planar tracker is professional-grade. EDITORS-PRO has `motion_tracking.rs` stub. |
| Mask drawing plugin | DaVinci **Magic Mask** (AI) + **Window** (Bezier), Premiere **Pen tool** mask | All native. EDITORS-PRO `effects/masking.rs` has Rectangle/Ellipse/Bezier/Luminance/Chroma/Depth. |
| Beat sync plugin | DaVinci **Audio Transient Detection**, Premiere **Mark audio beats** | Both detect transients. EDITORS-PRO has `analysis/beat_detect.rs` stub. |
| Proxy workflow plugin | DaVinci **Optimized Media** + **Proxy**, Premiere **Proxies**, EDITORS-PRO `proxy/` | All native. EDITORS-PRO has auto-proxy on import when res > threshold. |
| Batch export plugin | DaVinci **Render Queue**, Premiere **Adobe Media Encoder queue**, EDITORS-PRO `export_engine/batch.rs` | All native. EDITORS-PRO has the queue; UI in `batch_export_queue.dart`. |
| EDL / XML / AAF export plugin | DaVinci **Export → AAF / EDL / XML**, Premiere **Export → Final Cut Pro XML / AAF** | All native. EDITORS-PRO has `project/interop.rs` for EDL / FCPXML / OpenTimelineIO. |

## Format / codec

| You think you need... | The NLE already has... | Notes |
|---|---|---|
| ProRes encoder plugin | DaVinci, Premiere, FCP all encode ProRes natively | EDITORS-PRO `export_engine/encoder.rs` does H.264/H.265/VP9; ProRes is on the roadmap. |
| HDR metadata tool | DaVinci **HDR palette**, Premiere **Lumetri HDR**, FCP **HDR scopes** | All native. EDITORS-PRO `effects/color_space.rs` does HDR PQ/HLG tone mapping; metadata embedding is the gap. |
| Loudness QC tool | DaVinci **Loudness meter** (export report), Premiere **Loudness Radar** | Don't ship a deliverable without running this. EDITORS-PRO `analysis/loudness.rs` computes R128. |

## Audio effects not yet in EDITORS-PRO engine

These are the gaps where EDITORS-PRO currently has no native equivalent. The `dialogue-cleanup` skill documents the upgrade path.

| Need | EDITORS-PRO status | Workaround |
|---|---|---|
| De-noise (dialogue) | Not in engine | Use proxy of audio cleaned in DaVinci/Premiere, re-link in EDITORS-PRO |
| De-reverb | Not in engine | Same — clean externally, re-link |
| Spectral repair | Not in engine | iZotope RX externally |
| Pitch-preserving speed change | Not in engine | `effects/retiming.rs` has 7 interpolators; pitch preservation is a `video:` marker |
