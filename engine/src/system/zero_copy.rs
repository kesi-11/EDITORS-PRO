//! Zero-copy pipeline optimizations
//!
//! Provides buffer reuse strategies, in-place frame processing,
//! and slice-based operations to minimize allocations in the
//! rendering pipeline's hot paths.
//!
//! ## Key Optimizations
//!
//! 1. **FrameBuffer** — reusable RGBA buffer with in-place operations
//! 2. **DoubleBuffer** — swap-and-render pattern for GPU readback
//! 3. **SliceOps** — SIMD-friendly slice operations for pixel processing
//! 4. **FramePipeline** — chain of in-place frame transforms

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// A reusable RGBA frame buffer that avoids allocations.
///
/// The buffer can be cleared and reused across frames. It tracks
/// the actual content length separately from the allocated capacity
/// to support partial writes.
pub struct FrameBuffer {
    /// Pixel data (RGBA, row-major)
    data: Vec<u8>,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Whether the buffer has been written to
    dirty: bool,
}

impl FrameBuffer {
    /// Create a new frame buffer with the specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            data: vec![0u8; size],
            width,
            height,
            dirty: false,
        }
    }

    /// Get the buffer as a mutable byte slice
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.dirty = true;
        &mut self.data
    }

    /// Get the buffer as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the buffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the buffer size in bytes
    pub fn byte_len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer has been written to
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the buffer (sets all bytes to 0, marks as not dirty)
    pub fn clear(&mut self) {
        for byte in self.data.iter_mut() {
            *byte = 0;
        }
        self.dirty = false;
    }

    /// Resize the buffer for new dimensions.
    /// Only reallocates if the new size is larger.
    pub fn resize(&mut self, width: u32, height: u32) {
        let new_size = (width as usize) * (height as usize) * 4;
        if new_size > self.data.len() {
            self.data.resize(new_size, 0);
        }
        self.width = width;
        self.height = height;
        self.dirty = false;
    }

    /// Get a pixel at (x, y) as [R, G, B, A]
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        if idx + 4 <= self.data.len() {
            [self.data[idx], self.data[idx + 1], self.data[idx + 2], self.data[idx + 3]]
        } else {
            [0, 0, 0, 0]
        }
    }

    /// Set a pixel at (x, y) as [R, G, B, A]
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        if idx + 4 <= self.data.len() {
            self.data[idx] = rgba[0];
            self.data[idx + 1] = rgba[1];
            self.data[idx + 2] = rgba[2];
            self.data[idx + 3] = rgba[3];
            self.dirty = true;
        }
    }
}

impl Clone for FrameBuffer {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
            dirty: self.dirty,
        }
    }
}

/// Double buffer for GPU readback pattern.
///
/// While one buffer is being read back from the GPU (front),
/// the other is being rendered into (back). This overlaps
/// GPU compute with CPU readback to maximize throughput.
pub struct DoubleBuffer {
    buffers: [FrameBuffer; 2],
    /// Index of the current front buffer (for reading)
    front_idx: AtomicUsize,
}

impl DoubleBuffer {
    /// Create a double buffer with the specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            buffers: [
                FrameBuffer::new(width, height),
                FrameBuffer::new(width, height),
            ],
            front_idx: AtomicUsize::new(0),
        }
    }

    /// Get the back buffer (for writing/rendering)
    pub fn back(&self) -> &FrameBuffer {
        let front = self.front_idx.load(Ordering::Relaxed);
        &self.buffers[1 - front]
    }

    /// Get the back buffer as mutable (for writing/rendering)
    pub fn back_mut(&mut self) -> &mut FrameBuffer {
        let front = self.front_idx.load(Ordering::Relaxed);
        &mut self.buffers[1 - front]
    }

    /// Get the front buffer (for reading/display)
    pub fn front(&self) -> &FrameBuffer {
        let front = self.front_idx.load(Ordering::Relaxed);
        &self.buffers[front]
    }

    /// Swap front and back buffers
    pub fn swap(&self) {
        let front = self.front_idx.load(Ordering::Relaxed);
        self.front_idx.store(1 - front, Ordering::Relaxed);
    }

    /// Resize both buffers
    pub fn resize(&mut self, width: u32, height: u32) {
        self.buffers[0].resize(width, height);
        self.buffers[1].resize(width, height);
    }
}

// ─── SIMD-friendly slice operations ──────────────────────────────────────────

