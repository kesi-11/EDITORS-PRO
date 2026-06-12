//! Build script for editors-pro-engine.
//!
//! Generates C header files for the FFI bridge using cbindgen.

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
}
