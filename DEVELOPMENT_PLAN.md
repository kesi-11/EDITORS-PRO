# EDITORS-PRO — Comprehensive Phased Development Plan

**Project:** EDITORS-PRO — Professional Mobile Video Editor  
**Platform:** Android (Google Play Store)  
**Stack:** Flutter 3.x + Rust Engine + flutter_rust_bridge v2  
**Current State:** ~7,500 lines across 38+ files; scaffolded but not connected  
**MVP Goal:** Import video → Display first frame → Basic trim → Export trimmed clip  
**Document Version:** 1.0  
**Date:** 2026-03-04  

---

## Table of Contents

1. [Technology Stack Details](#1-technology-stack-details)
2. [Build System Setup](#2-build-system-setup)
3. [flutter_rust_bridge Integration Guide](#3-flutter_rust_bridge-integration-guide)
4. [Phase 0: Foundation Fixes](#4-phase-0-foundation-fixes)
5. [Phase 1: MVP — Import & Preview](#5-phase-1-mvp--import--preview)
6. [Phase 2: Timeline & Trim](#6-phase-2-timeline--trim)
7. [Phase 3: Export Pipeline](#7-phase-3-export-pipeline)
8. [Phase 4: Audio & Multi-track](#8-phase-4-audio--multi-track)
9. [Phase 5: Effects & Filters](#9-phase-5-effects--filters)
10. [Phase 6: Text & Overlays](#10-phase-6-text--overlays)
11. [Phase 7: Speed & Keyframes](#11-phase-7-speed--keyframes)
12. [Phase 8: GPU Acceleration](#12-phase-8-gpu-acceleration)
13. [Phase 9: Polish & Play Store](#13-phase-9-polish--play-store)
14. [Phase 10: Advanced Features](#14-phase-10-advanced-features)
15. [Testing Strategy](#15-testing-strategy)
16. [Google Play Store Requirements Checklist](#16-google-play-store-requirements-checklist)
17. [Performance Targets](#17-performance-targets)
18. [Risk Register](#18-risk-register)
19. [Architecture Decision Records](#19-architecture-decision-records)

---

## 1. Technology Stack Details

### Flutter Layer

| Dependency | Version | Purpose | Why This Choice |
|-----------|---------|---------|-----------------|
| Flutter SDK | 3.12.x | UI framework | Industry standard for cross-platform; excellent Android support; Skia rendering engine |
| Dart SDK | ^3.12.2 | Language | Required by Flutter; null-safe; strong typing |
| flutter_riverpod | ^2.6.1 | State management | Compile-time safety; testable; async support; no BuildContext dependency |
| go_router | ^14.8.1 | Navigation | Declarative routing; deep link support; type-safe |
| flutter_rust_bridge | ^2.9.0 | Rust FFI | Zero-copy data transfer; stream support; auto-generated bindings; v2 is production-ready |
| file_picker | ^9.2.1 | Media import | Cross-platform file selection; Android SAF support |
| path_provider | ^2.1.5 | Path resolution | App documents/cache directories |
| permission_handler | ^11.4.0 | Permissions | Runtime permission requests for Android 13+ |
| share_plus | ^10.1.4 | Share intent | Android share sheet for exported videos |
| drift | ^2.22.1 | SQLite ORM | Project metadata storage; deferred to Phase 2+ |
| uuid | ^4.5.1 | ID generation | Unique identifiers for clips, tracks, projects |
| intl | ^0.20.2 | i18n formatting | Time formatting; number formatting |
| shimmer | ^3.0.0 | Loading indicators | Skeleton loading states |

### Rust Engine

| Dependency | Version | Purpose | Why This Choice |
|-----------|---------|---------|-----------------|
| ffmpeg-next | 7.1 | Video decode/encode | FFmpeg 6.x bindings; industry standard codec support; hardware acceleration |
| serde + serde_json | 1.0 | Serialization | De facto Rust serialization; JSON for .epp compatibility |
| thiserror | 1.0 | Error types | Ergonomic error definitions; no runtime overhead |
| anyhow | 1.0 | Error propagation | Flexible error handling for internal chains |
| rayon | 1.10 | Parallelism | Data-parallel pixel processing; work-stealing thread pool |
| image | 0.25 | Image processing | Thumbnail generation; bilinear resize; format support |
| zip | 2.2 | .epp format | Compressed project files; standard zip compatibility |
| uuid | 1.0 | ID generation | Consistent with Flutter layer |
| chrono | 0.4 | Timestamps | Date/time handling for project metadata |
| bytemuck | 1.16 | Byte casting | GPU interop; zero-cost byte reinterpretation |
| log + env_logger | 0.4/0.11 | Logging | Unified logging; filtered by level |
| ndk-sys | 0.6 | Android NDK | MediaCodec hardware decoder access |
| jni | 0.21 | JNI bindings | Java Native Interface for Android integration |

### Android Build

| Component | Version | Purpose |
|-----------|---------|---------|
| compileSdk | 35 | Android 15 API |
| targetSdk | 35 | Required for Play Store (Aug 2025+) |
| minSdk | 24 | Android 7.0 — MediaCodec API baseline |
| NDK | 27.0.12077973 | Native compilation for arm64-v8a, armeabi-v7a, x86_64 |
| Kotlin | JVM 17 target | Android interop |
| Gradle | KTS | Type-safe build scripts |
| ExoPlayer | 2.19.1 | Fallback video playback (will be replaced by engine) |

### Why Rust for the Engine

1. **Performance:** Zero-cost abstractions; no GC pauses during frame processing
2. **Memory safety:** No buffer overflows in video parsing (security-critical)
3. **Concurrency:** Safe parallelism via `rayon` for pixel operations
4. **FFmpeg interop:** Direct C FFI without JNI overhead
5. **Cross-compile:** `cargo-ndk` produces optimized ARM binaries
6. **Binary size:** `strip = true` + LTO produces small .so files (~3-5MB)

### Why flutter_rust_bridge v2 (not FFI or dart:ffi)

1. **Auto-generated bindings** — no hand-written FFI code to maintain
2. **Stream support** — essential for export progress and frame delivery
3. **Zero-copy** — `Uint8List` ↔ `Vec<u8>` without copies via `ZeroCopyBuffer`
4. **Error propagation** — Rust `Result<T, E>` maps to Dart exceptions automatically
5. **Async support** — Rust `async` functions map to Dart `Future`
6. **Type safety** — generates typed Dart API from Rust signatures

---

## 2. Build System Setup

### Prerequisites

```
# System requirements
- Ubuntu 22.04+ or macOS 13+
- Android Studio 2024.1+
- Android SDK 35
- Android NDK 27.0.12077973
- Rust toolchain: stable (1.75+)
- Flutter SDK 3.12+
- cargo-ndk: cargo install cargo-ndk
- flutter_rust_bridge_codegen: cargo install flutter_rust_bridge_codegen
- LLVM/Clang 17+ (required by flutter_rust_bridge)
```

### Step-by-Step: From Zero to Running App

```bash
# 1. Clone the repository
git clone <repo-url> editors-pro
cd editors-pro

# 2. Install Rust targets for Android
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# 3. Install cargo-ndk
cargo install cargo-ndk

# 4. Install flutter_rust_bridge codegen
cargo install flutter_rust_bridge_codegen --version 2.9.0

# 5. Set Android NDK path
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/27.0.12077973

# 6. Build the Rust engine for Android
cd engine
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ../android/app/src/main/jniLibs build --release
cd ..

# 7. Get Flutter dependencies
flutter pub get

# 8. Generate bridge code (after API is defined — Phase 1)
flutter_rust_bridge_codegen generate

# 9. Run on connected Android device
flutter run

# 10. Build release APK
flutter build apk --release
```

### CI/CD Pipeline Setup (Phase 0)

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  rust-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd engine && cargo fmt --check
      - run: cd engine && cargo clippy -- -D warnings
      - run: cd engine && cargo test

  flutter-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with: { flutter-version: '3.12.x' }
      - run: flutter pub get
      - run: dart format --set-exit-if-changed .
      - run: dart analyze --fatal-infos
      - run: flutter test

  build-android:
    runs-on: ubuntu-latest
    needs: [rust-check, flutter-check]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-ndk
      - run: cd engine && cargo ndk -t arm64-v8a build --release
      - uses: subosito/flutter-action@v2
      - run: flutter pub get
      - run: flutter build apk --release
```

---

## 3. flutter_rust_bridge Integration Guide

### Architecture Overview

```
┌─────────────────────────────────────────────────┐
│ Flutter (Dart)                                  │
│                                                  │
│  lib/src/rust/                                   │
│  ├── api/                                        │
│  │   └── editors_pro_engine.dart  ← generated   │
│  ├── frb_generated.dart           ← generated   │
│  └── frb_generated.io.dart        ← generated   │
│                                                  │
│  Usage:                                          │
│  final api = EditorsProEngine();                 │
│  await api.initialize();                         │
│  final frame = await api.getFrame(timeMs: 0);   │
└──────────────────────┬──────────────────────────┘
                       │ FFI (zero-copy)
┌──────────────────────┴──────────────────────────┐
│ Rust Engine                                      │
│                                                  │
│  engine/src/api/                                 │
│  └── mod.rs  ← EditorsProEngine struct           │
│                                                  │
│  Generated:                                      │
│  engine/src/generated.rs                         │
│  engine/src/generated.io.rs                      │
└──────────────────────────────────────────────────┘
```

### Step-by-Step Integration (Phase 1)

#### Step 1: Create the Rust API file

The flutter_rust_bridge codegen needs a single "entry point" Rust file that defines the public API. Create `engine/src/api/bridge_api.rs`:

```rust
// engine/src/api/bridge_api.rs
// This file defines the public API that flutter_rust_bridge will expose to Dart.
// All types must be Serializable; all functions must take &self or be free functions.

use crate::api::{ClipInfo, MediaAssetInfo, ProjectInfo, TrackInfo};
use crate::export_engine::ExportSettings;
use crate::timeline::track::TrackType;

/// The main engine API — one instance per app session
pub struct EditorsProEngineApi {
    // Use interior mutability to allow &self methods (bridge requirement)
    inner: std::sync::Mutex<crate::api::EditorsProEngine>,
}

impl EditorsProEngineApi {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(crate::api::EditorsProEngine::new()),
        }
    }

    pub fn initialize(&self) -> Result<(), String> {
        self.inner.lock().unwrap().initialize()
    }

    pub fn create_project(&self, name: String, width: Option<u32>, height: Option<u32>, fps: Option<f32>) -> Result<ProjectInfo, String> {
        let settings = match (width, height, fps) {
            (Some(w), Some(h), Some(f)) => Some(crate::project::ProjectSettings {
                width: w, height: h, fps: f,
                ..Default::default()
            }),
            _ => None,
        };
        self.inner.lock().unwrap().create_project(&name, settings)
    }

    pub fn import_media(&self, file_path: String) -> Result<MediaAssetInfo, String> {
        self.inner.lock().unwrap().import_media(&file_path)
    }

    pub fn add_clip(&self, track_id: String, asset_id: String, start_ms: u64, duration_ms: u64) -> Result<ClipInfo, String> {
        self.inner.lock().unwrap().add_clip(&track_id, &asset_id, start_ms, duration_ms)
    }

    pub fn trim_clip(&self, clip_id: String, trim_start_ms: u64, trim_end_ms: u64) -> Result<(), String> {
        self.inner.lock().unwrap().trim_clip(&clip_id, trim_start_ms, trim_end_ms)
    }

    pub fn split_clip(&self, clip_id: String, time_ms: u64) -> Result<(), String> {
        self.inner.lock().unwrap().split_clip(&clip_id, time_ms)?;
        Ok(())
    }

    pub fn remove_clip(&self, clip_id: String) -> Result<(), String> {
        self.inner.lock().unwrap().remove_clip(&clip_id)
    }

    pub fn get_frame(&self, time_ms: u64) -> Result<Vec<u8>, String> {
        self.inner.lock().unwrap().get_frame(time_ms)
    }

    pub fn undo(&self) -> Result<(), String> {
        self.inner.lock().unwrap().undo()
    }

    pub fn redo(&self) -> Result<(), String> {
        self.inner.lock().unwrap().redo()
    }

    pub fn save_project(&self, path: String) -> Result<(), String> {
        self.inner.lock().unwrap().save_project(&path)
    }

    pub fn load_project(&self, path: String) -> Result<ProjectInfo, String> {
        self.inner.lock().unwrap().load_project(&path)
    }

    pub fn get_project_info(&self) -> Option<ProjectInfo> {
        self.inner.lock().unwrap().get_project_info()
    }

    pub fn get_timeline_duration(&self) -> u64 {
        self.inner.lock().unwrap().get_timeline_duration()
    }
}
```

#### Step 2: Run the code generator

```bash
# Install codegen if not installed
cargo install flutter_rust_bridge_codegen --version 2.9.0

# Generate bindings
flutter_rust_bridge_codegen generate \
  --rust-input engine/src/api/bridge_api.rs \
  --dart-output lib/src/rust/
```

#### Step 3: Generated file structure

```
lib/src/rust/
├── api/
│   └── editors_pro_engine_api.dart   # Generated Dart class matching Rust API
├── frb_generated.dart                # Core bridge infrastructure
├── frb_generated.io.dart             # I/O utilities
└── frb_generated.web.dart            # (Not needed for Android-only)

engine/src/
├── generated.rs                      # Generated Rust bridge code
└── generated.io.rs                   # Generated I/O code
```

#### Step 4: Initialize the bridge in Flutter

```dart
// lib/core/services/engine_service.dart
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:editors_pro/src/rust/api/editors_pro_engine_api.dart';
import 'package:editors_pro/src/rust/frb_generated.dart';

class EngineService {
  static EditorsProEngineApi? _api;

  static Future<EditorsProEngineApi> get instance async {
    if (_api != null) return _api!;
    await RustLib.init();
    _api = EditorsProEngineApi();
    await _api!.initialize();
    return _api!;
  }

  static void dispose() {
    _api = null;
  }
}
```

#### Step 5: Use in Riverpod providers

```dart
// lib/features/editor/providers/engine_bridge_provider.dart
final engineApiProvider = FutureProvider<EditorsProEngineApi>((ref) async {
  return EngineService.instance;
});
```

### Stream Support for Progress

For export progress, use flutter_rust_bridge's `StreamSink`:

```rust
// In bridge_api.rs
pub fn export_video_with_progress(
    &self,
    output_path: String,
    settings: ExportSettings,
    progress_sink: flutter_rust_bridge::StreamSink<ExportProgress>,
) -> Result<(), String> {
    let engine = self.inner.lock().unwrap();
    // Export loop sends progress via sink
    // progress_sink.add(ExportProgress { ... });
    Ok(())
}
```

```dart
// In Flutter
final progressStream = api.exportVideoWithProgress(
    outputPath: path,
    settings: settings,
);
await for (final progress in progressStream) {
    ref.read(editorProvider.notifier).setExporting(true, progress: progress.progress);
}
```

### Key API Patterns

| Pattern | Rust Signature | Dart Usage |
|---------|---------------|------------|
| Synchronous call | `pub fn get_timeline_duration(&self) -> u64` | `final dur = api.getTimelineDuration();` |
| Async call | `pub async fn get_frame(&self, time_ms: u64) -> Result<Vec<u8>, String>` | `final frame = await api.getFrame(timeMs: 0);` |
| Stream | `pub fn export_video(&self, ..., sink: StreamSink<Progress>) -> Result<()>` | `await for (final p in stream) { ... }` |
| Optional params | `pub fn create_project(&self, name: String, fps: Option<f32>)` | `api.createProject(name: "Test", fps: 60.0);` |
| Enum | `pub enum TrackType { Video, Audio, Text, Effect }` | `TrackType.video` |
| Binary data | `Result<Vec<u8>, String>` | `Uint8List` (zero-copy) |

---

## 4. Phase 0: Foundation Fixes

**Duration:** 1-2 weeks  
**Goal:** All critical audit issues resolved; app builds and runs cleanly; CI/CD operational; release signing configured

### Detailed Tasks

#### 0.1 — Fix .gitignore

| File | Change |
|------|--------|
| `.gitignore` | Add `engine/Cargo.lock` to version control (binary crates should lock); add `*.so`, `*.dll`, `*.dylib` for native libs; add `rust-toolchain.toml`; remove `*.lock` blanket rule (it excludes `Cargo.lock`) |
| `engine/.gitignore` | Create engine-specific gitignore: `/target`, `**/*.rs.bk` |

**Acceptance:** `git status` shows no tracked build artifacts; `Cargo.lock` is tracked

#### 0.2 — Fix Error Types (C13)

| File | Change |
|------|--------|
| `engine/src/lib.rs` | Ensure `EngineError` covers all variants |
| `engine/src/api/mod.rs` | Replace all `Result<_, String>` returns with `Result<_, EngineError>` or a new `BridgeError` |
| `engine/src/api/bridge_api.rs` (new) | Create typed error for bridge: `BridgeError` with `#[derive(Serialize, Deserialize)]` |

**New type:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeError {
    Engine(String),
    NotFound(String),
    InvalidInput(String),
    PermissionDenied(String),
}
```

**Acceptance:** No `Result<_, String>` in any Rust public API; all errors are typed

#### 0.3 — Remove Unsafe Send/Sync

| File | Change |
|------|--------|
| `engine/src/decoder/hardware.rs` | Verify `unsafe impl Send` has safety documentation; add `// SAFETY: HardwareDecoder is only accessed from the mutex-guarded API, ensuring single-threaded access` |
| `engine/src/decoder/software.rs` | Same documentation; verify no `impl Sync` exists |

**Acceptance:** All `unsafe` blocks have `// SAFETY:` comments; no `impl Sync` on decoder types

#### 0.4 — Fix SDK Version and Build Configuration

| File | Change |
|------|--------|
| `android/app/build.gradle.kts` | Change `signingConfig = signingConfigs.getByName("debug")` in release to proper release signing config |
| `android/app/build.gradle.kts` | Add `proguard-rules.pro` file content (see 0.6) |
| `android/app/build.gradle.kts` | Add `ndk { abiFilters += listOf("arm64-v8a") }` for initial release (drop x86 for Play Store) |
| `android/settings.gradle.kts` | Ensure NDK version is specified |

**Acceptance:** `./gradlew assembleRelease` builds successfully with release signing

#### 0.5 — Scoped Storage Migration

| File | Change |
|------|--------|
| `android/app/src/main/AndroidManifest.xml` | Remove `MANAGE_EXTERNAL_STORAGE` permission — Google Play rejects apps using this unless they are file managers |
| `android/app/src/main/AndroidManifest.xml` | Remove `tools:ignore="ScopedStorage"` |
| `android/app/src/main/AndroidManifest.xml` | Add `xmlns:tools="http://schemas.android.com/tools"` to `<manifest>` tag |
| `android/app/src/main/AndroidManifest.xml` | Remove `android:requestLegacyExternalStorage="true"` |
| `engine/src/api/mod.rs` | Update `import_media()` to accept SAF content URIs as well as file paths |
| `engine/src/project/mod.rs` | Update `MediaAsset.file_path` documentation to note that it can be a content URI |
| `lib/features/editor/presentation/editor_screen.dart` | Use `file_picker` which returns SAF URIs on Android 13+ |

**Acceptance:** No `MANAGE_EXTERNAL_STORAGE` in manifest; app reads media via `READ_MEDIA_VIDEO` / `READ_MEDIA_AUDIO` / `READ_MEDIA_IMAGES`

#### 0.6 — ProGuard Rules

Create `android/app/proguard-rules.pro`:

```proguard
# Rust engine native methods
-keep class com.editorspro.editors_pro.** { *; }

# FFmpeg
-keep class com.google.android.exoplayer2.** { *; }

# Flutter Rust Bridge
-keep class * extends java.lang.reflect.** { *; }
-keepclassmembers class * {
    native <methods>;
}

# Serde serialization
-keepclassmembers class **.serde.** {
    *;
}
```

**Acceptance:** Release build with minification succeeds and engine loads correctly

#### 0.7 — Release Signing Setup

| File | Change |
|------|--------|
| `android/app/build.gradle.kts` | Add release signing config using environment variables |
| `android/key.properties` (gitignored) | Store keystore path, passwords |

```kotlin
// In build.gradle.kts
val keystoreProperties = Properties()
val keystorePropertiesFile = rootProject.file("key.properties")
if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(keystorePropertiesFile.inputStream())
}

signingConfigs {
    create("release") {
        keyAlias = keystoreProperties["keyAlias"] as String?
        keyPassword = keystoreProperties["keyPassword"] as String?
        storeFile = keystoreProperties["storeFile"]?.let { file(it) }
        storePassword = keystoreProperties["storePassword"] as String?
    }
}

buildTypes {
    release {
        signingConfig = signingConfigs.getByName("release")
    }
}
```

**Acceptance:** `flutter build apk --release` produces a signed APK

#### 0.8 — Fix Remaining Audit Bugs

| Bug | File | Fix |
|-----|------|-----|
| B1: Decoder not switched between clips | `engine/src/api/mod.rs:get_frame()` | Track `current_file_path` in engine; re-open decoder when it changes |
| B2: Speed calculation overflow | `engine/src/timeline/clip.rs:source_duration_ms()` | Use `u64` arithmetic with `checked_mul` |
| B3: Timeline duration 0 | `engine/src/timeline/mod.rs:recalculate_duration()` | Return minimum 1ms; handle 0 in Flutter scrub bar |
| B4: Split at clip boundary | `engine/src/timeline/mod.rs:split_clip()` | Allow epsilon tolerance (±1ms) at boundaries |
| B5: Missing tools namespace | `android/app/src/main/AndroidManifest.xml` | Add `xmlns:tools` (done in 0.5) |
| B6: Playback timer never stops | `lib/features/editor/providers/editor_provider.dart:_startPlayback()` | Replace `Future.delayed` with `AnimationController` or `Ticker` with mounted check |

#### 0.9 — Remove Dead Dependencies

| Dependency | Location | Action |
|-----------|----------|--------|
| `parking_lot` | Not in current Cargo.toml (already removed) | Verify |
| `crossbeam-channel` | Not in current Cargo.toml (already removed) | Verify |
| `video_thumbnail` | `pubspec.yaml` | Remove — redundant with engine |
| `flutter_animate` | `pubspec.yaml` | Keep for Phase 6 text animations |
| `freezed_annotation` | `pubspec.yaml` | Remove if not using freezed |
| `json_annotation` | `pubspec.yaml` | Keep for bridge DTO serialization |
| `cbindgen` | `engine/Cargo.toml [build-dependencies]` | Remove — replaced by flutter_rust_bridge codegen |

**Acceptance:** `cargo tree` shows no unused dependencies; `dart pub deps` shows no unused deps

#### 0.10 — Responsive Layout Basics

| File | Change |
|------|--------|
| `lib/features/editor/presentation/editor_screen.dart` | Replace hardcoded `width: 240` with `MediaQuery`-based responsive width; use `LayoutBuilder` |
| `lib/features/editor/widgets/timeline_panel.dart` | Replace hardcoded `AppTheme.timelineMinHeight` with responsive calculation based on screen height |
| `lib/core/theme/app_theme.dart` | Add responsive breakpoints: `isTablet`, `isPhone` helpers |

**Acceptance:** Editor renders correctly on 5.5" phone (360dp width) and 10" tablet (800dp width)

### Rust Engine Tasks

- [ ] Fix `Result<_, String>` → `Result<_, EngineError>` in `api/mod.rs`
- [ ] Add `current_file_path` tracking to `EditorsProEngine` for decoder switching
- [ ] Fix `source_duration_ms()` overflow check in `clip.rs`
- [ ] Fix `recalculate_duration()` to return minimum 1ms
- [ ] Add epsilon tolerance to `split_clip()` boundary check
- [ ] Remove `cbindgen` from `build-dependencies`

### Flutter UI Tasks

- [ ] Replace `_startPlayback()` with `AnimationController` / `Ticker`
- [ ] Add responsive layout to editor screen
- [ ] Remove `video_thumbnail` from `pubspec.yaml`
- [ ] Add `mounted` check to all async callbacks

### Bridge Integration Tasks

- [ ] N/A (bridge not yet connected)

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_clip_duration_overflow` | Rust unit | Verify `source_duration_ms()` doesn't overflow with max values |
| `test_split_at_boundary` | Rust unit | Split at exact start/end with epsilon tolerance |
| `test_timeline_zero_duration` | Rust unit | Removing all clips doesn't cause division by zero |
| `test_decoder_switching` | Rust unit | Switching between two video files reopens decoder |
| `widget_test_responsive` | Flutter widget | Editor renders on small and large screens |

### Acceptance Criteria

- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `dart analyze --fatal-infos` passes with zero issues
- [ ] `flutter build apk --release` produces a signed APK
- [ ] No `MANAGE_EXTERNAL_STORAGE` in manifest
- [ ] All Rust tests pass (`cargo test`)
- [ ] App launches on Android device without crash
- [ ] CI/CD pipeline runs green on GitHub Actions

### Google Play Store Readiness Checklist (Phase 0)

- [x] targetSdkVersion 35
- [ ] Release signing configured with proper keystore
- [ ] ProGuard rules file exists and is correct
- [ ] No overly broad permissions (`MANAGE_EXTERNAL_STORAGE` removed)
- [ ] App does not crash on launch
- [ ] Version code and name set in `pubspec.yaml`

---

## 5. Phase 1: MVP — Import & Preview

**Duration:** 3-4 weeks  
**Goal:** User can import a video from gallery, see the first frame displayed in the preview viewport, and play/pause/seek through the video

### Detailed Tasks

#### 1.1 — Connect flutter_rust_bridge v2

| File | Task |
|------|------|
| `engine/src/api/bridge_api.rs` (new) | Create the bridge API wrapper using `Mutex<EditorsProEngine>` pattern (see Section 3) |
| `engine/src/lib.rs` | Add `pub mod api;` → ensure `bridge_api` is included |
| `engine/Cargo.toml` | Add `flutter_rust_bridge = "2.9"` as dependency |
| Run codegen | `flutter_rust_bridge_codegen generate` |
| `lib/src/rust/` (generated) | Verify generated Dart files exist |
| `lib/core/services/engine_service.dart` (new) | Create singleton service to initialize bridge |
| `lib/main.dart` | Call `RustLib.init()` before `runApp()` |

**Key decisions:**
- Use `Mutex<EditorsProEngine>` in the bridge API struct so all methods can take `&self` (required by flutter_rust_bridge v2)
- Do NOT use `RwLock` — the engine is always accessed sequentially from the Flutter UI thread

**Acceptance:** `RustLib.init()` completes without error; can call `api.initialize()` from Dart

#### 1.2 — Implement Video Import via File Picker

| File | Task |
|------|------|
| `lib/features/editor/providers/editor_provider.dart` | Add `importMedia()` method that uses `file_picker` to select a video, then calls `api.importMedia(filePath:)` |
| `lib/features/editor/presentation/editor_screen.dart:_importMedia()` | Replace stub with actual file picker call; handle SAF URIs by copying to app cache directory first |
| `lib/core/services/file_service.dart` (new) | Create service to: (1) Pick video file via `file_picker`, (2) Copy to app cache directory, (3) Return the local file path |
| `lib/core/services/permission_service.dart` (new) | Request `READ_MEDIA_VIDEO` permission before import |

**Android SAF URI handling:**
```dart
// When file_picker returns a content:// URI on Android 13+:
// 1. Open InputStream from content resolver
// 2. Copy to getApplicationCacheDirectory() + "/import/" + filename
// 3. Pass the local path to Rust engine
```

**Acceptance:** Tapping "Import" button → permission request → file picker → video appears in media library list with metadata (duration, resolution)

#### 1.3 — Decode and Display First Frame

| File | Task |
|------|------|
| `engine/src/decoder/hardware.rs` | Verify `open()`, `decode_frame_at()`, `get_video_info()` work correctly with test video |
| `engine/src/api/bridge_api.rs` | Ensure `get_frame()` is exposed and returns `Vec<u8>` RGBA data |
| `lib/features/editor/widgets/preview_viewport.dart` | Replace placeholder with `Image.memory()` that displays the RGBA frame data |
| `lib/features/editor/providers/editor_provider.dart` | Add `currentFrame` field (`Uint8List?`); add `loadFirstFrame()` that calls `api.getFrame(timeMs: 0)` and converts RGBA to RGB for `Image.memory` |
| `lib/core/services/frame_converter.dart` (new) | Convert RGBA `Vec<u8>` from Rust to a PNG/RGB format consumable by Flutter's `Image.memory()` |

**Critical implementation detail:** Flutter's `Image.memory()` expects a compressed format (PNG/JPEG), not raw RGBA. Options:
- **Option A (recommended for Phase 1):** Encode RGBA to PNG in Rust using `image` crate, return PNG bytes
- **Option B (later phase):** Use Flutter's `Texture` widget with `TextureRegistry` for zero-copy GPU texture sharing

```rust
// In bridge_api.rs
pub fn get_frame_png(&self, time_ms: u64, preview_width: u32, preview_height: u32) -> Result<Vec<u8>, String> {
    let rgba = self.inner.lock().unwrap().get_frame(time_ms)?;
    // Resize using image crate for preview
    let img = image::RgbaImage::from_raw(/* width */, /* height */, rgba)
        .ok_or("Invalid frame data")?;
    let resized = image::imageops::resize(&img, preview_width, preview_height, image::imageops::FilterType::Bilinear);
    let mut png_bytes = Vec::new();
    resized.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png_bytes)
}
```

**Acceptance:** Import video → first frame appears in preview viewport within 2 seconds

#### 1.4 — Basic Playback (Play/Pause/Seek)

| File | Task |
|------|------|
| `lib/features/editor/providers/editor_provider.dart` | Replace `_startPlayback()` `Future.delayed` loop with `AnimationController` that ticks at 30fps |
| `lib/features/editor/providers/editor_provider.dart` | On each tick, call `api.getFramePng(timeMs: currentTimeMs)` to get frame |
| `lib/features/editor/widgets/preview_viewport.dart` | Listen to frame updates and display them |
| `lib/features/editor/widgets/editor_toolbar.dart` | Wire play/pause button to `togglePlayback()` |
| `lib/features/editor/widgets/preview_viewport.dart` | Add seek bar (Slider) below the preview |

**Playback controller implementation:**
```dart
class PlaybackController extends ChangeNotifier {
  final Ticker _ticker;
  int _currentTimeMs = 0;
  int _durationMs = 0;
  bool _isPlaying = false;
  Duration _lastTick = Duration.zero;

  PlaybackController(TickerProvider vsync) : _ticker = vsync.createTicker(_onTick);

  void _onTick(Duration elapsed) {
    if (!_isPlaying) return;
    final delta = elapsed - _lastTick;
    _lastTick = elapsed;
    _currentTimeMs += delta.inMilliseconds;
    if (_currentTimeMs >= _durationMs) {
      _currentTimeMs = _durationMs;
      _isPlaying = false;
    }
    notifyListeners();
  }

  void play() { _isPlaying = true; _lastTick = _ticker.lastDuration; _ticker.start(); }
  void pause() { _isPlaying = false; _ticker.stop(); }
  void seekTo(int timeMs) { _currentTimeMs = timeMs.clamp(0, _durationMs); notifyListeners(); }
}
```

**Acceptance:** Press play → video plays at approximately 30fps; press pause → stops; drag seek bar → frame updates

#### 1.5 — Frame-by-Frame Scrubbing

| File | Task |
|------|------|
| `lib/features/editor/widgets/preview_viewport.dart` | Add frame step forward/backward buttons (←1 frame, →1 frame) |
| `lib/features/editor/widgets/preview_viewport.dart` | On scrub gesture, throttle frame requests to 15fps max to prevent UI jank |
| `engine/src/api/bridge_api.rs` | Add `step_frame(&self, direction: i32) -> Result<Vec<u8>, String>` that moves by `1000/fps` ms |

**Frame cache optimization (simple):**
```dart
// Cache the last 10 decoded frames for instant scrubbing
final _frameCache = <int, Uint8List>{};

Future<Uint8List?> getFrame(int timeMs) async {
  if (_frameCache.containsKey(timeMs)) return _frameCache[timeMs];
  final png = await api.getFramePng(timeMs: timeMs, previewWidth: 540, previewHeight: 960);
  _frameCache[timeMs] = png;
  if (_frameCache.length > 10) _frameCache.remove(_frameCache.keys.first);
  return png;
}
```

**Acceptance:** Scrubbing the seek bar displays frames without freezing; forward/backward step buttons work

### Rust Engine Tasks

- [ ] Create `bridge_api.rs` with `Mutex<EditorsProEngine>` wrapper
- [ ] Add `get_frame_png()` that encodes RGBA to PNG
- [ ] Add `step_frame()` for frame stepping
- [ ] Fix decoder switching (track `current_file_path`)
- [ ] Test hardware decoder open/decode/close cycle
- [ ] Implement `generate_thumbnail()` for media library (returns small PNG)

### Flutter UI Tasks

- [ ] Create `EngineService` singleton for bridge initialization
- [ ] Create `FileService` for SAF URI → local path conversion
- [ ] Create `PermissionService` for runtime permission requests
- [ ] Create `FrameConverter` for PNG byte display
- [ ] Replace placeholder `PreviewViewport` with live frame display
- [ ] Wire play/pause/seek with `AnimationController`
- [ ] Add frame step forward/backward buttons
- [ ] Add seek bar below preview
- [ ] Update `EditorNotifier` to use `PlaybackController`

### Bridge Integration Tasks

- [ ] Add `flutter_rust_bridge` to `engine/Cargo.toml`
- [ ] Create `engine/src/api/bridge_api.rs`
- [ ] Run `flutter_rust_bridge_codegen generate`
- [ ] Create `lib/core/services/engine_service.dart`
- [ ] Initialize `RustLib.init()` in `main.dart`
- [ ] Create `engine_bridge_provider.dart` for Riverpod integration
- [ ] Connect `import_media()` → `api.importMedia()`
- [ ] Connect `get_frame()` → `api.getFramePng()`

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_bridge_api_initialize` | Rust unit | `EditorsProEngineApi::new()` + `initialize()` returns Ok |
| `test_bridge_api_create_project` | Rust unit | Create project returns valid `ProjectInfo` |
| `test_bridge_api_import_media` | Rust unit | Import a test video file; verify `MediaAssetInfo` fields |
| `test_bridge_api_get_frame` | Rust unit | Decode frame from test video; verify non-empty PNG output |
| `test_decoder_hardware_open_close` | Rust unit | Open/close cycle doesn't leak resources |
| `test_frame_png_encoding` | Rust unit | RGBA data encodes to valid PNG |
| `widget_test_import_button` | Flutter widget | Import button exists and is tappable |
| `integration_test_import_preview` | Flutter integration | Import video → first frame appears in preview |

### Acceptance Criteria

- [ ] App launches → bridge initializes → no errors in logcat
- [ ] Tap "Import" → file picker opens → select MP4 → video appears in media library
- [ ] Tap video → first frame displays in preview viewport within 2 seconds
- [ ] Press play → video plays at ~30fps (may stutter on complex videos)
- [ ] Press pause → playback stops
- [ ] Drag seek bar → frame updates to correct position
- [ ] Frame step buttons advance/rewind by one frame
- [ ] No crashes during 60-second continuous playback

### Google Play Store Readiness Checklist (Phase 1)

- [ ] Runtime permission requests for `READ_MEDIA_VIDEO` with proper rationale
- [ ] Graceful fallback when permission denied (show explanation)
- [ ] No ANR (Application Not Responding) during video import
- [ ] Memory usage stays under 300MB during playback of 1080p video

---

## 6. Phase 2: Timeline & Trim

**Duration:** 3-4 weeks  
**Goal:** Working timeline with drag interaction; add clip to timeline; trim clip with start/end handles; split clip at playhead; undo/redo connected; save/load project

### Detailed Tasks

#### 2.1 — Timeline Data Sync: Rust as Source of Truth

| File | Task |
|------|------|
| `lib/features/editor/providers/timeline_provider.dart` (new) | Create provider that reads timeline state from Rust engine and exposes it to Flutter |
| `lib/features/editor/providers/editor_provider.dart` | Remove `durationMs` and `currentTimeMs` from local state; read from `timeline_provider` instead |
| `engine/src/api/bridge_api.rs` | Add `get_timeline_state() -> TimelineState` that returns full track/clip structure |
| `engine/src/api/mod.rs` | Create `TimelineState` DTO: `{ tracks: Vec<TrackDto>, duration_ms: u64 }` |

**Sync pattern:**
```
Flutter action → Rust engine call → engine updates timeline → Flutter re-reads state
(No dual state; Flutter is a read cache that refreshes after every mutation)
```

**Acceptance:** After any timeline mutation, calling `get_timeline_state()` returns the correct state

#### 2.2 — Add Clip to Timeline from Imported Media

| File | Task |
|------|--------|
| `lib/features/editor/presentation/editor_screen.dart:_MediaAssetItem` | Wire the "+" button to call `api.addClip(trackId:, assetId:, startMs: 0, durationMs: 0)` |
| `engine/src/api/bridge_api.rs` | Ensure `add_clip()` returns the updated timeline state |
| `lib/features/editor/widgets/timeline_panel.dart:_TrackRow` | Render clips from the Rust-provided timeline state |

**Acceptance:** Tap "+" on imported video → clip appears on "Video 1" track in timeline

#### 2.3 — Timeline Horizontal Scroll and Drag Interaction

| File | Task |
|------|--------|
| `lib/features/editor/widgets/timeline_panel.dart` | Wrap timeline content in `SingleChildScrollView(scrollDirection: Axis.horizontal)` |
| `lib/features/editor/widgets/timeline_panel.dart` | Implement clip drag with `GestureDetector.onHorizontalDragUpdate` |
| `lib/features/editor/widgets/timeline_panel.dart` | Add playhead drag to seek (tap/drag on ruler) |
| `lib/features/editor/widgets/timeline_panel.dart` | Synchronize horizontal scroll between ruler and track content |

**Acceptance:** Timeline scrolls horizontally for long projects; clips can be dragged left/right; playhead follows drag

#### 2.4 — Trim Clip with Start/End Handles

| File | Task |
|------|--------|
| `lib/features/editor/widgets/clip_trim_handles.dart` (new) | Create trim handle widgets at left and right edges of selected clip |
| `lib/features/editor/widgets/timeline_panel.dart:_ClipWidget` | When clip is selected, overlay trim handles |
| `lib/features/editor/providers/editor_provider.dart` | Add `trimClip(clipId, trimStartMs, trimEndMs)` that calls `api.trimClip()` |
| `engine/src/api/bridge_api.rs` | Ensure `trim_clip()` returns updated clip info |
| `lib/features/editor/widgets/inspector_panel.dart` | Show trim values (start/end time) in inspector when clip is selected |

**Trim handle implementation:**
```dart
class ClipTrimHandles extends StatelessWidget {
  // Left handle (trim start) and right handle (trim end)
  // Each is a 12px wide drag handle at the clip edge
  // Dragging updates trim_start_ms or trim_end_ms in real-time
  // On drag end, calls api.trimClip() to commit the change
}
```

**Acceptance:** Select clip → drag left handle to trim start → drag right handle to trim end → preview updates to show trimmed content

#### 2.5 — Split Clip at Playhead

| File | Task |
|------|--------|
| `lib/features/editor/widgets/editor_toolbar.dart` | Add "Split" button (scissors icon) that calls `splitAtPlayhead()` |
| `lib/features/editor/providers/editor_provider.dart` | Implement `splitAtPlayhead()` using `api.splitClip()` |
| `engine/src/api/bridge_api.rs` | Verify `split_clip()` works with epsilon tolerance |

**Acceptance:** Position playhead → tap "Split" → clip splits into two at playhead position; both clips are independently movable/trimmable

#### 2.6 — Undo/Redo Connected to Rust

| File | Task |
|------|--------|
| `lib/features/editor/widgets/editor_toolbar.dart` | Wire undo/redo buttons to `api.undo()` / `api.redo()` |
| `lib/features/editor/providers/editor_provider.dart` | After undo/redo, refresh timeline state from Rust |
| `engine/src/api/bridge_api.rs` | Add `can_undo() -> bool` and `can_redo() -> bool` |
| `lib/features/editor/providers/editor_provider.dart` | Listen to `can_undo`/`can_redo` to enable/disable toolbar buttons |

**Acceptance:** Trim clip → tap undo → clip returns to original state → tap redo → trim re-applies

#### 2.7 — Save/Load Project (.epp)

| File | Task |
|------|--------|
| `lib/features/editor/widgets/editor_toolbar.dart` | Add "Save" button |
| `lib/features/projects/providers/project_provider.dart` | Add `saveProject()` and `loadProject()` methods using `api.saveProject()` / `api.loadProject()` |
| `lib/features/projects/presentation/project_home_screen.dart` | List saved projects; tap to load |
| `engine/src/api/bridge_api.rs` | Ensure `save_project()` and `load_project()` work |
| `engine/src/project/format.rs` | Verify `.epp` format save/load (zip with JSON inside) |

**Project storage location:**
```dart
// Save to: getApplicationDocumentsDirectory()/projects/{projectId}.epp
// Thumbnails: getApplicationDocumentsDirectory()/thumbnails/{projectId}.png
```

**Acceptance:** Create project → add clips → save → close app → open app → project appears in list → tap to load → timeline shows all clips

### Rust Engine Tasks

- [ ] Create `TimelineState` DTO with all tracks/clips
- [ ] Add `get_timeline_state()` to bridge API
- [ ] Add `can_undo()` / `can_redo()` to bridge API
- [ ] Verify `save_project()` / `load_project()` with .epp format
- [ ] Add `generate_thumbnail()` that saves PNG to a given path
- [ ] Test undo/redo with all command types (AddClip, RemoveClip, TrimClip, SplitClip, MoveClip)

### Flutter UI Tasks

- [ ] Create `TimelineProvider` that reads from Rust
- [ ] Fix timeline horizontal scrolling
- [ ] Add clip drag gesture handling
- [ ] Create trim handle widgets
- [ ] Wire split button
- [ ] Wire undo/redo buttons
- [ ] Create save/load UI flow
- [ ] Update project home screen to list saved projects

### Bridge Integration Tasks

- [ ] Add `get_timeline_state()` to bridge API
- [ ] Add `can_undo()` / `can_redo()` to bridge API
- [ ] Re-run `flutter_rust_bridge_codegen generate` after each API change
- [ ] Create Riverpod providers for all bridge functions

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_add_clip_to_track` | Rust unit | Add clip to video track; verify it appears in timeline state |
| `test_trim_clip` | Rust unit | Trim clip start/end; verify new duration; undo restores original |
| `test_split_clip` | Rust unit | Split at midpoint; verify two clips; undo restores one |
| `test_undo_redo_chain` | Rust unit | Execute 5 commands; undo 3; redo 2; verify correct state |
| `test_save_load_epp` | Rust unit | Save project to .epp; load it back; verify identical state |
| `integration_test_timeline_interactions` | Flutter integration | Add clip → trim → split → undo → verify UI |
| `golden_test_trim_handles` | Flutter golden | Trim handles render correctly on selected clip |

### Acceptance Criteria

- [ ] Import video → add to timeline → clip appears on Video 1 track
- [ ] Drag clip left/right on timeline → position updates
- [ ] Select clip → trim handles appear → drag to trim → preview shows trimmed content
- [ ] Position playhead → split → two clips appear
- [ ] Undo/redo buttons work for all operations
- [ ] Save project → close → reopen → project loads with all edits intact
- [ ] Timeline scrolls horizontally for projects > 30 seconds
- [ ] No state sync issues between Flutter and Rust

### Google Play Store Readiness Checklist (Phase 2)

- [ ] App handles config changes (rotation) without losing project state
- [ ] Auto-save every 30 seconds (configurable)
- [ ] "Unsaved changes" dialog when navigating away from editor
- [ ] Project files are stored in app-private directory (not accessible to other apps)

---

## 7. Phase 3: Export Pipeline

**Duration:** 2-3 weeks  
**Goal:** Real FFmpeg encoding in export engine; progress reporting via stream; multiple resolution presets; foreground service for export; share exported video

### Detailed Tasks

#### 3.1 — Implement Real FFmpeg Encoding

| File | Task |
|------|------|
| `engine/src/export_engine/mod.rs` | Replace stub `export_video()` with actual FFmpeg encoding loop |
| `engine/src/export_engine/encoder.rs` (new) | Create FFmpeg encoder wrapper: open output context, configure codec, write frames |
| `engine/src/api/bridge_api.rs` | Add `export_video_with_progress()` using `StreamSink<ExportProgress>` |

**Encoding loop pseudocode:**
```rust
pub fn export_video(&self, output_path: &str, settings: &ExportSettings, progress_sink: &StreamSink<ExportProgress>) -> Result<ExportResult, EngineError> {
    // 1. Create output format context
    let mut octx = ffmpeg_next::format::output(&output_path)?;

    // 2. Add video stream with configured codec
    let codec = ffmpeg_next::encoder::find_by_name(settings.codec.ffmpeg_codec_name())
        .ok_or(EngineError::ExportError("Codec not found".into()))?;
    let mut ost = octx.add_stream(codec)?;
    let mut encoder = ost.codec().encoder().video()?;

    encoder.set_width(settings.width);
    encoder.set_height(settings.height);
    encoder.set_frame_rate(Rational::new(settings.fps as i32, 1));
    encoder.set_bit_rate(settings.bitrate_kbps as usize * 1000);
    encoder.set_pixel_format(ffmpeg_next::format::Pixel::YUV420P);
    encoder.open()?;
    ost.set_parameters(&encoder);

    // 3. Frame-by-frame render and encode
    let total_frames = (self.timeline.duration_ms as f64 * settings.fps as f64 / 1000.0) as u64;
    for frame_num in 0..total_frames {
        let time_ms = (frame_num as f64 * 1000.0 / settings.fps as f64) as u64;
        let rgba_frame = self.get_frame(time_ms)?;

        // Convert RGBA → YUV420P
        let yuv_frame = convert_rgba_to_yuv420p(&rgba_frame, settings.width, settings.height);

        // Encode
        encoder.send_frame(&yuv_frame)?;
        encoder.receive_and_write(&mut octx)?;

        // Report progress
        progress_sink.add(ExportProgress {
            progress: frame_num as f32 / total_frames as f32,
            current_frame: frame_num,
            total_frames,
            stage: ExportStage::Encoding,
            estimated_seconds_remaining: estimate_remaining(frame_num, total_frames, start_time),
        });
    }

    // 4. Flush encoder and finalize
    encoder.flush()?;
    octx.write_trailer()?;

    Ok(ExportResult { success: true, output_path: output_path.to_string(), ... })
}
```

**Acceptance:** Calling `export_video()` produces a valid MP4 file that plays in VLC

#### 3.2 — Progress Reporting via Stream

| File | Task |
|------|------|
| `engine/src/api/bridge_api.rs` | Use `StreamSink<ExportProgress>` in export function |
| `lib/features/export/providers/export_provider.dart` (new) | Listen to progress stream; update UI |
| `lib/features/export/presentation/export_screen.dart` | Show progress bar, estimated time, current stage |

**Flutter side:**
```dart
Future<void> startExport(ExportSettings settings) async {
  final api = await EngineService.instance;
  final outputDir = await getApplicationDocumentsDirectory();
  final outputPath = '${outputDir.path}/exports/${DateTime.now().millisecondsSinceEpoch}.mp4';

  final stream = api.exportVideoWithProgress(
    outputPath: outputPath,
    settings: settings,
  );

  await for (final progress in stream) {
    state = state.copyWith(
      exportProgress: progress.progress,
      exportStage: progress.stage.name,
      estimatedSecondsRemaining: progress.estimatedSecondsRemaining,
    );
  }
}
```

**Acceptance:** Export screen shows live progress bar from 0% to 100% with estimated time

#### 3.3 — Multiple Resolution Presets

| File | Task |
|------|--------|
| `lib/features/export/presentation/export_screen.dart` | Add resolution selector: 720p, 1080p, 4K, Social Vertical, Social Square |
| `engine/src/export_engine/mod.rs` | Ensure all presets work (already defined) |
| `lib/features/export/providers/export_provider.dart` | Map UI selection to `ExportSettings` presets |

**Acceptance:** User can select 720p or 1080p export; resulting file matches selected resolution

#### 3.4 — Foreground Service for Export

| File | Task |
|------|--------|
| `android/app/src/main/AndroidManifest.xml` | Add `<service android:name=".ExportService" android:foregroundServiceType="mediaPlayback" />` |
| `android/app/src/main/kotlin/com/editorspro/editors_pro/ExportService.kt` (new) | Create foreground service with notification that shows export progress |
| `lib/core/services/export_service.dart` (new) | Start/stop foreground service around export operations |
| `android/app/src/main/AndroidManifest.xml` | Add `POST_NOTIFICATIONS` permission (Android 13+) |

**Foreground service flow:**
```
1. Flutter calls platform channel to start ExportService
2. ExportService creates notification channel
3. ExportService starts foreground with notification
4. Rust engine encodes video
5. Progress updates via StreamSink (notification updates too)
6. On completion, ExportService stops foreground
```

**Acceptance:** Export continues even when app is minimized; notification shows progress; tapping notification returns to app

#### 3.5 — Share Exported Video

| File | Task |
|------|--------|
| `lib/features/export/presentation/export_screen.dart` | After export completes, show "Share" button |
| `lib/features/export/providers/export_provider.dart` | Use `share_plus` to share the exported file |

**Acceptance:** After export → tap "Share" → Android share sheet opens with video attached

### Rust Engine Tasks

- [ ] Implement FFmpeg encoding loop in `export_engine/encoder.rs`
- [ ] Implement RGBA → YUV420P conversion
- [ ] Add audio passthrough (copy audio from source without re-encoding for MVP)
- [ ] Add two-pass encoding support for higher quality
- [ ] Implement progress reporting via `StreamSink`
- [ ] Add file size validation (reject if output exceeds device storage)

### Flutter UI Tasks

- [ ] Redesign export screen with resolution presets
- [ ] Add progress bar with stage indicators
- [ ] Add estimated time remaining
- [ ] Add "Share" button after export
- [ ] Create export foreground service integration

### Bridge Integration Tasks

- [ ] Add `export_video_with_progress()` with `StreamSink<ExportProgress>`
- [ ] Re-run codegen
- [ ] Create `ExportProvider` that listens to progress stream

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_encoder_h264_720p` | Rust unit | Encode 100 frames to H.264 720p; verify output file |
| `test_encoder_h264_1080p` | Rust unit | Encode 100 frames to H.264 1080p; verify output file |
| `test_encoder_progress_stream` | Rust unit | Verify progress stream emits correct percentages |
| `test_rgba_to_yuv420p` | Rust unit | Verify color conversion accuracy (±1 value) |
| `test_export_trimmed_clip` | Rust integration | Import → trim → export → verify output duration matches trim |
| `integration_test_export_flow` | Flutter integration | Full export flow: import → add clip → trim → export → verify file |

### Acceptance Criteria

- [ ] Import video → trim → export → resulting MP4 plays correctly in VLC
- [ ] Export progress bar updates in real-time
- [ ] Exported file matches selected resolution preset
- [ ] Export continues when app is minimized (foreground service)
- [ ] Share button opens Android share sheet with video
- [ ] Export of 30-second 1080p video completes within 2x realtime (i.e., ≤60 seconds)
- [ ] No memory leaks during export (RSS stays under 500MB)

### Google Play Store Readiness Checklist (Phase 3)

- [ ] Foreground service notification shows app name and progress
- [ ] Export doesn't drain battery excessively (profile battery usage)
- [ ] No file corruption on export (verify MD5 of output matches re-export)
- [ ] App handles "low storage" gracefully (show error, don't crash)
- [ ] Exported files are written to app-specific directory or user-chosen location via SAF

---

## 8. Phase 4: Audio & Multi-track

**Duration:** 3-4 weeks  
**Goal:** Audio decode and playback; multi-track audio mixing; volume control per track; audio waveform visualization; audio ducking

### Detailed Tasks

#### 4.1 — Audio Decode and Playback

| File | Task |
|------|--------|
| `engine/src/audio/decoder.rs` (new) | Create audio decoder using FFmpeg: open file, decode PCM samples |
| `engine/src/audio/mod.rs` | Add `AudioDecoder` struct with `open()`, `decode_samples()`, `close()` |
| `engine/src/api/bridge_api.rs` | Add `get_audio_samples(time_ms: u64, duration_ms: u64) -> Vec<f32>` |
| `lib/core/services/audio_player_service.dart` (new) | Use Android `AudioTrack` via platform channel or `audioplayers` package to play PCM samples |
| `lib/features/editor/providers/editor_provider.dart` | Synchronize audio playback with video playback |

**Audio pipeline:**
```
FFmpeg decode → PCM float32 (44100Hz, stereo) → AudioTrack playback
                                                    ↕ synchronized with
Video decode → RGBA frame → Preview viewport
```

**Acceptance:** Playing a video with audio → audio is audible and synchronized with video (±50ms)

#### 4.2 — Multi-track Audio Mixing

| File | Task |
|------|--------|
| `engine/src/audio/mixer.rs` | Implement `AudioMixer::mix(tracks: Vec<AudioTrack>) -> Vec<f32>` |
| `engine/src/audio/mixer.rs` | Handle different sample rates (resample to project rate) |
| `engine/src/audio/mixer.rs` | Handle track offset (audio starting at different timeline positions) |
| `engine/src/api/bridge_api.rs` | Add `add_audio_track()` and `add_audio_clip()` |

**Mixing algorithm:**
```rust
fn mix_tracks(tracks: &[AudioTrackData], output_length: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; output_length * 2]; // stereo
    for track in tracks {
        let volume = track.volume;
        for (i, sample) in track.samples.iter().enumerate() {
            let output_idx = track.offset_samples + i;
            if output_idx < output.len() {
                output[output_idx] += sample * volume;
            }
        }
    }
    // Soft clipping
    for sample in output.iter_mut() {
        *sample = sample.tanh(); // Soft clip
    }
    output
}
```

**Acceptance:** Import two audio files → add both to timeline → play → hear both mixed together

#### 4.3 — Volume Control Per Track

| File | Task |
|------|--------|
| `engine/src/timeline/track.rs` | Volume field already exists; add `set_volume()` method |
| `lib/features/editor/widgets/inspector_panel.dart` | Add volume slider when track is selected |
| `lib/features/editor/widgets/timeline_panel.dart:_buildTrackHeaders` | Add small volume icon that shows current level |
| `engine/src/api/bridge_api.rs` | Add `set_track_volume(track_id: String, volume: f32)` |

**Acceptance:** Select audio track → drag volume slider → audio volume changes in real-time

#### 4.4 — Audio Waveform Visualization

| File | Task |
|------|--------|
| `engine/src/audio/waveform.rs` | Implement `generate_waveform(samples: &[f32], num_bins: usize) -> Vec<f32>` (peak values) |
| `engine/src/api/bridge_api.rs` | Add `get_waveform(asset_id: String, num_bins: usize) -> Vec<f32>` |
| `lib/features/editor/widgets/audio_waveform_painter.dart` (new) | CustomPainter that draws waveform on audio clip |
| `lib/features/editor/widgets/timeline_panel.dart:_ClipWidget` | Draw waveform inside audio clip widgets |

**Waveform rendering:**
```dart
class AudioWaveformPainter extends CustomPainter {
  final List<double> peaks; // 0.0 to 1.0

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = Color(0xFF4ECDC4).withOpacity(0.6);
    final barWidth = size.width / peaks.length;

    for (int i = 0; i < peaks.length; i++) {
      final barHeight = peaks[i] * size.height;
      final rect = Rect.fromCenter(
        center: Offset(i * barWidth + barWidth / 2, size.height / 2),
        width: barWidth - 1,
        height: barHeight,
      );
      canvas.drawRect(rect, paint);
    }
  }
}
```

**Acceptance:** Audio clips on timeline display a waveform visualization; waveform matches audio content

#### 4.5 — Audio Ducking

| File | Task |
|------|--------|
| `engine/src/audio/ducking.rs` (new) | Implement ducking: reduce background volume when voiceover is active |
| `engine/src/api/bridge_api.rs` | Add `set_ducking(track_id: String, enabled: bool, duck_level: f32)` |
| `lib/features/editor/widgets/inspector_panel.dart` | Add ducking toggle for audio tracks |

**Acceptance:** Enable ducking on music track → when voiceover plays, music volume automatically reduces

### Rust Engine Tasks

- [ ] Create `AudioDecoder` using FFmpeg
- [ ] Implement `AudioMixer::mix()` with volume, offset, and resampling
- [ ] Implement `generate_waveform()` peak extraction
- [ ] Implement audio ducking logic
- [ ] Add bridge API functions for all audio operations
- [ ] Handle audio format conversion (resample to project sample rate)

### Flutter UI Tasks

- [ ] Create `AudioPlayerService` for synchronized playback
- [ ] Create `AudioWaveformPainter` for waveform rendering
- [ ] Add volume slider to inspector panel
- [ ] Add ducking toggle to inspector
- [ ] Add "Add Audio" import option
- [ ] Synchronize audio/video playback timing

### Bridge Integration Tasks

- [ ] Add `get_audio_samples()`, `add_audio_clip()`, `set_track_volume()`, `get_waveform()`, `set_ducking()` to bridge
- [ ] Re-run codegen
- [ ] Create Riverpod providers for audio state

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_audio_decode` | Rust unit | Decode audio from test video; verify sample count and format |
| `test_audio_mix_stereo` | Rust unit | Mix two stereo tracks; verify output is correct sum |
| `test_audio_resample` | Rust unit | Resample 48000Hz to 44100Hz; verify length and quality |
| `test_waveform_generation` | Rust unit | Generate waveform peaks from audio; verify correct range |
| `test_ducking` | Rust unit | Enable ducking; verify volume reduces during voice segments |
| `integration_test_audio_playback` | Flutter integration | Import video with audio → play → verify audio is heard |

### Acceptance Criteria

- [ ] Audio plays synchronized with video (±50ms)
- [ ] Multiple audio tracks mix correctly
- [ ] Volume control works per track
- [ ] Waveform visualization displays on audio clips
- [ ] Audio ducking reduces background when voice is active
- [ ] No audio glitching during 60-second playback

### Google Play Store Readiness Checklist (Phase 4)

- [ ] Audio continues when screen is off (WAKE_LOCK + foreground service)
- [ ] Audio respects device volume settings
- [ ] No audio focus conflicts with other apps (request AUDIO_FOCUS)

---

## 9. Phase 5: Effects & Filters

**Duration:** 3-4 weeks  
**Goal:** Filter effects pipeline (brightness, contrast, saturation); real-time preview of effects; transition effects between clips; effect parameter controls

### Detailed Tasks

#### 5.1 — Filter Effects Pipeline

| File | Task |
|------|--------|
| `engine/src/effects/pipeline.rs` (new) | Create `EffectPipeline` that applies a chain of effects to a frame |
| `engine/src/effects/filters.rs` | Implement pixel-level filter functions using `rayon` for parallel processing |
| `engine/src/effects/mod.rs` | Add `apply_effects(frame: &mut FrameData, effects: &[Effect]) -> Result<(), EngineError>` |

**Parallel filter application with rayon:**
```rust
pub fn apply_brightness(data: &mut [u8], brightness: f32) {
    data.par_chunks_exact_mut(4).for_each(|pixel| {
        pixel[0] = (pixel[0] as f32 + brightness * 255.0).clamp(0.0, 255.0) as u8; // R
        pixel[1] = (pixel[1] as f32 + brightness * 255.0).clamp(0.0, 255.0) as u8; // G
        pixel[2] = (pixel[2] as f32 + brightness * 255.0).clamp(0.0, 255.0) as u8; // B
    });
}

pub fn apply_contrast(data: &mut [u8], contrast: f32) {
    let factor = (1.0 + contrast).max(0.0);
    data.par_chunks_exact_mut(4).for_each(|pixel| {
        pixel[0] = ((pixel[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
        pixel[1] = ((pixel[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
        pixel[2] = ((pixel[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
    });
}
```

**Acceptance:** Applying brightness + contrast filters to a frame processes in under 16ms on Snapdragon 8 Gen 2

#### 5.2 — Real-Time Preview of Effects

| File | Task |
|------|--------|
| `engine/src/api/bridge_api.rs` | Modify `get_frame_png()` to apply effects from clip's effect chain |
| `engine/src/timeline/clip.rs` | Add `effects: Vec<Effect>` field to `Clip` |
| `lib/features/editor/widgets/preview_viewport.dart` | Refresh preview when effects change |
| `lib/features/editor/widgets/inspector_panel.dart` | Show effect parameters with sliders |

**Effect application in get_frame:**
```rust
pub fn get_frame_png(&self, time_ms: u64, preview_width: u32, preview_height: u32) -> Result<Vec<u8>, String> {
    let mut frame = self.inner.lock().unwrap().get_frame(time_ms)?;

    // Apply effects from the active clip
    if let Some(clip) = self.inner.lock().unwrap().get_active_clip(time_ms) {
        let pipeline = EffectPipeline::new(&clip.effects);
        pipeline.apply(&mut frame)?;
    }

    // Resize and encode to PNG
    // ...
}
```

**Acceptance:** Dragging brightness slider → preview updates within 100ms

#### 5.3 — Transition Effects Between Clips

| File | Task |
|------|--------|
| `engine/src/effects/transitions.rs` | Implement cross-dissolve transition: blend two frames based on transition progress |
| `engine/src/effects/transitions.rs` | Implement wipe, slide, fade-to-black transitions |
| `engine/src/timeline/clip.rs` | Add `transition_in: Option<Transition>` and `transition_out: Option<Transition>` |
| `lib/features/editor/widgets/transition_picker.dart` (new) | UI for selecting transitions between clips |
| `engine/src/api/bridge_api.rs` | Add `add_transition(clip_id, transition_type, duration_ms)` |

**Cross-dissolve implementation:**
```rust
pub fn cross_dissolve(frame_a: &FrameData, frame_b: &FrameData, progress: f32) -> FrameData {
    let mut output = frame_a.clone();
    let alpha_b = progress;
    let alpha_a = 1.0 - progress;

    output.data.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
        let a_r = frame_a.data[i * 4] as f32 * alpha_a;
        let b_r = frame_b.data[i * 4] as f32 * alpha_b;
        pixel[0] = (a_r + b_r).clamp(0.0, 255.0) as u8;
        // ... same for G, B
    });

    output
}
```

**Acceptance:** Adding a cross-dissolve transition between two clips shows smooth blend during playback

#### 5.4 — Effect Parameter Controls

| File | Task |
|------|--------|
| `lib/features/editor/widgets/inspector_panel.dart` | Show effect parameters with labeled sliders (min/max/step from `EffectParameter`) |
| `lib/features/editor/widgets/inspector_panel.dart` | Add "Add Effect" button that shows filter catalog |
| `lib/features/editor/widgets/inspector_panel.dart` | Add "Remove Effect" button per effect |
| `engine/src/api/bridge_api.rs` | Add `add_effect(clip_id, filter_type)`, `remove_effect(clip_id, effect_id)`, `set_effect_parameter(clip_id, effect_id, param_name, value)` |

**Acceptance:** Select clip → tap "Add Effect" → choose Brightness → slider appears → drag slider → preview updates

### Rust Engine Tasks

- [ ] Create `EffectPipeline` with rayon-parallel filter application
- [ ] Implement all 11 filter types (brightness, contrast, saturation, hue, blur, sharpen, grayscale, sepia, invert, vignette, temperature)
- [ ] Implement 5+ transition types (cross-dissolve, wipe, slide, fade-to-black, dip-to-color)
- [ ] Add effects field to `Clip` struct
- [ ] Integrate effect application into `get_frame()` path
- [ ] Add bridge API for effect CRUD operations

### Flutter UI Tasks

- [ ] Redesign effects panel with filter catalog
- [ ] Add effect parameter sliders in inspector
- [ ] Create transition picker UI
- [ ] Show effect thumbnails (apply filter to a sample frame)
- [ ] Real-time preview updates on parameter change

### Bridge Integration Tasks

- [ ] Add `add_effect()`, `remove_effect()`, `set_effect_parameter()`, `add_transition()` to bridge
- [ ] Re-run codegen

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_brightness_filter` | Rust unit | Apply brightness +1.0 to white frame → all pixels become 255 |
| `test_contrast_filter` | Rust unit | Apply contrast 0.5 → verify pixel values |
| `test_blur_filter` | Rust unit | Apply Gaussian blur; verify output is smoother than input |
| `test_effect_pipeline_chain` | Rust unit | Apply brightness + contrast + saturation; verify correct order |
| `test_cross_dissolve_transition` | Rust unit | Blend two frames at 50%; verify average of both |
| `test_effect_performance` | Rust bench | 1080p frame with 3 effects < 16ms |
| `integration_test_effect_preview` | Flutter integration | Add brightness effect → verify preview changes |

### Acceptance Criteria

- [ ] All 11 filters apply correctly to frames
- [ ] Real-time preview updates within 100ms of parameter change
- [ ] Transitions render smoothly during playback
- [ ] Effect pipeline with 3+ effects processes 1080p in under 16ms
- [ ] Effects are saved/loaded with project

### Google Play Store Readiness Checklist (Phase 5)

- [ ] No OOM when applying multiple effects to 4K video
- [ ] Effect processing doesn't cause ANR (all on background thread)
- [ ] Thermal throttling doesn't cause app crash (monitor CPU temperature)

---

## 10. Phase 6: Text & Overlays

**Duration:** 2-3 weeks  
**Goal:** Text overlay system; font selection; text positioning (drag); text animations; subtitle/caption support

### Detailed Tasks

#### 6.1 — Text Overlay System

| File | Task |
|------|--------|
| `engine/src/effects/text_render.rs` | Implement text rendering using `fontdue` or `rusttype` crate for rasterizing glyphs onto frame |
| `engine/src/effects/text_render.rs` | Support font size, color, background, stroke, shadow |
| `engine/src/timeline/clip.rs` | TextClip variant with text content, font, position, style |
| `engine/src/api/bridge_api.rs` | Add `add_text_clip(track_id, text, position, style)` |

**Text rendering pipeline:**
```rust
pub fn render_text_on_frame(frame: &mut FrameData, text_clip: &TextClip) {
    let font = load_font(&text_clip.font_family);
    let glyphs = font.layout(&text_clip.text, text_clip.font_size);

    for glyph in glyphs {
        let (x, y) = glyph.position + text_clip.position;
        for (py, row) in glyph.pixels.iter().enumerate() {
            for (px, alpha) in row.iter().enumerate() {
                let fx = x + px as i32;
                let fy = y + py as i32;
                if fx >= 0 && fy >= 0 && fx < frame.width as i32 && fy < frame.height as i32 {
                    blend_pixel(frame, fx, fy, text_clip.color, *alpha);
                }
            }
        }
    }
}
```

**Acceptance:** Add text clip → text appears on preview at specified position with correct styling

#### 6.2 — Font Selection

| File | Task |
|------|--------|
| `engine/Cargo.toml` | Add `fontdue = "0.9"` for font rasterization |
| `assets/fonts/` (new directory) | Bundle 5-10 fonts (Inter, Roboto, Montserrat, Playfair Display, Oswald, etc.) |
| `pubspec.yaml` | Add font assets |
| `lib/features/editor/widgets/font_picker.dart` (new) | Grid of font previews; tap to select |
| `engine/src/api/bridge_api.rs` | Add `get_available_fonts() -> Vec<FontInfo>` |

**Acceptance:** Text panel shows font list; selecting a font updates text clip in preview

#### 6.3 — Text Positioning (Drag)

| File | Task |
|------|--------|
| `lib/features/editor/widgets/preview_viewport.dart` | Add `GestureDetector` for dragging text overlays |
| `lib/features/editor/widgets/text_overlay_handle.dart` (new) | Draggable handle for text position on preview |
| `engine/src/api/bridge_api.rs` | Add `set_text_position(clip_id, x, y)` |

**Acceptance:** Select text clip → drag text on preview → text moves to new position

#### 6.4 — Text Animations

| File | Task |
|------|--------|
| `engine/src/effects/text_render.rs` | Implement text animations: fade in/out, typewriter, slide in, scale up |
| `engine/src/timeline/clip.rs` | Add `text_animation: Option<TextAnimation>` to text clips |
| `lib/features/editor/widgets/inspector_panel.dart` | Add animation selector for text clips |

**Animation calculations:**
```rust
fn apply_text_animation(animation: &TextAnimation, progress: f32, text_clip: &mut TextClip) {
    match animation {
        TextAnimation::FadeIn { duration_ms } => {
            let fade_progress = (progress * animation_duration / *duration_ms as f32).min(1.0);
            text_clip.opacity = fade_progress;
        }
        TextAnimation::Typewriter { chars_per_second } => {
            let visible_chars = (progress * *chars_per_second as f32) as usize;
            text_clip.visible_text = text_clip.text.chars().take(visible_chars).collect();
        }
        // ... more animations
    }
}
```

**Acceptance:** Adding "Fade In" animation to text → text fades in during playback

#### 6.5 — Subtitle/Caption Support

| File | Task |
|------|--------|
| `engine/src/subtitle/parser.rs` (new) | Parse SRT and VTT subtitle files |
| `engine/src/api/bridge_api.rs` | Add `import_subtitles(file_path) -> Vec<SubtitleEntry>` |
| `lib/features/editor/widgets/subtitle_import.dart` (new) | UI for importing subtitle files |
| `engine/src/subtitle/mod.rs` (new) | Module for subtitle handling |

**Acceptance:** Import .srt file → subtitle clips appear on text track at correct timestamps

### Rust Engine Tasks

- [ ] Implement text rendering with `fontdue`
- [ ] Support text styles (font, size, color, stroke, shadow, background)
- [ ] Implement text animations (fade, typewriter, slide, scale)
- [ ] Add SRT/VTT subtitle parser
- [ ] Integrate text rendering into `get_frame()` pipeline

### Flutter UI Tasks

- [ ] Create text panel with presets (Title, Subtitle, Caption, Lower Third)
- [ ] Create font picker with previews
- [ ] Create text style editor (color, size, stroke, shadow)
- [ ] Create draggable text overlay handles on preview
- [ ] Create animation selector
- [ ] Create subtitle import UI

### Bridge Integration Tasks

- [ ] Add `add_text_clip()`, `set_text_position()`, `set_text_style()`, `get_available_fonts()`, `import_subtitles()` to bridge
- [ ] Re-run codegen

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_text_render_basic` | Rust unit | Render "Hello" at center; verify pixels are non-black |
| `test_text_font_change` | Rust unit | Render with different fonts; verify output differs |
| `test_text_animation_fade` | Rust unit | Fade in animation; verify opacity goes 0→1 over duration |
| `test_srt_parser` | Rust unit | Parse sample .srt; verify timestamps and text |
| `integration_test_text_overlay` | Flutter integration | Add text → drag → verify position updates |

### Acceptance Criteria

- [ ] Text overlays render on preview and in export
- [ ] Font selection works with 5+ bundled fonts
- [ ] Text can be dragged to any position on the preview
- [ ] Text animations play correctly during playback
- [ ] SRT subtitles import and display correctly
- [ ] Text is saved/loaded with project

### Google Play Store Readiness Checklist (Phase 6)

- [ ] Bundled fonts have appropriate licenses (OFL or public domain)
- [ ] No font loading crashes on devices with limited storage

---

## 11. Phase 7: Speed & Keyframes

**Duration:** 3-4 weeks  
**Goal:** Speed control with curves; keyframe animation system; position/scale/rotation keyframes; keyframe graph editor

### Detailed Tasks

#### 7.1 — Speed Control with Curves

| File | Task |
|------|--------|
| `engine/src/timeline/clip.rs` | Expand speed from `f32` to `SpeedCurve { segments: Vec<SpeedSegment> }` |
| `engine/src/timeline/speed_curve.rs` (new) | Define `SpeedSegment { start_ms, end_ms, start_speed, end_speed, easing }` |
| `engine/src/api/bridge_api.rs` | Add `set_clip_speed_curve(clip_id, curve)` |
| `lib/features/editor/widgets/speed_curve_editor.dart` (new) | Bezier curve editor for speed; drag control points |
| `lib/features/editor/widgets/inspector_panel.dart` | Add speed section with presets (0.25x, 0.5x, 1x, 2x, 4x) and custom curve |

**Speed curve evaluation:**
```rust
fn evaluate_speed_at(curve: &SpeedCurve, time_ms: u64) -> f32 {
    for segment in &curve.segments {
        if time_ms >= segment.start_ms && time_ms < segment.end_ms {
            let t = (time_ms - segment.start_ms) as f32 / (segment.end_ms - segment.start_ms) as f32;
            let eased_t = apply_easing(t, segment.easing);
            return segment.start_speed + (segment.end_speed - segment.start_speed) * eased_t;
        }
    }
    1.0 // default
}
```

**Acceptance:** Apply 0.5x speed to clip → playback is slower; apply ease-in curve → speed ramps smoothly

#### 7.2 — Keyframe Animation System

| File | Task |
|------|--------|
| `engine/src/timeline/keyframe.rs` (new) | Define `Keyframe<T> { time_ms, value, easing }` and `KeyframeTrack<T>` |
| `engine/src/timeline/clip.rs` | Add keyframe tracks: position_x, position_y, scale, rotation, opacity |
| `engine/src/api/bridge_api.rs` | Add `add_keyframe(clip_id, property, time_ms, value, easing)`, `remove_keyframe()`, `update_keyframe()` |

**Keyframe interpolation:**
```rust
fn interpolate_keyframes<T: Lerp>(keyframes: &[Keyframe<T>], time_ms: u64) -> T {
    if keyframes.is_empty() { return T::default(); }
    if time_ms <= keyframes[0].time_ms { return keyframes[0].value; }
    if time_ms >= keyframes.last().unwrap().time_ms { return keyframes.last().unwrap().value; }

    let (before, after) = find_surrounding_keyframes(keyframes, time_ms);
    let t = (time_ms - before.time_ms) as f32 / (after.time_ms - before.time_ms) as f32;
    let eased_t = apply_easing(t, before.easing);
    before.value.lerp(after.value, eased_t)
}
```

**Acceptance:** Add position keyframes at t=0 and t=5000 → clip moves smoothly from one position to another during playback

#### 7.3 — Position/Scale/Rotation Keyframes

| File | Task |
|------|--------|
| `engine/src/renderer/mod.rs` | Apply keyframe values when rendering frame: translate, scale, rotate the clip |
| `lib/features/editor/widgets/preview_viewport.dart` | Show transform handles (move, scale, rotate) on selected clip |
| `lib/features/editor/widgets/inspector_panel.dart` | Show keyframe diamond indicators on parameter sliders |

**Transform application in renderer:**
```rust
fn apply_clip_transform(frame: &mut FrameData, clip: &Clip, time_ms: u64) {
    let pos_x = clip.position_x_keyframes.interpolate(time_ms);
    let pos_y = clip.position_y_keyframes.interpolate(time_ms);
    let scale = clip.scale_keyframes.interpolate(time_ms);
    let rotation = clip.rotation_keyframes.interpolate(time_ms);

    // Apply affine transform
    let transform = AffineTransform::new(pos_x, pos_y, scale, rotation);
    transform.apply_to_frame(frame);
}
```

**Acceptance:** Add scale keyframe 1.0→2.0 → clip zooms in during playback; rotation keyframe → clip rotates

#### 7.4 — Keyframe Graph Editor

| File | Task |
|------|--------|
| `lib/features/editor/widgets/keyframe_graph_editor.dart` (new) | Full-screen graph editor: X-axis = time, Y-axis = value, drag keyframe points |
| `lib/features/editor/widgets/keyframe_graph_editor.dart` | Bezier handle editing for easing curves |
| `lib/features/editor/widgets/keyframe_graph_editor.dart` | Multi-property overlay (show position + scale on same graph) |

**Acceptance:** Open graph editor → see keyframe points → drag to adjust → playback reflects changes

### Rust Engine Tasks

- [ ] Create `SpeedCurve` with segment-based speed definition
- [ ] Create `Keyframe<T>` and `KeyframeTrack<T>` with interpolation
- [ ] Add keyframe tracks to `Clip` (position, scale, rotation, opacity)
- [ ] Apply transforms in renderer based on keyframe values
- [ ] Add easing functions (linear, ease-in, ease-out, ease-in-out, bezier)
- [ ] Add bridge API for all keyframe operations

### Flutter UI Tasks

- [ ] Create speed curve editor widget
- [ ] Create keyframe graph editor widget
- [ ] Add transform handles on preview viewport
- [ ] Add keyframe diamond indicators on inspector sliders
- [ ] Add speed presets panel
- [ ] Add "Add Keyframe" button on playback position

### Bridge Integration Tasks

- [ ] Add `set_clip_speed_curve()`, `add_keyframe()`, `remove_keyframe()`, `update_keyframe()` to bridge
- [ ] Re-run codegen

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_speed_curve_evaluation` | Rust unit | Verify speed at various time points matches curve |
| `test_keyframe_interpolation_linear` | Rust unit | Linear interpolation between two keyframes |
| `test_keyframe_interpolation_ease` | Rust unit | Ease-in-out interpolation; verify smooth curve |
| `test_clip_transform_translate` | Rust unit | Keyframe position 0→100; verify frame pixels shifted |
| `test_keyframe_save_load` | Rust unit | Keyframes survive project save/load cycle |
| `integration_test_speed_change` | Flutter integration | Change speed to 0.5x → verify slower playback |
| `integration_test_keyframe_animation` | Flutter integration | Add position keyframes → verify animation in preview |

### Acceptance Criteria

- [ ] Speed control works with presets and custom curves
- [ ] Ease-in/ease-out curves produce smooth speed transitions
- [ ] Keyframes can be added, edited, and removed
- [ ] Position, scale, rotation keyframes animate correctly during playback
- [ ] Keyframe graph editor shows and allows editing of all keyframes
- [ ] Keyframes are saved/loaded with project

### Google Play Store Readiness Checklist (Phase 7)

- [ ] No ANR when editing keyframes on complex projects
- [ ] Undo/redo works for keyframe operations

---

## 12. Phase 8: GPU Acceleration

**Duration:** 3-4 weeks  
**Goal:** wgpu integration for Vulkan; GPU shader pipeline; compute shader effects; hardware encoder integration

### Detailed Tasks

#### 8.1 — wgpu Integration for Vulkan

| File | Task |
|------|--------|
| `engine/Cargo.toml` | Uncomment `wgpu = "22.0"` and add `pollster = "0.3"` for async runtime |
| `engine/src/renderer/gpu.rs` | Implement `GpuRenderer` using wgpu: create device, configure surface |
| `engine/src/renderer/mod.rs` | Add `GpuRenderer` as alternative to `PreviewRenderer` |

**wgpu initialization on Android:**
```rust
pub async fn create_gpu_renderer() -> Result<GpuRenderer, EngineError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN, // Android uses Vulkan
        ..Default::default()
    });

    // On Android, surface comes from the Android window
    // For off-screen rendering (export), use headless
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None, // headless for now
        ..Default::default()
    }).await.ok_or(EngineError::RendererError("No GPU adapter".into()))?;

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("EDITORS-PRO GPU"),
        ..Default::default()
    }, None).await.map_err(|e| EngineError::RendererError(e.to_string()))?;

    Ok(GpuRenderer { device, queue, adapter })
}
```

**Acceptance:** GpuRenderer initializes on Android device with Vulkan support

#### 8.2 — GPU Shader Pipeline

| File | Task |
|------|--------|
| `engine/src/renderer/shader.rs` | Implement WGSL shader loading and compilation |
| `engine/src/renderer/shaders/brightness.wgsl` (new) | GPU brightness filter shader |
| `engine/src/renderer/shaders/blur.wgsl` (new) | GPU blur shader (separable Gaussian) |
| `engine/src/renderer/shaders/composite.wgsl` (new) | Multi-layer composition shader |

**Brightness shader example:**
```wgsl
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_2d<f32>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Params {
    brightness: f32,
    contrast: f32,
    saturation: f32,
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    if (id.x >= dims.x || id.y >= dims.y) { return; }

    var color = textureLoad(input_texture, id.xy, 0);
    color = vec4<f32>(
        (color.r - 0.5) * (1.0 + params.contrast) + 0.5 + params.brightness,
        (color.g - 0.5) * (1.0 + params.contrast) + 0.5 + params.brightness,
        (color.b - 0.5) * (1.0 + params.contrast) + 0.5 + params.brightness,
        color.a
    );

    textureStore(output_texture, id.xy, color);
}
```

**Acceptance:** GPU-accelerated brightness filter processes 1080p frame in under 2ms (vs 8ms on CPU)

#### 8.3 — Compute Shader Effects

| File | Task |
|------|--------|
| `engine/src/renderer/gpu.rs` | Create compute pipeline for each effect shader |
| `engine/src/effects/gpu_filters.rs` (new) | GPU implementations of all 11 filters |
| `engine/src/effects/mod.rs` | Add `apply_gpu_effects()` that uses compute shaders when available |

**Acceptance:** All 11 filters run on GPU; benchmark shows 5-10x speedup over CPU

#### 8.4 — Hardware Encoder Integration

| File | Task |
|------|--------|
| `engine/src/export_engine/hardware_encoder.rs` (new) | Use Android MediaCodec via NDK for H.264/H.265 hardware encoding |
| `engine/src/export_engine/mod.rs` | Auto-detect hardware encoder availability; fall back to software |
| `engine/Cargo.toml` | Verify `ndk-sys` is included for MediaCodec access |

**MediaCodec hardware encoding flow:**
```rust
fn encode_with_media_codec(frames: &[FrameData], settings: &ExportSettings) -> Result<ExportResult, EngineError> {
    // 1. Create MediaCodec encoder via NDK
    // 2. Configure with width, height, bitrate, fps
    // 3. Feed frames as input buffers
    // 4. Read encoded output buffers
    // 5. Mux into MP4 container
}
```

**Acceptance:** Export with hardware encoding is 3-5x faster than software encoding on supported devices

### Rust Engine Tasks

- [ ] Set up wgpu with Vulkan backend
- [ ] Implement GPU compute pipeline for effects
- [ ] Write WGSL shaders for all filters
- [ ] Implement GPU-accelerated compositing
- [ ] Add MediaCodec hardware encoder integration
- [ ] Implement fallback to CPU when GPU unavailable

### Flutter UI Tasks

- [ ] Add GPU usage indicator in settings
- [ ] Show "Hardware encoding" badge during export
- [ ] Add option to force software rendering (debug)

### Bridge Integration Tasks

- [ ] Add `is_gpu_available() -> bool` to bridge
- [ ] Modify export to auto-detect and use hardware encoder

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_wgpu_init` | Rust unit | GpuRenderer initializes on test device |
| `test_gpu_brightness_shader` | Rust unit | GPU brightness produces same output as CPU (±1 value) |
| `test_gpu_blur_shader` | Rust unit | GPU blur produces visually correct output |
| `test_hardware_encoder` | Rust integration | Export with hardware encoder; verify output plays |
| `bench_cpu_vs_gpu_filters` | Rust bench | Compare CPU vs GPU filter performance |

### Acceptance Criteria

- [ ] GPU renderer initializes on 90%+ of target Android devices
- [ ] GPU-accelerated filters are 5-10x faster than CPU
- [ ] Hardware encoding produces valid MP4 output
- [ ] Fallback to CPU works on devices without Vulkan
- [ ] No GPU memory leaks during extended editing sessions

### Google Play Store Readiness Checklist (Phase 8)

- [ ] App doesn't crash on devices without Vulkan support (graceful fallback)
- [ ] GPU memory usage stays under 512MB
- [ ] No GPU driver crashes on popular devices (Samsung, Pixel, Xiaomi)

---

## 13. Phase 9: Polish & Play Store

**Duration:** 2-3 weeks  
**Goal:** App icon and splash screen; onboarding flow; settings screen; crash reporting; performance profiling; Google Play Store submission

### Detailed Tasks

#### 9.1 — App Icon and Splash Screen

| File | Task |
|------|--------|
| `android/app/src/main/res/mipmap-*` | Design and add app icon at all densities (48px → 192px) |
| `android/app/src/main/res/drawable/launch_background.xml` | Design splash screen with brand colors and logo |
| `pubspec.yaml` | Add `flutter_native_splash` for splash screen generation |

**Acceptance:** App icon appears correctly on home screen; splash screen shows on launch

#### 9.2 — Onboarding Flow

| File | Task |
|------|--------|
| `lib/features/onboarding/presentation/onboarding_screen.dart` (new) | 3-page onboarding: (1) Welcome, (2) Import & Edit, (3) Export & Share |
| `lib/features/onboarding/providers/onboarding_provider.dart` (new) | Track if onboarding has been shown (shared_preferences) |
| `lib/app.dart` | Show onboarding on first launch |

**Acceptance:** First launch shows onboarding; subsequent launches skip to project home

#### 9.3 — Settings Screen

| File | Task |
|------|--------|
| `lib/features/settings/presentation/settings_screen.dart` (new) | Settings: default export resolution, auto-save interval, cache management, about |
| `lib/features/settings/providers/settings_provider.dart` (new) | Persist settings using shared_preferences |
| `lib/features/editor/widgets/editor_toolbar.dart` | Add settings button that navigates to settings |

**Settings to include:**
- Default export resolution (720p / 1080p / 4K)
- Default export codec (H.264 / H.265)
- Auto-save interval (30s / 60s / 5min / off)
- Cache management (clear cache, show cache size)
- GPU acceleration toggle
- About (version, licenses, privacy policy)

**Acceptance:** All settings persist across app restarts

#### 9.4 — Crash Reporting

| File | Task |
|------|--------|
| `pubspec.yaml` | Add `firebase_crashlytics: ^4.0.0` |
| `android/app/build.gradle.kts` | Add Firebase/Crashlytics plugin |
| `lib/main.dart` | Initialize Crashlytics; set up error handler |
| `android/app/src/main/AndroidManifest.xml` | Add `firebase_app_id` and `api_key` metadata |

**Also add Rust panic handling:**
```rust
// In bridge_api.rs
use std::panic;

pub fn initialize(&self) -> Result<(), String> {
    // Set up panic hook that reports to Crashlytics via platform channel
    panic::set_hook(Box::new(|info| {
        log::error!("Rust panic: {}", info);
        // TODO: Report to Crashlytics via platform channel
    }));

    crate::init_engine().map_err(|e| e.to_string())?;
    self.inner.lock().unwrap().initialized = true;
    Ok(())
}
```

**Acceptance:** Crashes are reported to Firebase console with stack traces

#### 9.5 — Performance Profiling

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Cold start time | < 3 seconds | Android Profiler |
| Frame decode (1080p) | < 16ms | Rust benchmark |
| Preview framerate | ≥ 24fps | Flutter DevTools |
| Export speed (1080p, H.264) | ≤ 2x realtime | Stopwatch on device |
| Memory usage (editing) | < 400MB | Android Profiler |
| Memory usage (export) | < 600MB | Android Profiler |
| APK size | < 50MB | `flutter build apk --release` output |
| ANR rate | 0% | Crashlytics |

**Actions based on profiling:**
- If cold start > 3s: lazy-load Rust engine, defer FFmpeg init
- If preview < 24fps: add frame skip logic, reduce preview resolution
- If memory > 600MB: implement frame cache eviction, downsample for preview

**Acceptance:** All performance targets met on Pixel 7 and Samsung Galaxy S23

#### 9.6 — Google Play Store Submission

| Item | Details |
|------|---------|
| Store listing | Title, short description, full description, screenshots (phone), feature graphic |
| Content rating | Complete IARC questionnaire (video editor = "Everyone" or "Low Maturity") |
| Data safety | Declare: no data collected, no data shared, local-only storage |
| Target SDK | 35 (required) |
| App signing | Use Google Play App Signing |
| AAB format | Upload .aab (not .apk) for smaller download size |
| Privacy policy | Required URL — create a simple privacy policy page |
| Screenshots | Minimum 4 screenshots: project home, editor, effects, export |
| Release notes | Detailed changelog for first release |

**Submission checklist:**
- [ ] All acceptance criteria from Phases 0-9 pass
- [ ] No crashes in 1-hour monkey test
- [ ] All permissions justified in store listing
- [ ] Privacy policy URL is valid
- [ ] Content rating completed
- [ ] AAB uploaded and reviewed
- [ ] Internal test track passes review
- [ ] Closed test track with 10+ testers for 14 days
- [ ] Open release

### Rust Engine Tasks

- [ ] Add Rust panic hook for crash reporting
- [ ] Optimize cold start (lazy initialization)
- [ ] Memory usage profiling and optimization

### Flutter UI Tasks

- [ ] Design and implement app icon
- [ ] Design and implement splash screen
- [ ] Create onboarding flow
- [ ] Create settings screen
- [ ] Add error boundaries to all screens

### Bridge Integration Tasks

- [ ] Ensure bridge handles engine crashes gracefully (show error dialog, not ANR)

### Testing Requirements

| Test | Type | Description |
|------|------|-------------|
| `test_cold_start_time` | Performance | App launches in < 3 seconds |
| `test_memory_leak_editing` | Performance | 30 minutes of editing; RSS doesn't grow unbounded |
| `test_crash_recovery` | Flutter integration | Kill app during editing; reopen; project auto-saved |
| `monkey_test` | Manual | 1-hour automated UI stress test; no crashes |
| `test_aab_build` | Build | `flutter build appbundle --release` succeeds |

### Acceptance Criteria

- [ ] App icon and splash screen are professional quality
- [ ] Onboarding shows on first launch only
- [ ] Settings persist and work correctly
- [ ] Crashlytics reports crashes with full stack traces
- [ ] All performance targets met
- [ ] Google Play Store submission is approved

### Google Play Store Readiness Checklist (Phase 9 — Final)

- [ ] App icon: 512x512 PNG
- [ ] Feature graphic: 1024x500 PNG
- [ ] Screenshots: 4+ phone screenshots (16:9 aspect ratio)
- [ ] Short description: ≤80 characters
- [ ] Full description: ≤4000 characters
- [ ] Privacy policy URL
- [ ] Content rating: IARC completed
- [ ] Data safety form: completed (no data collected)
- [ ] Target SDK: 35
- [ ] App signing: enrolled in Google Play App Signing
- [ ] Release type: AAB
- [ ] No MANAGE_EXTERNAL_STORAGE permission
- [ ] All permissions justified
- [ ] No hardcoded API keys
- [ ] Minimum viable product quality (no placeholder screens)

---

## 14. Phase 10: Advanced Features

**Duration:** Ongoing (post-launch)  
**Goal:** Chroma key; auto captions; templates; proxy workflow; cloud sync

### 10.1 — Chroma Key (Green Screen)

| Estimated Duration | 2-3 weeks |
|-------------------|-----------|

| File | Task |
|------|------|
| `engine/src/effects/chroma_key.rs` (new) | Implement chroma key: replace specified color range with transparency |
| `engine/src/effects/chroma_key.rs` | Color distance calculation in HSV space; spill suppression; edge softness |
| `lib/features/editor/widgets/inspector_panel.dart` | Add chroma key controls: color picker, tolerance, softness, spill |

**Acceptance:** Select green background → enable chroma key → green becomes transparent, showing layer below

### 10.2 — Auto Captions (Whisper)

| Estimated Duration | 3-4 weeks |
|-------------------|-----------|

| File | Task |
|------|------|
| `engine/Cargo.toml` | Add `whisper-rs = "0.12"` (Rust bindings for OpenAI Whisper) |
| `engine/src/audio/transcription.rs` (new) | Transcribe audio using Whisper; return timestamped segments |
| `engine/src/api/bridge_api.rs` | Add `transcribe_audio(asset_id) -> Vec<TranscriptionSegment>` with `StreamSink` progress |
| `lib/features/editor/widgets/auto_caption.dart` (new) | UI for triggering transcription and applying captions |

**Acceptance:** Tap "Auto Caption" → audio transcribed → subtitle clips created on text track with correct timestamps

### 10.3 — Templates System

| Estimated Duration | 3-4 weeks |
|-------------------|-----------|

| File | Task |
|------|------|
| `engine/src/template/mod.rs` (new) | Define template format: pre-configured timeline with placeholder clips |
| `engine/src/template/builder.rs` (new) | Template instantiation: replace placeholders with user media |
| `lib/features/templates/presentation/template_browser.dart` (new) | Grid of template previews |
| `assets/templates/` | Bundle 10-20 pre-built templates (social media formats, transitions) |

**Template structure:**
```rust
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory, // Social, Cinematic, Tutorial, etc.
    pub preview_path: String,
    pub placeholder_slots: Vec<PlaceholderSlot>, // "Drop your video here"
    pub timeline_template: Timeline, // Pre-configured with effects, transitions
    pub duration_ms: u64,
    pub aspect_ratio: (u32, u32), // 16:9, 9:16, 1:1
}
```

**Acceptance:** Browse templates → select one → replace placeholders with media → export finished video

### 10.4 — Proxy Workflow

| Estimated Duration | 2-3 weeks |
|-------------------|-----------|

| File | Task |
|------|------|
| `engine/src/proxy/mod.rs` (new) | Generate proxy (low-res) copies of imported media for smooth editing |
| `engine/src/proxy/generator.rs` (new) | Encode proxy at 480p/720p using FFmpeg; store in app cache |
| `engine/src/api/bridge_api.rs` | Auto-generate proxies for files > 1080p; use proxies for preview, originals for export |
| `lib/features/settings/presentation/settings_screen.dart` | Proxy quality setting (off, 480p, 720p) |

**Acceptance:** Import 4K video → proxy generated at 720p → editing is smooth → export uses original 4K

### 10.5 — Cloud Sync

| Estimated Duration | 4-6 weeks |
|-------------------|-----------|

| File | Task |
|------|------|
| `engine/src/cloud/mod.rs` (new) | Cloud storage integration (Google Drive, Dropbox) |
| `lib/features/cloud/providers/cloud_provider.dart` (new) | Authentication and sync logic |
| `lib/features/cloud/presentation/cloud_screen.dart` (new) | Cloud project browser |

**This is a major feature requiring careful design around:**
- Authentication (OAuth2)
- Conflict resolution (simultaneous edits on two devices)
- Large file handling (only sync .epp project files, not source media)
- Privacy considerations

**Acceptance:** Save project → sync to cloud → open on another device → project loads with all edits

---

## 15. Testing Strategy

### Testing Pyramid

```
         ╱  E2E  ╲         ← Few, slow, expensive
        ╱─────────╲
       ╱Integration╲       ← Moderate number
      ╱─────────────╲
     ╱  Widget Tests ╲     ← Many, medium speed
    ╱─────────────────╲
   ╱   Unit Tests      ╲   ← Most, fast, cheap
  ╱─────────────────────╲
```

### Rust Unit Tests

**Location:** `engine/src/*/tests.rs` or inline `#[cfg(test)]` modules

| Module | Tests | Priority |
|--------|-------|----------|
| `timeline` | Add/remove/move clips, split, trim, undo/redo | Phase 2 |
| `decoder` | Open/close, decode frame, hardware vs software | Phase 1 |
| `effects` | Each filter, pipeline chaining, edge cases | Phase 5 |
| `audio` | Decode, mix, resample, ducking | Phase 4 |
| `export` | Encode H.264, H.265, presets | Phase 3 |
| `project` | Save/load .epp, version migration | Phase 2 |
| `keyframe` | Interpolation, easing, multi-property | Phase 7 |

**Running:** `cargo test` — target: 80% line coverage for engine modules

### Rust Benchmarks

**Location:** `engine/benches/`

```rust
// engine/benches/frame_decode.rs
#[bench]
fn bench_decode_1080p_frame(b: &mut Bencher) {
    let mut decoder = HardwareDecoder::new();
    decoder.open("test_assets/sample_1080p.mp4").unwrap();
    b.iter(|| decoder.decode_frame_at(5000));
}

#[bench]
fn bench_apply_brightness_1080p(b: &mut Bencher) {
    let frame = FrameData::blank(1920, 1080);
    b.iter(|| apply_brightness(&mut frame.data.clone(), 0.5));
}
```

**Running:** `cargo bench` — track performance regression in CI

### Flutter Widget Tests

| Widget | Test | Priority |
|--------|------|----------|
| `PreviewViewport` | Renders frame data without crash | Phase 1 |
| `TimelinePanel` | Shows tracks and clips | Phase 2 |
| `ClipTrimHandles` | Handles appear on selected clip | Phase 2 |
| `InspectorPanel` | Shows correct properties | Phase 2 |
| `ExportScreen` | Shows progress bar | Phase 3 |
| `FontPicker` | Shows font list | Phase 6 |

### Flutter Integration Tests

**Location:** `integration_test/`

| Test | Steps | Priority |
|------|-------|----------|
| `import_and_preview` | Launch → import → see first frame | Phase 1 |
| `timeline_interactions` | Import → add to timeline → trim → split | Phase 2 |
| `export_flow` | Import → trim → export → verify file | Phase 3 |
| `audio_playback` | Import video with audio → play → verify sound | Phase 4 |
| `effects_preview` | Add brightness effect → verify preview | Phase 5 |
| `text_overlay` | Add text → drag → verify position | Phase 6 |

**Running:** `flutter test integration_test/`

### Golden Tests

**Purpose:** Detect visual regressions in UI components

```dart
testWidgets('Timeline renders correctly', (tester) async {
  await tester.pumpWidget(testApp);
  await expectLater(
    find.byType(TimelinePanel),
    matchesGoldenFile('goldens/timeline_panel.png'),
  );
});
```

**Running:** `flutter test --update-goldens` (update) / `flutter test` (verify)

### Performance Benchmarks (Flutter)

```dart
testWidgets('Frame decode performance', (tester) async {
  final binding = tester.binding as AutomatedTestWidgetsFlutterBinding;
  binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  final stopwatch = Stopwatch()..start();
  for (int i = 0; i < 30; i++) {
    await api.getFramePng(timeMs: i * 33);
  }
  stopwatch.stop();

  // Should decode 30 frames in under 1 second (30fps target)
  expect(stopwatch.elapsedMilliseconds, lessThan(1000));
});
```

---

## 16. Google Play Store Requirements Checklist

### Policies & Compliance

| Requirement | Status | Phase |
|-------------|--------|-------|
| Target SDK 35 | Done | 0 |
| No `MANAGE_EXTERNAL_STORAGE` | Must fix | 0 |
| Scoped storage compliance | Must fix | 0 |
| Content rating (IARC) | Pending | 9 |
| Data safety declaration | Pending | 9 |
| Privacy policy URL | Pending | 9 |
| No deceptive behavior | ✅ | — |
| No malware/unwanted software | ✅ | — |
| No impersonation | ✅ | — |

### Technical Requirements

| Requirement | Status | Phase |
|-------------|--------|-------|
| App signing (Google Play) | Must set up | 0 |
| AAB format | Must configure | 9 |
| 64-bit native libraries (arm64-v8a) | Must build | 0 |
| No debug signing in release | Must fix | 0 |
| ProGuard/R8 rules | Must create | 0 |
| No hardcoded API keys | Must audit | 0 |
| WebView policy compliance | N/A | — |
| Background execution limits | Must verify | 3 |
| Notification channels (Android 8+) | Must implement | 3 |
| Scoped storage (Android 11+) | Must fix | 0 |
| Exact alarm permission (Android 12+) | N/A | — |
| Notification permission (Android 13+) | Must implement | 3 |

### Permissions Justification

| Permission | Justification |
|-----------|---------------|
| `READ_MEDIA_VIDEO` | Required to import video files from gallery |
| `READ_MEDIA_AUDIO` | Required to import audio files for background music |
| `READ_MEDIA_IMAGES` | Required to import images for overlays |
| `READ_EXTERNAL_STORAGE` | Backward compatibility for Android 12 and below |
| `FOREGROUND_SERVICE` | Required for export process that continues when app is minimized |
| `FOREGROUND_SERVICE_MEDIA_PLAYBACK` | Required for audio playback during export |
| `POST_NOTIFICATIONS` | Required to show export progress notification |
| `WAKE_LOCK` | Required to prevent CPU sleep during export |
| `INTERNET` | Future cloud sync feature |

### Store Listing Assets

| Asset | Size | Format |
|-------|------|--------|
| App icon | 512x512 | PNG (32-bit) |
| Feature graphic | 1024x500 | PNG/JPEG |
| Phone screenshots (min 4) | 16:9 ratio | PNG/JPEG |
| Seven-inch screenshots | 16:9 ratio | PNG/JPEG |
| Ten-inch screenshots | 16:9 ratio | PNG/JPEG |

---

## 17. Performance Targets

### Runtime Performance

| Metric | Target | Critical Threshold | Measurement |
|--------|--------|--------------------|-------------|
| Cold start | < 3s | < 5s | `adb shell am start -W` |
| Warm start | < 1s | < 2s | `adb shell am start -W` |
| Frame decode (720p) | < 5ms | < 10ms | Rust benchmark |
| Frame decode (1080p) | < 10ms | < 16ms | Rust benchmark |
| Frame decode (4K) | < 30ms | < 50ms | Rust benchmark |
| Preview FPS (1080p) | ≥ 30fps | ≥ 24fps | Flutter DevTools |
| Preview FPS (4K w/ proxy) | ≥ 30fps | ≥ 24fps | Flutter DevTools |
| Filter application (CPU) | < 16ms | < 33ms | Rust benchmark |
| Filter application (GPU) | < 3ms | < 8ms | Rust benchmark |
| Export speed (H.264 1080p, SW) | ≤ 2x realtime | ≤ 3x | Stopwatch |
| Export speed (H.264 1080p, HW) | ≤ 0.5x realtime | ≤ 1x | Stopwatch |

### Memory Targets

| Scenario | Target | Critical | Measurement |
|----------|--------|----------|-------------|
| App idle | < 100MB | < 150MB | Android Profiler |
| Editing 1080p video | < 400MB | < 500MB | Android Profiler |
| Editing 4K video (with proxy) | < 400MB | < 500MB | Android Profiler |
| Export (1080p) | < 600MB | < 800MB | Android Profiler |
| Export (4K) | < 1GB | < 1.5GB | Android Profiler |

### Storage Targets

| Item | Target |
|------|--------|
| APK size (download) | < 40MB |
| AAB size (download) | < 30MB |
| Installed size | < 80MB |
| Cache per project | < 100MB |
| Proxy per 5-min 4K video | < 200MB |

### Battery Targets

| Scenario | Target |
|----------|--------|
| 30-minute editing session | < 10% battery drain |
| 5-minute export (1080p) | < 3% battery drain |

---

## 18. Risk Register

| ID | Risk | Probability | Impact | Mitigation |
|----|------|-------------|--------|------------|
| R1 | **flutter_rust_bridge v2 incompatibility** — codegen fails or produces incorrect bindings | Medium | High | Test bridge setup in Phase 1 Week 1; have fallback plan using manual FFI with `dart:ffi` |
| R2 | **FFmpeg build failure on Android NDK 27** — linking errors, missing codecs | Medium | Critical | Use pre-built FFmpeg Android binaries from `ffmpeg-kit` as fallback; test NDK build early |
| R3 | **Hardware decoder not available on some devices** — MediaCodec failures on low-end devices | High | Medium | Implement software decoder fallback; test on 5+ device models |
| R4 | **Memory pressure on 4K video** — OOM during 4K editing | High | High | Implement proxy workflow early (Phase 10); limit preview resolution; add memory monitoring |
| R5 | **Google Play rejection** — due to permissions, content rating, or policy violations | Low | Critical | Follow checklist strictly; test with internal track before public release |
| R6 | **Audio/video sync drift** — accumulated timing error during playback | Medium | High | Use monotonic clock; resync every 1 second; measure drift in testing |
| R7 | **Performance regression** — effects pipeline too slow for real-time preview | Medium | High | Benchmark every phase; GPU acceleration in Phase 8; reduce preview resolution if needed |
| R8 | **File corruption** — .epp save interrupted by crash | Low | High | Write to temp file, then atomic rename; auto-save to separate file; add CRC32 checksum |
| R9 | **Android 16+ breaking changes** — new permission model, background restrictions | Medium | Medium | Monitor Android Developer Preview; keep targetSdk updated |
| R10 | **Rust cross-compilation issues** — ARM64 build fails on CI | Medium | Medium | Use `cargo-ndk`; test build in CI from Phase 0; cache build artifacts |
| R11 | **Flutter 4.0 migration** — breaking changes in future Flutter version | Low | Medium | Pin Flutter version in CI; migrate only when stable; test on beta channel early |
| R12 | **FFmpeg patent/licensing** — H.264/H.265 codec licensing for commercial use | Medium | Critical | Use only open-source codecs in free version; license codecs for paid version; consult legal |

### Top 3 Risks to Monitor

1. **R2 (FFmpeg build failure)** — Test in Phase 1 Week 1; have `ffmpeg-kit` as backup
2. **R4 (4K memory pressure)** — Implement proxy workflow before marketing 4K support
3. **R12 (FFmpeg licensing)** — Legal review before commercial release; consider VP9/AV1 as patent-free alternatives

---

## 19. Architecture Decision Records

### ADR-001: Flutter + Rust Architecture

**Status:** Accepted  
**Date:** 2026-03-04  
**Context:** Need a mobile video editor with professional-grade performance that runs on Android.  
**Decision:** Use Flutter for UI and Rust for the video processing engine, connected via flutter_rust_bridge v2.  
**Consequences:**
- (+) Rust gives native performance for video decode/encode/effects
- (+) Flutter gives rapid UI development and Material Design
- (+) flutter_rust_bridge provides type-safe, zero-copy communication
- (-) Two languages to maintain; two build systems
- (-) Bridge code generation adds complexity to build pipeline
- (-) Debugging across the FFI boundary is harder

### ADR-002: Rust Engine as Single Source of Truth

**Status:** Accepted  
**Date:** 2026-03-04  
**Context:** The audit found dual state management — Flutter Riverpod and Rust engine maintain independent timeline state with no sync.  
**Decision:** The Rust engine is the single source of truth for all project state. Flutter reads state from Rust after every mutation and does not maintain independent state.  
**Consequences:**
- (+) No state sync bugs
- (+) Simpler mental model: Flutter → command → Rust → state → Flutter reads back
- (-) Every UI update requires a bridge call (slight latency)
- (-) Flutter cannot work offline from the engine (but engine is always available)

### ADR-003: Mutex-Based Bridge API

**Status:** Accepted  
**Date:** 2026-03-04  
**Context:** flutter_rust_bridge v2 requires `&self` methods (not `&mut self`). The engine uses `&mut self` extensively.  
**Decision:** Wrap the engine in `std::sync::Mutex<EditorsProEngine>` inside the bridge API struct. All bridge methods acquire the lock, perform the operation, and release.  
**Consequences:**
- (+) Compatible with flutter_rust_bridge v2
- (+) Thread-safe (only one thread accesses engine at a time)
- (-) No concurrent access to engine (but we don't need it — Flutter UI is single-threaded)
- (-) Must be careful not to hold the lock across long operations (export should release lock periodically)

### ADR-004: PNG Frame Transfer (Phase 1), GPU Texture (Phase 8)

**Status:** Accepted  
**Date:** 2026-03-04  
**Context:** Flutter's `Image.memory()` requires compressed image data, not raw RGBA pixels.  
**Decision:** In Phase 1, encode frames to PNG in Rust using the `image` crate and transfer PNG bytes via the bridge. In Phase 8, switch to GPU texture sharing via Flutter's `Texture` widget and wgpu.  
**Consequences:**
- (+) Phase 1: Simple implementation; works immediately
- (+) Phase 8: Zero-copy GPU texture sharing for maximum performance
- (-) PNG encoding adds ~5ms per frame in Phase 1
- (-) PNG transfer uses more bandwidth than raw RGBA (but PNG compresses well)

### ADR-005: Scoped Storage Compliance

**Status:** Accepted  
**Date:** 2026-03-04  
**Context:** Google Play rejects apps using `MANAGE_EXTERNAL_STORAGE`. Android 13+ uses granular media permissions.  
**Decision:** Remove `MANAGE_EXTERNAL_STORAGE`. Use `READ_MEDIA_VIDEO/AUDIO/IMAGES` for reading. Use SAF (Storage Access Framework) for writing exported files. Copy imported files to app cache for Rust access.  
**Consequences:**
- (+) Play Store compliant
- (+) Respects user privacy
- (-) Extra step to copy imported files to app cache (uses storage temporarily)
- (-) SAF content URIs need conversion to file paths for Rust/FFmpeg

### ADR-006: Command Pattern for Undo/Redo

**Status:** Accepted (pre-existing)  
**Date:** Pre-existing architecture  
**Context:** Editor apps need undo/redo.  
**Decision:** Use the Command pattern. Each mutation (add clip, trim, split) is a `Command` object that implements `execute()` and `undo()`. A `CommandHistory` maintains the undo/redo stacks.  
**Consequences:**
- (+) Clean undo/redo implementation
- (+) Commands can be composed (macro commands)
- (-) Each command holds cloned state for undo (memory overhead)
- (-) All mutations must go through the command system (discipline required)

### ADR-007: .epp Project Format

**Status:** Accepted (pre-existing)  
**Date:** Pre-existing architecture  
**Context:** Need a project file format that can evolve over time without breaking old projects.  
**Decision:** Use a ZIP container with a JSON manifest inside. Include version number for migration. Add CRC32 checksum for integrity.  
**Consequences:**
- (+) Compressed storage
- (+) Extensible (add new files to the ZIP)
- (+) Version migration built in
- (-) Slightly slower save/load than raw JSON
- (-) ZIP library dependency

### ADR-008: arm64-v8a Only for Initial Release

**Status:** Proposed  
**Date:** 2026-03-04  
**Context:** Supporting armeabi-v7a and x86_64 increases APK size and CI build time.  
**Decision:** Target arm64-v8a only for the initial Google Play release. Add armeabi-v7a support in a follow-up release if demand exists.  
**Consequences:**
- (+) Smaller APK (~30MB vs ~50MB)
- (+) Faster CI builds
- (+) All modern Android devices (2019+) support arm64
- (-) Excludes some older/budget devices with only armeabi-v7a

---

## Appendix A: File Structure After All Phases

```
editors-pro/
├── .github/
│   └── workflows/
│       └── ci.yml
├── android/
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── jniLibs/           # Rust .so files
│   │   │   │   └── arm64-v8a/
│   │   │   │       └── libeditors_pro_engine.so
│   │   │   ├── kotlin/com/editorspro/editors_pro/
│   │   │   │   ├── MainActivity.kt
│   │   │   │   └── ExportService.kt
│   │   │   ├── res/
│   │   │   │   ├── mipmap-*/     # App icon
│   │   │   │   └── drawable/     # Splash screen
│   │   │   └── AndroidManifest.xml
│   │   ├── proguard-rules.pro
│   │   └── build.gradle.kts
│   ├── build.gradle.kts
│   ├── settings.gradle.kts
│   └── key.properties (gitignored)
├── engine/
│   ├── src/
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── bridge_api.rs     # flutter_rust_bridge entry point
│   │   │   └── commands.rs
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── decoder.rs
│   │   │   ├── mixer.rs
│   │   │   ├── waveform.rs
│   │   │   ├── ducking.rs
│   │   │   └── transcription.rs
│   │   ├── decoder/
│   │   │   ├── mod.rs
│   │   │   ├── hardware.rs
│   │   │   └── software.rs
│   │   ├── effects/
│   │   │   ├── mod.rs
│   │   │   ├── filters.rs
│   │   │   ├── pipeline.rs
│   │   │   ├── transitions.rs
│   │   │   ├── text_render.rs
│   │   │   ├── chroma_key.rs
│   │   │   └── gpu_filters.rs
│   │   ├── export_engine/
│   │   │   ├── mod.rs
│   │   │   ├── encoder.rs
│   │   │   └── hardware_encoder.rs
│   │   ├── project/
│   │   │   ├── mod.rs
│   │   │   └── format.rs
│   │   ├── proxy/
│   │   │   ├── mod.rs
│   │   │   └── generator.rs
│   │   ├── renderer/
│   │   │   ├── mod.rs
│   │   │   ├── gpu.rs
│   │   │   └── shader.rs
│   │   ├── shaders/              # WGSL shader files
│   │   │   ├── brightness.wgsl
│   │   │   ├── blur.wgsl
│   │   │   └── composite.wgsl
│   │   ├── subtitle/
│   │   │   ├── mod.rs
│   │   │   └── parser.rs
│   │   ├── template/
│   │   │   ├── mod.rs
│   │   │   └── builder.rs
│   │   ├── timeline/
│   │   │   ├── mod.rs
│   │   │   ├── clip.rs
│   │   │   ├── command.rs
│   │   │   ├── keyframe.rs
│   │   │   ├── speed_curve.rs
│   │   │   └── track.rs
│   │   ├── lib.rs
│   │   └── generated.rs          # flutter_rust_bridge generated
│   ├── benches/
│   │   ├── frame_decode.rs
│   │   └── effects_pipeline.rs
│   ├── tests/
│   │   └── integration_tests.rs
│   ├── Cargo.toml
│   └── build.rs
├── assets/
│   ├── fonts/                    # Bundled fonts
│   └── templates/                # Pre-built templates
├── integration_test/
│   ├── import_and_preview_test.dart
│   ├── timeline_interactions_test.dart
│   ├── export_flow_test.dart
│   └── audio_playback_test.dart
├── lib/
│   ├── main.dart
│   ├── app.dart
│   ├── src/
│   │   └── rust/                 # flutter_rust_bridge generated
│   │       ├── api/
│   │       │   └── editors_pro_engine_api.dart
│   │       ├── frb_generated.dart
│   │       └── frb_generated.io.dart
│   ├── core/
│   │   ├── constants/
│   │   │   └── app_constants.dart
│   │   ├── extensions/
│   │   │   └── context_extensions.dart
│   │   ├── services/
│   │   │   ├── engine_service.dart
│   │   │   ├── file_service.dart
│   │   │   ├── permission_service.dart
│   │   │   ├── export_service.dart
│   │   │   └── audio_player_service.dart
│   │   └── theme/
│   │       └── app_theme.dart
│   ├── data/
│   │   └── models/
│   │       └── project_model.dart
│   └── features/
│       ├── editor/
│       │   ├── presentation/
│       │   │   └── editor_screen.dart
│       │   ├── providers/
│       │   │   ├── editor_provider.dart
│       │   │   ├── timeline_provider.dart
│       │   │   ├── engine_bridge_provider.dart
│       │   │   └── export_provider.dart
│       │   └── widgets/
│       │       ├── preview_viewport.dart
│       │       ├── timeline_panel.dart
│       │       ├── editor_toolbar.dart
│       │       ├── inspector_panel.dart
│       │       ├── clip_trim_handles.dart
│       │       ├── audio_waveform_painter.dart
│       │       ├── font_picker.dart
│       │       ├── text_overlay_handle.dart
│       │       ├── transition_picker.dart
│       │       ├── speed_curve_editor.dart
│       │       ├── keyframe_graph_editor.dart
│       │       └── subtitle_import.dart
│       ├── export/
│       │   └── presentation/
│       │       └── export_screen.dart
│       ├── onboarding/
│       │   ├── presentation/
│       │   │   └── onboarding_screen.dart
│       │   └── providers/
│       │       └── onboarding_provider.dart
│       ├── projects/
│       │   ├── presentation/
│       │   │   └── project_home_screen.dart
│       │   └── providers/
│       │       └── project_provider.dart
│       ├── settings/
│       │   ├── presentation/
│       │   │   └── settings_screen.dart
│       │   └── providers/
│       │       └── settings_provider.dart
│       └── templates/
│           └── presentation/
│               └── template_browser.dart
├── test/
│   └── widget_test.dart
├── pubspec.yaml
├── pubspec.lock
├── .gitignore
├── DEVELOPMENT_PLAN.md
├── AUDIT.md
├── AUDIT_REPORT.md
└── README.md
```

---

## Appendix B: Phase Dependency Graph

```
Phase 0 (Foundation)
    ↓
Phase 1 (Import & Preview)  ← MVP CRITICAL PATH START
    ↓
Phase 2 (Timeline & Trim)   ← MVP CRITICAL PATH
    ↓
Phase 3 (Export Pipeline)    ← MVP CRITICAL PATH END
    ↓
Phase 4 (Audio) ──────────── Phase 5 (Effects) ───── Phase 6 (Text)
    ↓                           ↓                       ↓
    └─────────────────────── Phase 7 (Speed & Keyframes)
                                ↓
                        Phase 8 (GPU Acceleration)
                                ↓
                        Phase 9 (Polish & Play Store)
                                ↓
                        Phase 10 (Advanced Features)
```

**MVP Critical Path:** Phases 0 → 1 → 2 → 3  
**Estimated MVP Duration:** 9-13 weeks  
**Estimated Full Release Duration:** 25-35 weeks  

---

## Appendix C: Weekly Milestone Checklist

Use this checklist at the end of each week to track progress:

```markdown
## Week [X] — Phase [Y]

### Completed This Week
- [ ] ...

### Blocked By
- [ ] ...

### Next Week Plan
- [ ] ...

### Phase Acceptance Criteria Status
- [ ] Criterion 1: ✅ / ❌ / 🔄
- [ ] Criterion 2: ✅ / ❌ / 🔄
- [ ] ...

### Performance Metrics
- Frame decode time: ___ms
- Preview FPS: ___fps
- Memory usage: ___MB
- APK size: ___MB
```

---

*This document is a living reference. Update it as phases are completed, decisions are revised, and new information emerges. After completing each phase, review the acceptance criteria, update the phase status, and confirm the next phase's tasks are still accurate.*