/// In-place alpha blend of a source over a destination.
///
/// Both slices must be the same length and a multiple of 4 (RGBA).
/// This is the zero-copy equivalent of creating a new Vec.
///
/// Formula: dst = src * src_alpha + dst * (1 - src_alpha)
pub fn blend_rgba_in_place(dst: &mut [u8], src: &[u8]) {
    assert_eq!(dst.len(), src.len());
    assert!(dst.len() % 4 == 0, "Slice length must be a multiple of 4");

    for chunk_idx in 0..(dst.len() / 4) {
        let i = chunk_idx * 4;
        let src_alpha = src[i + 3] as f32 / 255.0;
        let dst_alpha = 1.0 - src_alpha;

        dst[i] = ((src[i] as f32 * src_alpha) + (dst[i] as f32 * dst_alpha)).min(255.0) as u8;
        dst[i + 1] = ((src[i + 1] as f32 * src_alpha) + (dst[i + 1] as f32 * dst_alpha)).min(255.0) as u8;
        dst[i + 2] = ((src[i + 2] as f32 * src_alpha) + (dst[i + 2] as f32 * dst_alpha)).min(255.0) as u8;
        dst[i + 3] = ((src[i + 3] as f32 * src_alpha) + (dst[i + 3] as f32 * dst_alpha)).min(255.0) as u8;
    }
}

/// In-place opacity adjustment for RGBA data.
///
/// Multiplies all alpha values by the given opacity factor.
pub fn apply_opacity_in_place(data: &mut [u8], opacity: f32) {
    assert!(data.len() % 4 == 0, "Slice length must be a multiple of 4");
    let alpha = opacity.clamp(0.0, 1.0);

    for chunk_idx in 0..(data.len() / 4) {
        let i = chunk_idx * 4;
        data[i + 3] = ((data[i + 3] as f32 * alpha).min(255.0)) as u8;
    }
}

/// In-place brightness adjustment for RGBA data.
///
/// Adds `delta` to each RGB channel, clamping to [0, 255].
pub fn adjust_brightness_in_place(data: &mut [u8], delta: i16) {
    assert!(data.len() % 4 == 0, "Slice length must be a multiple of 4");

    for chunk_idx in 0..(data.len() / 4) {
        let i = chunk_idx * 4;
        data[i] = (data[i] as i16 + delta).clamp(0, 255) as u8;
        data[i + 1] = (data[i + 1] as i16 + delta).clamp(0, 255) as u8;
        data[i + 2] = (data[i + 2] as i16 + delta).clamp(0, 255) as u8;
        // Alpha unchanged
    }
}

/// In-place contrast adjustment for RGBA data.
///
/// Multiplies each RGB channel by `factor`, centered around 128.
pub fn adjust_contrast_in_place(data: &mut [u8], factor: f32) {
    assert!(data.len() % 4 == 0, "Slice length must be a multiple of 4");

    for chunk_idx in 0..(data.len() / 4) {
        let i = chunk_idx * 4;
        data[i] = (((data[i] as f32 - 128.0) * factor) + 128.0).clamp(0.0, 255.0) as u8;
        data[i + 1] = (((data[i + 1] as f32 - 128.0) * factor) + 128.0).clamp(0.0, 255.0) as u8;
        data[i + 2] = (((data[i + 2] as f32 - 128.0) * factor) + 128.0).clamp(0.0, 255.0) as u8;
    }
}

/// In-place grayscale conversion for RGBA data.
///
/// Uses the luminance formula: Y = 0.299R + 0.587G + 0.114B
pub fn grayscale_in_place(data: &mut [u8]) {
    assert!(data.len() % 4 == 0, "Slice length must be a multiple of 4");

    for chunk_idx in 0..(data.len() / 4) {
        let i = chunk_idx * 4;
        let gray = (0.299 * data[i] as f32
            + 0.587 * data[i + 1] as f32
            + 0.114 * data[i + 2] as f32) as u8;
        data[i] = gray;
        data[i + 1] = gray;
        data[i + 2] = gray;
        // Alpha unchanged
    }
}

/// In-place inversion for RGBA data.
///
/// Inverts each RGB channel: value = 255 - value
pub fn invert_in_place(data: &mut [u8]) {
    assert!(data.len() % 4 == 0, "Slice length must be a multiple of 4");

    for chunk_idx in 0..(data.len() / 4) {
        let i = chunk_idx * 4;
        data[i] = 255 - data[i];
        data[i + 1] = 255 - data[i + 1];
        data[i + 2] = 255 - data[i + 2];
        // Alpha unchanged
    }
}

