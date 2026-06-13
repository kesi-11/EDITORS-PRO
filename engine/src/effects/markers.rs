//! Timeline Markers — Color-coded markers with navigation, chapter export.
//!
//! Supports 8 marker colors, 7 marker types, navigation between markers,
//! and chapter/YouTube chapter export.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Marker color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Gray,
}

impl MarkerColor {
    pub fn all() -> &'static [MarkerColor] {
        &[Self::Red, Self::Orange, Self::Yellow, Self::Green, Self::Blue, Self::Purple, Self::Pink, Self::Gray]
    }
    pub fn hex(&self) -> &'static str {
        match self {
            Self::Red => "#FF4444", Self::Orange => "#FF8800", Self::Yellow => "#FFCC00",
            Self::Green => "#44CC44", Self::Blue => "#4488FF", Self::Purple => "#8844FF",
            Self::Pink => "#FF44AA", Self::Gray => "#888888",
        }
    }
}

/// Marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    Standard,
    Chapter,
    Comment,
    Todo,
    Error,
    MusicBeat,
    Custom,
}

/// A timeline marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    pub id: String,
    pub name: String,
    pub position_ms: f64,
    pub duration_ms: f64,
    pub color: MarkerColor,
    pub marker_type: MarkerType,
    pub comment: String,
    pub metadata: HashMap<String, String>,
}

impl Marker {
    pub fn new(name: &str, position_ms: f64, color: MarkerColor) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            position_ms,
            duration_ms: 0.0,
            color,
            marker_type: MarkerType::Standard,
            comment: String::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn chapter(name: &str, position_ms: f64) -> Self {
        Self { marker_type: MarkerType::Chapter, ..Self::new(name, position_ms, MarkerColor::Orange) }
    }
}

/// Marker manager for a timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerManager {
    pub markers: Vec<Marker>,
}

impl MarkerManager {
    pub fn new() -> Self { Self { markers: Vec::new() } }

    pub fn add(&mut self, marker: Marker) { self.markers.push(marker); self.markers.sort_by(|a, b| a.position_ms.partial_cmp(&b.position_ms).unwrap()); }

    pub fn remove(&mut self, id: &str) -> Option<Marker> {
        if let Some(pos) = self.markers.iter().position(|m| m.id == id) { Some(self.markers.remove(pos)) } else { None }
    }

    pub fn next_after(&self, time_ms: f64) -> Option<&Marker> {
        self.markers.iter().find(|m| m.position_ms > time_ms)
    }

    pub fn prev_before(&self, time_ms: f64) -> Option<&Marker> {
        self.markers.iter().rev().find(|m| m.position_ms < time_ms)
    }

    pub fn at_time(&self, time_ms: f64) -> Option<&Marker> {
        self.markers.iter().find(|m| time_ms >= m.position_ms && time_ms <= m.position_ms + m.duration_ms.max(1.0))
    }

    pub fn chapters(&self) -> Vec<&Marker> {
        let mut ch: Vec<_> = self.markers.iter().filter(|m| m.marker_type == MarkerType::Chapter).collect();
        ch.sort_by(|a, b| a.position_ms.partial_cmp(&b.position_ms).unwrap());
        ch
    }

    /// Export as YouTube chapter format: "0:00 Chapter Name"
    pub fn export_youtube_chapters(&self) -> String {
        self.chapters().iter().map(|m| {
            let total_secs = (m.position_ms / 1000.0) as u64;
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            let hours = mins / 60;
            if hours > 0 { format!("{}:{:02}:{:02} {}", hours, mins % 60, secs, m.name) }
            else { format!("{}:{:02} {}", mins, secs, m.name) }
        }).collect::<Vec<_>>().join("\n")
    }

    pub fn by_color(&self, color: MarkerColor) -> Vec<&Marker> {
        self.markers.iter().filter(|m| m.color == color).collect()
    }

    pub fn by_type(&self, marker_type: MarkerType) -> Vec<&Marker> {
        self.markers.iter().filter(|m| m.marker_type == marker_type).collect()
    }

