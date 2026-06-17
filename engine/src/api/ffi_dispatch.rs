//! JSON-RPC-style FFI dispatcher for the EDITORS-PRO engine.
//!
//! ## Why this exists
//!
//! `flutter_rust_bridge` v2 generates per-method Dart bindings from the
//! Rust source via `flutter_rust_bridge_codegen generate`. The previous
//! `lib/src/rust/frb_generated.dart` and `lib/src/rust/api/bridge_api.dart`
//! were stubs that threw `UnimplementedError` on every call, leaving the
//! entire engine unreachable from Flutter.
//!
//! As a Phase A pragmatic bridge, this module exposes a single C-ABI
//! function `editors_pro_dispatch` that accepts a JSON method name and
//! JSON arguments, dispatches to the corresponding `EditorsProEngineApi`
//! method, and returns a JSON response. The Dart side calls this via
//! `dart:ffi` — no codegen required.
//!
//! When the team later runs `flutter_rust_bridge_codegen generate`, the
//! generated per-method bindings will replace this dispatcher and the
//! Dart `_call` shim. The dispatcher remains as a fallback for
//! headless-test environments where FRB codegen isn't available.
//!
//! ## ABI
//!
//! ```c
//! char* editors_pro_dispatch(const char* method, const char* args_json);
//! ```
//!
//! - `method`: UTF-8 JSON string naming the engine method (e.g. `"initialize"`).
//! - `args_json`: UTF-8 JSON object string of method arguments.
//! - Returns: UTF-8 JSON string of `{"ok": true, "data": ...}` on success,
//!   or `{"ok": false, "error": "..."}` on failure. The caller MUST free
//!   the returned buffer with `editors_pro_free_string`.
//!
//! All dispatch calls are serialized through a global `Mutex<EditorsProEngineApi>`
//! to satisfy the `&self` contract of the bridge API.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::api::bridge_api::EditorsProEngineApi;

/// Payload returned by the dispatcher.
#[derive(serde::Serialize)]
struct DispatchOk<'a, T: serde::Serialize> {
    ok: bool,
    data: &'a T,
}

#[derive(serde::Serialize)]
struct DispatchErr {
    ok: bool,
    error: String,
}

/// Global engine instance.
///
/// Initialized lazily on first dispatch call. Using a `Lazy<Mutex<...>>`
/// keeps the FFI surface simple — there's exactly one engine per process,
/// which is the intended usage for a mobile app.
static ENGINE: Lazy<Mutex<EditorsProEngineApi>> =
    Lazy::new(|| Mutex::new(EditorsProEngineApi::new()));

/// Convert a Rust `Result<T, String>` into a JSON string suitable for
/// returning across the FFI boundary.
fn to_json_string<T: serde::Serialize>(result: Result<T, String>) -> String {
    match result {
        Ok(value) => to_json_ok(&value),
        Err(err) => to_json_err(err),
    }
}

/// Serialize a successful value as `{"ok": true, "data": ...}`.
fn to_json_ok<T: serde::Serialize>(value: &T) -> String {
    let wrapper = DispatchOk { ok: true, data: value };
    serde_json::to_string(&wrapper).unwrap_or_else(|e| {
        to_json_err(format!("Serialization failed: {}", e))
    })
}

