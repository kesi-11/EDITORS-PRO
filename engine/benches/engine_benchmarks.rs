//! Criterion benchmarks for EDITORS-PRO engine
//!
//! Benchmarks critical paths in the engine:
//! - Audio waveform generation
//! - Color space conversions
//! - Effect processing (brightness, contrast, blur)
//! - Keyframe interpolation
//! - Timeline operations (add/remove/split clips)
//! - Project serialization/deserialization
//! - Compositing blend modes
//! - Noise reduction filters

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// ─── Waveform Generation ─────────────────────────────────────

fn bench_waveform_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("waveform");

    for &sample_count in &[1000, 10000, 100000] {
        let samples: Vec<f32> = (0..sample_count)
            .map(|i| (i as f32 * 0.01).sin() * 0.8)
            .collect();

        group.bench_with_input(
            BenchmarkId::new("from_samples", sample_count),
            &samples,
            |b, samples| {
                b.iter(|| {
                    editors_pro_engine::audio::waveform::WaveformData::from_samples(
                        black_box(samples),
                        44100,
                        2,
                        200,
                    )
                })
            },
        );
    }

    group.finish();
}

// ─── Color Conversions ───────────────────────────────────────

fn bench_color_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_conversion");

    group.bench_function("rgb_to_hsl", |b| {
        b.iter(|| {
            for r in 0..=255u8 {
                for g in 0..=255u8 {
                    let _ = editors_pro_engine::utils::math::rgb_to_hsl(
                        black_box(r),
                        black_box(g),
                        black_box(128),
                    );
                }
            }
        })
    });

    group.bench_function("hsl_to_rgb", |b| {
        b.iter(|| {
            for h in (0..360).step_by(30) {
                for s in (0..=100).step_by(25) {
                    let _ = editors_pro_engine::utils::math::hsl_to_rgb(
                        black_box(h as f32),
                        black_box(s as f32 / 100.0),
                        black_box(0.5),
                    );
                }
            }
        })
    });

    group.finish();
}

// ─── Timeline Operations ─────────────────────────────────────

fn bench_timeline_operations(c: &mut Criterion) {
    use editors_pro_engine::timeline::clip::Clip;
    use editors_pro_engine::timeline::track::{Track, TrackType};
    use editors_pro_engine::timeline::Timeline;

    let mut group = c.benchmark_group("timeline");

    group.bench_function("add_clip", |b| {
        b.iter(|| {
            let mut track = Track::new("V1".into(), TrackType::Video, 0);
            for i in 0..100 {
                let clip = Clip::new("asset-1", i * 1000, 1000);
                track.add_clip(clip);
            }
            black_box(&track);
        })
    });

    group.bench_function("timeline_split_clip", |b| {
        b.iter(|| {
            let mut tl = Timeline::new();
            tl.add_track(TrackType::Video, Some("V1".into()));
            let track_id = tl.tracks[0].id.clone();
            let clip = Clip::new("asset-1", 0, 10000);
            let clip_id = clip.id.clone();
            tl.add_clip_to_track(&track_id, clip).unwrap();
            let _ = tl.split_clip(&clip_id, 5000);
        })
    });

    group.finish();
}

// ─── Project Serialization ───────────────────────────────────

fn bench_project_serialization(c: &mut Criterion) {
    use editors_pro_engine::project::Project;

    let mut group = c.benchmark_group("project");

    let project = Project::new("Benchmark Project", None);

    group.bench_function("serialize_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&project)).unwrap();
            black_box(json);
        })
    });

    group.bench_function("serialize_and_deserialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&project)).unwrap();
            let parsed: Project = serde_json::from_str(&json).unwrap();
            black_box(parsed);
        })
    });

    group.finish();
}

// ─── Compositing Blend Modes ─────────────────────────────────

