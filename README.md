# EDITORS-PRO

Professional-grade mobile video editor built with **Flutter + Rust** for Android.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    EDITORS-PRO STACK                      │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  🎨 UI LAYER — Flutter 3.x (Dart)                       │
│  ├── State Management: Riverpod 2.x                     │
│  ├── Navigation: GoRouter                               │
│  ├── Timeline Widget: CustomPainter + Canvas             │
│  ├── Preview: Texture widget (platform view)             │
│  ├── Performance Overlay: Real-time FPS/memory/GPU      │
│  └── Dark Theme: Professional editing UI                 │
│                                                          │
│  🔗 BRIDGE — flutter_rust_bridge v2                     │
│  ├── Zero-copy data transfer                            │
│  ├── Async message passing                              │
│  ├── Stream support for progress events                 │
│  └── 60+ API methods exposed to Flutter                 │
│                                                          │
│  ⚙️ NATIVE ENGINE — Rust                                │
│  ├── Video I/O: ffmpeg-next (FFmpeg 7.x bindings)       │
│  ├── GPU Compute: wgpu (Vulkan on Android)              │
│  ├── Effects Pipeline: Custom shader system (10 WGSL)   │
│  ├── Timeline Engine: Custom (frame-accurate seek)       │
│  ├── Audio Engine: Multi-track mixer, ducking, waveform  │
│  ├── Project Format: .epp (ZIP + JSON + CRC32)          │
│  ├── Export: FFmpeg encoder (H.264/H.265/VP9)           │
│  ├── Buffer Pool: Zero-allocation frame processing      │
│  ├── LRU Cache: O(1) eviction with hit/miss stats       │
│  ├── Priority Scheduler: Critical/Normal/Background     │
│  ├── Profiler: Span-based timing, FrameTimer, throughput │
│  └── Error Handling: Structured errors with recovery     │
│                                                          │
│  🗄️ DATA LAYER                                          │
│  ├── Local DB: SQLite (via drift in Flutter)             │
│  ├── Project Files: .epp format (zipped JSON + CRC32)   │
│  ├── Storage: SAF + MediaStore (Android 13+)             │
│  └── Cloud Sync: Conflict-free sync protocol             │
│                                                          │
│  📱 ANDROID INTEGRATION                                  │
│  ├── SAF: Storage Access Framework for content URIs      │
│  ├── MediaStore: Gallery-visible exports                 │
│  ├── ExportService: Foreground service + notifications   │
│  ├── AudioTrack: Low-latency PCM playback               │
│  └── Permissions: Android 13+ scoped storage            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Stats

| Metric | Count |
|--------|-------|
| Rust source files | 98 |
| WGSL GPU shaders | 10 |
| Dart source files | 72 |
| Total lines of code | ~75,000 |
| Rust unit/integration tests | ~270 |
| Flutter widget/service tests | ~30 |
| Criterion benchmark groups | 11 |
| Bridge API methods | 60+ |
| Effect types | 13+ |

## Project Structure

```
EDITORS-PRO/
├── lib/                            # Flutter application
│   ├── main.dart                   # Entry point
│   ├── app.dart                    # Root widget + router
│   ├── core/
│   │   ├── theme/                  # Dark theme, colors, typography
│   │   ├── constants/              # App-wide constants
│   │   ├── services/               # Engine, export, audio, storage, profiling
│   │   └── extensions/             # Dart extensions
│   ├── features/                   # Feature-first architecture
│   │   ├── editor/                 # Timeline, preview, effects, text, audio
│   │   ├── export/                 # Export flow with progress
│   │   ├── projects/               # Project management
│   │   ├── cloud/                  # Cloud sync UI
│   │   ├── templates/              # Template browser
│   │   ├── settings/               # User preferences
│   │   └── onboarding/             # First-run experience
│   ├── data/                       # Drift database, models
│   └── src/rust/                   # Generated bridge code
├── engine/                         # Rust native engine
│   ├── src/
│   │   ├── api/                    # Bridge API (60+ methods)
│   │   ├── timeline/               # Timeline, clips, tracks, commands
│   │   ├── decoder/                # HW/SW video decoding
│   │   ├── renderer/               # GPU/CPU compositing
│   │   ├── effects/                # 13+ effect types
│   │   ├── audio/                  # Mixer, waveform, ducking, transcription
│   │   ├── export_engine/          # FFmpeg encoding pipeline
│   │   ├── project/                # .epp format, keyframes, settings
│   │   ├── codec/                  # Hardware encoder (MediaCodec)
│   │   ├── storage/                # Cache, LRU, project store, proxy
│   │   ├── system/                 # Memory, profiler, buffer pool, zero-copy
│   │   ├── pipeline/               # Preview and render pipelines
│   │   ├── proxy/                  # Proxy generation
│   │   ├── cloud/                  # Sync manager, conflict resolution
│   │   ├── subtitle/               # SRT/VTT parser
│   │   ├── template/               # Template builder
│   │   ├── analysis/               # Loudness, waveform
│   │   └── utils/                  # Math, async ops, priority scheduler
│   └── benches/                    # Criterion benchmarks
├── android/                        # Android integration
│   └── app/src/main/kotlin/
│       ├── MainActivity.kt         # Platform channels (export, audio, storage)
│       ├── ExportService.kt        # Foreground service + notifications
│       └── StorageIntegration.kt   # SAF + MediaStore
├── .github/workflows/ci.yml       # CI/CD pipeline
└── docs/                           # Architecture documentation
```

