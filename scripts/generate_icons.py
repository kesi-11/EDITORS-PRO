#!/usr/bin/env python3
"""Generate professional SVG icons for EDITORS-PRO video editor."""

import os

SVG_DIR = "/home/z/my-project/EDITORS-PRO/assets/icons/svg"
os.makedirs(SVG_DIR, exist_ok=True)

def write_svg(name, content):
    with open(os.path.join(SVG_DIR, f"{name}.svg"), "w") as f:
        f.write(content.strip())
    print(f"  ✓ {name}.svg")

# ─── Logo ─────────────────────────────────────────────────────────
write_svg("logo", """
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none">
  <defs>
    <linearGradient id="lg" x1="0" y1="0" x2="48" y2="48">
      <stop offset="0%" stop-color="#6C5CE7"/>
      <stop offset="100%" stop-color="#A29BFE"/>
    </linearGradient>
  </defs>
  <rect width="48" height="48" rx="12" fill="url(#lg)"/>
  <path d="M14 16l10 8-10 8V16z" fill="white" opacity="0.95"/>
  <path d="M24 16l10 8-10 8V16z" fill="white" opacity="0.6"/>
</svg>
""")

# ─── Navigation ───────────────────────────────────────────────────
write_svg("back", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M19 12H5M12 19l-7-7 7-7"/>
</svg>
""")

write_svg("close", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <path d="M18 6L6 18M6 6l12 12"/>
</svg>
""")

write_svg("menu", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <path d="M4 6h16M4 12h16M4 18h16"/>
</svg>
""")

write_svg("settings", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="3"/>
  <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
</svg>
""")

write_svg("search", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="11" cy="11" r="8"/>
  <path d="M21 21l-4.35-4.35"/>
</svg>
""")

write_svg("more_horizontal", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
  <circle cx="5" cy="12" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="19" cy="12" r="2"/>
</svg>
""")

write_svg("chevron_down", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M6 9l6 6 6-6"/>
</svg>
""")

write_svg("chevron_right", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M9 18l6-6-6-6"/>
</svg>
""")

# ─── Editor Toolbar ───────────────────────────────────────────────
write_svg("undo", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M3 7v6h6"/><path d="M21 17a9 9 0 00-9-9 9 9 0 00-6.69 3L3 13"/>
</svg>
""")

write_svg("redo", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M21 7v6h-6"/><path d="M3 17a9 9 0 019-9 9 9 0 016.69 3L21 13"/>
</svg>
""")

write_svg("save", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z"/>
  <polyline points="17 21 17 13 7 13 7 21"/>
  <polyline points="7 3 7 8 15 8"/>
</svg>
""")

write_svg("export", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
  <polyline points="7 10 12 15 17 10"/>
  <line x1="12" y1="15" x2="12" y2="3"/>
</svg>
""")

write_svg("share", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
  <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
  <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
</svg>
""")

# ─── Playback ─────────────────────────────────────────────────────
write_svg("play", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
  <path d="M8 5v14l11-7z"/>
</svg>
""")

write_svg("pause", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
  <rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>
</svg>
""")

write_svg("skip_back", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
  <path d="M11 12L21 4v16zM3 4h2v16H3z"/>
</svg>
""")

write_svg("skip_forward", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
  <path d="M13 12L3 4v16zM19 4h2v16h-2z"/>
</svg>
""")

write_svg("speed", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 12l3.5-3.5"/>
  <path d="M20.3 18c.4-1 .7-2.2.7-3.4C21 9.8 17 6 12 6s-9 3.8-9 8.6c0 1.2.3 2.4.7 3.4"/>
  <circle cx="12" cy="12" r="1.5" fill="currentColor"/>
</svg>
""")

# ─── Media & Files ────────────────────────────────────────────────
write_svg("import", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
  <polyline points="17 8 12 3 7 8"/>
  <line x1="12" y1="3" x2="12" y2="15"/>
</svg>
""")

write_svg("video", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="2" y="4" width="15" height="16" rx="2"/>
  <path d="M17 8l5-3v14l-5-3"/>
</svg>
""")

write_svg("audio", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M9 18V5l12-2v13"/>
  <circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
</svg>
""")

write_svg("image", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="3" y="3" width="18" height="18" rx="2"/>
  <circle cx="8.5" cy="8.5" r="1.5"/>
  <polyline points="21 15 16 10 5 21"/>
</svg>
""")

write_svg("file", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
  <polyline points="14 2 14 8 20 8"/>
</svg>
""")

write_svg("folder", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
</svg>
""")

