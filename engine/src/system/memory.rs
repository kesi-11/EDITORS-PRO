//! Memory monitoring and pressure detection
//!
//! Tracks the engine's memory consumption and detects memory pressure
//! conditions. When pressure is detected, the engine can proactively
//! release caches and reduce quality to avoid OOM crashes.
//!
//! ## Memory Pressure Thresholds (Android)
//!
//! - **Normal**: RSS < 60% of available memory
//! - **Warning**: RSS 60-80% of available memory → release preview caches
//! - **Critical**: RSS > 80% of available memory → release all caches, reduce quality

use super::{MemoryPressureLevel, SystemMetrics};

/// Memory monitor that tracks RSS and detects pressure conditions
pub struct MemoryMonitor {
    /// Peak RSS observed since engine start
    peak_rss: u64,
    /// Threshold for warning level (fraction of available memory, 0.0-1.0)
    warning_threshold: f32,
    /// Threshold for critical level (fraction of available memory, 0.0-1.0)
    critical_threshold: f32,
    /// Whether to log memory metrics periodically
    log_metrics: bool,
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self {
            peak_rss: 0,
            warning_threshold: 0.6,
            critical_threshold: 0.8,
            log_metrics: true,
        }
    }
}

impl MemoryMonitor {
    /// Create a new memory monitor with default thresholds
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a memory monitor with custom thresholds
    pub fn with_thresholds(warning: f32, critical: f32) -> Self {
        Self {
            peak_rss: 0,
            warning_threshold: warning.clamp(0.0, 1.0),
            critical_threshold: critical.clamp(0.0, 1.0),
            log_metrics: true,
        }
    }

    /// Get the current RSS (Resident Set Size) of this process in bytes.
    ///
    /// On Linux/Android, this reads from `/proc/self/statm`.
    /// Returns 0 if the value cannot be determined.
    pub fn current_rss(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
                let fields: Vec<&str> = contents.split_whitespace().collect();
                if fields.len() >= 2 {
                    // Field 2 is RSS in pages
                    if let Ok(pages) = fields[1].parse::<u64>() {
                        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };
                        return pages * page_size;
                    }
                }
            }
        }

        // Fallback: estimate from allocation tracking
        0
    }

    /// Get available system memory in bytes.
    ///
    /// On Linux/Android, this reads from `/proc/meminfo`.
    /// Returns 0 if the value cannot be determined.
    pub fn available_system_memory(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
                for line in contents.lines() {
                    if line.starts_with("MemAvailable:") {
                        // Format: "MemAvailable:    1234567 kB"
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024;
                            }
                        }
                    }
                }
            }
        }

        0
    }

    /// Get total system memory in bytes.
    ///
    /// On Linux/Android, this reads from `/proc/meminfo`.
    /// Returns 0 if the value cannot be determined.
    pub fn total_system_memory(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
                for line in contents.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024;
                            }
                        }
                    }
                }
            }
        }

        0
    }

    /// Determine the current memory pressure level
    pub fn pressure_level(&self) -> MemoryPressureLevel {
        let rss = self.current_rss();
        let available = self.available_system_memory();

        if available == 0 || rss == 0 {
            // Can't determine, assume normal
            return MemoryPressureLevel::Normal;
        }

        let usage_ratio = rss as f32 / available as f32;

        if usage_ratio >= self.critical_threshold {
            MemoryPressureLevel::Critical
        } else if usage_ratio >= self.warning_threshold {
            MemoryPressureLevel::Warning
        } else {
            MemoryPressureLevel::Normal
        }
    }

    /// Collect a full system metrics snapshot
    pub fn collect_metrics(
        &mut self,
        cached_frames: usize,
        cached_audio_buffers: usize,
    ) -> SystemMetrics {
        let rss = self.current_rss();
        if rss > self.peak_rss {
            self.peak_rss = rss;
        }

        let pressure = self.pressure_level();

        if self.log_metrics && pressure != MemoryPressureLevel::Normal {
            log::warn!(
                "Memory pressure: {} (RSS: {}MB, Peak: {}MB, Available: {}MB)",
                pressure,
                rss / (1024 * 1024),
                self.peak_rss / (1024 * 1024),
                self.available_system_memory() / (1024 * 1024),
            );
        }

        SystemMetrics {
            memory_rss_bytes: rss,
            memory_peak_bytes: self.peak_rss,
            system_available_bytes: self.available_system_memory(),
            system_total_bytes: self.total_system_memory(),
            pressure_level: pressure,
            cached_frames,
            cached_audio_buffers,
        }
    }

    /// Check if caches should be released based on memory pressure.
    ///
    /// Returns `true` if memory is under warning or critical pressure
    /// and the engine should proactively release caches.
    pub fn should_release_caches(&self) -> bool {
        matches!(self.pressure_level(), MemoryPressureLevel::Warning | MemoryPressureLevel::Critical)
    }

    /// Check if the engine should reduce preview quality due to memory pressure.
    ///
    /// Returns `true` only under critical pressure.
    pub fn should_reduce_quality(&self) -> bool {
        matches!(self.pressure_level(), MemoryPressureLevel::Critical)
    }

    /// Format a byte count as a human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_monitor_new() {
        let monitor = MemoryMonitor::new();
        assert_eq!(monitor.peak_rss, 0);
        assert!((monitor.warning_threshold - 0.6).abs() < 0.01);
        assert!((monitor.critical_threshold - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_custom_thresholds() {
        let monitor = MemoryMonitor::with_thresholds(0.5, 0.7);
        assert!((monitor.warning_threshold - 0.5).abs() < 0.01);
        assert!((monitor.critical_threshold - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_thresholds_clamped() {
        let monitor = MemoryMonitor::with_thresholds(-0.5, 1.5);
        assert!((monitor.warning_threshold - 0.0).abs() < 0.01);
        assert!((monitor.critical_threshold - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(MemoryMonitor::format_bytes(0), "0 B");
        assert_eq!(MemoryMonitor::format_bytes(512), "512 B");
        assert_eq!(MemoryMonitor::format_bytes(1024), "1.0 KB");
        assert_eq!(MemoryMonitor::format_bytes(1048576), "1.0 MB");
        assert_eq!(MemoryMonitor::format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_collect_metrics() {
        let mut monitor = MemoryMonitor::new();
        let metrics = monitor.collect_metrics(5, 3);
        // On non-Linux, RSS will be 0
        assert!(metrics.cached_frames == 5);
        assert!(metrics.cached_audio_buffers == 3);
    }

    #[test]
    fn test_pressure_level_default() {
        assert_eq!(MemoryPressureLevel::default(), MemoryPressureLevel::Normal);
    }
}
