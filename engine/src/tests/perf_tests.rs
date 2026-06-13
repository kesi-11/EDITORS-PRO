//! Performance profiling tests
//!
//! Integration tests for the performance profiling, buffer pool,
//! LRU cache, zero-copy pipeline, and priority scheduler modules.

use crate::system::buffer_pool::{BufferPool, BufferPoolConfig};
use crate::system::lru_cache::{LruCache, LruCacheConfig};
use crate::system::profiler::{
    FrameTimer, Profiler, SpanGuard, ThroughputTracker, format_duration_ns,
    is_profiling_enabled, set_profiling_enabled, span, span_if_enabled,
};
use crate::system::zero_copy::{
    DoubleBuffer, FrameBuffer, FramePipeline, adjust_brightness_in_place,
    adjust_contrast_in_place, apply_opacity_in_place, blend_rgba_in_place,
    grayscale_in_place, invert_in_place, sepia_in_place,
    BrightnessTransform, ContrastTransform, GrayscaleTransform, OpacityTransform,
};
use crate::utils::priority_scheduler::{PriorityScheduler, TaskPriority};
use std::time::Duration;

// ─── Profiler Integration Tests ──────────────────────────────────────────────

#[test]
fn test_profiler_multiple_spans() {
    let profiler = Profiler::new();
    for i in 0..10 {
        profiler.record_span(&format!("op_{}", i % 3), Duration::from_millis(i + 1));
    }
    let stats = profiler.get_all_stats();
    assert_eq!(stats.len(), 3); // 3 unique span names
    let total_calls: u64 = stats.iter().map(|s| s.call_count).sum();
    assert_eq!(total_calls, 10);
}

#[test]
fn test_profiler_span_guard_timing() {
    set_profiling_enabled(true);
    let guard = span("test_span");
    std::thread::sleep(Duration::from_millis(10));
    let duration = guard.finish();
    assert!(duration >= Duration::from_millis(10));
    set_profiling_enabled(false);
}

#[test]
fn test_profiler_conditional_span() {
    set_profiling_enabled(false);
    assert!(span_if_enabled("disabled_span").is_none());
    set_profiling_enabled(true);
    assert!(span_if_enabled("enabled_span").is_some());
    set_profiling_enabled(false);
}

#[test]
fn test_frame_timer_simulation() {
    let mut timer = FrameTimer::new(60, 30.0);
    // Simulate 30fps (budget is ~33.3ms per frame)
    for _ in 0..30 {
        timer.record(Duration::from_millis(33));
    }
    assert!(timer.is_on_budget());
    assert!(timer.average_fps() > 25.0);
    assert!(timer.drop_rate() < 0.1);
}

#[test]
fn test_frame_timer_frame_drops() {
    let mut timer = FrameTimer::new(30, 30.0);
    // Some frames within budget, some way over
    for _ in 0..15 {
        timer.record(Duration::from_millis(20)); // fast
    }
    for _ in 0..15 {
        timer.record(Duration::from_millis(80)); // slow → dropped
    }
    assert!(timer.dropped_frames() >= 10);
    assert!(timer.drop_rate() > 0.3);
}

#[test]
fn test_throughput_tracker_export_simulation() {
    let tracker = ThroughputTracker::new("export_test");
    // Simulate exporting 100 frames of 1080p (~8.3MB each)
    for _ in 0..100 {
        tracker.record_items(1);
        tracker.record_bytes(1920 * 1080 * 4);
    }
    assert_eq!(tracker.total_items(), 100);
    assert!(tracker.total_bytes() > 0);
    let summary = tracker.format_summary();
    assert!(summary.contains("export_test"));
}

// ─── Buffer Pool Integration Tests ───────────────────────────────────────────

#[test]
fn test_buffer_pool_frame_workflow() {
    let pool = BufferPool::with_config(BufferPoolConfig {
        max_per_class: 4,
        prewarm: false,
    });

    // Simulate frame processing: allocate → fill → process → return
    for _ in 0..5 {
        let mut buf = pool.allocate(1920 * 1080 * 4);
        // Simulate filling the buffer
        let data = buf.as_mut_slice();
        data[0] = 255;
        data[1] = 128;
        data[2] = 64;
        data[3] = 255;
        // Buffer returned to pool on drop
    }

    // After 5 iterations with max_per_class=4, pool should have reused buffers
    let reuses = pool.stats().reuses.load(std::sync::atomic::Ordering::Relaxed);
    assert!(reuses >= 1, "Should have reused buffers, got {} reuses", reuses);
}