/// Serialize an error as `{"ok": false, "error": "..."}`.
fn to_json_err(msg: impl Into<String>) -> String {
    serde_json::to_string(&DispatchErr {
        ok: false,
        error: msg.into(),
    })
    .unwrap_or_else(|_| r#"{"ok":false,"error":"unknown"}"#.to_string())
}

/// Dispatch a JSON method call to the engine.
///
/// See module docs for the ABI contract.
///
/// # Safety
///
/// - `method` and `args_json` must be valid non-null pointers to NUL-terminated
///   UTF-8 C strings.
/// - The returned pointer MUST be freed by `editors_pro_free_string` to avoid
///   a memory leak.
#[no_mangle]
pub unsafe extern "C" fn editors_pro_dispatch(
    method: *const c_char,
    args_json: *const c_char,
) -> *mut c_char {
    // Parse method name.
    let method_str = if method.is_null() {
        return leak_cstring(r#"{"ok":false,"error":"method is null"}"#);
    } else {
        match CStr::from_ptr(method).to_str() {
            Ok(s) => s,
            Err(_) => {
                return leak_cstring(
                    r#"{"ok":false,"error":"method is not valid UTF-8"}"#,
                );
            }
        }
    };

    // Parse args JSON. Null or empty is treated as `{}`.
    let args_str: &str = if args_json.is_null() {
        "{}"
    } else {
        match CStr::from_ptr(args_json).to_str() {
            Ok(s) if s.is_empty() => "{}",
            Ok(s) => s,
            Err(_) => {
                return leak_cstring(
                    r#"{"ok":false,"error":"args_json is not valid UTF-8"}"#,
                );
            }
        }
    };

    let args: serde_json::Value = match serde_json::from_str(args_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!(r#"{{"ok":false,"error":"args_json parse error: {}"}}"#, e);
            return leak_cstring(&msg);
        }
    };

    // Acquire engine lock and dispatch.
    let result = std::panic::catch_unwind(|| {
        let mut api = ENGINE.lock().expect("engine mutex poisoned");
        dispatch_method(&mut api, method_str, &args)
    });

    let json_str = match result {
        Ok(json) => json,
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Rust panic during dispatch".to_string()
            };
            to_json_string::<()>(Err(format!("Panic: {}", msg)))
        }
    };

    leak_cstring(&json_str)
}

/// Free a string returned by `editors_pro_dispatch`.
///
/// # Safety
///
/// `ptr` must be a non-null pointer previously returned by
/// `editors_pro_dispatch`. Passing any other pointer is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn editors_pro_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // Reconstruct the CString and drop it to reclaim memory.
    let _ = CString::from_raw(ptr);
}