fn bench_blend_modes(c: &mut Criterion) {
    use editors_pro_engine::effects::compositing::BlendMode;

    let mut group = c.benchmark_group("blend_modes");

    let base_pixel = [128u8, 128, 128, 255];
    let blend_pixel = [200u8, 100, 50, 200];

    for mode in [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::SoftLight,
        BlendMode::ColorDodge,
    ] {
        group.bench_with_input(
            BenchmarkId::new("blend_pixel", format!("{:?}", mode)),
            &mode,
            |b, mode| {
                b.iter(|| {
                    for _ in 0..1000 {
                        let _ = editors_pro_engine::effects::compositing::blend_pixels(
                            black_box(&base_pixel),
                            black_box(&blend_pixel),
                            black_box(mode),
                        );
                    }
                })
            },
        );
    }

    group.finish();
}

// ─── Keyframe Interpolation ──────────────────────────────────

fn bench_keyframe_interpolation(c: &mut Criterion) {
    use editors_pro_engine::timeline::keyframe::{InterpolationType, Keyframe, KeyframeTrack};

    let mut group = c.benchmark_group("keyframe");

    group.bench_function("linear_interpolation_1000_keyframes", |b| {
        let mut track = KeyframeTrack::new("position_x");
        for i in 0..1000 {
            track.add_keyframe(Keyframe::new(i as u64 * 10, i as f32, InterpolationType::Linear));
        }
        b.iter(|| {
            for t in (0..10000).step_by(100) {
                let _ = track.interpolate_at(black_box(t));
            }
        })
    });

    group.finish();
}

// ─── Speed Curve Evaluation ──────────────────────────────────

fn bench_speed_curve(c: &mut Criterion) {
    use editors_pro_engine::timeline::speed_curve::{EasingType, SpeedCurve, SpeedSegment};

    let mut group = c.benchmark_group("speed_curve");

    group.bench_function("evaluate_speed_constant", |b| {
        let curve = SpeedCurve::constant(1.0);
        b.iter(|| {
            for t in (0..10000).step_by(10) {
                let _ = curve.evaluate_speed_at(black_box(t));
            }
        })
    });

    group.bench_function("evaluate_speed_ramp", |b| {
        let curve = SpeedCurve::ramp(vec![
            SpeedSegment { start_ms: 0, end_ms: 5000, start_speed: 0.5, end_speed: 2.0, easing: EasingType::EaseInOut },
            SpeedSegment { start_ms: 5000, end_ms: 10000, start_speed: 2.0, end_speed: 1.0, easing: EasingType::EaseOut },
        ]);
        b.iter(|| {
            for t in (0..10000).step_by(10) {
                let _ = curve.evaluate_speed_at(black_box(t));
            }
        })
    });

    group.finish();
}

// ─── Buffer Pool ──────────────────────────────────────────────────

fn bench_buffer_pool(c: &mut Criterion) {
    use editors_pro_engine::system::buffer_pool::{BufferPool, BufferPoolConfig};

    let mut group = c.benchmark_group("buffer_pool");

    group.bench_function("allocate_1080p_new", |b| {
        b.iter(|| {
            let pool = BufferPool::with_config(BufferPoolConfig {
                max_per_class: 4,
                prewarm: false,
            });
            let buf = pool.allocate(1920 * 1080 * 4);
            black_box(buf);
        })
    });

    group.bench_function("allocate_1080p_reuse", |b| {
        let pool = BufferPool::with_config(BufferPoolConfig {
            max_per_class: 4,
            prewarm: false,
        });
        b.iter(|| {
            let buf = pool.allocate(1920 * 1080 * 4);
            drop(buf);
        })
    });

    group.finish();
}

// ─── LRU Cache ────────────────────────────────────────────────────

