//! Templates system for pre-built video projects
//!
//! Templates provide pre-configured timelines with placeholder clips
//! that users can quickly fill with their own media for fast video creation.
//! Each template includes effects, transitions, text overlays, and timing.

pub mod builder;

use crate::timeline::Timeline;
use serde::{Deserialize, Serialize};

/// Template category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TemplateCategory {
    Social,      // Instagram, TikTok, YouTube Shorts
    Cinematic,   // Film-like effects and transitions
    Tutorial,    // Step-by-step educational
    Vlog,        // Personal vlog style
    Business,    // Corporate/presentation
    Celebration, // Birthday, holiday, etc.
}

impl TemplateCategory {
    pub fn display_name(&self) -> &str {
        match self {
            TemplateCategory::Social => "Social",
            TemplateCategory::Cinematic => "Cinematic",
            TemplateCategory::Tutorial => "Tutorial",
            TemplateCategory::Vlog => "Vlog",
            TemplateCategory::Business => "Business",
            TemplateCategory::Celebration => "Celebration",
        }
    }

    pub fn all_categories() -> &'static [TemplateCategory] {
        &[
            TemplateCategory::Social,
            TemplateCategory::Cinematic,
            TemplateCategory::Tutorial,
            TemplateCategory::Vlog,
            TemplateCategory::Business,
            TemplateCategory::Celebration,
        ]
    }

    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "social" => Some(TemplateCategory::Social),
            "cinematic" => Some(TemplateCategory::Cinematic),
            "tutorial" => Some(TemplateCategory::Tutorial),
            "vlog" => Some(TemplateCategory::Vlog),
            "business" => Some(TemplateCategory::Business),
            "celebration" => Some(TemplateCategory::Celebration),
            _ => None,
        }
    }
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A placeholder slot in a template where user media goes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderSlot {
    /// Unique ID for this slot
    pub id: String,
    /// Human-readable label (e.g., "Drop your intro video here")
    pub label: String,
    /// Track index in the timeline where this clip belongs
    pub track_index: usize,
    /// Position in the timeline (ms)
    pub start_ms: u64,
    /// Expected duration (ms) — 0 means any duration
    pub expected_duration_ms: u64,
    /// Media type expected
    pub media_type: PlaceholderMediaType,
    /// Whether this slot has been filled
    pub is_filled: bool,
}

/// Type of media expected for a placeholder
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PlaceholderMediaType {
    Video,
    Image,
    Any,
}

impl PlaceholderMediaType {
    pub fn display_name(&self) -> &str {
        match self {
            PlaceholderMediaType::Video => "Video",
            PlaceholderMediaType::Image => "Image",
            PlaceholderMediaType::Any => "Video or Image",
        }
    }
}

/// A pre-built template for quick video creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    /// Preview image path (thumbnail)
    pub preview_path: String,
    /// Placeholder slots that users need to fill
    pub placeholder_slots: Vec<PlaceholderSlot>,
    /// Pre-configured timeline with effects, transitions, text
    pub timeline_template: Timeline,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Aspect ratio as (width, height)
    pub aspect_ratio: (u32, u32),
    /// Tags for search/filtering
    pub tags: Vec<String>,
    /// Number of times this template has been used
    pub use_count: u32,
}

impl Template {
    /// Create a blank template with default timeline
    pub fn new(name: &str, category: TemplateCategory) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            category,
            preview_path: String::new(),
            placeholder_slots: Vec::new(),
            timeline_template: Timeline::new(),
            duration_ms: 0,
            aspect_ratio: (16, 9),
            tags: Vec::new(),
            use_count: 0,
        }
    }

    /// Check if all placeholder slots are filled
    pub fn is_complete(&self) -> bool {
        self.placeholder_slots.iter().all(|s| s.is_filled)
    }

    /// Get unfilled slots
    pub fn unfilled_slots(&self) -> Vec<&PlaceholderSlot> {
        self.placeholder_slots
            .iter()
            .filter(|s| !s.is_filled)
            .collect()
    }

    /// Create a template with a specific timeline and settings
    pub fn with_timeline(
        name: &str,
        category: TemplateCategory,
        timeline: Timeline,
        duration_ms: u64,
        aspect_ratio: (u32, u32),
        description: &str,
        tags: Vec<&str>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category,
            preview_path: String::new(),
            placeholder_slots: Vec::new(),
            timeline_template: timeline,
            duration_ms,
            aspect_ratio,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            use_count: 0,
        }
    }
}