/// Leak a `CString` so its memory is owned by the caller.
///
/// The caller MUST free it with `editors_pro_free_string`.
fn leak_cstring(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new(r#"{"ok":false,"error":"internal nul error"}"#).unwrap())
        .into_raw()
}

/// Dispatch a single method call. Returns the JSON response string.
///
/// This is a giant match on `method`. Only a subset of the full 60+ API
/// is wired here initially — enough to support the Phase A end-to-end
/// smoke test (initialize → create_project → import_media → add_track →
/// add_clip → trim_clip → split_clip → get_frame → export). The remaining
/// methods will be added as they're needed, or replaced entirely when
/// `flutter_rust_bridge_codegen generate` is run.
fn dispatch_method(
    api: &mut EditorsProEngineApi,
    method: &str,
    args: &serde_json::Value,
) -> String {
    match method {
        // ─── Lifecycle ────────────────────────────────────────────────────
        "initialize" => to_json_string::<()>(api.initialize()),
        "get_engine_version" => to_json_ok(&crate::engine_version().to_string()),
        "force_reset_engine" => to_json_string::<()>(api.force_reset_engine()),
        "recover_project" => {
            let path: Option<String> = args
                .get("auto_save_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            to_json_string(api.recover_project(path))
        }

        // ─── Project Operations ───────────────────────────────────────────
        "create_project" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string();
            let settings = args.get("settings").and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    serde_json::from_value(v.clone()).ok()
                }
            });
            to_json_string(api.create_project(name, settings))
        }
        "save_project" => {
            let path = match args.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => return to_json_err("missing 'path'"),
            };
            to_json_string::<()>(api.save_project(path))
        }
        "load_project" => {
            let path = match args.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => return to_json_err("missing 'path'"),
            };
            to_json_string(api.load_project(path))
        }
        "get_project_info" => to_json_ok(&api.get_project_info()),
        "get_timeline_duration" => to_json_ok(&api.get_timeline_duration()),
        "can_undo" => to_json_ok(&api.can_undo()),
        "can_redo" => to_json_ok(&api.can_redo()),

        // ─── Media ────────────────────────────────────────────────────────
        "import_media" => {
            let path = match args.get("file_path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => return to_json_err("missing 'file_path'"),
            };
            to_json_string(api.import_media(path))
        }

        // ─── Timeline ─────────────────────────────────────────────────────
        "add_track" => {
            let track_type = match args.get("track_type").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => return to_json_err("missing 'track_type'"),
            };
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            to_json_string(api.add_track(track_type, name))
        }
        "add_clip" => {
            let track_id = match args.get("track_id").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => return to_json_err("missing 'track_id'"),
            };
            let asset_id = match args.get("asset_id").and_then(|v| v.as_str()) {
                Some(a) => a.to_string(),
                None => return to_json_err("missing 'asset_id'"),
            };
            let start_ms = parse_u64(args, "start_ms");
            let duration_ms = parse_u64(args, "duration_ms");
            to_json_string(api.add_clip(track_id, asset_id, start_ms, duration_ms))
        }
        "trim_clip" => {
            let clip_id = match args.get("clip_id").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => return to_json_err("missing 'clip_id'"),
            };
            let trim_start_ms = parse_u64(args, "trim_start_ms");
            let trim_end_ms = parse_u64(args, "trim_end_ms");
            to_json_string::<()>(api.trim_clip(clip_id, trim_start_ms, trim_end_ms))
        }
        "split_clip" => {
            let clip_id = match args.get("clip_id").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => return to_json_err("missing 'clip_id'"),
            };
            let time_ms = parse_u64(args, "time_ms");
            to_json_string(api.split_clip(clip_id, time_ms))
        }
        "remove_clip" => {
            let clip_id = match args.get("clip_id").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => return to_json_err("missing 'clip_id'"),
            };
            to_json_string::<()>(api.remove_clip(clip_id))
        }
        "get_timeline_state" => to_json_ok(&api.get_timeline_state()),
        "undo" => to_json_string::<()>(api.undo()),
        "redo" => to_json_string::<()>(api.redo()),

        // ─── Cache management (Phase B.12) ────────────────────────────────
        // Mutation methods (add_clip, trim_clip, etc.) call
        // `invalidate_frame_cache` automatically before returning, so
        // the next `get_frame` re-decodes from source. Exposing it
        // here lets the Dart side manually flush the cache if it
        // suspects staleness (e.g. after a file was modified on disk).
        "invalidate_frame_cache" => to_json_string::<()>(api.invalidate_frame_cache()),

        // ─── Preview ──────────────────────────────────────────────────────
        "get_frame" => {
            let time_ms = parse_u64(args, "time_ms");
            to_json_string(api.get_frame(time_ms))
        }

        // ─── Export ───────────────────────────────────────────────────────
        "export_video" => {
            let output_path = match args.get("output_path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => return to_json_err("missing 'output_path'"),
            };
            let settings = match args.get("settings") {
                Some(v) if !v.is_null() => match serde_json::from_value(v.clone()) {
                    Ok(s) => s,
                    Err(e) => return to_json_err(format!("invalid 'settings': {}", e)),
                },
                _ => return to_json_err("missing 'settings'"),
            };
            to_json_string(api.export_video(output_path, settings))
        }
        "cancel_export" => to_json_string::<()>(api.cancel_export()),

        // ─── GPU ──────────────────────────────────────────────────────────
        "is_gpu_available" => to_json_ok(&api.is_gpu_available()),
        "get_gpu_info" => to_json_ok(&api.get_gpu_info()),
        "set_gpu_acceleration" => {
            let enabled = args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            to_json_string::<()>(api.set_gpu_acceleration(enabled))
        }

        // ─── Profiling (free functions, not methods on EditorsProEngineApi) ───
        "set_profiling_enabled" => {
            let enabled = args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            crate::api::bridge_api::set_profiling_enabled(enabled);
            to_json_ok(&())
        }
        "is_profiling_enabled" => to_json_ok(&crate::api::bridge_api::is_profiling_enabled()),
        "get_performance_snapshot" => {
            to_json_ok(&crate::api::bridge_api::get_performance_snapshot())
        }
        "get_profiler_report" => {
            to_json_ok(&crate::api::bridge_api::get_profiler_report())
        }
        "reset_profiler" => {
            crate::api::bridge_api::reset_profiler();
            to_json_ok(&())
        }

        // ─── System metrics ───────────────────────────────────────────────
        "get_system_metrics" => to_json_ok(&api.get_system_metrics()),
        "is_memory_pressure" => to_json_ok(&api.is_memory_pressure()),
        "get_memory_pressure_level" => {
            to_json_ok(&crate::api::bridge_api::get_memory_pressure_level())
        }
        "get_memory_usage_bytes" => {
            to_json_ok(&crate::api::bridge_api::get_memory_usage_bytes())
        }

        // ─── Catch-all ────────────────────────────────────────────────────
        _ => to_json_err(format!("unknown method: {}", method)),
    }
}

