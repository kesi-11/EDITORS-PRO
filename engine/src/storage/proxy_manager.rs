use anyhow::{Context, Result};
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};

/// Proxy video manager for generating and managing lower-resolution proxy files.
pub struct ProxyManager {
    proxy_dir: String,
    proxy_resolution: (u32, u32),
    proxy_bitrate: u64,
}

impl ProxyManager {
    /// Create a new proxy manager with the given proxy directory.
    pub fn new(proxy_dir: &str) -> Self {
        Self {
            proxy_dir: proxy_dir.to_string(),
            proxy_resolution: (960, 540),
            proxy_bitrate: 2_000_000,
        }
    }

    /// Create a proxy manager with custom settings.
    pub fn with_settings(
        proxy_dir: &str,
        resolution: (u32, u32),
        bitrate: u64,
    ) -> Self {
        Self {
            proxy_dir: proxy_dir.to_string(),
            proxy_resolution: resolution,
            proxy_bitrate: bitrate,
        }
    }

    /// Generate a proxy video for the given source file.
    /// Returns the path to the generated proxy file.
    pub fn generate_proxy(&self, source_path: &str) -> Result<String> {
        let proxy_path = self.compute_proxy_path(source_path);

        // Ensure the proxy directory exists
        if let Some(parent) = Path::new(&proxy_path).parent() {
            fs::create_dir_all(parent)
                .context("Failed to create proxy directory")?;
        }

        // Check if proxy already exists and is newer than source
        if self.is_proxy_fresh(source_path) {
            debug!("Proxy already exists and is fresh: {}", proxy_path);
            return Ok(proxy_path);
        }

        // In a complete implementation, this would use FFmpeg to transcode:
        // ffmpeg -i source -vf scale=960:540 -c:v libx264 -b:v 2M -an proxy.mp4
        info!(
            "Generating proxy for {} at {}x{} @ {}bps -> {}",
            source_path,
            self.proxy_resolution.0,
            self.proxy_resolution.1,
            self.proxy_bitrate,
            proxy_path
        );

        // Create a placeholder proxy file
        // Real implementation would call the encoder module
        fs::write(&proxy_path, b"PROXY_PLACEHOLDER")
            .context("Failed to write proxy file")?;

        Ok(proxy_path)
    }

    /// Get the proxy path for a given source file, if the proxy exists.
    pub fn get_proxy_path(&self, source_path: &str) -> Option<String> {
        let proxy_path = self.compute_proxy_path(source_path);
        if Path::new(&proxy_path).exists() {
            Some(proxy_path)
        } else {
            None
        }
    }

    /// Check if a proxy is available for the given source.
    pub fn is_proxy_available(&self, source_path: &str) -> bool {
        self.get_proxy_path(source_path).is_some()
    }

    /// Delete the proxy file for the given source.
    pub fn delete_proxy(&self, source_path: &str) -> Result<()> {
        let proxy_path = self.compute_proxy_path(source_path);
        if Path::new(&proxy_path).exists() {
            fs::remove_file(&proxy_path)
                .context("Failed to delete proxy file")?;
            debug!("Deleted proxy: {}", proxy_path);
        }
        Ok(())
    }