#[test]
fn test_buffer_pool_memory_pressure() {
    let pool = BufferPool::new();
    {
        let _buf1 = pool.allocate(1920 * 1080 * 4);
        let _buf2 = pool.allocate(1280 * 720 * 4);
    }
    let bytes = pool.stats().pooled_bytes.load(std::sync::atomic::Ordering::Relaxed);
    assert!(bytes > 0, "Should have pooled bytes after buffer returns");

    pool.release_all();
    let after = pool.stats().pooled_bytes.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(after, 0, "Should have 0 pooled bytes after release_all");
}

// ─── LRU Cache Integration Tests ─────────────────────────────────────────────

#[test]
fn test_lru_cache_frame_cache_workflow() {
    let mut cache = LruCache::with_budget_mb(50);

    // Simulate caching decoded frames
    for i in 0..20 {
        let frame_data = vec![0u8; 1920 * 1080 * 4]; // 8.3MB per frame
        cache.put(&format!("frame_{}", i), frame_data, 8_294_400);
    }

    // Not all 20 frames fit in 50MB budget
    assert!(cache.len() <= 10, "Should have evicted some frames");

    // Hit rate should be low (most were evicted)
    // Access the remaining frames
    for i in 15..20 {
        let _ = cache.get(&format!("frame_{}", i));
    }

    let stats = cache.stats();
    let misses = stats.misses.load(std::sync::atomic::Ordering::Relaxed);
    assert!(misses > 0, "Should have cache misses from evictions");
}

#[test]
fn test_lru_cache_hot_frame_promotion() {
    let mut cache = LruCache::new(LruCacheConfig {
        max_bytes: 20_000_000, // 20MB
        ..LruCacheConfig::default()
    });

    // Cache frame 0 (2MB)
    cache.put("frame_0", vec![0u8; 2_000_000], 2_000_000);

    // Fill cache with other frames
    for i in 1..10 {
        cache.put(&format!("frame_{}", i), vec![0u8; 2_000_000], 2_000_000);
        // Access frame_0 to keep it hot
        let _ = cache.get("frame_0");
    }

    // frame_0 should still be in cache (it was promoted each time)
    assert!(cache.contains("frame_0"), "Hot frame should survive evictions");
}

// ─── Zero-Copy Pipeline Integration Tests ────────────────────────────────────

#[test]
fn test_zero_copy_pipeline_chain() {
    let mut pipeline = FramePipeline::new();
    pipeline.add(BrightnessTransform { delta: 30 });
    pipeline.add(ContrastTransform { factor: 1.2 });
    pipeline.add(OpacityTransform { opacity: 0.9 });

    let mut buffer = FrameBuffer::new(1920, 1080);
    // Fill with a gradient
    for y in 0..1080 {
        for x in 0..1920 {
            let r = ((x as f32 / 1920.0) * 255.0) as u8;
            let g = ((y as f32 / 1080.0) * 255.0) as u8;
            buffer.set_pixel(x, y, [r, g, 128, 255]);
        }
    }

    pipeline.apply(&mut buffer);
    assert!(buffer.is_dirty());
}

#[test]
fn test_double_buffer_rendering_simulation() {
    let mut db = DoubleBuffer::new(960, 540);

    // Simulate render loop: render to back, swap, read from front
    for frame in 0..10 {
        let back = db.back_mut();
        back.clear();
        // Simulate rendering
        back.set_pixel(0, 0, [frame as u8, 128, 64, 255]);
        db.swap();

        // Read from front (previously rendered frame)
        let front = db.front();
        if frame > 0 {
            let pixel = front.pixel(0, 0);
            assert_eq!(pixel[0], (frame - 1) as u8);
        }
    }
}

#[test]
fn test_in_place_operations_correctness() {
    // Test that all in-place operations produce correct results
    let mut data = [128u8, 128, 128, 255];

    // Brightness
    adjust_brightness_in_place(&mut data, 50);
    assert_eq!(data[0], 178);
    adjust_brightness_in_place(&mut data, -50);
    assert_eq!(data[0], 128);

    // Contrast
    adjust_contrast_in_place(&mut data, 1.5);
    // 128 is the center, should remain 128 after contrast
    assert_eq!(data[0], 128);

    // Grayscale
    let mut color_data = [255u8, 0, 0, 255]; // Red
    grayscale_in_place(&mut color_data);
    assert_eq!(color_data[0], color_data[1]);
    assert_eq!(color_data[1], color_data[2]);

    // Invert
    let mut inv_data = [0u8, 128, 255, 255];
    invert_in_place(&mut inv_data);
    assert_eq!(inv_data[0], 255);
    assert_eq!(inv_data[1], 127);
    assert_eq!(inv_data[2], 0);

    // Opacity
    let mut alpha_data = [255u8, 255, 255, 255];
    apply_opacity_in_place(&mut alpha_data, 0.5);
    assert_eq!(alpha_data[3], 127); // Alpha halved
}

