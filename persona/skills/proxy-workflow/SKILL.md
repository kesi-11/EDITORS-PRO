---
name: proxy-workflow
description: >
  Proxy Workflow. Use when the user says "proxy", "offline edit", "low-res preview", "proxy generation", "swap to full", "4k lag", "timeline lag", "preview lag",
  or whenever they describe a workflow involving proxy.
license: MIT
---

# Proxy Workflow

## The trick

Proxies are low-res transcodes of your source media used for editing. You cut with proxies (fast, smooth scrubbing), then swap to full-res at picture-lock for color and export. EDITORS-PRO auto-generates proxies when source res exceeds the threshold; the `proxy_status_badge.dart` widget shows proxy status.

Proxies are 1/4 or 1/8 resolution, often in a fast codec (ProRes Proxy, DNxHR LB). They keep the timeline responsive on phones and older hardware. The full-res swap is automatic at export time.

## When to use

Always when editing 4K+ on a phone or older hardware. Whenever the timeline lags. When the source codec is heavy (HEVC, AV1) and the hardware can't decode in real time.

## When NOT to use

When editing 1080p on a fast desktop with hardware decode — you may not need proxies. When the source codec is already light (ProRes Proxy).

## Examples

**Amateur** ❌: Editing 4K HEVC directly on a phone, timeline lagging, scrubbing one frame at a time, getting frustrated and giving up.

**Professional** ✅: Proxies on. Cut smoothly. Swap to full at picture-lock. Verify color on full-res before export. The proxy is a tool, not the deliverable.

## Intensity

| Level | Enforcement |
|---|---|
| `lite` | Proxies at 1/4 res. Always on for 4K+ sources. |
| `full` | Proxies at 1/4 res. Swap to full at picture-lock. Verify color on full-res. |
| `ultra` | Proxies at 1/8 res for cutting. Full-res for color. Original RAW for DI. Document the proxy generation settings. |

## Safety carve-outs (never cut)

This skill must enforce the following pinned safety phrases. A reword that drops one of these from this file trips CI (see `scripts/check-video-invariants.js`):

- `data loss`
- `delivery spec`

## Boundaries

Proxy workflow covers the generation, use, and swapping of low-res transcodes. It does not cover the source media management (project's job) or the export (delivery-encode-ladder).
