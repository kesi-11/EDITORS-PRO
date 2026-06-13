use log::{debug, warn};
use std::collections::HashMap;

/// A simple LRU cache entry with size tracking.
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    last_access: u64,
}

/// LRU cache manager for frames and thumbnails.
pub struct CacheManager {
    frame_cache: HashMap<String, HashMap<u64, CacheEntry>>,
    thumbnail_cache: HashMap<String, CacheEntry>,
    memory_budget: u64,
    used_memory: u64,
    access_counter: u64,
}

impl CacheManager {
    /// Create a new cache manager with the given memory budget in megabytes.
    pub fn new(max_memory_mb: u32) -> Self {
        Self {
            frame_cache: HashMap::new(),
            thumbnail_cache: HashMap::new(),
            memory_budget: (max_memory_mb as u64) * 1024 * 1024,
            used_memory: 0,
            access_counter: 0,
        }
    }

    /// Get a cached frame for the given project and frame index.
    pub fn get_frame(&mut self, project_id: &str, frame_idx: u64) -> Option<Vec<u8>> {
        self.access_counter += 1;
        if let Some(project_cache) = self.frame_cache.get_mut(project_id) {
            if let Some(entry) = project_cache.get_mut(&frame_idx) {
                entry.last_access = self.access_counter;
                debug!("Cache hit: frame {} for project {}", frame_idx, project_id);
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Store a frame in the cache.
    pub fn put_frame(&mut self, project_id: &str, frame_idx: u64, data: Vec<u8>) {
        let data_size = data.len() as u64;
        self.ensure_space(data_size);

        let entry = CacheEntry {
            data,
            last_access: self.access_counter,
        };

        self.frame_cache
            .entry(project_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(frame_idx, entry);

        self.used_memory += data_size;
        debug!(
            "Cached frame {} for project {} (size: {} bytes, total: {} bytes)",
            frame_idx, project_id, data_size, self.used_memory
        );
    }

    /// Get a cached thumbnail for the given path.
    pub fn get_thumbnail(&mut self, path: &str) -> Option<Vec<u8>> {
        self.access_counter += 1;
        if let Some(entry) = self.thumbnail_cache.get_mut(path) {
            entry.last_access = self.access_counter;
            debug!("Thumbnail cache hit: {}", path);
            return Some(entry.data.clone());
        }
        None
    }

    /// Store a thumbnail in the cache.
    pub fn put_thumbnail(&mut self, path: &str, data: Vec<u8>) {
        let data_size = data.len() as u64;
        self.ensure_space(data_size);

        let entry = CacheEntry {
            data,
            last_access: self.access_counter,
        };

        // If overwriting, adjust memory usage
        if let Some(old) = self.thumbnail_cache.insert(path.to_string(), entry) {
            self.used_memory -= old.data.len() as u64;
        }
        self.used_memory += data_size;
    }

    /// Evict all cached data for a specific project.
    pub fn evict(&mut self, project_id: &str) {
        if let Some(project_cache) = self.frame_cache.remove(project_id) {
            for (_, entry) in project_cache {
                self.used_memory -= entry.data.len() as u64;
            }
        }
        debug!("Evicted cache for project {}", project_id);
    }

    /// Get the total memory usage in bytes.
    pub fn get_memory_usage(&self) -> u64 {
        self.used_memory
    }

    /// Get the memory budget in bytes.
    pub fn get_memory_budget(&self) -> u64 {
        self.memory_budget
    }

    /// Get the cache hit ratio (not tracked precisely; returns utilization).
    pub fn utilization(&self) -> f32 {
        if self.memory_budget == 0 {
            0.0
        } else {
            self.used_memory as f32 / self.memory_budget as f32
        }
    }

    /// Evict the least recently used entries until there's enough space.
    fn ensure_space(&mut self, needed: u64) {
        while self.used_memory + needed > self.memory_budget && self.used_memory > 0 {
            self.evict_lru();
        }
    }

    /// Find and remove the least recently used cache entry.
    fn evict_lru(&mut self) {
        // Find LRU frame
        let mut lru_project: Option<String> = None;
        let mut lru_frame: Option<u64> = None;
        let mut lru_time = u64::MAX;

        for (project_id, project_cache) in &self.frame_cache {
            for (frame_idx, entry) in project_cache {
                if entry.last_access < lru_time {
                    lru_time = entry.last_access;
                    lru_project = Some(project_id.clone());
                    lru_frame = Some(*frame_idx);
                }
            }
        }

        // Find LRU thumbnail
        let mut lru_thumb: Option<String> = None;
        let mut lru_thumb_time = u64::MAX;
        for (path, entry) in &self.thumbnail_cache {
            if entry.last_access < lru_thumb_time {
                lru_thumb_time = entry.last_access;
                lru_thumb = Some(path.clone());
            }
        }

        // Evict whichever is older
        if lru_time <= lru_thumb_time {
            if let (Some(project), Some(frame)) = (lru_project, lru_frame) {
                if let Some(project_cache) = self.frame_cache.get_mut(&project) {
                    if let Some(entry) = project_cache.remove(&frame) {
                        self.used_memory -= entry.data.len() as u64;
                        debug!("Evicted LRU frame {} from project {}", frame, project);
                    }
                }
                // Clean up empty project caches
                if let Some(project_cache) = self.frame_cache.get(&project) {
                    if project_cache.is_empty() {
                        self.frame_cache.remove(&project);
                    }
                }
            }
        } else if let Some(path) = lru_thumb {
            if let Some(entry) = self.thumbnail_cache.remove(&path) {
                self.used_memory -= entry.data.len() as u64;
                debug!("Evicted LRU thumbnail: {}", path);
            }
        }
    }

    /// Clear all caches.
    pub fn clear(&mut self) {
        self.frame_cache.clear();
        self.thumbnail_cache.clear();
        self.used_memory = 0;
        debug!("All caches cleared");
    }

    /// Get the number of cached frames across all projects.
    pub fn cached_frame_count(&self) -> usize {
        self.frame_cache.values().map(|c| c.len()).sum()
    }

    /// Get the number of cached thumbnails.
    pub fn cached_thumbnail_count(&self) -> usize {
        self.thumbnail_cache.len()
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_new() {
        let cache = CacheManager::new(100);
        assert_eq!(cache.get_memory_usage(), 0);
        assert_eq!(cache.cached_frame_count(), 0);
    }

    #[test]
    fn test_cache_put_get_frame() {
        let mut cache = CacheManager::new(100);
        cache.put_frame("proj1", 0, vec![1, 2, 3, 4]);
        let result = cache.get_frame("proj1", 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_cache_miss_frame() {
        let mut cache = CacheManager::new(100);
        let result = cache.get_frame("proj1", 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_put_get_thumbnail() {
        let mut cache = CacheManager::new(100);
        cache.put_thumbnail("/path/to/file.mp4", vec![5, 6, 7, 8]);
        let result = cache.get_thumbnail("/path/to/file.mp4");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![5, 6, 7, 8]);
    }

    #[test]
    fn test_cache_evict_project() {
        let mut cache = CacheManager::new(100);
        cache.put_frame("proj1", 0, vec![1, 2, 3, 4]);
        cache.put_frame("proj1", 1, vec![5, 6, 7, 8]);
        cache.evict("proj1");
        assert!(cache.get_frame("proj1", 0).is_none());
        assert!(cache.get_frame("proj1", 1).is_none());
        assert_eq!(cache.get_memory_usage(), 0);
    }

    #[test]
    fn test_cache_memory_tracking() {
        let mut cache = CacheManager::new(100);
        cache.put_frame("proj1", 0, vec![0u8; 1024]);
        assert_eq!(cache.get_memory_usage(), 1024);
        cache.evict("proj1");
        assert_eq!(cache.get_memory_usage(), 0);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = CacheManager::new(1); // 1MB budget
        cache.put_frame("proj1", 0, vec![0u8; 512 * 1024]);
        cache.put_frame("proj1", 1, vec![0u8; 512 * 1024]);
        // Now cache is full. Adding another should trigger LRU eviction.
        cache.put_frame("proj1", 2, vec![0u8; 512 * 1024]);
        // Frame 0 should have been evicted (LRU)
        assert!(cache.get_frame("proj1", 0).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CacheManager::new(100);
        cache.put_frame("proj1", 0, vec![1, 2, 3]);
        cache.put_thumbnail("/path", vec![4, 5, 6]);
        cache.clear();
        assert_eq!(cache.cached_frame_count(), 0);
        assert_eq!(cache.cached_thumbnail_count(), 0);
        assert_eq!(cache.get_memory_usage(), 0);
    }
}
