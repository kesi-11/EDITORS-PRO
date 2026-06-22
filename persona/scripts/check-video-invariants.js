#!/usr/bin/env node
/**
 * check-video-invariants.js
 *
 * Pins safety-critical phrases that MUST appear in the canonical ruleset
 * (AGENTS.md) and in every skill that claims to touch the relevant domain.
 *
 * Inspired by ponytail's check-rule-copies.js — a reword that drops a
 * safety-critical phrase from any file trips CI.
 *
 * Run:  node persona/scripts/check-video-invariants.js
 * Exit: 0 if all invariants hold, 1 otherwise.
 */

const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const AGENTS_MD = path.join(ROOT, 'AGENTS.md');
const SKILLS_DIR = path.join(ROOT, 'skills');

/**
 * Each invariant pins a phrase that must appear in AGENTS.md AND in every
 * skill listed under `skills`. If any file is missing the phrase, fail.
 *
 * `phrase` is matched as a substring (case-sensitive — these are technical
 * terms). `mustAlsoAppearIn` lists the skill directory names that must
 * contain the phrase.
 */
const INVARIANTS = [
  {
    name: 'EBU R128 loudness target',
    phrase: '−23 LUFS',
    mustAlsoAppearIn: ['loudness-target', 'broadcast-legal', 'delivery-encode-ladder'],
  },
  {
    name: 'ATSC A/85 loudness target',
    phrase: '−24 LKFS',
    mustAlsoAppearIn: ['loudness-target', 'broadcast-legal'],
  },
  {
    name: 'True-peak ceiling',
    phrase: 'dBTP',
    mustAlsoAppearIn: ['loudness-target', 'broadcast-legal', 'delivery-encode-ladder'],
  },
  {
    name: 'Legal color range',
    phrase: 'legal range',
    mustAlsoAppearIn: ['broadcast-legal', 'color-scopes', 'hdr-delivery'],
  },
  {
    name: 'Title-safe area',
    phrase: 'title-safe',
    mustAlsoAppearIn: ['broadcast-legal'],
  },
  {
    name: 'Frame-rate compliance',
    phrase: 'frame-rate',
    mustAlsoAppearIn: ['broadcast-legal', 'delivery-encode-ladder'],
  },
  {
    name: 'Field-order compliance',
    phrase: 'field-order',
    mustAlsoAppearIn: ['broadcast-legal'],
  },
  {
    name: 'Color space tagging',
    phrase: 'color space',
    mustAlsoAppearIn: ['broadcast-legal', 'hdr-delivery', 'color-scopes'],
  },
  {
    name: 'Delivery spec compliance',
    phrase: 'delivery spec',
    mustAlsoAppearIn: ['broadcast-legal', 'delivery-encode-ladder', 'format-interop'],
  },
  {
    name: 'Data loss prevention',
    phrase: 'data loss',
    mustAlsoAppearIn: ['broadcast-legal', 'batch-export'],
  },
  {
    name: 'LUT cube format',
    phrase: '.cube',
    mustAlsoAppearIn: ['lut-management', 'color-match-shots'],
  },
  {
    name: 'Vectorscope',
    phrase: 'vectorscope',
    mustAlsoAppearIn: ['color-scopes', 'broadcast-legal'],
  },
  {
    name: 'Waveform monitor',
    phrase: 'waveform',
    mustAlsoAppearIn: ['color-scopes', 'broadcast-legal'],
  },
  {
    name: 'ProRes codec',
    phrase: 'ProRes',
    mustAlsoAppearIn: ['delivery-encode-ladder', 'format-interop'],
  },
  {
    name: 'Stabilization ceiling',
    phrase: '3D camera solve',
    mustAlsoAppearIn: ['video-stabilization'],
  },
];

function readFile(p) {
  try {
    return fs.readFileSync(p, 'utf8');
  } catch (e) {
    return null;
  }
}

function check() {
  const errors = [];
  const warnings = [];

  const agents = readFile(AGENTS_MD);
  if (!agents) {
    errors.push(`FATAL: ${AGENTS_MD} not found or unreadable`);
    return { errors, warnings, totalMarkers: 0 };
  }

  let totalChecks = 0;
  let passedChecks = 0;

  for (const inv of INVARIANTS) {
    // Check AGENTS.md
    totalChecks++;
    if (agents.includes(inv.phrase)) {
      passedChecks++;
    } else {
      errors.push(`AGENTS.md is missing pinned phrase "${inv.phrase}" (${inv.name})`);
    }

    // Check each named skill
    for (const skillName of inv.mustAlsoAppearIn) {
      const skillPath = path.join(SKILLS_DIR, skillName, 'SKILL.md');
      const skillContent = readFile(skillPath);
      totalChecks++;
      if (skillContent === null) {
        errors.push(`Skill file not found: ${skillPath}`);
        continue;
      }
      if (skillContent.includes(inv.phrase)) {
        passedChecks++;
      } else {
        errors.push(
          `${skillPath}\n  is missing pinned phrase "${inv.phrase}" (${inv.name})\n  This safety-critical invariant must appear verbatim.`
        );
      }
    }
  }

  return { errors, warnings, totalChecks, passedChecks };
}

function main() {
  console.log('Checking video persona invariants...\n');

  const { errors, warnings, totalChecks, passedChecks } = check();

  if (warnings.length > 0) {
    console.log('Warnings:');
    for (const w of warnings) console.log(`  ⚠️  ${w}`);
    console.log('');
  }

  if (errors.length === 0) {
    console.log(`✅ All ${passedChecks}/${totalChecks} invariant checks passed.`);
    console.log(`   ${INVARIANTS.length} safety-critical phrases pinned across AGENTS.md + skills.`);
    process.exit(0);
  } else {
    console.log(`❌ ${errors.length} invariant violation(s) found.`);
    console.log(`   ${passedChecks}/${totalChecks} checks passed.\n`);
    for (const e of errors) {
      console.log(`  ✗ ${e}`);
    }
    console.log('\nTo fix: add the missing phrase verbatim to the listed file(s).');
    console.log('Safety-critical phrases must not be silently reworded out of any skill.');
    process.exit(1);
  }
}

main();
