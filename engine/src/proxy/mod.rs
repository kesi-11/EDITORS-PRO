//! Proxy workflow for smooth editing of high-resolution media
//!
//! Generates lower-resolution copies ("proxies") of imported media for
//! smooth timeline editing. When exporting, the original full-resolution
//! media is used instead of proxies for maximum quality.
//!
//! ## Architecture
//!
//! 1. On media import, check if resolution > threshold (e.g., 1080p)
//! 2. If yes, generate a proxy at the configured quality (480p/720p)
//! 3. Store proxy in app cache directory alongside metadata
//! 4. During preview/editing, use proxy for frame decoding
//! 5. During export, use original media for full quality

pub mod generator;

use serde::{Deserialize, Serialize};

/// Proxy quality settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProxyQuality {
    Off,  // No proxy generation
    P360, // 360p
    P480, // 480p
    P720, // 720p
}

impl ProxyQuality {
    /// Get the target width for this proxy quality level.
    pub fn target_width(&self) -> u32 {
        match self {
            ProxyQuality::Off => 0,
            ProxyQuality::P360 => 640,
            ProxyQuality::P480 => 854,
            ProxyQuality::P720 => 1280,
        }
    }

    /// Get the target height for this proxy quality level.
    pub fn target_height(&self) -> u32 {
        match self {
            ProxyQuality::Off => 0,
            ProxyQuality::P360 => 360,
            ProxyQuality::P480 => 480,
            ProxyQuality::P720 => 720,
        }
    }

    /// Get the display name for this proxy quality level.
    pub fn display_name(&self) -> &str {
        match self {
            ProxyQuality::Off => "Off",
            ProxyQuality::P360 => "360p",
            ProxyQuality::P480 => "480p",
            ProxyQuality::P720 => "720p",
        }
    }

    /// Parse a proxy quality from a string (case-insensitive).
    ///
    /// Returns `None` if the string doesn't match any known quality level.
    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "off" => Some(ProxyQuality::Off),
            "360p" | "360" => Some(ProxyQuality::P360),
            "480p" | "480" => Some(ProxyQuality::P480),
            "720p" | "720" => Some(ProxyQuality::P720),
            _ => None,
        }
    }

    /// Get all quality levels (excluding Off).
    pub fn all_qualities() -> Vec<ProxyQuality> {
        vec![ProxyQuality::P360, ProxyQuality::P480, ProxyQuality::P720]
    }
}

/// Proxy metadata stored alongside the proxy file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMetadata {
    /// ID of the original asset this proxy is for
    pub original_asset_id: String,
    /// Path to the original (full-resolution) media file
    pub original_path: String,
    /// Path to the generated proxy file
    pub proxy_path: String,
    /// Quality level of this proxy
    pub quality: ProxyQuality,
    /// Width of the original media in pixels
    pub original_width: u32,
    /// Height of the original media in pixels
    pub original_height: u32,
    /// Width of the proxy in pixels
    pub proxy_width: u32,
    /// Height of the proxy in pixels
    pub proxy_height: u32,
    /// Unix timestamp when the proxy was generated
    pub generated_at: i64,
    /// Size of the proxy file in bytes
    pub file_size_bytes: u64,
}

/// Proxy manager that tracks proxy status for all assets.
///
/// The manager maintains an in-memory index of all generated proxies
/// and provides methods to query proxy paths, generate new proxies,
/// and manage the proxy cache.
pub struct ProxyManager {
    /// Active proxies keyed by asset ID
    proxies: std::collections::HashMap<String, ProxyMetadata>,
    /// Configured proxy quality
    quality: ProxyQuality,
    /// Resolution threshold for proxy generation (generate proxy if source exceeds this)
    threshold_width: u32,
    /// Resolution threshold for proxy generation
    threshold_height: u32,
}

impl ProxyManager {
    /// Create a new proxy manager with the given quality setting.
    ///
    /// The default threshold is 1080p (1920x1080) — media larger than
    /// this will have proxies generated.
    pub fn new(quality: ProxyQuality) -> Self {
        Self {
            proxies: std::collections::HashMap::new(),
            quality,
            threshold_width: 1920,
            threshold_height: 1080,
        }
    }

    /// Check whether a proxy should be generated for the given resolution.
    ///
    /// Returns `true` if:
    /// - Proxy quality is not Off
    /// - The source resolution exceeds the threshold
    pub fn should_generate_proxy(&self, width: u32, height: u32) -> bool {
        if self.quality == ProxyQuality::Off {
            return false;
        }
        width > self.threshold_width || height > self.threshold_height
    }

    /// Get the proxy path for an asset, if a proxy exists.
    pub fn get_proxy_path(&self, asset_id: &str) -> Option<&str> {
        self.proxies.get(asset_id).map(|m| m.proxy_path.as_str())
    }