/// In-place sepia tone filter for RGBA data.
pub fn sepia_in_place(data: &mut [u8]) {
    assert!(data.len() % 4 == 0, "Slice length must be a multiple of 4");

    for chunk_idx in 0..(data.len() / 4) {
        let i = chunk_idx * 4;
        let r = data[i] as f32;
        let g = data[i + 1] as f32;
        let b = data[i + 2] as f32;

        let new_r = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0);
        let new_g = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0);
        let new_b = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0);

        data[i] = new_r as u8;
        data[i + 1] = new_g as u8;
        data[i + 2] = new_b as u8;
    }
}

// ─── Frame pipeline for chained in-place transforms ──────────────────────────

/// An in-place frame transform
pub trait FrameTransform: Send + Sync {
    /// Apply the transform to RGBA pixel data in place
    fn apply(&self, data: &mut [u8], width: u32, height: u32);

    /// Name of this transform (for profiling)
    fn name(&self) -> &str;
}

/// A brightness transform
pub struct BrightnessTransform {
    pub delta: i16,
}

impl FrameTransform for BrightnessTransform {
    fn apply(&self, data: &mut [u8], _width: u32, _height: u32) {
        adjust_brightness_in_place(data, self.delta);
    }
    fn name(&self) -> &str { "brightness" }
}

/// A contrast transform
pub struct ContrastTransform {
    pub factor: f32,
}

impl FrameTransform for ContrastTransform {
    fn apply(&self, data: &mut [u8], _width: u32, _height: u32) {
        adjust_contrast_in_place(data, self.factor);
    }
    fn name(&self) -> &str { "contrast" }
}

/// A grayscale transform
pub struct GrayscaleTransform;

impl FrameTransform for GrayscaleTransform {
    fn apply(&self, data: &mut [u8], _width: u32, _height: u32) {
        grayscale_in_place(data);
    }
    fn name(&self) -> &str { "grayscale" }
}

/// An opacity transform
pub struct OpacityTransform {
    pub opacity: f32,
}

impl FrameTransform for OpacityTransform {
    fn apply(&self, data: &mut [u8], _width: u32, _height: u32) {
        apply_opacity_in_place(data, self.opacity);
    }
    fn name(&self) -> &str { "opacity" }
}

/// A chain of in-place frame transforms
pub struct FramePipeline {
    transforms: Vec<Box<dyn FrameTransform>>,
}

impl FramePipeline {
    /// Create an empty pipeline
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    /// Add a transform to the pipeline
    pub fn add<T: FrameTransform + 'static>(&mut self, transform: T) {
        self.transforms.push(Box::new(transform));
    }

    /// Apply all transforms in sequence to a frame buffer
    pub fn apply(&self, buffer: &mut FrameBuffer) {
        let width = buffer.width;
        let height = buffer.height;
        for transform in &self.transforms {
            transform.apply(buffer.as_mut_bytes(), width, height);
        }
    }

    /// Apply all transforms to raw pixel data
    pub fn apply_raw(&self, data: &mut [u8], width: u32, height: u32) {
        for transform in &self.transforms {
            transform.apply(data, width, height);
        }
    }

    /// Get the number of transforms in the pipeline
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Get the names of all transforms in the pipeline
    pub fn transform_names(&self) -> Vec<&str> {
        self.transforms.iter().map(|t| t.name()).collect()
    }
}

