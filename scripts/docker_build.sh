#!/usr/bin/env bash
# Docker build entry point — compiles the Rust engine for Android arm64-v8a
set -euo pipefail

echo "============================================================"
echo "  EDITORS-PRO — Docker Android Build"
echo "============================================================"
echo ""

# Build the Rust engine with FFmpeg support
cd /project/engine

echo "Building Rust engine for aarch64-linux-android..."
cargo ndk -t arm64-v8a build --release --features ffmpeg

echo ""
echo "✓ Build complete!"
echo ""

# Copy the .so to jniLibs
JNI_DIR="/project/android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$JNI_DIR"
cp target/aarch64-linux-android/release/libeditors_pro_engine.so "$JNI_DIR/"
echo "✓ Copied libeditors_pro_engine.so to: $JNI_DIR"

# Copy FFmpeg .so files to jniLibs
cp ${FFMPEG_DIR}/lib/*.so "$JNI_DIR/" 2>/dev/null || true
echo "✓ Copied FFmpeg .so files to: $JNI_DIR"

echo ""
echo "============================================================"
echo "  Build SUCCESS! Now run on your host machine:"
echo "    flutter run"
echo "  Or build APK:"
echo "    flutter build apk --release"
echo "============================================================"