    /// Get the proxy metadata for an asset, if a proxy exists.
    pub fn get_proxy_metadata(&self, asset_id: &str) -> Option<&ProxyMetadata> {
        self.proxies.get(asset_id)
    }

    /// Register a new proxy with the manager.
    ///
    /// If a proxy already exists for this asset, it will be replaced.
    pub fn register_proxy(&mut self, metadata: ProxyMetadata) {
        self.proxies
            .insert(metadata.original_asset_id.clone(), metadata);
    }

    /// Remove a proxy from the manager by asset ID.
    ///
    /// Returns the removed metadata, if any.
    pub fn remove_proxy(&mut self, asset_id: &str) -> Option<ProxyMetadata> {
        self.proxies.remove(asset_id)
    }

    /// Set the proxy quality setting.
    ///
    /// Changing this does not regenerate existing proxies — call
    /// the generator to regenerate if needed.
    pub fn set_quality(&mut self, quality: ProxyQuality) {
        self.quality = quality;
    }

    /// Get the current proxy quality setting.
    pub fn quality(&self) -> ProxyQuality {
        self.quality
    }

    /// Set the resolution threshold for proxy generation.
    pub fn set_threshold(&mut self, width: u32, height: u32) {
        self.threshold_width = width;
        self.threshold_height = height;
    }

    /// Get the number of active proxies.
    pub fn active_proxy_count(&self) -> usize {
        self.proxies.len()
    }

    /// Get the total size of all proxy files in bytes.
    pub fn total_proxy_size_bytes(&self) -> u64 {
        self.proxies.values().map(|m| m.file_size_bytes).sum()
    }

    /// Check whether an asset has a proxy registered.
    pub fn has_proxy(&self, asset_id: &str) -> bool {
        self.proxies.contains_key(asset_id)
    }

    /// Get the effective video path for preview playback.
    ///
    /// Returns the proxy path if available, otherwise the original path.
    /// This is the method the decoder should use during preview/editing.
    pub fn preview_path<'a>(&'a self, asset_id: &str, original_path: &'a str) -> &'a str {
        self.get_proxy_path(asset_id).unwrap_or(original_path)
    }

    /// Clear all proxy registrations (does not delete files).
    pub fn clear(&mut self) {
        self.proxies.clear();
    }

    /// Iterate over all registered proxies.
    ///
    /// Returns an iterator yielding (`asset_id`, `&ProxyMetadata`) pairs.
    pub fn proxies_iter(&self) -> impl Iterator<Item = (&String, &ProxyMetadata)> {
        self.proxies.iter()
    }
}

