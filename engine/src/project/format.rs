//! .epp format - Custom project file format
//!
//! The .epp (EDITORS-PRO Project) format is a zipped JSON structure
//! that contains all project data including timeline, media references,
//! and settings. Designed for version compatibility and migration.

use serde::{Deserialize, Serialize};
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::Path;

use super::Project;

/// Current format version
const FORMAT_VERSION: u32 = 1;

/// The .epp file format structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EppFormat {
    /// Format version for migration support
    pub version: u32,
    /// The project data
    pub project: Project,
    /// Format-specific metadata
    pub metadata: EppMetadata,
}

/// Metadata about the .epp file itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EppMetadata {
    pub created_with: String,
    pub created_at: i64,
    pub checksum: Option<String>,
}

impl EppFormat {
    /// Create an EppFormat from a Project
    pub fn from_project(project: &Project) -> Self {
        Self {
            version: FORMAT_VERSION,
            project: project.clone(),
            metadata: EppMetadata {
                created_with: format!("EDITORS-PRO Engine v{}", env!("CARGO_PKG_VERSION")),
                created_at: chrono::Utc::now().timestamp_millis(),
                checksum: None,
            },
        }
    }

    /// Convert back to a Project
    pub fn to_project(self) -> Project {
        self.project
    }

    /// Save the project to a .epp file (zipped JSON)
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let file = std::fs::File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("project.json", options)
            .map_err(|e| format!("Failed to create zip entry: {}", e))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write zip entry: {}", e))?;
        zip.finish()
            .map_err(|e| format!("Failed to finalize zip: {}", e))?;

        log::info!("Project saved as .epp (zipped) to {:?}", path);
        Ok(())
    }

    /// Load a project from a .epp file (zipped JSON)
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        let mut json = String::new();
        {
            let mut project_file = archive.by_name("project.json")
                .map_err(|e| format!("Failed to find project.json in archive: {}", e))?;
            project_file.read_to_string(&mut json)
                .map_err(|e| format!("Failed to read project.json: {}", e))?;
        }

        let format: EppFormat = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization failed: {}", e))?;

        // Check version compatibility
        if format.version > FORMAT_VERSION {
            log::warn!(
                "Project file version {} is newer than engine version {}. Some features may not work.",
                format.version, FORMAT_VERSION
            );
        }

        // Run migrations if needed
        let migrated = Self::migrate(format)?;

        log::info!("Project loaded from .epp file: {:?}", path);
        Ok(migrated)
    }

    /// Migrate project data from older format versions
    fn migrate(format: EppFormat) -> Result<EppFormat, String> {
        match format.version {
            1 => Ok(format), // Current version, no migration needed
            v if v > FORMAT_VERSION => {
                // Future version - try to load with warnings
                log::warn!("Loading project from future format version {}", v);
                Ok(format)
            }
            0 => {
                // Legacy format migration
                log::info!("Migrating project from version 0 to version 1");
                Ok(EppFormat {
                    version: FORMAT_VERSION,
                    project: format.project,
                    metadata: format.metadata,
                })
            }
            _ => Err(format!("Unknown format version: {}", format.version)),
        }
    }

    /// Get the current format version
    pub fn current_version() -> u32 {
        FORMAT_VERSION
    }
}
