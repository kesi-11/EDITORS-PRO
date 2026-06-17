//! Storage subsystem
//!
//! Provides caching primitives and project persistence.
//!
//! NOTE: `project_store` (SQLite-backed) and `proxy_manager` were duplicate
//! implementations of functionality that already exists in `crate::project`
//! and `crate::proxy`. They have been removed in Phase A of the upgrade plan
//! to eliminate the orphan-module dead code and the conflicting type names.
//! The canonical implementations are:
//!   - `crate::project::Project` for project data
//!   - `crate::proxy::ProxyManager` for proxy workflow
//!   - `crate::storage::lru_cache::LruCache` for O(1) LRU caching
//!   - `crate::storage::cache_manager::CacheManager` for frame/thumbnail caches

pub mod cache_manager;
pub mod lru_cache;

pub use cache_manager::CacheManager;
pub use lru_cache::LruCache;