write_svg("cloud", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z"/>
</svg>
""")

# ─── Timeline ─────────────────────────────────────────────────────
write_svg("split", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <line x1="12" y1="2" x2="12" y2="22"/>
  <path d="M4 8h4v8H4z" fill="currentColor" opacity="0.3"/>
  <path d="M16 8h4v8h-4z" fill="currentColor" opacity="0.3"/>
  <path d="M4 8h4v8H4z"/>
  <path d="M16 8h4v8h-4z"/>
</svg>
""")

write_svg("delete", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <polyline points="3 6 5 6 21 6"/>
  <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
  <line x1="10" y1="11" x2="10" y2="17"/>
  <line x1="14" y1="11" x2="14" y2="17"/>
</svg>
""")

write_svg("copy", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="9" y="9" width="13" height="13" rx="2"/>
  <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
</svg>
""")

write_svg("paste", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2"/>
  <rect x="8" y="2" width="8" height="4" rx="1"/>
</svg>
""")

write_svg("lock", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="3" y="11" width="18" height="11" rx="2"/>
  <path d="M7 11V7a5 5 0 0110 0v4"/>
</svg>
""")

write_svg("unlock", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="3" y="11" width="18" height="11" rx="2"/>
  <path d="M7 11V7a5 5 0 019.9-1"/>
</svg>
""")

write_svg("visible", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
  <circle cx="12" cy="12" r="3"/>
</svg>
""")

write_svg("hidden", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M17.94 17.94A10.07 10.07 0 0112 20c-7 0-11-8-11-8a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 4c7 0 11 8 11 8a18.5 18.5 0 01-2.16 3.19m-6.72-1.07a3 3 0 11-4.24-4.24"/>
  <line x1="1" y1="1" x2="23" y2="23"/>
</svg>
""")

write_svg("zoom_in", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
  <line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/>
</svg>
""")

write_svg("zoom_out", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
  <line x1="8" y1="11" x2="14" y2="11"/>
</svg>
""")

# ─── Effects & Filters ────────────────────────────────────────────
write_svg("effects", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 3l1.912 5.813a2 2 0 001.275 1.275L21 12l-5.813 1.912a2 2 0 00-1.275 1.275L12 21l-1.912-5.813a2 2 0 00-1.275-1.275L3 12l5.813-1.912a2 2 0 001.275-1.275L12 3z"/>
</svg>
""")

write_svg("filters", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
</svg>
""")

write_svg("transitions", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <rect x="2" y="4" width="8" height="16" rx="1" fill="currentColor" opacity="0.15"/>
  <rect x="14" y="4" width="8" height="16" rx="1" fill="currentColor" opacity="0.15"/>
  <path d="M10 4L14 12L10 20" stroke="currentColor" stroke-width="2"/>
  <rect x="2" y="4" width="8" height="16" rx="1"/>
  <rect x="14" y="4" width="8" height="16" rx="1"/>
</svg>
""")

write_svg("color_wheel", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
  <circle cx="12" cy="12" r="9"/>
  <circle cx="12" cy="12" r="3"/>
  <line x1="12" y1="3" x2="12" y2="9"/><line x1="12" y1="15" x2="12" y2="21"/>
  <line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/>
  <line x1="5.64" y1="5.64" x2="9.88" y2="9.88"/><line x1="14.12" y1="14.12" x2="18.36" y2="18.36"/>
  <line x1="5.64" y1="18.36" x2="9.88" y2="14.12"/><line x1="14.12" y1="9.88" x2="18.36" y2="5.64"/>
</svg>
""")

write_svg("blur", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="12" cy="12" r="10" opacity="0.2" stroke-width="4"/>
  <circle cx="12" cy="12" r="6" opacity="0.3" stroke-width="3"/>
  <circle cx="12" cy="12" r="2" fill="currentColor"/>
</svg>
""")

write_svg("brightness", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="12" cy="12" r="5"/>
  <line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/>
  <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
  <line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>
  <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
</svg>
""")

write_svg("contrast", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="12" cy="12" r="10"/>
  <path d="M12 2a10 10 0 010 20z" fill="currentColor" opacity="0.3"/>
</svg>
""")

write_svg("saturation", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <path d="M12 2.69l5.66 5.66a8 8 0 11-11.31 0z"/>
  <path d="M12 2.69v18.62" opacity="0.3"/>
</svg>
""")

# ─── Text ─────────────────────────────────────────────────────────
write_svg("text", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <polyline points="4 7 4 4 20 4 20 7"/>
  <line x1="9" y1="20" x2="15" y2="20"/>
  <line x1="12" y1="4" x2="12" y2="20"/>
</svg>
""")

write_svg("text_bold", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
  <path d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z"/>
  <path d="M6 12h9a4 4 0 014 4 4 4 0 01-4 4H6z"/>
</svg>
""")

write_svg("text_italic", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <line x1="19" y1="4" x2="10" y2="4"/>
  <line x1="14" y1="20" x2="5" y2="20"/>
  <line x1="15" y1="4" x2="9" y2="20"/>
</svg>
""")