#[test]
fn test_blend_rgba_in_place_correctness() {
    // Blend semi-transparent red over black
    let mut dst = [0u8, 0, 0, 255];
    let src = [255u8, 0, 0, 128]; // 50% alpha red
    blend_rgba_in_place(&mut dst, &src);
    assert!(dst[0] > 0, "Result should have some red");
}

// ─── Priority Scheduler Integration Tests ────────────────────────────────────

#[test]
fn test_scheduler_priority_execution() {
    let scheduler = PriorityScheduler::new(2);
    let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    // Block the worker with a long task
    let r_block = std::sync::Arc::clone(&results);
    scheduler.submit_normal(move || {
        std::thread::sleep(Duration::from_millis(100));
        r_block.lock().unwrap().push("normal_block".to_string());
    });

    std::thread::sleep(Duration::from_millis(10));

    // Submit tasks at different priorities while worker is busy
    let r_crit = std::sync::Arc::clone(&results);
    scheduler.submit_critical(move || {
        r_crit.lock().unwrap().push("critical".to_string());
    });

    let r_bg = std::sync::Arc::clone(&results);
    scheduler.submit_background(move || {
        r_bg.lock().unwrap().push("background".to_string());
    });

    std::thread::sleep(Duration::from_millis(300));

    let res = results.lock().unwrap();
    assert!(res.contains(&"critical".to_string()));
    assert!(res.contains(&"background".to_string()));
}

#[test]
fn test_scheduler_high_throughput() {
    let scheduler = PriorityScheduler::new(4);
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));

    for _ in 0..500 {
        let c = std::sync::Arc::clone(&counter);
        scheduler.submit_normal(move || {
            c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        });
    }

    std::thread::sleep(Duration::from_millis(1000));
    let count = counter.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(count, 500, "All tasks should complete");
}

// ─── End-to-End Performance Simulation ───────────────────────────────────────

#[test]
fn test_end_to_end_frame_pipeline() {
    set_profiling_enabled(true);

    let profiler = Profiler::new();
    let mut frame_timer = FrameTimer::new(60, 24.0);
    let mut buffer_pool = BufferPool::with_config(BufferPoolConfig {
        max_per_class: 4,
        prewarm: false,
    });
    let mut cache = LruCache::with_budget_mb(100);
    let throughput = ThroughputTracker::new("frame_pipeline");

    // Simulate a frame rendering pipeline
    for frame_idx in 0..30 {
        let _decode_span = SpanGuard::new("decode");

        // Check cache
        let cache_key = format!("frame_{}", frame_idx);
        let _cache_hit = cache.get(&cache_key);

        // Allocate buffer
        let mut buffer = buffer_pool.allocate(960 * 540 * 4);

        // Simulate decode
        std::thread::sleep(Duration::from_micros(500));

        // Simulate render
        let _render_span = SpanGuard::new("render");
        std::thread::sleep(Duration::from_micros(1000));

        // Record frame timing
        frame_timer.record(Duration::from_micros(1500));
        throughput.record_items(1);
        throughput.record_bytes(960 * 540 * 4);

        // Cache the frame
        cache.put(&cache_key, buffer.as_slice().to_vec(), buffer.len() as u64);

        drop(buffer);
    }

    // Verify metrics
    assert!(frame_timer.average_fps() > 0.0);
    assert!(throughput.total_items() == 30);
    assert!(cache.len() > 0);

    let pool_hit_rate = buffer_pool.stats().hit_rate();
    let cache_hit_rate = cache.stats().hit_ratio();

    // Log summary
    profiler.log_report();
    log::info!("Frame timer: {:.1} fps, drop rate: {:.1}%",
        frame_timer.average_fps(),
        frame_timer.drop_rate() * 100.0
    );
    log::info!("Buffer pool hit rate: {:.1}%", pool_hit_rate * 100.0);
    log::info!("Cache hit rate: {:.1}%", cache_hit_rate * 100.0);
    log::info!("{}", throughput.format_summary());

    set_profiling_enabled(false);
}

#[test]
fn test_format_duration_variants() {
    assert_eq!(format_duration_ns(500), "500ns");
    assert_eq!(format_duration_ns(1_500), "1.5us");
    assert_eq!(format_duration_ns(2_500_000), "2.50ms");
    assert_eq!(format_duration_ns(1_500_000_000), "1.50s");
    assert_eq!(format_duration_ns(0), "0ns");
}
