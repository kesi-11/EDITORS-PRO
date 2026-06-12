//! System monitoring utilities
//!
//! Provides memory pressure monitoring, performance metrics collection,
//! and system resource tracking for the editing engine.

pub mod memory;

use serde::{Deserialize, Serialize};

/// System resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Current RSS (Resident Set Size) in bytes
    pub memory_rss_bytes: u64,
    /// Peak RSS since engine start in bytes
    pub memory_peak_bytes: u64,
    /// Available system memory in bytes (0 if unknown)
    pub system_available_bytes: u64,
    /// Total system memory in bytes (0 if unknown)
    pub system_total_bytes: u64,
    /// Memory pressure level
    pub pressure_level: MemoryPressureLevel,
    /// Number of decoded frames currently cached
    pub cached_frames: usize,
    /// Number of audio buffers currently cached
    pub cached_audio_buffers: usize,
}

/// Memory pressure level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPressureLevel {
    /// Plenty of memory available
    Normal,
    /// Memory is getting tight, consider releasing caches
    Warning,
    /// Critical memory pressure, must release non-essential resources
    Critical,
}

impl Default for MemoryPressureLevel {
    fn default() -> Self {
        MemoryPressureLevel::Normal
    }
}

impl std::fmt::Display for MemoryPressureLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryPressureLevel::Normal => write!(f, "Normal"),
            MemoryPressureLevel::Warning => write!(f, "Warning"),
            MemoryPressureLevel::Critical => write!(f, "Critical"),
        }
    }
}
