//! # EDITORS-PRO Engine
//!
//! Native video editing engine built in Rust for maximum performance
//! and memory safety. Designed to be called from Flutter via bridge.

pub mod api;
pub mod audio;
pub mod decoder;
pub mod effects;
pub mod export_engine;
pub mod project;
pub mod renderer;
pub mod subtitle;
pub mod system;
pub mod timeline;

use log::LevelFilter;

/// Initialize the editing engine.
/// Must be called once before any other engine operations.
/// Sets up logging and FFmpeg libraries.
pub fn init_engine() -> Result<(), EngineError> {
    // Initialize logger (only first call wins, repeated calls are safe)
    let _ = env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .try_init();

    log::info!(
        "EDITORS-PRO Engine v{} initializing",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize FFmpeg libraries
    ffmpeg_next::init().map_err(|e| {
        log::error!("Failed to initialize FFmpeg: {}", e);
        EngineError::InitializationFailed(format!("FFmpeg init error: {}", e))
    })?;

    log::info!("Engine initialized successfully");
    Ok(())
}

/// Get the engine version string
pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Top-level engine error type
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Decoder error: {0}")]
    DecoderError(String),

    #[error("Renderer error: {0}")]
    RendererError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Project error: {0}")]
    ProjectError(String),

    #[error("Timeline error: {0}")]
    TimelineError(String),

    #[error("Bridge error: {0}")]
    BridgeError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type EngineResult<T> = Result<T, EngineError>;
