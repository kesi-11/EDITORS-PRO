# EDITORS-PRO

Professional mobile video editor built with **Flutter + Rust** for Android.

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
│  └── Animations: flutter_animate                         │
│                                                          │
│  🔗 BRIDGE — flutter_rust_bridge v2                     │
│  ├── Zero-copy data transfer                            │
│  ├── Async message passing                              │
│  └── Stream support for progress events                 │
│                                                          │
│  ⚙️ NATIVE ENGINE — Rust                                │
│  ├── Video I/O: ffmpeg-next (FFmpeg 6.x bindings)       │
│  ├── GPU Compute: wgpu (Vulkan on Android)              │
│  ├── Effects Pipeline: Custom shader system (WGSL)       │
│  ├── Timeline Engine: Custom (frame-accurate seek)       │
│  ├── Audio Engine: cpal + rubery (playback/processing)   │
│  ├── Project Format: Custom .epp format (zipped JSON)    │
│  └── Export: FFmpeg encoder (H.264/H.265/VP9)           │
│                                                          │
│  🗄️ DATA LAYER                                          │
│  ├── Local DB: SQLite (via drift in Flutter)             │
│  ├── Project Files: Custom .epp format (zipped JSON)     │
│  └── Assets: Local filesystem + content provider         │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Project Structure

```
EDITORS-PRO/
├── app/ (Flutter application root)
│   ├── lib/
│   │   ├── main.dart              # Entry point
│   │   ├── app.dart               # Root widget + router
│   │   ├── core/
│   │   │   ├── theme/             # Dark theme, colors, typography
│   │   │   ├── constants/         # App-wide constants
│   │   │   └── extensions/        # Dart extensions
│   │   ├── features/              # Feature-first architecture
│   │   │   ├── timeline/          # Timeline UI + logic
│   │   │   ├── editor/            # Preview + controls
│   │   │   ├── media/             # Import + library
│   │   │   ├── effects/           # Effects panel
│   │   │   ├── audio/             # Audio controls
│   │   │   ├── text/              # Text overlay system
│   │   │   ├── export/            # Export flow
│   │   │   └── projects/          # Project management
│   │   ├── data/
│   │   │   ├── models/            # Data models (freezed)
│   │   │   └── repositories/      # Data access layer
│   │   └── services/
│   │       ├── rust_bridge/       # Generated bridge code
│   │       ├── platform/          # Platform channels
│   │       └── permissions/       # Permission handling
│   ├── android/                   # Android-specific config
│   └── pubspec.yaml               # Flutter dependencies
├── engine/                         # Rust native engine
│   ├── src/
│   │   ├── lib.rs                 # Engine entry + init
│   │   ├── api/                   # Public API for Flutter
│   │   ├── timeline/              # Timeline data model
│   │   │   ├── track.rs           # Track model
│   │   │   ├── clip.rs            # Clip model
│   │   │   └── command.rs         # Undo/redo system
│   │   ├── decoder/               # Video/audio decoding
│   │   │   ├── hardware.rs        # HW-accelerated (MediaCodec)
│   │   │   └── software.rs        # SW fallback (FFmpeg)
│   │   ├── renderer/              # Frame composition
│   │   │   ├── gpu.rs             # wgpu GPU renderer
│   │   │   └── shader.rs          # WGSL shaders + CPU fallback
│   │   ├── effects/               # Visual effects
│   │   │   ├── filters.rs         # Color filters
│   │   │   ├── transitions.rs     # Clip transitions
│   │   │   └── text_render.rs     # Text overlays
│   │   ├── audio/                 # Audio processing
│   │   │   ├── mixer.rs           # Audio mixing
│   │   │   └── waveform.rs        # Waveform generation
│   │   ├── export_engine/         # Export pipeline
│   │   └── project/               # Project serialization
│   │       └── format.rs          # .epp file format
│   └── Cargo.toml                 # Rust dependencies
└── docs/                           # Architecture docs
```

## Getting Started

### Prerequisites

- Flutter 3.44+ (stable channel)
- Rust 1.96+ (stable)
- Android SDK with NDK 27+
- Java 17

### Setup

```bash
# 1. Install Flutter
git clone https://github.com/flutter/flutter.git -b stable
export PATH="$HOME/flutter/bin:$PATH"

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 3. Add Android targets for Rust
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk

# 4. Set Android NDK path
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973

# 5. Get Flutter dependencies
cd editors-pro
flutter pub get

# 6. Build Rust engine for Android
cd engine
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build --release

# 7. Run the app
cd ..
flutter run
```

## Development Roadmap

### Phase 1: Skeleton MVP (Current)
- [x] Project scaffold and architecture
- [x] Rust engine core modules
- [x] Flutter UI screens (home, editor, export)
- [x] Timeline widget with tracks and clips
- [ ] Bridge connection (flutter_rust_bridge)
- [ ] Import video + display first frame
- [ ] Basic trim + export

### Phase 2: Timeline + Layers
- [ ] Custom timeline widget (drag-to-reorder)
- [ ] Audio waveform display
- [ ] Text overlay with animations
- [ ] Multi-track preview rendering

### Phase 3: Effects + Speed
- [ ] GPU shader pipeline (wgpu)
- [ ] Filter effects (brightness, contrast, etc.)
- [ ] Speed curves (ease in/out)
- [ ] Transitions between clips

### Phase 4: Pro Features
- [ ] Keyframe animation system
- [ ] Chroma key (green screen)
- [ ] Auto captions (Whisper)
- [ ] Advanced export (4K, ProRes)

### Phase 5: Cloud + Scale
- [ ] Cloud project sync
- [ ] Template marketplace
- [ ] Multi-device editing

## Engine Architecture

### Timeline System
The timeline is the central data model. It contains:
- **Tracks**: Vertical lanes (video, audio, text, effect)
- **Clips**: Horizontal segments on tracks with timing, trim, speed
- **Commands**: Full undo/redo via Command pattern

### Rendering Pipeline
1. For each visible frame, the renderer:
   - Decodes the video frame at the current timestamp (HW accel first)
   - Applies effects pipeline (filters, transitions)
   - Composites text overlays
   - Returns RGBA frame data to Flutter

### Export Pipeline
1. Iterate through all frames at the target FPS
2. Render each frame through the full pipeline
3. Encode with FFmpeg (H.264/H.265/VP9)
4. Write to output file with progress reporting

## API Surface (Rust → Flutter)

| Method | Description |
|--------|-------------|
| `create_project` | Create a new editing project |
| `import_media` | Import video/audio/image file |
| `add_track` | Add a timeline track |
| `add_clip` | Place a clip on a track |
| `trim_clip` | Trim clip from start/end |
| `split_clip` | Split clip at timestamp |
| `move_clip` | Move clip position |
| `remove_clip` | Delete a clip |
| `get_frame` | Render single frame for preview |
| `export_video` | Export final video with settings |
| `undo/redo` | Undo/redo last action |
| `save_project` | Persist project to .epp file |
| `load_project` | Load project from .epp file |

## License

Private project — All rights reserved.
