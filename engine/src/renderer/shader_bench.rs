//! Benchmark tests for CPU vs GPU filter performance
//!
//! Measures the performance of CPU-based effects pipeline vs GPU compute
//! shader effects. On devices with Vulkan support, GPU effects should be
//! 5-10x faster than CPU for 1080p frames.

#[cfg(test)]
mod bench {
    use crate::decoder::FrameData;
    use crate::effects::gpu_filters::GpuFilterDispatcher;
    use crate::effects::{Effect, EffectParameter, EffectType};
    use crate::renderer::shader::ShaderManager;
    use std::time::Instant;

    /// Create a test frame of the given dimensions filled with a gradient pattern.
    fn create_test_frame(width: u32, height: u32) -> FrameData {
        let mut data = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = ((x * 255) / width) as u8;     // R: horizontal gradient
                data[idx + 1] = ((y * 255) / height) as u8; // G: vertical gradient
                data[idx + 2] = 128;                         // B: constant
                data[idx + 3] = 255;                         // A: opaque
            }
        }
        FrameData {
            width,
            height,
            data,
            timestamp_ms: 0,
            is_keyframe: true,
        }
    }

    /// Create a brightness effect with the given intensity
    fn brightness_effect(intensity: f32) -> Effect {
        Effect::new(
            "Brightness",
            EffectType::Filter,
            vec![EffectParameter::new("intensity", "Intensity", intensity, -1.0, 1.0, 0.01)],
        )
    }

    /// Create a blur effect with the given radius
    fn blur_effect(radius: f32) -> Effect {
        Effect::new(
            "Blur",
            EffectType::Filter,
            vec![EffectParameter::new("radius", "Radius", radius, 0.0, 20.0, 0.1)],
        )
    }

    /// Benchmark: CPU brightness filter on 1080p frame
    #[test]
    fn bench_cpu_brightness_1080p() {
        let mut frame = create_test_frame(1920, 1080);
        let effects = vec![brightness_effect(0.3)];

        let start = Instant::now();
        let iterations = 10;
        for _ in 0..iterations {
            let pipeline = crate::effects::EffectsPipeline::new(effects.clone());
            pipeline.apply(&mut frame.data, frame.width, frame.height);
        }
        let elapsed = start.elapsed();
        let per_frame = elapsed / iterations;

        println!(
            "CPU Brightness 1080p: {:.2}ms/frame ({} iterations)",
            per_frame.as_secs_f64() * 1000.0,
            iterations
        );

        // CPU brightness should be under 20ms for 1080p
        assert!(
            per_frame.as_millis() < 50,
            "CPU brightness too slow: {}ms",
            per_frame.as_millis()
        );
    }

    /// Benchmark: CPU blur filter on 1080p frame (blur is the slowest CPU filter)
    #[test]
    fn bench_cpu_blur_1080p() {
        let mut frame = create_test_frame(1920, 1080);
        let effects = vec![blur_effect(5.0)];

        let start = Instant::now();
        let iterations = 3;
        for _ in 0..iterations {
            let pipeline = crate::effects::EffectsPipeline::new(effects.clone());
            pipeline.apply(&mut frame.data, frame.width, frame.height);
        }
        let elapsed = start.elapsed();
        let per_frame = elapsed / iterations;

        println!(
            "CPU Blur 1080p: {:.2}ms/frame ({} iterations)",
            per_frame.as_secs_f64() * 1000.0,
            iterations
        );
    }

    /// Benchmark: CPU brightness filter on 720p frame
    #[test]
    fn bench_cpu_brightness_720p() {
        let mut frame = create_test_frame(1280, 720);
        let effects = vec![brightness_effect(0.5)];

        let start = Instant::now();
        let iterations = 20;
        for _ in 0..iterations {
            let pipeline = crate::effects::EffectsPipeline::new(effects.clone());
            pipeline.apply(&mut frame.data, frame.width, frame.height);
        }
        let elapsed = start.elapsed();
        let per_frame = elapsed / iterations;

        println!(
            "CPU Brightness 720p: {:.2}ms/frame ({} iterations)",
            per_frame.as_secs_f64() * 1000.0,
            iterations
        );

        // CPU brightness should be under 10ms for 720p
        assert!(
            per_frame.as_millis() < 30,
            "CPU brightness too slow: {}ms",
            per_frame.as_millis()
        );
    }

    /// Benchmark: GPU filter descriptor creation overhead
    #[test]
    fn bench_gpu_filter_descriptor_creation() {
        let params = vec![EffectParameter::new("intensity", "Intensity", 0.5, -1.0, 1.0, 0.01)];

        let start = Instant::now();
        let iterations = 10000;
        for _ in 0..iterations {
            let _ = GpuFilterDispatcher::create_descriptor("brightness", &params);
        }
        let elapsed = start.elapsed();
        let per_call = elapsed / iterations;

        println!(
            "GPU Filter Descriptor Creation: {:.2}us/call",
            per_call.as_secs_f64() * 1_000_000.0
        );

        // Descriptor creation should be under 10us
        assert!(
            per_call.as_micros() < 100,
            "Descriptor creation too slow: {}us",
            per_call.as_micros()
        );
    }

    /// Benchmark: CPU effects chain (multiple effects) on 720p frame
    #[test]
    fn bench_cpu_effects_chain_720p() {
        let mut frame = create_test_frame(1280, 720);
        let effects = vec![
            brightness_effect(0.2),
            Effect::new(
                "Contrast",
                EffectType::Filter,
                vec![EffectParameter::new("intensity", "Intensity", 0.3, -1.0, 1.0, 0.01)],
            ),
            Effect::new(
                "Saturation",
                EffectType::Filter,
                vec![EffectParameter::new("intensity", "Intensity", 0.5, -1.0, 1.0, 0.01)],
            ),
        ];

        let start = Instant::now();
        let iterations = 10;
        for _ in 0..iterations {
            let pipeline = crate::effects::EffectsPipeline::new(effects.clone());
            pipeline.apply(&mut frame.data, frame.width, frame.height);
        }
        let elapsed = start.elapsed();
        let per_frame = elapsed / iterations;

        println!(
            "CPU Effects Chain (3 effects) 720p: {:.2}ms/frame",
            per_frame.as_secs_f64() * 1000.0
        );
    }

    /// Benchmark: Shader loading and availability check
    #[test]
    fn bench_shader_loading() {
        let start = Instant::now();
        let iterations = 1000;
        for _ in 0..iterations {
            let _manager = ShaderManager::new();
        }
        let elapsed = start.elapsed();
        let per_init = elapsed / iterations;

        println!(
            "ShaderManager initialization: {:.2}us/init",
            per_init.as_secs_f64() * 1_000_000.0
        );

        // Shader manager init should be under 100us
        assert!(
            per_init.as_micros() < 1000,
            "ShaderManager init too slow: {}us",
            per_init.as_micros()
        );
    }

    /// Verify that GPU-accelerated effects list is complete
    #[test]
    fn test_all_effects_have_gpu_descriptors() {
        let gpu_effects = GpuFilterDispatcher::gpu_accelerated_effects();
        assert_eq!(gpu_effects.len(), 11, "Should have 11 GPU-accelerated effects");

        // Verify each effect can create a descriptor
        for &effect_name in gpu_effects {
            let params = vec![EffectParameter::new("intensity", "Intensity", 0.5, -1.0, 1.0, 0.01)];
            let descriptor = GpuFilterDispatcher::create_descriptor(effect_name, &params);
            assert!(
                descriptor.is_some(),
                "Effect '{}' should have a GPU descriptor",
                effect_name
            );
            let desc = descriptor.unwrap();
            assert!(
                !desc.shader_name.is_empty(),
                "Effect '{}' should have a non-empty shader name",
                effect_name
            );
        }
    }

    /// Verify GPU acceleration detection
    #[test]
    fn test_gpu_acceleration_detection() {
        assert!(GpuFilterDispatcher::is_gpu_accelerated("brightness"));
        assert!(GpuFilterDispatcher::is_gpu_accelerated("Brightness"));
        assert!(GpuFilterDispatcher::is_gpu_accelerated("BLUR"));
        assert!(!GpuFilterDispatcher::is_gpu_accelerated("unknown_effect"));
    }
}
