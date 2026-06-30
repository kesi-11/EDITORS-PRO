#!/usr/bin/env node
/**
 * video-mode-tracker.js
 *
 * UserPromptSubmit hook for the EDITORS-PRO video persona.
 *
 * - Parses /video [level] and "stop video" / "normal mode" from the
 *   user's prompt. Updates the .video-active flag file.
 * - If the mode changed, emits the new ruleset (filtered by mode) as
 *   additional context for the agent. Otherwise emits nothing.
 *
 * Inspired by ponytail's ponytail-mode-tracker.js. Discipline:
 * isDeactivationCommand requires the WHOLE message to be the command,
 * so "add a normal mode toggle" doesn't accidentally turn the persona off.
 *
 * Invocation:
 *   node persona/hooks/video-mode-tracker.js <prompt>
 *
 *   (or read prompt from stdin if no arg)
 */

const { parseModeCommand, getActiveMode, setActiveMode, getDefaultMode } = require('./video-config');
const { getVideoInstructions, getCompactInstruction } = require('./video-instructions');

function readPrompt() {
  if (process.argv[2]) return process.argv[2];
  // Read from stdin (some hosts pipe the prompt)
  try {
    return fs.readFileSync(0, 'utf8');
  } catch (e) {
    return '';
  }
}

const fs = require('fs');

function main() {
  const prompt = readPrompt();
  const projectRoot = process.cwd();

  const cmd = parseModeCommand(prompt);
  if (!cmd) {
    // Not a mode-change command — emit nothing (the SessionStart
    // already injected the ruleset; re-injecting would bloat context).
    process.stdout.write('');
    return;
  }

  let newMode = cmd.mode;
  if (newMode === null) {
    // /video alone — use default
    newMode = getDefaultMode();
  }

  const oldMode = getActiveMode(projectRoot);
  if (newMode === oldMode) {
    // No-op — emit a one-line confirmation
    process.stdout.write(`[video persona] Already in ${newMode} mode.\n`);
    return;
  }

  setActiveMode(newMode, projectRoot);
  process.stderr.write(`[video persona] Mode: ${oldMode} → ${newMode}\n`);

  if (newMode === 'off') {
    process.stdout.write('[video persona] Deactivated. Normal mode.\n');
    return;
  }

  // Emit the new ruleset (filtered by mode)
  process.stdout.write(getVideoInstructions(newMode));
}

main();
