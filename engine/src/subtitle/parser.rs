//! SRT subtitle parser
//!
//! Parses SubRip (.srt) format files into structured `SubtitleEntry`
//! objects that can be used to create text clips on the timeline.

use std::fs;
use std::path::Path;

/// A single subtitle entry parsed from an SRT file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubtitleEntry {
    /// Sequential index of the subtitle (1-based in SRT)
    pub index: u32,
    /// Start time in milliseconds
    pub start_ms: u64,
    /// End time in milliseconds
    pub end_ms: u64,
    /// Subtitle text content (may contain multiple lines)
    pub text: String,
}

/// Phase E.18: styling options for subtitle rendering.
///
/// Controls the visual appearance of subtitles on the preview and in
/// exported video. Stored alongside the project so the same style is
/// applied consistently across all subtitle clips.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubtitleStyle {
    /// Font family name (e.g., "Inter", "Arial"). Must be a font that's
    /// bundled with the app or available on the device.
    pub font_family: String,

    /// Font size in pixels at the project's native resolution.
    pub font_size: f32,

    /// Whether the text is bold.
    pub bold: bool,

    /// Whether the text is italic.
    pub italic: bool,

    /// Whether the text is underlined.
    pub underline: bool,

    /// Text color as RGBA (0.0-1.0 per channel).
    pub text_color: [f32; 4],

    /// Outline (stroke) color as RGBA.
    pub outline_color: [f32; 4],

    /// Outline thickness in pixels. 0 disables the outline.
    pub outline_width: f32,

    /// Shadow color as RGBA.
    pub shadow_color: [f32; 4],

    /// Shadow offset in pixels (dx, dy). (0, 0) disables the shadow.
    pub shadow_offset: [f32; 2],

    /// Shadow blur radius in pixels.
    pub shadow_blur: f32,

    /// Background color as RGBA (for the "box" behind the text).
    /// Fully transparent (alpha=0) disables the background.
    pub background_color: [f32; 4],

    /// Horizontal alignment: -1 = left, 0 = center, 1 = right.
    pub alignment: i32,

    /// Vertical position: 0 = top, 0.5 = middle, 1 = bottom.
    /// Defaults to 0.9 (near the bottom, like most subtitles).
    pub vertical_position: f32,

    /// Animation preset for subtitle appearance.
    pub animation: SubtitleAnimation,
}

/// Phase E.18: animation presets for subtitle appearance.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleAnimation {
    /// No animation — subtitle appears instantly.
    None,
    /// Fade in from transparent.
    FadeIn,
    /// Slide up from below.
    SlideUp,
    /// Slide in from the left.
    SlideLeft,
    /// Scale up from 50% to 100%.
    ScaleUp,
    /// Typewriter effect (characters appear one at a time).
    Typewriter,
    /// Bounce in (overshoot then settle).
    Bounce,
}

impl Default for SubtitleStyle {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            font_size: 48.0,
            bold: false,
            italic: false,
            underline: false,
            text_color: [1.0, 1.0, 1.0, 1.0], // white
            outline_color: [0.0, 0.0, 0.0, 1.0], // black
            outline_width: 2.0,
            shadow_color: [0.0, 0.0, 0.0, 0.6], // semi-transparent black
            shadow_offset: [2.0, 2.0],
            shadow_blur: 4.0,
            background_color: [0.0, 0.0, 0.0, 0.0], // transparent
            alignment: 0,                            // center
            vertical_position: 0.9,                  // near bottom
            animation: SubtitleAnimation::FadeIn,
        }
    }
}

/// Parse an SRT file and return all subtitle entries
pub fn parse_srt_file(file_path: &str) -> Result<Vec<SubtitleEntry>, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("SRT file not found: {}", file_path));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read SRT file '{}': {}", file_path, e))?;

    parse_srt_content(&content)
}