/// Built-in template presets
pub fn built_in_templates() -> Vec<Template> {
    vec![
        // 1. Social Intro — 9:16, 15s, 3 video slots + 1 text
        {
            let mut timeline = Timeline::with_settings(crate::timeline::TimelineSettings {
                width: 1080,
                height: 1920,
                fps: 30.0,
                sample_rate: 44100,
                background_color: "#000000".to_string(),
            });
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Video".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Title".to_string()),
            );

            Template {
                id: "tmpl-social-intro".to_string(),
                name: "Social Intro".to_string(),
                description:
                    "Eye-catching vertical intro for Instagram Reels and TikTok with animated title and transitions"
                        .to_string(),
                category: TemplateCategory::Social,
                preview_path: "assets/templates/social_intro.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-social-intro-video-1".to_string(),
                        label: "Opening hook clip".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-social-intro-video-2".to_string(),
                        label: "Main content clip".to_string(),
                        track_index: 0,
                        start_ms: 5000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-social-intro-video-3".to_string(),
                        label: "Closing clip".to_string(),
                        track_index: 0,
                        start_ms: 10000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-social-intro-text-1".to_string(),
                        label: "Title text overlay".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 4000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 15000,
                aspect_ratio: (9, 16),
                tags: vec![
                    "social".to_string(),
                    "tiktok".to_string(),
                    "instagram".to_string(),
                    "intro".to_string(),
                    "vertical".to_string(),
                ],
                use_count: 0,
            }
        },
        // 2. Cinematic Widescreen — 16:9, 30s, 4 video slots + 2 text
        {
            let mut timeline = Timeline::with_settings(crate::timeline::TimelineSettings {
                width: 1920,
                height: 1080,
                fps: 30.0,
                ..Default::default()
            });
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Main Video".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Title Cards".to_string()),
            );

            Template {
                id: "tmpl-cinematic-widescreen".to_string(),
                name: "Cinematic Widescreen".to_string(),
                description:
                    "Dramatic widescreen opener with cinematic transitions, title cards, and slow reveals"
                        .to_string(),
                category: TemplateCategory::Cinematic,
                preview_path: "assets/templates/cinematic_widescreen.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-cinematic-video-1".to_string(),
                        label: "Opening wide shot".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 8000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-cinematic-video-2".to_string(),
                        label: "Reveal shot".to_string(),
                        track_index: 0,
                        start_ms: 8000,
                        expected_duration_ms: 7000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-cinematic-video-3".to_string(),
                        label: "Action sequence".to_string(),
                        track_index: 0,
                        start_ms: 15000,
                        expected_duration_ms: 8000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-cinematic-video-4".to_string(),
                        label: "Final establishing shot".to_string(),
                        track_index: 0,
                        start_ms: 23000,
                        expected_duration_ms: 7000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-cinematic-text-1".to_string(),
                        label: "Opening title card".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-cinematic-text-2".to_string(),
                        label: "Closing title card".to_string(),
                        track_index: 1,
                        start_ms: 25000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 30000,
                aspect_ratio: (16, 9),
                tags: vec![
                    "cinematic".to_string(),
                    "widescreen".to_string(),
                    "film".to_string(),
                    "dramatic".to_string(),
                    "opener".to_string(),
                ],
                use_count: 0,
            }
        },
        // 3. Tutorial Steps — 16:9, 60s, 5 video slots + 5 text
        {
            let mut timeline =
                Timeline::with_settings(crate::timeline::TimelineSettings::default());
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Screen Recording".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Step Titles".to_string()),
            );

            let mut slots = Vec::new();
            for i in 0..5 {
                let step_start = i as u64 * 12000;
                slots.push(PlaceholderSlot {
                    id: format!("slot-tutorial-video-{}", i + 1),
                    label: format!("Step {} recording", i + 1),
                    track_index: 0,
                    start_ms: step_start,
                    expected_duration_ms: 12000,
                    media_type: PlaceholderMediaType::Video,
                    is_filled: false,
                });
                slots.push(PlaceholderSlot {
                    id: format!("slot-tutorial-text-{}", i + 1),
                    label: format!("Step {} title", i + 1),
                    track_index: 1,
                    start_ms: step_start,
                    expected_duration_ms: 4000,
                    media_type: PlaceholderMediaType::Image,
                    is_filled: false,
                });
            }

            Template {
                id: "tmpl-tutorial-steps".to_string(),
                name: "Tutorial Steps".to_string(),
                description: "Step-by-step tutorial with 5 numbered sections, each with screen recording and title overlay".to_string(),
                category: TemplateCategory::Tutorial,
                preview_path: "assets/templates/tutorial_steps.png".to_string(),
                placeholder_slots: slots,
                timeline_template: timeline,
                duration_ms: 60000,
                aspect_ratio: (16, 9),
                tags: vec!["tutorial".to_string(), "steps".to_string(), "education".to_string(), "how-to".to_string(), "instructional".to_string()],
                use_count: 0,
            }
        },
        // 4. Vlog Highlight — 9:16, 20s, 4 video slots
        {
            let mut timeline = Timeline::with_settings(crate::timeline::TimelineSettings {
                width: 1080,
                height: 1920,
                fps: 30.0,
                sample_rate: 44100,
                background_color: "#1A1A2E".to_string(),
            });
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Highlight Clips".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Captions".to_string()),
            );

            Template {
                id: "tmpl-vlog-highlight".to_string(),
                name: "Vlog Highlight".to_string(),
                description: "Quick vertical vlog highlight reel with snappy cuts and energetic pacing".to_string(),
                category: TemplateCategory::Vlog,
                preview_path: "assets/templates/vlog_highlight.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-vlog-video-1".to_string(),
                        label: "Highlight clip 1".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-vlog-video-2".to_string(),
                        label: "Highlight clip 2".to_string(),
                        track_index: 0,
                        start_ms: 5000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-vlog-video-3".to_string(),
                        label: "Highlight clip 3".to_string(),
                        track_index: 0,
                        start_ms: 10000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-vlog-video-4".to_string(),
                        label: "Highlight clip 4".to_string(),
                        track_index: 0,
                        start_ms: 15000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 20000,
                aspect_ratio: (9, 16),
                tags: vec![
                    "vlog".to_string(),
                    "highlight".to_string(),
                    "vertical".to_string(),
                    "tiktok".to_string(),
                    "instagram".to_string(),
                ],
                use_count: 0,
            }
        },
        // 5. Business Presentation — 16:9, 45s, 3 video + 4 text
        {
            let mut timeline =
                Timeline::with_settings(crate::timeline::TimelineSettings::default());
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Presentation".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Lower Thirds".to_string()),
            );

            Template {
                id: "tmpl-business-presentation".to_string(),
                name: "Business Presentation".to_string(),
                description: "Professional corporate presentation with video sections and lower-third text callouts".to_string(),
                category: TemplateCategory::Business,
                preview_path: "assets/templates/business_presentation.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-biz-video-1".to_string(),
                        label: "Introduction clip".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 15000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-biz-video-2".to_string(),
                        label: "Main content clip".to_string(),
                        track_index: 0,
                        start_ms: 15000,
                        expected_duration_ms: 15000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-biz-video-3".to_string(),
                        label: "Conclusion clip".to_string(),
                        track_index: 0,
                        start_ms: 30000,
                        expected_duration_ms: 15000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-biz-text-1".to_string(),
                        label: "Speaker name title".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-biz-text-2".to_string(),
                        label: "Key point 1 callout".to_string(),
                        track_index: 1,
                        start_ms: 15000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-biz-text-3".to_string(),
                        label: "Key point 2 callout".to_string(),
                        track_index: 1,
                        start_ms: 22000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-biz-text-4".to_string(),
                        label: "Closing title card".to_string(),
                        track_index: 1,
                        start_ms: 38000,
                        expected_duration_ms: 7000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 45000,
                aspect_ratio: (16, 9),
                tags: vec![
                    "business".to_string(),
                    "presentation".to_string(),
                    "corporate".to_string(),
                    "professional".to_string(),
                    "pitch".to_string(),
                ],
                use_count: 0,
            }
        },
        // 6. Celebration Card — 1:1, 10s, 2 video + 2 text
        {
            let mut timeline = Timeline::with_settings(crate::timeline::TimelineSettings {
                width: 1080,
                height: 1080,
                fps: 30.0,
                sample_rate: 44100,
                background_color: "#2D1B69".to_string(),
            });
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Celebration Clips".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Greeting Text".to_string()),
            );

            Template {
                id: "tmpl-celebration-card".to_string(),
                name: "Celebration Card".to_string(),
                description: "Festive square video card with photo/video slots, confetti effects, and greeting text overlays".to_string(),
                category: TemplateCategory::Celebration,
                preview_path: "assets/templates/celebration_card.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-celebration-video-1".to_string(),
                        label: "Celebration moment 1".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-celebration-video-2".to_string(),
                        label: "Celebration moment 2".to_string(),
                        track_index: 0,
                        start_ms: 5000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-celebration-text-1".to_string(),
                        label: "Greeting headline".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-celebration-text-2".to_string(),
                        label: "Personal message".to_string(),
                        track_index: 1,
                        start_ms: 5000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 10000,
                aspect_ratio: (1, 1),
                tags: vec![
                    "celebration".to_string(),
                    "card".to_string(),
                    "birthday".to_string(),
                    "instagram".to_string(),
                    "square".to_string(),
                ],
                use_count: 0,
            }
        },
        // 7. Instagram Reel — 9:16, 15s, 3 video + 1 text
        {
            let mut timeline = Timeline::with_settings(crate::timeline::TimelineSettings {
                width: 1080,
                height: 1920,
                fps: 30.0,
                sample_rate: 44100,
                background_color: "#000000".to_string(),
            });
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Reel Clips".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Caption".to_string()),
            );

            Template {
                id: "tmpl-instagram-reel".to_string(),
                name: "Instagram Reel".to_string(),
                description: "Instagram Reel template with trendy cuts, text caption overlay, and music-sync transitions".to_string(),
                category: TemplateCategory::Social,
                preview_path: "assets/templates/instagram_reel.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-reel-video-1".to_string(),
                        label: "Hook clip".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-reel-video-2".to_string(),
                        label: "Main content clip".to_string(),
                        track_index: 0,
                        start_ms: 5000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-reel-video-3".to_string(),
                        label: "Outro clip".to_string(),
                        track_index: 0,
                        start_ms: 10000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-reel-text-1".to_string(),
                        label: "Caption overlay".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 6000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 15000,
                aspect_ratio: (9, 16),
                tags: vec![
                    "instagram".to_string(),
                    "reel".to_string(),
                    "social".to_string(),
                    "vertical".to_string(),
                    "trending".to_string(),
                ],
                use_count: 0,
            }
        },
        // 8. Product Showcase — 1:1, 20s, 3 video + 3 text
        {
            let mut timeline = Timeline::with_settings(crate::timeline::TimelineSettings {
                width: 1080,
                height: 1080,
                fps: 30.0,
                sample_rate: 44100,
                background_color: "#F5F5F5".to_string(),
            });
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Product Shots".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Feature Labels".to_string()),
            );

            Template {
                id: "tmpl-product-showcase".to_string(),
                name: "Product Showcase".to_string(),
                description:
                    "Clean product showcase with feature highlight labels and professional transitions"
                        .to_string(),
                category: TemplateCategory::Business,
                preview_path: "assets/templates/product_showcase.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-product-video-1".to_string(),
                        label: "Hero product shot".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 7000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-product-video-2".to_string(),
                        label: "Product detail closeup".to_string(),
                        track_index: 0,
                        start_ms: 7000,
                        expected_duration_ms: 7000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-product-video-3".to_string(),
                        label: "Lifestyle / in-use shot".to_string(),
                        track_index: 0,
                        start_ms: 14000,
                        expected_duration_ms: 6000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-product-text-1".to_string(),
                        label: "Product name label".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-product-text-2".to_string(),
                        label: "Feature highlight 1".to_string(),
                        track_index: 1,
                        start_ms: 7000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-product-text-3".to_string(),
                        label: "Price / CTA label".to_string(),
                        track_index: 1,
                        start_ms: 14000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 20000,
                aspect_ratio: (1, 1),
                tags: vec![
                    "product".to_string(),
                    "showcase".to_string(),
                    "ecommerce".to_string(),
                    "square".to_string(),
                    "business".to_string(),
                ],
                use_count: 0,
            }
        },
        // 9. Travel Montage — 16:9, 30s, 5 video + 1 text
        {
            let mut timeline =
                Timeline::with_settings(crate::timeline::TimelineSettings::default());
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Travel Clips".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Location Title".to_string()),
            );

            let mut slots = Vec::new();
            for i in 0..5 {
                slots.push(PlaceholderSlot {
                    id: format!("slot-travel-video-{}", i + 1),
                    label: format!("Travel clip {}", i + 1),
                    track_index: 0,
                    start_ms: i as u64 * 6000,
                    expected_duration_ms: 6000,
                    media_type: PlaceholderMediaType::Video,
                    is_filled: false,
                });
            }
            slots.push(PlaceholderSlot {
                id: "slot-travel-text-1".to_string(),
                label: "Destination title card".to_string(),
                track_index: 1,
                start_ms: 0,
                expected_duration_ms: 6000,
                media_type: PlaceholderMediaType::Image,
                is_filled: false,
            });

            Template {
                id: "tmpl-travel-montage".to_string(),
                name: "Travel Montage".to_string(),
                description:
                    "Fast-paced travel montage with speed-ramp transitions and location title card"
                        .to_string(),
                category: TemplateCategory::Cinematic,
                preview_path: "assets/templates/travel_montage.png".to_string(),
                placeholder_slots: slots,
                timeline_template: timeline,
                duration_ms: 30000,
                aspect_ratio: (16, 9),
                tags: vec![
                    "travel".to_string(),
                    "montage".to_string(),
                    "speed-ramp".to_string(),
                    "adventure".to_string(),
                    "cinematic".to_string(),
                ],
                use_count: 0,
            }
        },
        // 10. Quick Tutorial — 16:9, 30s, 3 video + 3 text
        {
            let mut timeline =
                Timeline::with_settings(crate::timeline::TimelineSettings::default());
            timeline.add_track(
                crate::timeline::track::TrackType::Video,
                Some("Demo Recording".to_string()),
            );
            timeline.add_track(
                crate::timeline::track::TrackType::Text,
                Some("Step Labels".to_string()),
            );

            Template {
                id: "tmpl-quick-tutorial".to_string(),
                name: "Quick Tutorial".to_string(),
                description: "Short-form tutorial with 3 demo clips and step-by-step text labels, perfect for quick how-tos".to_string(),
                category: TemplateCategory::Tutorial,
                preview_path: "assets/templates/quick_tutorial.png".to_string(),
                placeholder_slots: vec![
                    PlaceholderSlot {
                        id: "slot-qt-video-1".to_string(),
                        label: "Step 1 demo clip".to_string(),
                        track_index: 0,
                        start_ms: 0,
                        expected_duration_ms: 10000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-qt-video-2".to_string(),
                        label: "Step 2 demo clip".to_string(),
                        track_index: 0,
                        start_ms: 10000,
                        expected_duration_ms: 10000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-qt-video-3".to_string(),
                        label: "Step 3 demo clip".to_string(),
                        track_index: 0,
                        start_ms: 20000,
                        expected_duration_ms: 10000,
                        media_type: PlaceholderMediaType::Video,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-qt-text-1".to_string(),
                        label: "Step 1 label".to_string(),
                        track_index: 1,
                        start_ms: 0,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-qt-text-2".to_string(),
                        label: "Step 2 label".to_string(),
                        track_index: 1,
                        start_ms: 10000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                    PlaceholderSlot {
                        id: "slot-qt-text-3".to_string(),
                        label: "Step 3 label".to_string(),
                        track_index: 1,
                        start_ms: 20000,
                        expected_duration_ms: 5000,
                        media_type: PlaceholderMediaType::Image,
                        is_filled: false,
                    },
                ],
                timeline_template: timeline,
                duration_ms: 30000,
                aspect_ratio: (16, 9),
                tags: vec![
                    "tutorial".to_string(),
                    "quick".to_string(),
                    "how-to".to_string(),
                    "education".to_string(),
                    "demo".to_string(),
                ],
                use_count: 0,
            }
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_category_display_name() {
        assert_eq!(TemplateCategory::Social.display_name(), "Social");
        assert_eq!(TemplateCategory::Cinematic.display_name(), "Cinematic");
        assert_eq!(TemplateCategory::Tutorial.display_name(), "Tutorial");
        assert_eq!(TemplateCategory::Vlog.display_name(), "Vlog");
        assert_eq!(TemplateCategory::Business.display_name(), "Business");
        assert_eq!(TemplateCategory::Celebration.display_name(), "Celebration");
    }

    #[test]
    fn test_template_category_from_str_lossy() {
        assert_eq!(
            TemplateCategory::from_str_lossy("social"),
            Some(TemplateCategory::Social)
        );
        assert_eq!(
            TemplateCategory::from_str_lossy("Cinematic"),
            Some(TemplateCategory::Cinematic)
        );
        assert_eq!(
            TemplateCategory::from_str_lossy("TUTORIAL"),
            Some(TemplateCategory::Tutorial)
        );
        assert_eq!(TemplateCategory::from_str_lossy("unknown"), None);
    }

    #[test]
    fn test_template_category_all_categories() {
        let cats = TemplateCategory::all_categories();
        assert_eq!(cats.len(), 6);
    }

    #[test]
    fn test_placeholder_media_type_display_name() {
        assert_eq!(PlaceholderMediaType::Video.display_name(), "Video");
        assert_eq!(PlaceholderMediaType::Image.display_name(), "Image");
        assert_eq!(PlaceholderMediaType::Any.display_name(), "Video or Image");
    }

    #[test]
    fn test_template_new() {
        let tmpl = Template::new("Test", TemplateCategory::Social);
        assert_eq!(tmpl.name, "Test");
        assert_eq!(tmpl.category, TemplateCategory::Social);
        assert!(tmpl.placeholder_slots.is_empty());
        assert!(!tmpl.is_complete()); // No slots = not complete (empty check)
        assert_eq!(tmpl.unfilled_slots().len(), 0);
    }

    #[test]
    fn test_template_is_complete() {
        let mut tmpl = Template::new("Test", TemplateCategory::Social);
        tmpl.placeholder_slots.push(PlaceholderSlot {
            id: "slot-1".to_string(),
            label: "Test".to_string(),
            track_index: 0,
            start_ms: 0,
            expected_duration_ms: 5000,
            media_type: PlaceholderMediaType::Video,
            is_filled: false,
        });
        assert!(!tmpl.is_complete());
        assert_eq!(tmpl.unfilled_slots().len(), 1);

        tmpl.placeholder_slots[0].is_filled = true;
        assert!(tmpl.is_complete());
        assert_eq!(tmpl.unfilled_slots().len(), 0);
    }

    #[test]
    fn test_built_in_templates_count() {
        let templates = built_in_templates();
        assert_eq!(templates.len(), 10);
    }

    #[test]
    fn test_built_in_templates_categories() {
        let templates = built_in_templates();
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Social));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Cinematic));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Tutorial));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Vlog));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Business));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Celebration));
    }

    #[test]
    fn test_built_in_templates_have_unique_ids() {
        let templates = built_in_templates();
        let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        let unique_ids: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_built_in_templates_have_slots() {
        let templates = built_in_templates();
        for tmpl in &templates {
            assert!(
                !tmpl.placeholder_slots.is_empty(),
                "Template '{}' has no slots",
                tmpl.name
            );
        }
    }

    #[test]
    fn test_built_in_templates_aspect_ratios() {
        let templates = built_in_templates();
        // Social Intro should be 9:16
        let social_intro = templates
            .iter()
            .find(|t| t.id == "tmpl-social-intro")
            .unwrap();
        assert_eq!(social_intro.aspect_ratio, (9, 16));

        // Cinematic Widescreen should be 16:9
        let cinematic = templates
            .iter()
            .find(|t| t.id == "tmpl-cinematic-widescreen")
            .unwrap();
        assert_eq!(cinematic.aspect_ratio, (16, 9));

        // Celebration Card should be 1:1
        let celebration = templates
            .iter()
            .find(|t| t.id == "tmpl-celebration-card")
            .unwrap();
        assert_eq!(celebration.aspect_ratio, (1, 1));
    }

    #[test]
    fn test_built_in_templates_durations() {
        let templates = built_in_templates();

        let social_intro = templates.iter().find(|t| t.id == "tmpl-social-intro").unwrap();
        assert_eq!(social_intro.duration_ms, 15000);

        let cinematic = templates.iter().find(|t| t.id == "tmpl-cinematic-widescreen").unwrap();
        assert_eq!(cinematic.duration_ms, 30000);

        let tutorial = templates.iter().find(|t| t.id == "tmpl-tutorial-steps").unwrap();
        assert_eq!(tutorial.duration_ms, 60000);

        let vlog = templates.iter().find(|t| t.id == "tmpl-vlog-highlight").unwrap();
        assert_eq!(vlog.duration_ms, 20000);

        let biz = templates.iter().find(|t| t.id == "tmpl-business-presentation").unwrap();
        assert_eq!(biz.duration_ms, 45000);

        let celebration = templates.iter().find(|t| t.id == "tmpl-celebration-card").unwrap();
        assert_eq!(celebration.duration_ms, 10000);

        let reel = templates.iter().find(|t| t.id == "tmpl-instagram-reel").unwrap();
        assert_eq!(reel.duration_ms, 15000);

        let product = templates.iter().find(|t| t.id == "tmpl-product-showcase").unwrap();
        assert_eq!(product.duration_ms, 20000);

        let travel = templates.iter().find(|t| t.id == "tmpl-travel-montage").unwrap();
        assert_eq!(travel.duration_ms, 30000);

        let quick = templates.iter().find(|t| t.id == "tmpl-quick-tutorial").unwrap();
        assert_eq!(quick.duration_ms, 30000);
    }

    #[test]
    fn test_built_in_templates_slot_counts() {
        let templates = built_in_templates();

        // Social Intro: 3 video + 1 text = 4
        let social_intro = templates.iter().find(|t| t.id == "tmpl-social-intro").unwrap();
        assert_eq!(social_intro.placeholder_slots.len(), 4);

        // Cinematic Widescreen: 4 video + 2 text = 6
        let cinematic = templates.iter().find(|t| t.id == "tmpl-cinematic-widescreen").unwrap();
        assert_eq!(cinematic.placeholder_slots.len(), 6);

        // Tutorial Steps: 5 video + 5 text = 10
        let tutorial = templates.iter().find(|t| t.id == "tmpl-tutorial-steps").unwrap();
        assert_eq!(tutorial.placeholder_slots.len(), 10);

        // Vlog Highlight: 4 video = 4
        let vlog = templates.iter().find(|t| t.id == "tmpl-vlog-highlight").unwrap();
        assert_eq!(vlog.placeholder_slots.len(), 4);

        // Business Presentation: 3 video + 4 text = 7
        let biz = templates.iter().find(|t| t.id == "tmpl-business-presentation").unwrap();
        assert_eq!(biz.placeholder_slots.len(), 7);

        // Celebration Card: 2 video + 2 text = 4
        let celebration = templates.iter().find(|t| t.id == "tmpl-celebration-card").unwrap();
        assert_eq!(celebration.placeholder_slots.len(), 4);

        // Instagram Reel: 3 video + 1 text = 4
        let reel = templates.iter().find(|t| t.id == "tmpl-instagram-reel").unwrap();
        assert_eq!(reel.placeholder_slots.len(), 4);

        // Product Showcase: 3 video + 3 text = 6
        let product = templates.iter().find(|t| t.id == "tmpl-product-showcase").unwrap();
        assert_eq!(product.placeholder_slots.len(), 6);

        // Travel Montage: 5 video + 1 text = 6
        let travel = templates.iter().find(|t| t.id == "tmpl-travel-montage").unwrap();
        assert_eq!(travel.placeholder_slots.len(), 6);

        // Quick Tutorial: 3 video + 3 text = 6
        let quick = templates.iter().find(|t| t.id == "tmpl-quick-tutorial").unwrap();
        assert_eq!(quick.placeholder_slots.len(), 6);
    }

    #[test]
    fn test_built_in_templates_have_tags() {
        let templates = built_in_templates();
        for tmpl in &templates {
            assert!(!tmpl.tags.is_empty(), "Template '{}' has no tags", tmpl.name);
        }
    }

    #[test]
    fn test_template_serialization() {
        let tmpl = Template::new("Test Serialize", TemplateCategory::Vlog);
        let json = serde_json::to_string(&tmpl).expect("Failed to serialize");
        let deserialized: Template = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.name, "Test Serialize");
        assert_eq!(deserialized.category, TemplateCategory::Vlog);
    }
}
