# EDITORS-PRO — Build Guide

## Quick Start (No FFmpeg — UI Only)

This is the **easiest way to get the app running** on your device. Video decode/encode will be stubs, but the full UI works.

```cmd
cd C:\Users\moham\OneDrive\Desktop\EDITORS-PRO

:: Step 1: Get Flutter dependencies
flutter pub get

:: Step 2: Build Rust engine WITHOUT FFmpeg
cd engine
cargo ndk -t arm64-v8a build --release --no-default-features
cd ..

:: Step 3: Copy the compiled engine to jniLibs
mkdir android\app\src\main\jniLibs\arm64-v8a
copy engine\target\aarch64-linux-android\release\libeditors_pro_engine.so android\app\src\main\jniLibs\arm64-v8a\

:: Step 4: Run the app
flutter run
```

## Full Build (With FFmpeg — Complete Functionality)

### Prerequisites

1. **Flutter SDK 3.x** — https://docs.flutter.dev/get-started/install
2. **Android Studio** with Android SDK (API 34+)
3. **Rust** — https://rustup.rs
4. **cargo-ndk** — `cargo install cargo-ndk`
5. **flutter_rust_bridge_codegen** — `cargo install flutter_rust_bridge_codegen --version 2.9.0`
6. **FFmpeg for Android arm64-v8a** (see below)

### Step 1: Install Rust Android Target

```cmd
rustup target add aarch64-linux-android
```

### Step 2: Get FFmpeg for Android

#### Option A: Download Pre-built FFmpeg (Recommended)

1. Download from **ffmpeg-kit** releases:
   - https://github.com/nicehash/ffmpeg-kit/releases
   - Get the `ffmpeg-kit-full-gpl-*.android-arm64-v8a.zip`

2. Extract and organize:
   ```
   engine/ffmpeg-android/
   ├── lib/
   │   ├── libavcodec.so (or .a)
   │   ├── libavformat.so
   │   ├── libavutil.so
   │   ├── libswscale.so
   │   ├── libswresample.so
   │   ├── libavfilter.so
   │   └── pkgconfig/
   │       ├── libavcodec.pc
   │       ├── libavformat.pc
   │       └── ...
   └── include/
       ├── libavcodec/
       ├── libavformat/
       ├── libavutil/
       └── ...
   ```

3. Copy FFmpeg .so files to Android jniLibs:
   ```cmd
   mkdir android\app\src\main\jniLibs\arm64-v8a
   copy engine\ffmpeg-android\lib\*.so android\app\src\main\jniLibs\arm64-v8a\
   ```

#### Option B: Build FFmpeg from Source (Advanced)

Use the Docker method (see below) or build FFmpeg manually:
```bash
# On Linux or WSL2
git clone https://github.com/nicehash/FFmpeg-Android
cd FFmpeg-Android
./build.sh -a arm64-v8a
```

### Step 3: Build the Engine with FFmpeg

```cmd
:: Set environment variables for FFmpeg
set FFMPEG_DIR=C:\Users\moham\OneDrive\Desktop\EDITORS-PRO\engine\ffmpeg-android
set PKG_CONFIG_PATH=%FFMPEG_DIR%\lib\pkgconfig
set PKG_CONFIG_SYSROOT_DIR=%FFMPEG_DIR%

:: Build the engine
cd engine
cargo ndk -t arm64-v8a build --release
cd ..
```

### Step 4: Copy Engine and Run

```cmd
:: Copy the compiled engine
copy engine\target\aarch64-linux-android\release\libeditors_pro_engine.so android\app\src\main\jniLibs\arm64-v8a\

:: Run
flutter run
```

### Step 5: Build Release APK

```cmd
flutter build apk --release
```

The APK will be at: `build\app\outputs\flutter-apk\app-release.apk`

---

## Docker Build (Most Reliable — Linux/Mac/WSL2)

This avoids all Windows cross-compilation issues by building in a Docker container.

```bash
# Build the Docker image (one-time)
docker build -t editors-pro-builder -f Dockerfile.build .

# Run the build (mounts your project directory)
docker run --rm -v "$(pwd):/project" editors-pro-builder

# Then on your host machine:
flutter run
```

---

## WSL2 Build (Windows Users)

If you're on Windows, using WSL2 for the Rust build is recommended:

```bash
# In WSL2 (Ubuntu)
sudo apt update && sudo apt install -y build-essential cmake pkg-config yasm nasm

# Install Android NDK
wget https://dl.google.com/android/repository/android-ndk-r26d-linux.zip
unzip android-ndk-r26d-linux.zip
export ANDROID_NDK_HOME=$PWD/android-ndk-r26d

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add aarch64-linux-android
cargo install cargo-ndk

# Build FFmpeg for Android (see Dockerfile.build for the configure flags)
# Then build the engine:
cd /mnt/c/Users/moham/OneDrive/Desktop/EDITORS-PRO/engine
cargo ndk -t arm64-v8a build --release
```

Then switch back to Windows CMD for Flutter:
```cmd
flutter run
```

---

## Troubleshooting

### Error: `pkg-config has not been configured to support cross-compilation`
**Solution**: You need FFmpeg libraries compiled for Android. Either:
1. Build without FFmpeg: `cargo ndk -t arm64-v8a build --release --no-default-features`
2. Use the Docker build method
3. Use WSL2 and build FFmpeg for Android

### Error: `Could not find ffmpeg with vcpkg`
**Solution**: This means FFmpeg C libraries aren't found. Set `FFMPEG_DIR` environment variable.

### Error: `rustup target install aarch64-linux-android`
**Solution**: Run `rustup target add aarch64-linux-android`

### Flutter can't find the native library
**Solution**: Make sure `libeditors_pro_engine.so` is in:
```
android/app/src/main/jniLibs/arm64-v8a/
```

### Build is slow
**Solution**: Use `--release` for smaller, faster binaries. Debug builds are much larger.

---

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `ffmpeg` | ✅ Yes | Enables FFmpeg video decode/encode. Requires FFmpeg C libraries. |
| (no features) | — | Build without FFmpeg. UI works, video I/O returns errors. |

Build commands:
```cmd
:: With FFmpeg (full functionality)
cargo ndk -t arm64-v8a build --release

:: Without FFmpeg (dev/UI testing)
cargo ndk -t arm64-v8a build --release --no-default-features
```