## Features

### Core Editing
- Multi-track timeline with drag-to-reorder
- Frame-accurate seeking and trimming
- Split, move, and delete clips
- Full undo/redo via Command pattern
- Speed curves with easing (linear, ease-in/out, bezier)
- Keyframe animation (position, scale, rotation, opacity)

### Effects & Filters
- 13+ built-in effects (brightness, contrast, saturation, blur, etc.)
- 10 GPU compute shaders (WGSL) for real-time processing
- Chroma key (green screen) with eyedropper
- Blend modes (Normal, Multiply, Screen, Overlay, SoftLight, ColorDodge)
- Color space management (sRGB, Display P3, Rec. 2020, Rec. 709)
- Film grain overlay with stock presets
- Noise reduction with sigma estimation
- Lens correction with built-in profiles
- Masking (rectangle, ellipse, bezier)
- Compositing layers

### Audio
- Multi-track audio mixer with per-track volume/pan
- Audio ducking (auto-duck on voice)
- Real-time VU meters
- Waveform visualization
- Audio transcription (Whisper-compatible)

### Export
- H.264, H.265, VP9 codecs
- 720p to 4K output
- Two-pass encoding for quality
- AAC audio muxing
- Foreground service with progress notification
- MediaStore integration (gallery-visible)

### Professional Features
- Nested sequences
- Multi-camera editing
- Professional audio mixer
- Markers and regions
- Retime controls
- Effect presets
- Workspace layouts
- Auto-save and crash recovery

### Performance
- Zero-allocation buffer pool (500x allocation speedup)
- O(1) LRU cache with hit/miss tracking
- Priority task scheduler (Critical/Normal/Background)
- In-place pixel operations (no allocations in hot path)
- Double-buffered GPU readback
- Adaptive preview quality
- Memory pressure monitoring with auto-eviction

### Infrastructure
- 270+ Rust unit/integration tests
- 30+ Flutter widget/service tests
- 11 Criterion benchmark groups
- GitHub Actions CI/CD with cargo-ndk builds
- Code coverage (cargo-llvm-cov + Flutter coverage)
- Automated APK builds on push

## Getting Started

### Prerequisites

- Flutter 3.44+ (stable channel)
- Rust 1.96+ (stable)
- Android SDK with NDK 27+
- Java 17

### Setup

```bash
# 1. Clone the repository
git clone https://github.com/kesi-11/EDITORS-PRO.git
cd EDITORS-PRO

# 2. Get Flutter dependencies
flutter pub get

# 3. Build Rust engine for Android
cd engine
cargo ndk -t arm64-v8a build --release
cd ..

# 4. Run the app
flutter run
```

### Running Tests

```bash
# Rust tests
cd engine
cargo test

# Rust benchmarks
cargo bench

# Flutter tests
cd ..
flutter test
flutter test --coverage
```

### Building for Release

```bash
# Build Rust engine
cd engine
cargo ndk -t arm64-v8a build --release
cp target/aarch64-linux-android/release/libeditors_pro_engine.so \
   ../android/app/src/main/jniLibs/arm64-v8a/
cd ..

# Build APK
flutter build apk --release
```

## Development Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 1-2 | Flutter + Rust scaffold, bridge integration | ✅ |
| 3 | Export pipeline with FFmpeg | ✅ |
| 4 | Audio & multi-track + polish, storage | ✅ |
| 5 | Effects & filters | ✅ |
| 7 | Speed & keyframes | ✅ |
| 7-10 | GPU acceleration, advanced features | ✅ |
| 11 | Core engine additions | ✅ |
| 12 | S-Tier professional features | ✅ |
| 13 | Professional workflow features | ✅ |
| 14 | Bridge API for Phase 12-13 | ✅ |
| 15 | Testing & CI/CD | ✅ |
| 16 | Performance profiling & optimization | ✅ |
| 17 | Flutter-Rust bridge codegen | ✅ |
| 18 | Android integration (SAF, MediaStore) | ✅ |
| 19 | Error handling & crash reporting | ✅ |
| 20 | Polish & QA | ✅ |

## License

Private project — All rights reserved.
