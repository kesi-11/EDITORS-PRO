//! Build script for editors-pro-engine.
//!
//! Previously this file ran `cbindgen` to generate C headers for an FFI
//! bridge. As of Phase A of the upgrade plan, cbindgen has been removed
//! because `flutter_rust_bridge` v2 generates its own bindings from the
//! Rust source directly — there is no need for a separate C header.
//!
//! The only remaining responsibility of this build script is to wire up
//! FFmpeg library search paths when cross-compiling for Android with the
//! `ffmpeg` feature enabled.

fn main() {
    // docs.rs builds don't need any linker configuration.
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    // When building for Android with FFmpeg, set up library paths.
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("android") && std::env::var("CARGO_FEATURE_FFMPEG").is_ok() {
        // Check for FFMPEG_DIR environment variable
        if let Ok(ffmpeg_dir) = std::env::var("FFMPEG_DIR") {
            let ffmpeg_path = std::path::Path::new(&ffmpeg_dir);

            // Add library search path
            let lib_dir = ffmpeg_path.join("lib");
            if lib_dir.exists() {
                println!("cargo:rustc-link-search=native={}", lib_dir.display());
                println!("cargo:warning=FFmpeg lib dir: {}", lib_dir.display());
            }

            // Add include path
            let include_dir = ffmpeg_path.join("include");
            if include_dir.exists() {
                // This helps ffmpeg-sys-next find headers during build
                println!("cargo:warning=FFmpeg include dir: {}", include_dir.display());
            }

            // Link FFmpeg libraries
            println!("cargo:rustc-link-lib=avcodec");
            println!("cargo:rustc-link-lib=avformat");
            println!("cargo:rustc-link-lib=avutil");
            println!("cargo:rustc-link-lib=swscale");
            println!("cargo:rustc-link-lib=swresample");
            println!("cargo:rustc-link-lib=avfilter");
        } else {
            println!("cargo:warning=FFMPEG_DIR not set. FFmpeg libraries may not be found.");
            println!("cargo:warning=Set FFMPEG_DIR to the directory containing lib/ and include/");
            println!("cargo:warning=Or build without FFmpeg: cargo ndk -t arm64-v8a build --release --no-default-features");
        }
    }

    // Re-run if these env vars change.
    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
}
