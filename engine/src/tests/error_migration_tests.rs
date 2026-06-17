//! Tests for the Phase C.17 EngineError migration.
//!
//! These tests verify that:
//! 1. The new `Other` variant exists and round-trips through Display.
//! 2. `From<String> for EngineError` works (legacy `Result<_, String>`
//!    can be converted via `?`).
//! 3. `From<&str> for EngineError` works.
//! 4. The `?` operator works for converting `Result<T, String>` to
//!    `Result<T, EngineError>`.

use crate::EngineError;

#[test]
fn test_other_variant_display() {
    let err = EngineError::Other("something went wrong".to_string());
    assert_eq!(err.to_string(), "something went wrong");
}

#[test]
fn test_from_string_for_engine_error() {
    let err: EngineError = "legacy error message".to_string().into();
    match err {
        EngineError::Other(s) => assert_eq!(s, "legacy error message"),
        _ => panic!("expected Other variant"),
    }
}

#[test]
fn test_from_str_for_engine_error() {
    let err: EngineError = "literal error".into();
    match err {
        EngineError::Other(s) => assert_eq!(s, "literal error"),
        _ => panic!("expected Other variant"),
    }
}

#[test]
fn test_question_mark_operator_converts_string_to_engine_error() {
    fn legacy_function() -> Result<i32, String> {
        Err("legacy failure".to_string())
    }

    fn modern_function() -> Result<i32, EngineError> {
        // The `?` operator should convert `String` → `EngineError`
        // via the `From<String>` impl.
        let value = legacy_function()?;
        Ok(value)
    }

    let result = modern_function();
    assert!(result.is_err());
    match result.unwrap_err() {
        EngineError::Other(s) => assert_eq!(s, "legacy failure"),
        _ => panic!("expected Other variant from ? conversion"),
    }
}

#[test]
fn test_specific_variants_take_precedence_over_other() {
    // When migrating, callers should prefer specific variants.
    let decoder_err = EngineError::DecoderError("FFmpeg failed".to_string());
    assert_eq!(decoder_err.to_string(), "Decoder error: FFmpeg failed");

    let export_err = EngineError::ExportError("H.264 encoder not found".to_string());
    assert_eq!(export_err.to_string(), "Export error: H.264 encoder not found");
}

#[test]
fn test_io_error_auto_conversion() {
    // The `#[from] std::io::Error` attribute generates this impl.
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let engine_err: EngineError = io_err.into();
    match engine_err {
        EngineError::IoError(_) => {} // OK
        _ => panic!("expected IoError variant"),
    }
}

#[test]
fn test_serde_error_auto_conversion() {
    let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
    let engine_err: EngineError = json_err.into();
    match engine_err {
        EngineError::SerializationError(_) => {} // OK
        _ => panic!("expected SerializationError variant"),
    }
}

/// Verify that all 12 variants exist and can be constructed.
/// This guards against accidental removal of variants during refactors.
#[test]
fn test_all_variants_constructible() {
    let _ = EngineError::InitializationFailed("".to_string());
    let _ = EngineError::DecoderError("".to_string());
    let _ = EngineError::RendererError("".to_string());
    let _ = EngineError::ExportError("".to_string());
    let _ = EngineError::ProjectError("".to_string());
    let _ = EngineError::TimelineError("".to_string());
    let _ = EngineError::ProxyError("".to_string());
    let _ = EngineError::BridgeError("".to_string());
    let _ = EngineError::InvalidState("".to_string());
    let _ = EngineError::IoError(std::io::Error::new(std::io::ErrorKind::Other, ""));
    let _ = EngineError::SerializationError(serde_json::from_str::<()>("x").unwrap_err());
    let _ = EngineError::Other("".to_string());
}