impl Default for ProxyManager {
    fn default() -> Self {
        Self::new(ProxyQuality::P480)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_quality_target_dimensions() {
        assert_eq!(ProxyQuality::P360.target_width(), 640);
        assert_eq!(ProxyQuality::P360.target_height(), 360);
        assert_eq!(ProxyQuality::P480.target_width(), 854);
        assert_eq!(ProxyQuality::P480.target_height(), 480);
        assert_eq!(ProxyQuality::P720.target_width(), 1280);
        assert_eq!(ProxyQuality::P720.target_height(), 720);
        assert_eq!(ProxyQuality::Off.target_width(), 0);
        assert_eq!(ProxyQuality::Off.target_height(), 0);
    }

    #[test]
    fn test_proxy_quality_display_name() {
        assert_eq!(ProxyQuality::Off.display_name(), "Off");
        assert_eq!(ProxyQuality::P360.display_name(), "360p");
        assert_eq!(ProxyQuality::P480.display_name(), "480p");
        assert_eq!(ProxyQuality::P720.display_name(), "720p");
    }

    #[test]
    fn test_proxy_quality_from_str() {
        assert_eq!(
            ProxyQuality::from_str_lossy("480p"),
            Some(ProxyQuality::P480)
        );
        assert_eq!(
            ProxyQuality::from_str_lossy("720p"),
            Some(ProxyQuality::P720)
        );
        assert_eq!(ProxyQuality::from_str_lossy("off"), Some(ProxyQuality::Off));
        assert_eq!(ProxyQuality::from_str_lossy("1080p"), None);
        assert_eq!(ProxyQuality::from_str_lossy("invalid"), None);
    }

    #[test]
    fn test_proxy_manager_should_generate() {
        let manager = ProxyManager::new(ProxyQuality::P480);
        // 4K video should generate proxy
        assert!(manager.should_generate_proxy(3840, 2160));
        // 1080p at threshold should NOT generate (needs to exceed)
        assert!(!manager.should_generate_proxy(1920, 1080));
        // 720p should not generate
        assert!(!manager.should_generate_proxy(1280, 720));
    }

    #[test]
    fn test_proxy_manager_should_not_generate_when_off() {
        let manager = ProxyManager::new(ProxyQuality::Off);
        assert!(!manager.should_generate_proxy(3840, 2160));
    }

    #[test]
    fn test_proxy_manager_register_and_get() {
        let mut manager = ProxyManager::new(ProxyQuality::P480);
        let metadata = ProxyMetadata {
            original_asset_id: "asset-1".to_string(),
            original_path: "/original/video.mp4".to_string(),
            proxy_path: "/cache/proxies/asset-1_proxy.mp4".to_string(),
            quality: ProxyQuality::P480,
            original_width: 3840,
            original_height: 2160,
            proxy_width: 854,
            proxy_height: 480,
            generated_at: 1700000000,
            file_size_bytes: 5_000_000,
        };

        manager.register_proxy(metadata);
        assert!(manager.has_proxy("asset-1"));
        assert_eq!(
            manager.get_proxy_path("asset-1"),
            Some("/cache/proxies/asset-1_proxy.mp4")
        );
    }

    #[test]
    fn test_proxy_manager_remove() {
        let mut manager = ProxyManager::new(ProxyQuality::P480);
        let metadata = ProxyMetadata {
            original_asset_id: "asset-1".to_string(),
            original_path: "/original/video.mp4".to_string(),
            proxy_path: "/cache/proxies/asset-1_proxy.mp4".to_string(),
            quality: ProxyQuality::P480,
            original_width: 3840,
            original_height: 2160,
            proxy_width: 854,
            proxy_height: 480,
            generated_at: 1700000000,
            file_size_bytes: 5_000_000,
        };

        manager.register_proxy(metadata);
        let removed = manager.remove_proxy("asset-1");
        assert!(removed.is_some());
        assert!(!manager.has_proxy("asset-1"));
    }

    #[test]
    fn test_proxy_manager_total_size() {
        let mut manager = ProxyManager::new(ProxyQuality::P480);
        for i in 0..3 {
            let metadata = ProxyMetadata {
                original_asset_id: format!("asset-{}", i),
                original_path: format!("/original/{}.mp4", i),
                proxy_path: format!("/cache/proxies/asset-{}_proxy.mp4", i),
                quality: ProxyQuality::P480,
                original_width: 3840,
                original_height: 2160,
                proxy_width: 854,
                proxy_height: 480,
                generated_at: 1700000000,
                file_size_bytes: 5_000_000,
            };
            manager.register_proxy(metadata);
        }
        assert_eq!(manager.active_proxy_count(), 3);
        assert_eq!(manager.total_proxy_size_bytes(), 15_000_000);
    }

    #[test]
    fn test_proxy_manager_preview_path() {
        let mut manager = ProxyManager::new(ProxyQuality::P480);
        let metadata = ProxyMetadata {
            original_asset_id: "asset-1".to_string(),
            original_path: "/original/video.mp4".to_string(),
            proxy_path: "/cache/proxies/asset-1_proxy.mp4".to_string(),
            quality: ProxyQuality::P480,
            original_width: 3840,
            original_height: 2160,
            proxy_width: 854,
            proxy_height: 480,
            generated_at: 1700000000,
            file_size_bytes: 5_000_000,
        };
        manager.register_proxy(metadata);

        // Should return proxy path for asset with proxy
        assert_eq!(
            manager.preview_path("asset-1", "/original/video.mp4"),
            "/cache/proxies/asset-1_proxy.mp4"
        );
        // Should return original path for asset without proxy
        assert_eq!(
            manager.preview_path("asset-2", "/original/other.mp4"),
            "/original/other.mp4"
        );
    }

    #[test]
    fn test_proxy_manager_set_quality() {
        let mut manager = ProxyManager::new(ProxyQuality::P480);
        assert_eq!(manager.quality(), ProxyQuality::P480);
        manager.set_quality(ProxyQuality::P720);
        assert_eq!(manager.quality(), ProxyQuality::P720);
    }

    #[test]
    fn test_proxy_manager_clear() {
        let mut manager = ProxyManager::new(ProxyQuality::P480);
        let metadata = ProxyMetadata {
            original_asset_id: "asset-1".to_string(),
            original_path: "/original/video.mp4".to_string(),
            proxy_path: "/cache/proxies/asset-1_proxy.mp4".to_string(),
            quality: ProxyQuality::P480,
            original_width: 3840,
            original_height: 2160,
            proxy_width: 854,
            proxy_height: 480,
            generated_at: 1700000000,
            file_size_bytes: 5_000_000,
        };
        manager.register_proxy(metadata);
        assert_eq!(manager.active_proxy_count(), 1);
        manager.clear();
        assert_eq!(manager.active_proxy_count(), 0);
    }
}
