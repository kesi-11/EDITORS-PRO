@echo off
REM ============================================================================
REM  EDITORS-PRO — Windows Build Setup
REM  This script helps you set up the build environment on Windows
REM ============================================================================
echo.
echo ============================================================
echo   EDITORS-PRO - Windows Build Setup
echo ============================================================
echo.

REM --- Step 1: Check prerequisites ---
echo [1/6] Checking prerequisites...

where flutter >nul 2>&1
if %errorlevel% neq 0 (
    echo   X Flutter not found. Install from https://docs.flutter.dev/get-started/install
    echo     After install, add Flutter to your PATH
    goto :error
)
echo   + Flutter found

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo   X Rust/Cargo not found. Install from https://rustup.rs
    goto :error
)
echo   + Rust/Cargo found

where cargo-ndk >nul 2>&1
if %errorlevel% neq 0 (
    echo   ! cargo-ndk not found. Installing...
    cargo install cargo-ndk
    if %errorlevel% neq 0 (
        echo   X Failed to install cargo-ndk
        goto :error
    )
)
echo   + cargo-ndk found

REM --- Step 2: Install Android target ---
echo.
echo [2/6] Ensuring Rust Android target is installed...
rustup target add aarch64-linux-android
echo   + aarch64-linux-android target ready

REM --- Step 3: Flutter dependencies ---
echo.
echo [3/6] Installing Flutter dependencies...
cd /d "%~dp0.."
flutter pub get
if %errorlevel% neq 0 (
    echo   X flutter pub get failed
    goto :error
)
echo   + Flutter dependencies installed

REM --- Step 4: FFmpeg setup ---
echo.
echo [4/6] Checking FFmpeg for Android...
echo.
echo   IMPORTANT: Building with FFmpeg requires Android FFmpeg libraries.
echo   You have TWO options:
echo.
echo   === OPTION A: Build WITHOUT FFmpeg (recommended for first build) ===
echo   This gives you a working app but video decode/encode will be stubs.
echo   Use this to verify the Flutter UI works before setting up FFmpeg.
echo.
echo     cd engine
echo     cargo ndk -t arm64-v8a build --release --no-default-features
echo.
echo   === OPTION B: Build WITH FFmpeg (full functionality) ===
echo   Step 1: Download FFmpeg Android arm64-v8a build from:
echo           https://github.com/nicehash/ffmpeg-kit/releases
echo   Step 2: Extract to: engine\ffmpeg-android\
echo           (lib\*.so and include\*.h)
echo   Step 3: Copy .so files to: android\app\src\main\jniLibs\arm64-v8a\
echo   Step 4: Set environment variables and build:
echo.
echo     set FFMPEG_DIR=%CD%\engine\ffmpeg-android
echo     set PKG_CONFIG_PATH=%FFMPEG_DIR%\lib\pkgconfig
echo     set PKG_CONFIG_SYSROOT_DIR=%FFMPEG_DIR%
echo     cd engine
echo     cargo ndk -t arm64-v8a build --release
echo.

REM --- Step 5: flutter_rust_bridge_codegen ---
echo [5/6] Checking flutter_rust_bridge_codegen...
where flutter_rust_bridge_codegen >nul 2>&1
if %errorlevel% neq 0 (
    echo   ! flutter_rust_bridge_codegen not found. Installing...
    cargo install flutter_rust_bridge_codegen
    if %errorlevel% neq 0 (
        echo   X Failed to install flutter_rust_bridge_codegen
        echo     Try: cargo install flutter_rust_bridge_codegen --version 2.9.0
        goto :error
    )
)
echo   + flutter_rust_bridge_codegen found

REM --- Step 6: Summary ---
echo.
echo [6/6] Setup Summary
echo.
echo ============================================================
echo   EDITORS-PRO Build Setup Complete!
echo.
echo   QUICK START (no FFmpeg, UI-only):
echo     cd engine
echo     cargo ndk -t arm64-v8a build --release --no-default-features
echo     cd ..
echo     flutter run
echo.
echo   FULL BUILD (with FFmpeg):
echo     1. Download FFmpeg Android arm64 libs (see above)
echo     2. set FFMPEG_DIR=path\to\ffmpeg-android
echo     3. cd engine ^&^& cargo ndk -t arm64-v8a build --release
echo     4. cd .. ^&^& flutter run
echo.
echo   BUILD APK:
echo     flutter build apk --release
echo ============================================================
echo.
goto :end

:error
echo.
echo Setup failed. Please fix the errors above and re-run.
echo.

:end
pause
