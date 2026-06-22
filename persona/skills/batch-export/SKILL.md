---
name: batch-export
description: >
  Batch Export Queue. Use when the user says "batch export", "queue export", "multiple exports", "export queue", "render queue", "parallel export", "batch encode",
  or whenever they describe a workflow involving batch export.
license: MIT
---

# Batch Export Queue

## The trick

Batch export queues multiple export jobs (different resolutions, codecs, or platform targets) and runs them in sequence (or in parallel if the hardware supports it). EDITORS-PRO's `export_engine/batch.rs` provides the queue; the Flutter `batch_export_queue.dart` widget exposes it.

Workflow: (1) add jobs to the queue (e.g., 1080p H.264 for YouTube, 1080×1920 H.264 for TikTok, 4K ProRes master), (2) set the order (master first, then platform encodes), (3) start the queue, (4) the foreground `ExportService.kt` handles one job at a time, with notifications between.

## When to use

Delivering to multiple platforms. Mastering + delivery in one pass. Generating vertical + horizontal versions of the same edit.

## When NOT to use

For a single export — just use the regular export. When the platforms have different edits (not just different encodes) — those are separate projects.

## Examples

**Amateur** ❌: Exporting one version, then exporting another, manually, repeating for every platform. Hours of waiting.

**Professional** ✅: Queue the master, then the platform encodes. Start the queue. Walk away. Come back to all the deliverables.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Queue 2-3 social encodes. Single-pass. |
| `full` | Queue master + platform encodes. 2-pass VBR for streaming. Verify each encode. |
| `ultra` | Per-spec encode ladder for each platform. Full QC on each output. Document in QC report. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `data loss`
- `ProRes`
- `delivery spec`

## Boundaries

Batch export covers queueing multiple export jobs. It does not cover the encode settings per job (delivery-encode-ladder) or the loudness per platform (loudness-target).
