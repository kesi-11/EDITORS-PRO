#!/usr/bin/env node
/**
 * video-activate.js
 *
 * SessionStart hook for the EDITORS-PRO video persona.
 *
 * - Writes the .video-active flag file with the default mode (or keeps
 *   existing mode if flag file already exists).
 * - Emits the canonical ruleset as hidden context for the agent.
 * - Nudges the user to wire up a statusline if they don't have one.
 *
 * Inspired by ponytail's ponytail-activate.js.
 *
 * Invocation:
 *   node persona/hooks/video-activate.js
 *
 * Stdout is the instructions to inject into the agent's context.
 * Hooks wrap this in host-specific output shapes (Claude raw stdout,
 * Codex {systemMessage, hookSpecificOutput}, Copilot {additionalContext}).
 */

const fs = require('fs');
const path = require('path');
const {
  getDefaultMode,
  getActiveMode,
  setActiveMode,
  flagFilePath,
} = require('./video-config');
const { getVideoInstructions } = require('./video-instructions');

function main() {
  const projectRoot = process.cwd();
  const flagPath = flagFilePath(projectRoot);

  // Determine active mode
  let mode = getActiveMode(projectRoot);
  if (mode === 'off') {
    // First session — use default
    mode = getDefaultMode();
    setActiveMode(mode, projectRoot);
  }

  // Inject canonical ruleset
  const instructions = getVideoInstructions(mode);
  process.stdout.write(instructions);

  // Nudge: statusline setup
  // (kept minimal — the actual statusline is in video-statusline.sh)
  const claudeSettings = path.join(process.env.HOME || '', '.claude', 'settings.json');
  if (process.platform !== 'win32' && fs.existsSync(path.dirname(claudeSettings))) {
    try {
      const settings = JSON.parse(fs.readFileSync(claudeSettings, 'utf8'));
      if (!settings.statusLine) {
        process.stderr.write(
          `\n[video persona] Tip: add a statusline to ~/.claude/settings.json to see the active mode:\n` +
          `  "statusLine": { "type": "command", "command": "bash ${path.resolve(__dirname, 'video-statusline.sh')}" }\n`
        );
      }
    } catch (e) {
      // ignore
    }
  }
}

main();