/// Parse a u64 from a JSON value, accepting both `u64` and numeric strings
/// (since Dart's `BigInt` is serialized as a string for values > 2^53).
fn parse_u64(args: &serde_json::Value, key: &str) -> u64 {
    args.get(key)
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
        })
        .unwrap_or(0)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_json_string_ok() {
        let s = to_json_string(Ok(42u64));
        assert!(s.contains(r#""ok":true"#));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_to_json_string_err() {
        let s = to_json_string::<u64>(Err("boom".to_string()));
        assert!(s.contains(r#""ok":false"#));
        assert!(s.contains("boom"));
    }

    #[test]
    fn test_parse_u64_from_number() {
        let args = serde_json::json!({"start_ms": 12345});
        assert_eq!(parse_u64(&args, "start_ms"), 12345);
    }

    #[test]
    fn test_parse_u64_from_string() {
        // BigInt-serialized values come through as strings.
        let args = serde_json::json!({"start_ms": "12345"});
        assert_eq!(parse_u64(&args, "start_ms"), 12345);
    }

    #[test]
    fn test_parse_u64_missing_key() {
        let args = serde_json::json!({});
        assert_eq!(parse_u64(&args, "start_ms"), 0);
    }

    /// End-to-end smoke test: dispatch the `initialize` method through the
    /// FFI boundary and verify we get a JSON-serialized success response.
    /// This exercises the global Mutex<EditorsProEngineApi> as well as the
    /// JSON serialization path.
    #[test]
    fn test_dispatch_initialize() {
        let method = CString::new("initialize").unwrap();
        let args = CString::new("{}").unwrap();
        let raw = unsafe {
            editors_pro_dispatch(method.as_ptr(), args.as_ptr())
        };
        assert!(!raw.is_null());
        let s = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe { editors_pro_free_string(raw) };
        assert!(s.contains(r#""ok":true"#), "got: {}", s);
    }

    /// End-to-end smoke test: dispatch an unknown method and verify we get
    /// a graceful error rather than a panic.
    #[test]
    fn test_dispatch_unknown_method() {
        let method = CString::new("definitely_not_a_real_method").unwrap();
        let args = CString::new("{}").unwrap();
        let raw = unsafe {
            editors_pro_dispatch(method.as_ptr(), args.as_ptr())
        };
        assert!(!raw.is_null());
        let s = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe { editors_pro_free_string(raw) };
        assert!(s.contains(r#""ok":false"#), "got: {}", s);
        assert!(s.contains("unknown method"), "got: {}", s);
    }

    /// Verify that dispatching to `get_engine_version` returns a string
    /// matching the crate version.
    #[test]
    fn test_dispatch_get_engine_version() {
        let method = CString::new("get_engine_version").unwrap();
        let args = CString::new("{}").unwrap();
        let raw = unsafe {
            editors_pro_dispatch(method.as_ptr(), args.as_ptr())
        };
        assert!(!raw.is_null());
        let s = unsafe { CStr::from_ptr(raw) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe { editors_pro_free_string(raw) };
        assert!(s.contains(r#""ok":true"#), "got: {}", s);
        assert!(
            s.contains(env!("CARGO_PKG_VERSION")),
            "expected version {}, got: {}",
            env!("CARGO_PKG_VERSION"),
            s
        );
    }
}
