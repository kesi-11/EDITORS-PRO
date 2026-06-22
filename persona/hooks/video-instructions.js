/**
 * video-instructions.js
 *
 * Reads the canonical AGENTS.md ruleset and produces the active-mode
 * instructions to inject into the agent's context. Filters by intensity
 * level so the agent only sees the rules relevant to the current delivery
 * tier. Inspired by ponytail's ponytail-instructions.js — one shared
 * builder so all hosts emit identical rules.
 *
 * Used by:
 *   - hooks/video-activate.js     (SessionStart injection)
 *   - hooks/video-mode-tracker.js (UserPromptSubmit mode-change injection)
 *   - persona-mcp server (if added later)
 *   - any future host adapter
 */

const fs = require('fs');
const path = require('path');

const AGENTS_MD_PATH = path.resolve(__dirname, '..', 'AGENTS.md');

/**
 * The fallback instructions if AGENTS.md can't be read. Kept short and
 * safety-critical — if the file is missing, the persona still enforces
 * the never-cut list.
 */
const FALLBACK = `# EDITORS-PRO Persona — fallback (AGENTS.md unavailable)

You are a professional videographer. Lazy means efficient, not careless.

The ladder: YAGNI → reuse preset → NLE-native → platform-native → installed plugin → one node → minimum graph.

Never cut, regardless of intensity:
- Broadcast loudness (−23 LUFS EBU R128, −24 LKFS ATSC A/85, platform targets)
- True-peak ceiling (≤ −1 dBTP streaming, ≤ −2 dBTP broadcast)
- Legal color range (Rec.709 16–235 / 64–940)
- Title-safe (90% broadcast, 80% social captions)
- Frame-rate and field-order compliance
- Color space and gamma tagging on encode
- Anything in the delivery spec
- Anything that prevents data loss

Mark every shortcut with a \`video:\` comment and its ceiling.

WARNING: AGENTS.md could not be read; using fallback. Restore the canonical ruleset at persona/AGENTS.md.`;

/**
 * Filter the ruleset by intensity level. The full ruleset is always
 * injected; lite/ultra add a context paragraph at the top.
 */
function filterByMode(content, mode) {
  const header = `ACTIVE MODE: ${mode.toUpperCase()}\n\n`;

  if (mode === 'lite') {
    return header +
      `Enforcement posture: SOCIAL CUT — vertical, captions, platform LUFS, one-pass grade.\n` +
      `Skips allowed: legalizer pass, formal QC, scene-referred grade, ACES.\n` +
      `Still required: platform loudness (YouTube −14, TikTok −18), true-peak ≤ −1 dBTP, caption title-safe 80%, frame-rate match.\n\n` +
      content;
  }
  if (mode === 'full') {
    return header +
      `Enforcement posture: BROADCAST DEFAULT — legal range, −23 LUFS EBU R128, true-peak ≤ −2 dBTP, title-safe, full QC.\n` +
      `No skips on the never-cut list.\n\n` +
      content;
  }
  if (mode === 'ultra') {
    return header +
      `Enforcement posture: FEATURE / FESTIVAL GRADE — 10-bit minimum, ACES, scene-referred, full scopes, loudness per spec, frame-rate & field-order verified.\n` +
      `No skips. Every \`video:\` marker is debt that must be retired before delivery.\n\n` +
      content;
  }
  return content; // 'off' or unknown — return raw
}

/**
 * Get the active-mode instructions to inject into agent context.
 * Returns the canonical ruleset with a mode-specific header.
 */
function getVideoInstructions(mode) {
  let content;
  try {
    content = fs.readFileSync(AGENTS_MD_PATH, 'utf8');
  } catch (e) {
    content = FALLBACK;
  }
  return filterByMode(content, mode);
}

/**
 * Compact form for statusline / one-line displays.
 */
function getCompactInstruction(mode) {
  const map = {
    lite: 'VIDEO:LITE — social cut, platform LUFS, one-pass grade',
    full: 'VIDEO:FULL — broadcast, −23 LUFS, legal range, title-safe',
    ultra: 'VIDEO:ULTRA — feature grade, ACES, 10-bit, full QC',
    off: 'VIDEO:OFF — normal mode',
  };
  return map[mode] || map.off;
}

module.exports = {
  AGENTS_MD_PATH,
  FALLBACK,
  getVideoInstructions,
  getCompactInstruction,
};
