/**
 * video-config.js
 *
 * Shared config + helpers for the video persona hooks.
 * Inspired by ponytail's ponytail-config.js.
 *
 * Resolution order for default mode:
 *   1. VIDEO_DEFAULT_MODE env var
 *   2. ~/.config/editors-pro/video.json
 *   3. 'full' (broadcast default — the safe default for pro work)
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const VALID_MODES = new Set(['lite', 'full', 'ultra', 'off']);
const SAFE_DEFAULT_MODE = 'full'; // broadcast default — the safe pro default

/**
 * Resolve the default persona mode.
 */
function getDefaultMode() {
  // 1. env var
  const env = process.env.VIDEO_DEFAULT_MODE;
  if (env && VALID_MODES.has(env.toLowerCase())) {
    return env.toLowerCase();
  }

  // 2. config file
  const cfgPath = path.join(
    process.env.XDG_CONFIG_HOME || path.join(os.homedir(), '.config'),
    'editors-pro',
    'video.json'
  );
  try {
    const cfg = JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
    if (cfg.defaultMode && VALID_MODES.has(cfg.defaultMode.toLowerCase())) {
      return cfg.defaultMode.toLowerCase();
    }
  } catch (e) {
    // fall through
  }

  // 3. safe default
  return SAFE_DEFAULT_MODE;
}

/**
 * Path to the .video-active flag file in the project root.
 * The flag file stores the currently active mode (or 'off').
 */
function flagFilePath(projectRoot) {
  return path.join(projectRoot || process.cwd(), '.video-active');
}

/**
 * Read the active mode from the flag file. Returns 'off' if not active
 * or file missing.
 */
function getActiveMode(projectRoot) {
  try {
    const content = fs.readFileSync(flagFilePath(projectRoot), 'utf8').trim();
    if (VALID_MODES.has(content)) {
      return content;
    }
    return 'off';
  } catch (e) {
    return 'off';
  }
}

/**
 * Write the active mode to the flag file.
 */
function setActiveMode(mode, projectRoot) {
  if (!VALID_MODES.has(mode)) {
    throw new Error(`Invalid mode: ${mode}. Must be one of: ${Array.from(VALID_MODES).join(', ')}`);
  }
  fs.writeFileSync(flagFilePath(projectRoot), mode, 'utf8');
}

/**
 * Detect whether a user prompt is a persona activation/deactivation command.
 * Mirrors ponytail's isDeactivationCommand discipline — the WHOLE message
 * must be the command, so "add a normal mode toggle" doesn't accidentally
 * turn the persona off.
 */
function parseModeCommand(prompt) {
  const trimmed = prompt.trim().toLowerCase();
  if (trimmed === '') return null;

  // /video [level]
  const m = trimmed.match(/^\/video\s+(lite|full|ultra|off)$/);
  if (m) return { kind: 'set', mode: m[1] };

  // /video alone = activate with default
  if (trimmed === '/video') return { kind: 'set', mode: null };

  // stop video / normal mode — must be the whole message
  if (trimmed === 'stop video' || trimmed === 'normal mode') {
    return { kind: 'set', mode: 'off' };
  }

  return null;
}

/**
 * Check whether a path is shell-safe (alphanumeric + limited punctuation).
 * Used when the statusline echoes paths back to the user.
 */
function isShellSafe(s) {
  return /^[A-Za-z0-9._\-\/\s]+$/.test(s);
}

module.exports = {
  VALID_MODES,
  SAFE_DEFAULT_MODE,
  getDefaultMode,
  flagFilePath,
  getActiveMode,
  setActiveMode,
  parseModeCommand,
  isShellSafe,
};
