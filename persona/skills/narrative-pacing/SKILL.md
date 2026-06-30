---
name: narrative-pacing
description: >
  Narrative Pacing. Use when the user says "pacing", "rhythm of the edit", "tighten the cut", "breathing room", "hold on the shot", "let it breathe", "draggy", "rushed", "rhythm",
  or whenever they describe a workflow involving pacing.
license: MIT
---

# Narrative Pacing

## The trick

Pacing is the rhythm of the cut — when to hold, when to cut, when to leave silence. The ladder: (1) cut anything that doesn't serve the story, (2) hold on reactions, not actions, (3) trust silence — don't fill every gap with B-roll, (4) cut on motion (entering/leaving frame, gesture completion), (5) match emotional beats, not just visual beats.

The amateur move is to cut to keep the viewer's attention with rapid-fire shots. The pro move is to cut to serve the moment — sometimes that means a 6-second hold on a face, sometimes that means a hard cut on a gesture mid-motion. Pace serves story, not the other way around.

## When to use

Always. Pacing is the editor's primary storytelling tool.

## When NOT to use

Never. Pacing applies to every edit.

## Examples

**Amateur** ❌: Cutting every 1-2 seconds to "keep it engaging." The result is exhausting and meaningless — the viewer can't absorb anything.

**Professional** ✅: Cut to serve the moment. Reaction shots held longer than expected. Hard cuts on motion. Silence left in. The edit breathes. Pacing matches the emotional arc, not the music.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Tight cuts for social attention span. 1-3 second average shot length. |
| `full` | Pacing serves the story. Variable shot length. Reaction holds. |
| `ultra` | Frame-precise pacing. Emotional arc mapped. Pacing documented in the editor's notes. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `delivery spec`

## Boundaries

Narrative pacing covers the rhythm of the cut. It does not cover shot selection (the director/DP's job), color (color-scopes), or audio (loudness-target).