    /// Clean up proxy files that are not in the list of active sources.
    pub fn cleanup_unused(&self, active_sources: &[String]) -> Result<()> {
        let proxy_dir = Path::new(&self.proxy_dir);
        if !proxy_dir.exists() {
            return Ok(());
        }

        let active_proxy_paths: Vec<String> = active_sources
            .iter()
            .map(|s| self.compute_proxy_path(s))
            .collect();

        let entries = fs::read_dir(proxy_dir)
            .context("Failed to read proxy directory")?;

        let mut deleted_count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let path_str = path.to_string_lossy().to_string();
                if !active_proxy_paths.contains(&path_str) {
                    if fs::remove_file(&path).is_ok() {
                        deleted_count += 1;
                    }
                }
            }
        }

        if deleted_count > 0 {
            info!("Cleaned up {} unused proxy files", deleted_count);
        }
        Ok(())
    }

    /// Compute the proxy file path for a given source file.
    fn compute_proxy_path(&self, source_path: &str) -> String {
        let source = Path::new(source_path);
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let extension = source
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");

        // Include a hash of the source path to avoid collisions
        let hash = self.simple_hash(source_path);

        PathBuf::from(&self.proxy_dir)
            .join(format!("{}_proxy_{}.{}", stem, hash, extension))
            .to_string_lossy()
            .to_string()
    }

    /// Check if the proxy is newer than the source (freshness check).
    fn is_proxy_fresh(&self, source_path: &str) -> bool {
        let proxy_path = self.compute_proxy_path(source_path);

        let source_meta = fs::metadata(source_path).ok();
        let proxy_meta = fs::metadata(&proxy_path).ok();

        match (source_meta, proxy_meta) {
            (Some(src), Some(prx)) => {
                if let (Ok(src_time), Ok(prx_time)) =
                    (src.modified(), prx.modified())
                {
                    prx_time >= src_time
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Simple hash function for generating unique proxy names.
    fn simple_hash(&self, s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    /// Get the proxy resolution.
    pub fn proxy_resolution(&self) -> (u32, u32) {
        self.proxy_resolution
    }

    /// Get the proxy bitrate.
    pub fn proxy_bitrate(&self) -> u64 {
        self.proxy_bitrate
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_proxy_dir() -> String {
        let dir = std::env::temp_dir().join(format!("editors_pro_proxy_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir.to_str().unwrap().to_string()
    }

    #[test]
    fn test_proxy_manager_new() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::new(&dir);
        assert_eq!(manager.proxy_resolution(), (960, 540));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_proxy_manager_with_settings() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::with_settings(&dir, (640, 360), 1_000_000);
        assert_eq!(manager.proxy_resolution(), (640, 360));
        assert_eq!(manager.proxy_bitrate(), 1_000_000);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_proxy_manager_generate() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::new(&dir);
        // Create a dummy source file
        let source_dir = std::env::temp_dir().join("editors_pro_source");
        fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("test_video.mp4");
        fs::write(&source, b"fake video").unwrap();

        let result = manager.generate_proxy(source.to_str().unwrap());
        assert!(result.is_ok());
        let proxy_path = result.unwrap();
        assert!(Path::new(&proxy_path).exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&source_dir);
    }

    #[test]
    fn test_proxy_manager_get_path() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::new(&dir);
        let result = manager.get_proxy_path("/nonexistent/video.mp4");
        assert!(result.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_proxy_manager_is_available() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::new(&dir);
        assert!(!manager.is_proxy_available("/nonexistent/video.mp4"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_proxy_manager_delete() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::new(&dir);
        let source_dir = std::env::temp_dir().join("editors_pro_source_del");
        fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("del_video.mp4");
        fs::write(&source, b"fake").unwrap();

        let proxy_path = manager.generate_proxy(source.to_str().unwrap()).unwrap();
        assert!(Path::new(&proxy_path).exists());

        manager.delete_proxy(source.to_str().unwrap()).unwrap();
        assert!(!Path::new(&proxy_path).exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&source_dir);
    }

    #[test]
    fn test_proxy_manager_cleanup_unused() {
        let dir = temp_proxy_dir();
        let manager = ProxyManager::new(&dir);

        // Create a proxy file directly
        let proxy_file = PathBuf::from(&dir).join("unused_proxy.mp4");
        fs::write(&proxy_file, b"unused").unwrap();

        // Create a source and its proxy
        let source_dir = std::env::temp_dir().join("editors_pro_source_cleanup");
        fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("active.mp4");
        fs::write(&source, b"active").unwrap();
        let active_proxy = manager.generate_proxy(source.to_str().unwrap()).unwrap();

        // Cleanup with only the active source
        let result = manager.cleanup_unused(&[source.to_str().unwrap().to_string()]);
        assert!(result.is_ok());

        // The active proxy should still exist
        assert!(Path::new(&active_proxy).exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&source_dir);
    }
}
