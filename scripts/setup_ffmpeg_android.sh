#!/usr/bin/env bash
# ============================================================================
# setup_ffmpeg_android.sh
# Downloads pre-built FFmpeg libraries for Android arm64-v8a
# These are required for the Rust engine to cross-compile with ffmpeg-next
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FFMPEG_DIR="$PROJECT_ROOT/engine/ffmpeg-android"

echo "============================================================"
echo "  EDITORS-PRO — FFmpeg Android Setup"
echo "============================================================"
echo ""

# Create output directory
mkdir -p "$FFMPEG_DIR"
cd "$FFMPEG_DIR"

echo "This script downloads pre-built FFmpeg for Android arm64-v8a."
echo ""

# Check for required tools
command -v curl >/dev/null 2>&1 || { echo "ERROR: curl is required"; exit 1; }
command -v unzip >/dev/null 2>&1 || { echo "ERROR: unzip is required"; exit 1; }

if [ -f "$FFMPEG_DIR/lib/libavcodec.so" ] || [ -f "$FFMPEG_DIR/lib/libavcodec.a" ]; then
    echo "FFmpeg Android libraries already exist at: $FFMPEG_DIR"
    echo "Delete that directory and re-run to re-download."
    exit 0
fi

# ---------------------------------------------------------------------------
# Download pre-built FFmpeg from ffmpeg-kit
# ---------------------------------------------------------------------------
FFMPEG_KIT_VERSION="6.0-2"
FFMPEG_KIT_URL="https://github.com/nicehash/ffmpeg-kit/releases/download/v${FFMPEG_KIT_VERSION}/ffmpeg-kit-full-gpl-${FFMPEG_KIT_VERSION}-android-arm64-v8a.zip"

TEMP_ZIP="$FFMPEG_DIR/ffmpeg-android.zip"

echo "Downloading FFmpeg Android pre-built libraries..."
echo "URL: $FFMPEG_KIT_URL"
echo ""

curl -L --fail --progress-bar -o "$TEMP_ZIP" "$FFMPEG_KIT_URL" || {
    echo ""
    echo "ERROR: Could not download pre-built FFmpeg."
    echo ""
    echo "Please manually download FFmpeg for Android arm64-v8a:"
    echo "  Recommended: https://github.com/nicehash/ffmpeg-kit/releases"
    echo "  Alternative: Build from source with https://github.com/nicehash/FFmpeg-Android"
    echo ""
    echo "Place files as:"
    echo "  .so/.a files -> $FFMPEG_DIR/lib/"
    echo "  .h files     -> $FFMPEG_DIR/include/"
    exit 1
}

echo ""
echo "Extracting..."
unzip -o -q "$TEMP_ZIP" -d "$FFMPEG_DIR/extracted" || true

# Find and copy the libraries
FOUND_LIB=0
for dir in $(find "$FFMPEG_DIR/extracted" -type d -name "arm64-v8a" 2>/dev/null); do
    if [ -d "$dir/lib" ]; then
        mkdir -p "$FFMPEG_DIR/lib" "$FFMPEG_DIR/include"
        cp -r "$dir/lib/"*.so "$FFMPEG_DIR/lib/" 2>/dev/null || true
        cp -r "$dir/lib/"*.a "$FFMPEG_DIR/lib/" 2>/dev/null || true
        cp -r "$dir/include/"* "$FFMPEG_DIR/include/" 2>/dev/null || true
        FOUND_LIB=1
        break
    fi
done

if [ "$FOUND_LIB" -eq 0 ]; then
    for dir in $(find "$FFMPEG_DIR/extracted" -type d -name "lib" 2>/dev/null | head -5); do
        if ls "$dir"/libavcodec.* 1>/dev/null 2>&1; then
            mkdir -p "$FFMPEG_DIR/lib"
            cp -r "$dir/"*.so "$FFMPEG_DIR/lib/" 2>/dev/null || true
            cp -r "$dir/"*.a "$FFMPEG_DIR/lib/" 2>/dev/null || true
            FOUND_LIB=1
        fi
    done
    for dir in $(find "$FFMPEG_DIR/extracted" -type d -name "include" 2>/dev/null | head -5); do
        if ls "$dir"/libavcodec 1>/dev/null 2>&1; then
            mkdir -p "$FFMPEG_DIR/include"
            cp -r "$dir/"* "$FFMPEG_DIR/include/" 2>/dev/null || true
            FOUND_LIB=1
        fi
    done
fi

rm -rf "$FFMPEG_DIR/extracted" "$TEMP_ZIP"

if [ "$FOUND_LIB" -eq 0 ]; then
    echo "WARNING: Could not auto-extract FFmpeg. Manual setup required."
    echo "Place .so files in: $FFMPEG_DIR/lib/"
    echo "Place .h files in: $FFMPEG_DIR/include/"
    exit 1
fi

echo "✓ FFmpeg Android libraries ready"
echo ""

# Copy .so to jniLibs for Flutter
JNI_DIR="$PROJECT_ROOT/android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$JNI_DIR"
cp "$FFMPEG_DIR/lib/"*.so "$JNI_DIR/" 2>/dev/null || true
echo "✓ Copied .so to: $JNI_DIR"

# Create pkg-config files
PC_DIR="$FFMPEG_DIR/lib/pkgconfig"
mkdir -p "$PC_DIR"

for lib in libavcodec libavformat libavutil libswscale libswresample libavfilter; do
    cat > "$PC_DIR/${lib}.pc" << PCFILE
prefix=${FFMPEG_DIR}
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: ${lib}
Description: FFmpeg ${lib}
Version: 7.1.0
Libs: -L\${libdir} -l${lib#lib}
Cflags: -I\${includedir}
PCFILE
done

echo "✓ Created pkg-config files"
echo ""
echo "============================================================"
echo "  FFmpeg Android setup COMPLETE!"
echo ""
echo "  Build with FFmpeg:"
echo "    cd engine && cargo ndk -t arm64-v8a build --release"
echo ""
echo "  Build without FFmpeg (dev mode):"
echo "    cd engine && cargo ndk -t arm64-v8a build --release --no-default-features"
echo "============================================================"