write_svg("text_align_left", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <line x1="17" y1="10" x2="3" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/>
  <line x1="21" y1="14" x2="3" y2="14"/><line x1="17" y1="18" x2="3" y2="18"/>
</svg>
""")

write_svg("text_align_center", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <line x1="18" y1="10" x2="6" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/>
  <line x1="21" y1="14" x2="3" y2="14"/><line x1="18" y1="18" x2="6" y2="18"/>
</svg>
""")

# ─── Keyframes ────────────────────────────────────────────────────
write_svg("keyframe", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor" opacity="0.8">
  <path d="M12 4l8 8-8 8-8-8z" stroke="currentColor" stroke-width="1.5" fill="currentColor" opacity="0.2"/>
</svg>
""")

write_svg("keyframe_add", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <path d="M12 4l8 8-8 8-8-8z" fill="currentColor" opacity="0.15"/>
  <line x1="12" y1="8" x2="12" y2="16"/>
  <line x1="8" y1="12" x2="16" y2="12"/>
</svg>
""")

write_svg("timeline", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <rect x="1" y="6" width="22" height="4" rx="1" fill="currentColor" opacity="0.15"/>
  <rect x="1" y="14" width="22" height="4" rx="1" fill="currentColor" opacity="0.1"/>
  <line x1="8" y1="4" x2="8" y2="20" stroke="#FF4444" stroke-width="1.5"/>
</svg>
""")

# ─── Tools ────────────────────────────────────────────────────────
write_svg("crop", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M6.13 1L6 16a2 2 0 002 2h15"/>
  <path d="M1 6.13L16 6a2 2 0 012 2v15"/>
</svg>
""")

write_svg("transform", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/>
</svg>
""")

write_svg("mask", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="10"/>
  <circle cx="12" cy="12" r="4" fill="currentColor" opacity="0.3"/>
</svg>
""")

write_svg("scissors", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/>
  <line x1="20" y1="4" x2="8.12" y2="15.88"/>
  <line x1="14.47" y1="14.48" x2="20" y2="20"/>
  <line x1="8.12" y1="8.12" x2="12" y2="12"/>
</svg>
""")

write_svg("hand", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M18 11V6a2 2 0 00-4 0v1M14 10V4a2 2 0 00-4 0v6M10 10.5V5a2 2 0 00-4 0v9l-1.5-1.5a2 2 0 00-3 3l4.5 4.5A6 6 0 0014 20h2a6 6 0 006-6v-3a2 2 0 00-4 0"/>
</svg>
""")

write_svg("pointer", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/>
  <path d="M13 13l6 6"/>
</svg>
""")

write_svg("marker", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 2L2 7l10 5 10-5-10-5z" fill="currentColor" opacity="0.15"/>
  <path d="M2 17l10 5 10-5"/>
  <path d="M2 12l10 5 10-5"/>
  <path d="M12 2L2 7l10 5 10-5-10-5z"/>
</svg>
""")

# ─── Status ───────────────────────────────────────────────────────
write_svg("check", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
  <polyline points="20 6 9 17 4 12"/>
</svg>
""")

write_svg("warning", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
  <line x1="12" y1="9" x2="12" y2="13"/>
  <line x1="12" y1="17" x2="12.01" y2="17"/>
</svg>
""")

write_svg("error", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="10"/>
  <line x1="15" y1="9" x2="9" y2="15"/>
  <line x1="9" y1="9" x2="15" y2="15"/>
</svg>
""")

write_svg("info", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <circle cx="12" cy="12" r="10"/>
  <line x1="12" y1="16" x2="12" y2="12"/>
  <line x1="12" y1="8" x2="12.01" y2="8"/>
</svg>
""")

write_svg("gpu", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="4" y="6" width="16" height="12" rx="2"/>
  <path d="M8 2v4M12 2v4M16 2v4"/>
  <circle cx="9" cy="12" r="2" fill="currentColor" opacity="0.4"/>
  <circle cx="15" cy="12" r="2" fill="currentColor" opacity="0.4"/>
  <line x1="4" y1="22" x2="20" y2="22" opacity="0.3"/>
</svg>
""")

write_svg("cpu", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="4" y="4" width="16" height="16" rx="2"/>
  <rect x="9" y="9" width="6" height="6"/>
  <line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/>
  <line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/>
  <line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/>
  <line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/>
</svg>
""")

# ─── Project Home ─────────────────────────────────────────────────
write_svg("plus", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
  <line x1="12" y1="5" x2="12" y2="19"/>
  <line x1="5" y1="12" x2="19" y2="12"/>
</svg>
""")

write_svg("project", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="2" y="3" width="20" height="14" rx="2"/>
  <path d="M8 21h8"/>
  <path d="M12 17v4"/>
  <path d="M7 8l3 3-3 3" opacity="0.5"/>
  <line x1="13" y1="14" x2="17" y2="14" opacity="0.5"/>
</svg>
""")

