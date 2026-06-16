//! Build script for editors-pro-engine.
//!
//! Generates C header files for the FFI bridge using cbindgen.
//! When the `ffmpeg` feature is enabled for Android cross-compilation,
//! configures the correct library search paths.

fn main() {
    // Only generate headers if cbindgen is available and we're not
    // in a check/build that doesn't need it.
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    match cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
    {
        Ok(bindings) => {
            let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "target".to_string());
            let header_path = format!("{}/editors_pro_engine.h", out_dir);
            bindings.write_to_file(&header_path);
            println!("cargo:warning=Generated C header: {}", header_path);
        }
        Err(e) => {
            // cbindgen may fail if the crate doesn't compile yet.
            // This is not a hard error for the build.
            println!("cargo:warning=cbindgen failed: {}", e);
        }
    }

    // When building for Android with FFmpeg, set up library paths
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
}