impl Default for FramePipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_new() {
        let buf = FrameBuffer::new(4, 4);
        assert_eq!(buf.width(), 4);
        assert_eq!(buf.height(), 4);
        assert_eq!(buf.byte_len(), 64); // 4*4*4
        assert!(!buf.is_dirty());
    }

    #[test]
    fn test_frame_buffer_set_pixel() {
        let mut buf = FrameBuffer::new(2, 2);
        buf.set_pixel(0, 0, [255, 128, 64, 255]);
        let pixel = buf.pixel(0, 0);
        assert_eq!(pixel, [255, 128, 64, 255]);
        assert!(buf.is_dirty());
    }

    #[test]
    fn test_frame_buffer_clear() {
        let mut buf = FrameBuffer::new(2, 2);
        buf.set_pixel(0, 0, [255, 255, 255, 255]);
        buf.clear();
        assert!(!buf.is_dirty());
        let pixel = buf.pixel(0, 0);
        assert_eq!(pixel, [0, 0, 0, 0]);
    }

    #[test]
    fn test_frame_buffer_resize() {
        let mut buf = FrameBuffer::new(4, 4);
        buf.resize(8, 8);
        assert_eq!(buf.width(), 8);
        assert_eq!(buf.height(), 8);
        assert!(buf.byte_len() >= 256);
    }

    #[test]
    fn test_double_buffer() {
        let mut db = DoubleBuffer::new(4, 4);
        db.back_mut().set_pixel(0, 0, [255, 0, 0, 255]);
        db.swap();
        let pixel = db.front().pixel(0, 0);
        assert_eq!(pixel, [255, 0, 0, 255]);
    }

    #[test]
    fn test_blend_rgba_in_place() {
        let mut dst = [0u8, 0, 0, 255]; // opaque black
        let src = [255u8, 0, 0, 128]; // semi-transparent red
        blend_rgba_in_place(&mut dst, &src);
        // Result should be a blend of red and black
        assert!(dst[0] > 0, "Red channel should be > 0 after blending");
    }

    #[test]
    fn test_apply_opacity_in_place() {
        let mut data = [255u8, 128, 64, 255];
        apply_opacity_in_place(&mut data, 0.5);
        assert!(data[3] < 255, "Alpha should be reduced");
    }

    #[test]
    fn test_adjust_brightness_in_place() {
        let mut data = [100u8, 100, 100, 255];
        adjust_brightness_in_place(&mut data, 50);
        assert_eq!(data[0], 150);
        assert_eq!(data[3], 255); // Alpha unchanged
    }

    #[test]
    fn test_adjust_brightness_clamp() {
        let mut data = [200u8, 200, 200, 255];
        adjust_brightness_in_place(&mut data, 100);
        assert_eq!(data[0], 255); // Clamped
    }

    #[test]
    fn test_adjust_contrast_in_place() {
        let mut data = [128u8, 128, 128, 255];
        adjust_contrast_in_place(&mut data, 2.0);
        // 128 is the center, should remain 128 after contrast
        assert_eq!(data[0], 128);
    }

    #[test]
    fn test_grayscale_in_place() {
        let mut data = [255u8, 0, 0, 255]; // Pure red
        grayscale_in_place(&mut data);
        // Red contributes 0.299 * 255 ≈ 76
        assert!(data[0] > 0, "Gray should be > 0 for red input");
        assert_eq!(data[0], data[1]); // All channels equal
    }

    #[test]
    fn test_invert_in_place() {
        let mut data = [0u8, 128, 255, 255];
        invert_in_place(&mut data);
        assert_eq!(data[0], 255);
        assert_eq!(data[1], 127);
        assert_eq!(data[2], 0);
        assert_eq!(data[3], 255); // Alpha unchanged
    }

    #[test]
    fn test_sepia_in_place() {
        let mut data = [255u8, 255, 255, 255]; // White
        sepia_in_place(&mut data);
        // Sepia of white should be warm-tinted
        assert!(data[0] > 200); // R should be high
        assert!(data[1] > 180); // G should be high
        assert!(data[2] > 130); // B should be moderate
    }

    #[test]
    fn test_frame_pipeline() {
        let mut pipeline = FramePipeline::new();
        pipeline.add(BrightnessTransform { delta: 50 });
        pipeline.add(ContrastTransform { factor: 1.5 });

        let mut buf = FrameBuffer::new(2, 2);
        buf.set_pixel(0, 0, [100, 100, 100, 255]);
        pipeline.apply(&mut buf);
        assert!(buf.is_dirty());
    }

    #[test]
    fn test_frame_pipeline_names() {
        let mut pipeline = FramePipeline::new();
        pipeline.add(BrightnessTransform { delta: 10 });
        pipeline.add(GrayscaleTransform);
        let names = pipeline.transform_names();
        assert_eq!(names, vec!["brightness", "grayscale"]);
    }

    #[test]
    fn test_frame_pipeline_empty() {
        let pipeline = FramePipeline::new();
        assert!(pipeline.is_empty());
        let mut buf = FrameBuffer::new(2, 2);
        pipeline.apply(&mut buf); // No-op
    }

    #[test]
    fn test_frame_buffer_clone() {
        let mut buf = FrameBuffer::new(2, 2);
        buf.set_pixel(0, 0, [42, 42, 42, 255]);
        let cloned = buf.clone();
        assert_eq!(cloned.pixel(0, 0), [42, 42, 42, 255]);
    }

    #[test]
    fn test_pixel_out_of_bounds() {
        let buf = FrameBuffer::new(2, 2);
        let pixel = buf.pixel(10, 10);
        assert_eq!(pixel, [0, 0, 0, 0]); // Returns transparent black
    }
}