    pub fn count(&self) -> usize { self.markers.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_new() {
        let m = Marker::new("Test", 5000.0, MarkerColor::Red);
        assert_eq!(m.name, "Test");
        assert_eq!(m.position_ms, 5000.0);
        assert_eq!(m.color, MarkerColor::Red);
    }

    #[test]
    fn test_chapter_marker() {
        let m = Marker::chapter("Intro", 0.0);
        assert_eq!(m.marker_type, MarkerType::Chapter);
        assert_eq!(m.color, MarkerColor::Orange);
    }

    #[test]
    fn test_marker_manager_add() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::new("A", 1000.0, MarkerColor::Blue));
        mgr.add(Marker::new("B", 3000.0, MarkerColor::Green));
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn test_marker_manager_sorted() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::new("B", 3000.0, MarkerColor::Blue));
        mgr.add(Marker::new("A", 1000.0, MarkerColor::Red));
        assert_eq!(mgr.markers[0].name, "A");
    }

    #[test]
    fn test_marker_manager_remove() {
        let mut mgr = MarkerManager::new();
        let m = Marker::new("Test", 1000.0, MarkerColor::Red);
        let id = m.id.clone();
        mgr.add(m);
        assert!(mgr.remove(&id).is_some());
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_next_after() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::new("A", 1000.0, MarkerColor::Red));
        mgr.add(Marker::new("B", 3000.0, MarkerColor::Blue));
        let next = mgr.next_after(1500.0);
        assert!(next.is_some());
        assert_eq!(next.unwrap().name, "B");
    }

    #[test]
    fn test_prev_before() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::new("A", 1000.0, MarkerColor::Red));
        mgr.add(Marker::new("B", 3000.0, MarkerColor::Blue));
        let prev = mgr.prev_before(2500.0);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().name, "A");
    }

    #[test]
    fn test_chapters() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::chapter("Intro", 0.0));
        mgr.add(Marker::new("Note", 1000.0, MarkerColor::Red));
        mgr.add(Marker::chapter("Main", 5000.0));
        assert_eq!(mgr.chapters().len(), 2);
    }

    #[test]
    fn test_export_youtube() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::chapter("Intro", 0.0));
        mgr.add(Marker::chapter("Main", 120000.0));
        let output = mgr.export_youtube_chapters();
        assert!(output.contains("0:00 Intro"));
        assert!(output.contains("2:00 Main"));
    }

    #[test]
    fn test_by_color() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::new("Red1", 1000.0, MarkerColor::Red));
        mgr.add(Marker::new("Blue1", 2000.0, MarkerColor::Blue));
        mgr.add(Marker::new("Red2", 3000.0, MarkerColor::Red));
        assert_eq!(mgr.by_color(MarkerColor::Red).len(), 2);
    }

    #[test]
    fn test_by_type() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::chapter("Ch1", 0.0));
        mgr.add(Marker::new("Note", 1000.0, MarkerColor::Red));
        assert_eq!(mgr.by_type(MarkerType::Chapter).len(), 1);
    }

    #[test]
    fn test_marker_colors_count() { assert_eq!(MarkerColor::all().len(), 8); }

    #[test]
    fn test_marker_color_hex() { assert_eq!(MarkerColor::Red.hex(), "#FF4444"); }

    #[test]
    fn test_at_time() {
        let mut mgr = MarkerManager::new();
        let mut m = Marker::new("Range", 1000.0, MarkerColor::Green);
        m.duration_ms = 500.0;
        mgr.add(m);
        assert!(mgr.at_time(1200.0).is_some());
        assert!(mgr.at_time(2000.0).is_none());
    }

    #[test]
    fn test_youtube_hours() {
        let mut mgr = MarkerManager::new();
        mgr.add(Marker::chapter("Long", 7200000.0)); // 2 hours
        let output = mgr.export_youtube_chapters();
        assert!(output.contains("2:00:00"));
    }

    #[test]
    fn test_empty_manager() {
        let mgr = MarkerManager::new();
        assert_eq!(mgr.count(), 0);
        assert!(mgr.next_after(0.0).is_none());
        assert!(mgr.chapters().is_empty());
    }
}