/// Parse SRT-formatted string content into subtitle entries
pub fn parse_srt_content(content: &str) -> Result<Vec<SubtitleEntry>, String> {
    let mut entries = Vec::new();

    // SRT entries are separated by blank lines
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 3 {
            // A valid SRT block must have at least: index, timestamp, text
            continue;
        }

        // Parse index (first line)
        let index: u32 = match lines[0].trim().parse() {
            Ok(n) => n,
            Err(_) => continue, // Skip malformed entries
        };

        // Parse timestamp (second line)
        // Format: HH:MM:SS,mmm --> HH:MM:SS,mmm
        let timestamp_line = lines[1].trim();
        let (start_ms, end_ms) = match parse_srt_timestamp(timestamp_line) {
            Some((s, e)) => (s, e),
            None => continue, // Skip entries with invalid timestamps
        };

        // Remaining lines are the subtitle text
        let text: String = lines[2..].join("\n").trim().to_string();
        if text.is_empty() {
            continue;
        }

        entries.push(SubtitleEntry {
            index,
            start_ms,
            end_ms,
            text,
        });
    }

    log::info!("Parsed {} subtitle entries from SRT content", entries.len());
    Ok(entries)
}

/// Parse an SRT timestamp line into (start_ms, end_ms)
///
/// Expected format: `HH:MM:SS,mmm --> HH:MM:SS,mmm`
/// Example: `00:01:23,456 --> 00:01:25,789`
fn parse_srt_timestamp(line: &str) -> Option<(u64, u64)> {
    let parts: Vec<&str> = line.split("-->").collect();
    if parts.len() != 2 {
        return None;
    }

    let start_ms = parse_timecode(parts[0].trim())?;
    let end_ms = parse_timecode(parts[1].trim())?;

    Some((start_ms, end_ms))
}

/// Parse a single timecode into milliseconds
///
/// Expected format: `HH:MM:SS,mmm` or `HH:MM:SS.mmm`
fn parse_timecode(tc: &str) -> Option<u64> {
    // Replace comma with period for consistent parsing
    let tc = tc.replace(',', ".");

    let main_parts: Vec<&str> = tc.split(':').collect();
    if main_parts.len() != 3 {
        return None;
    }

    let hours: u64 = main_parts[0].trim().parse().ok()?;
    let minutes: u64 = main_parts[1].trim().parse().ok()?;

    let sec_parts: Vec<&str> = main_parts[2].split('.').collect();
    if sec_parts.len() != 2 {
        return None;
    }

    let seconds: u64 = sec_parts[0].trim().parse().ok()?;
    let millis: u64 = sec_parts[1].trim().parse().ok()?;

    Some(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timecode() {
        assert_eq!(parse_timecode("00:00:01,000"), Some(1000));
        assert_eq!(parse_timecode("00:01:00,000"), Some(60000));
        assert_eq!(parse_timecode("01:00:00,000"), Some(3600000));
        assert_eq!(parse_timecode("01:23:45,678"), Some(5025678));
        assert_eq!(parse_timecode("00:00:01.500"), Some(1500));
    }

    #[test]
    fn test_parse_srt_content() {
        let content = "\
1
00:00:01,000 --> 00:00:04,000
Hello, world!

2
00:00:05,000 --> 00:00:08,000
This is a test subtitle.

3
00:00:10,000 --> 00:00:13,500
Multiple lines
are supported too.
";

        let entries = parse_srt_content(content).unwrap();
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].index, 1);
        assert_eq!(entries[0].start_ms, 1000);
        assert_eq!(entries[0].end_ms, 4000);
        assert_eq!(entries[0].text, "Hello, world!");

        assert_eq!(entries[1].index, 2);
        assert_eq!(entries[1].start_ms, 5000);
        assert_eq!(entries[1].end_ms, 8000);
        assert_eq!(entries[1].text, "This is a test subtitle.");

        assert_eq!(entries[2].index, 3);
        assert_eq!(entries[2].start_ms, 10000);
        assert_eq!(entries[2].end_ms, 13500);
        assert_eq!(entries[2].text, "Multiple lines\nare supported too.");
    }

    #[test]
    fn test_parse_srt_malformed() {
        // Empty content
        let entries = parse_srt_content("").unwrap();
        assert!(entries.is_empty());

        // Missing timestamp
        let content = "1\nHello";
        let entries = parse_srt_content(content).unwrap();
        assert!(entries.is_empty());
    }
}
