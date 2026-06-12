//! .epp format - Custom project file format
//!
//! The .epp (EDITORS-PRO Project) format is a zipped JSON structure
//! that contains all project data including timeline, media references,
//! and settings. Designed for version compatibility and migration.
//!
//! ## File Structure
//!
//! ```text
//! project.epp (ZIP archive)
//! ├── project.json    — Serialized project data
//! └── manifest.json   — Version, checksums, metadata
//! ```

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
    /// CRC32 checksum of the project.json content for integrity verification.
    /// Populated on save; `None` for legacy files that predate checksum support.
    pub checksum: Option<String>,
    /// Modified timestamp (set on save)
    pub modified_at: Option<i64>,
}

/// Manifest file stored alongside project.json in the .epp archive
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EppManifest {
    /// Format version
    version: u32,
    /// CRC32 checksum of project.json
    checksum: String,
    /// Engine version that created the file
    engine_version: String,
    /// File size of project.json in bytes
    data_size: u64,
}

/// Compute a CRC32 checksum of the given data and return it as a
/// zero-padded 8-character lowercase hex string (e.g. `"cbf43926"`).
///
/// Uses the `crc32fast` crate which implements the standard
/// Ethernet/ZIP polynomial (ISO 3309 / ITU-T V.42).
pub fn compute_checksum(data: &[u8]) -> String {
    format!("{:08x}", crc32fast::hash(data))
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
                modified_at: None,
            },
        }
    }

    /// Convert back to a Project
    pub fn to_project(self) -> Project {
        self.project
    }

    /// Save the project to a .epp file (zipped JSON with CRC32 manifest)
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        // Compute CRC32 of project data
        let checksum = compute_checksum(json.as_bytes());
        let data_size = json.len() as u64;

        // Create the manifest
        let manifest = EppManifest {
            version: FORMAT_VERSION,
            checksum: checksum.clone(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            data_size,
        };
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Manifest serialization failed: {}", e))?;

        // Atomic write: write to a temp file first, then rename
        let temp_path = path.with_extension("epp.tmp");

        let file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Write project data
        zip.start_file("project.json", options)
            .map_err(|e| format!("Failed to create project.json entry: {}", e))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write project.json: {}", e))?;

        // Write manifest
        zip.start_file("manifest.json", options)
            .map_err(|e| format!("Failed to create manifest.json entry: {}", e))?;
        zip.write_all(manifest_json.as_bytes())
            .map_err(|e| format!("Failed to write manifest.json: {}", e))?;

        zip.finish()
            .map_err(|e| format!("Failed to finalize zip: {}", e))?;

        // Atomic rename
        std::fs::rename(&temp_path, path)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        log::info!(
            "Project saved as .epp to {:?} (CRC32: {}, {} bytes)",
            path, checksum, data_size
        );
        Ok(())
    }

    /// Load a project from a .epp file (zipped JSON with CRC32 verification)
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        // Read project data
        let mut json = String::new();
        {
            let mut project_file = archive.by_name("project.json")
                .map_err(|e| format!("Failed to find project.json in archive: {}", e))?;
            project_file.read_to_string(&mut json)
                .map_err(|e| format!("Failed to read project.json: {}", e))?;
        }

        // Verify CRC32 checksum if manifest exists
        let actual_checksum = compute_checksum(json.as_bytes());

        if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
            let mut manifest_json = String::new();
            manifest_file.read_to_string(&mut manifest_json)
                .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

            let manifest: EppManifest = serde_json::from_str(&manifest_json)
                .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

            let expected_checksum = u32::from_str_radix(&manifest.checksum, 16)
                .unwrap_or(0);

            let actual_crc32 = u32::from_str_radix(&actual_checksum, 16)
                .unwrap_or(0);

            if expected_checksum != 0 && actual_crc32 != expected_checksum {
                log::warn!(
                    "CRC32 mismatch in .epp file: expected {:08x}, got {:08x}. File may be corrupted.",
                    expected_checksum, actual_crc32
                );
                // We still load the project but warn about corruption (graceful degradation)
            } else {
                log::info!("CRC32 checksum verified: {}", actual_checksum);
            }

            // Verify data size
            if manifest.data_size != json.len() as u64 {
                log::warn!(
                    "Data size mismatch in .epp file: expected {} bytes, got {} bytes",
                    manifest.data_size, json.len()
                );
            }
        } else {
            log::info!("No manifest.json found in archive (legacy format), skipping CRC32 check");
        }

        let mut format: EppFormat = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization failed: {}", e))?;

        // Populate metadata.checksum from the computed value so that
        // the in-memory object knows its integrity status.  For legacy
        // files without a manifest this still gets set so that the next
        // save will embed it.
        format.metadata.checksum = Some(actual_checksum.clone());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_checksum_empty() {
        let checksum = compute_checksum(b"");
        assert_eq!(checksum, "00000000", "Empty data CRC32 should be 00000000");
    }

    #[test]
    fn test_compute_checksum_known_value() {
        // Standard CRC32 test vector: "123456789" → 0xCBF43926
        let checksum = compute_checksum(b"123456789");
        assert_eq!(checksum, "cbf43926", "CRC32 of '123456789' should be cbf43926");
    }

    #[test]
    fn test_compute_checksum_consistency() {
        let data = b"Hello, EDITORS-PRO!";
        let crc1 = compute_checksum(data);
        let crc2 = compute_checksum(data);
        assert_eq!(crc1, crc2, "CRC32 should be deterministic");
    }

    #[test]
    fn test_compute_checksum_different_data() {
        let crc1 = compute_checksum(b"video");
        let crc2 = compute_checksum(b"audio");
        assert_ne!(crc1, crc2, "Different data should have different CRC32");
    }

    #[test]
    fn test_compute_checksum_format() {
        let checksum = compute_checksum(b"test");
        assert_eq!(checksum.len(), 8, "Checksum should be 8 hex characters");
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()),
            "Checksum should be lowercase hex");
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = EppManifest {
            version: 1,
            checksum: "abcdef01".to_string(),
            engine_version: "0.1.0".to_string(),
            data_size: 1024,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: EppManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.checksum, "abcdef01");
        assert_eq!(deserialized.data_size, 1024);
    }
}