write_svg("template", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="3" y="3" width="18" height="18" rx="2"/>
  <path d="M3 9h18" opacity="0.3"/>
  <path d="M9 21V9" opacity="0.3"/>
</svg>
""")

write_svg("recent", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="10"/>
  <polyline points="12 6 12 12 16 14"/>
</svg>
""")

write_svg("phone", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="5" y="2" width="14" height="20" rx="2"/>
  <line x1="12" y1="18" x2="12.01" y2="18"/>
</svg>
""")

write_svg("monitor", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="2" y="3" width="20" height="14" rx="2"/>
  <line x1="8" y1="21" x2="16" y2="21"/>
  <line x1="12" y1="17" x2="12" y2="21"/>
</svg>
""")

write_svg("film", """
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <rect x="2" y="2" width="20" height="20" rx="2.18"/>
  <line x1="7" y1="2" x2="7" y2="22"/>
  <line x1="17" y1="2" x2="17" y2="22"/>
  <line x1="2" y1="12" x2="22" y2="12"/>
  <line x1="2" y1="7" x2="7" y2="7"/>
  <line x1="2" y1="17" x2="7" y2="17"/>
  <line x1="17" y1="17" x2="22" y2="17"/>
  <line x1="17" y1="7" x2="22" y2="7"/>
</svg>
""")

# ─── Onboarding Illustrations ─────────────────────────────────────
write_svg("onboarding_import", """
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120" fill="none">
  <circle cx="60" cy="60" r="55" fill="#6C5CE7" opacity="0.08"/>
  <circle cx="60" cy="60" r="42" fill="#6C5CE7" opacity="0.12"/>
  <rect x="35" y="25" width="50" height="65" rx="6" stroke="#6C5CE7" stroke-width="2.5" fill="none"/>
  <path d="M50 55L60 65L70 55" stroke="#A29BFE" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
  <line x1="60" y1="38" x2="60" y2="62" stroke="#A29BFE" stroke-width="3" stroke-linecap="round"/>
  <path d="M38 82h44" stroke="#6C5CE7" stroke-width="2" opacity="0.4"/>
</svg>
""")

write_svg("onboarding_edit", """
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120" fill="none">
  <circle cx="60" cy="60" r="55" fill="#00CEC9" opacity="0.08"/>
  <circle cx="60" cy="60" r="42" fill="#00CEC9" opacity="0.12"/>
  <rect x="20" y="38" width="35" height="20" rx="4" stroke="#00CEC9" stroke-width="2" fill="#00CEC9" fill-opacity="0.15"/>
  <rect x="20" y="62" width="25" height="16" rx="4" stroke="#00B894" stroke-width="2" fill="#00B894" fill-opacity="0.15"/>
  <rect x="60" y="38" width="40" height="40" rx="4" stroke="#00CEC9" stroke-width="2" fill="#00CEC9" fill-opacity="0.1"/>
  <path d="M70 52L80 52" stroke="#00CEC9" stroke-width="2" stroke-linecap="round"/>
  <path d="M70 60L90 60" stroke="#00CEC9" stroke-width="2" stroke-linecap="round" opacity="0.6"/>
  <path d="M70 68L85 68" stroke="#00CEC9" stroke-width="2" stroke-linecap="round" opacity="0.3"/>
  <line x1="48" y1="30" x2="48" y2="80" stroke="#FF4444" stroke-width="1.5" opacity="0.5"/>
</svg>
""")

write_svg("onboarding_export", """
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120" fill="none">
  <circle cx="60" cy="60" r="55" fill="#FD79A8" opacity="0.08"/>
  <circle cx="60" cy="60" r="42" fill="#FD79A8" opacity="0.12"/>
  <rect x="30" y="30" width="50" height="35" rx="6" stroke="#FD79A8" stroke-width="2" fill="none"/>
  <path d="M55 65L55 85" stroke="#FD79A8" stroke-width="2.5" stroke-linecap="round"/>
  <path d="M47 78L55 87L63 78" stroke="#FD79A8" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M35 95h40" stroke="#FD79A8" stroke-width="2" opacity="0.4" stroke-linecap="round"/>
  <circle cx="55" cy="47" r="8" stroke="#FD79A8" stroke-width="2" fill="#FD79A8" fill-opacity="0.15"/>
  <path d="M55 43L55 51M51 47L59 47" stroke="#FD79A8" stroke-width="2" stroke-linecap="round"/>
</svg>
""")

print(f"\n✅ Generated {len(os.listdir(SVG_DIR))} SVG icons in {SVG_DIR}")