fn bench_lru_cache(c: &mut Criterion) {
    use editors_pro_engine::storage::lru_cache::{LruCache, LruCacheConfig};

    let mut group = c.benchmark_group("lru_cache");

    group.bench_function("put_and_get_1000", |b| {
        let mut cache = LruCache::new(LruCacheConfig {
            max_bytes: 100_000_000,
            ..LruCacheConfig::default()
        });
        b.iter(|| {
            for i in 0..1000 {
                cache.put(&format!("key_{}", i), vec![0u8; 100], 100);
            }
            for i in 0..1000 {
                let _ = cache.get(&format!("key_{}", i));
            }
        })
    });

    group.bench_function("eviction_under_pressure", |b| {
        b.iter(|| {
            let mut cache = LruCache::new(LruCacheConfig {
                max_bytes: 10_000,
                ..LruCacheConfig::default()
            });
            for i in 0..1000 {
                cache.put(&format!("key_{}", i), vec![0u8; 100], 100);
            }
            black_box(&cache);
        })
    });

    group.finish();
}

// ─── Zero-Copy Operations ─────────────────────────────────────────

fn bench_zero_copy_operations(c: &mut Criterion) {
    use editors_pro_engine::system::zero_copy::*;

    let mut group = c.benchmark_group("zero_copy");

    // 1080p frame data (1920 * 1080 * 4 = 8,294,400 bytes)
    let frame_size = 1920 * 1080 * 4;

    group.bench_function("brightness_1080p", |b| {
        let mut data = vec![128u8; frame_size];
        b.iter(|| {
            adjust_brightness_in_place(&mut data, 50);
        })
    });

    group.bench_function("contrast_1080p", |b| {
        let mut data = vec![128u8; frame_size];
        b.iter(|| {
            adjust_contrast_in_place(&mut data, 1.5);
        })
    });

    group.bench_function("grayscale_1080p", |b| {
        let mut data = vec![128u8; frame_size];
        b.iter(|| {
            grayscale_in_place(&mut data);
        })
    });

    group.bench_function("invert_1080p", |b| {
        let mut data = vec![128u8; frame_size];
        b.iter(|| {
            invert_in_place(&mut data);
        })
    });

    group.bench_function("sepia_1080p", |b| {
        let mut data = vec![128u8; frame_size];
        b.iter(|| {
            sepia_in_place(&mut data);
        })
    });

    group.bench_function("opacity_1080p", |b| {
        let mut data = vec![255u8; frame_size];
        b.iter(|| {
            apply_opacity_in_place(&mut data, 0.75);
        })
    });

    group.bench_function("blend_rgba_1080p", |b| {
        let mut dst = vec![0u8; frame_size];
        let src = vec![128u8; frame_size];
        b.iter(|| {
            blend_rgba_in_place(&mut dst, &src);
        })
    });

    group.finish();
}

// ─── Frame Pipeline ───────────────────────────────────────────────

fn bench_frame_pipeline(c: &mut Criterion) {
    use editors_pro_engine::system::zero_copy::*;

    let mut group = c.benchmark_group("frame_pipeline");

    group.bench_function("single_transform_1080p", |b| {
        let mut pipeline = FramePipeline::new();
        pipeline.add(BrightnessTransform { delta: 30 });
        let mut buffer = FrameBuffer::new(1920, 1080);
        b.iter(|| {
            buffer.clear();
            pipeline.apply(&mut buffer);
        })
    });

    group.bench_function("chain_4_transforms_1080p", |b| {
        let mut pipeline = FramePipeline::new();
        pipeline.add(BrightnessTransform { delta: 20 });
        pipeline.add(ContrastTransform { factor: 1.3 });
        pipeline.add(OpacityTransform { opacity: 0.9 });
        pipeline.add(GrayscaleTransform);
        let mut buffer = FrameBuffer::new(1920, 1080);
        b.iter(|| {
            buffer.clear();
            pipeline.apply(&mut buffer);
        })
    });

    group.bench_function("double_buffer_swap_1080p", |b| {
        let mut db = DoubleBuffer::new(1920, 1080);
        b.iter(|| {
            db.back_mut().clear();
            db.swap();
            black_box(db.front());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_waveform_generation,
    bench_color_conversions,
    bench_timeline_operations,
    bench_project_serialization,
    bench_blend_modes,
    bench_keyframe_interpolation,
    bench_speed_curve,
    bench_buffer_pool,
    bench_lru_cache,
    bench_zero_copy_operations,
    bench_frame_pipeline,
);

criterion_main!(benches);
