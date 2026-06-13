use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crc32fast::Hasher;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

use super::timeline::Timeline;

/// Project metadata stored alongside the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub version: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            author: String::new(),
            description: String::new(),
            tags: Vec::new(),
            version: crate::ENGINE_VERSION.to_string(),
        }
    }
}

/// The top-level project structure representing an EDITORS-PRO project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub timeline: Timeline,
    pub metadata: HashMap<String, String>,
    pub project_metadata: ProjectMetadata,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project with the given name and default settings.
    pub fn new(name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            width: 1920,
            height: 1080,
            fps: 30.0,
            timeline: Timeline::new(),
            metadata: HashMap::new(),
            project_metadata: ProjectMetadata::default(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Create a new project with custom resolution and fps.
    pub fn with_settings(name: &str, width: u32, height: u32, fps: f64) -> Self {
        let mut project = Self::new(name);
        project.width = width;
        project.height = height;
        project.fps = fps;
        project
    }

    /// Save the project to a .epp file (ZIP containing project.json + CRC32 checksum).
    pub fn save(&self, path: &str) -> Result<()> {
        let json_data = serde_json::to_string_pretty(self)
            .context("Failed to serialize project to JSON")?;

        // Compute CRC32 of the JSON data
        let mut hasher = Hasher::new();
        hasher.update(json_data.as_bytes());
        let crc = hasher.finalize();

        // Create ZIP archive
        let file = fs::File::create(path)
            .context(format!("Failed to create file: {}", path))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Write project.json
        zip.start_file("project.json", options)
            .context("Failed to start project.json in ZIP")?;
        zip.write_all(json_data.as_bytes())
            .context("Failed to write project.json")?;

        // Write checksum.crc32
        zip.start_file("checksum.crc32", options)
            .context("Failed to start checksum.crc32 in ZIP")?;
        zip.write_all(format!("{}", crc).as_bytes())
            .context("Failed to write checksum.crc32")?;

        // Write manifest
        zip.start_file("manifest.json", options)
            .context("Failed to start manifest.json in ZIP")?;
        let manifest = serde_json::json!({
            "version": crate::ENGINE_VERSION,
            "format": "epp",
            "created_at": self.created_at.to_rfc3339(),
            "modified_at": self.modified_at.to_rfc3339(),
        });
        zip.write_all(manifest.to_string().as_bytes())
            .context("Failed to write manifest.json")?;

        zip.finish()
            .context("Failed to finalize ZIP archive")?;

        info!("Project saved to {} (CRC32: {:08X})", path, crc);
        Ok(())
    }

    /// Load a project from a .epp file (verify CRC32, deserialize JSON from ZIP).
    pub fn load(path: &str) -> Result<Self> {
        let file = fs::File::open(path)
            .context(format!("Failed to open file: {}", path))?;
        let mut archive = zip::ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;

        // Read project.json
        let mut json_data = String::new();
        {
            let mut project_file = archive
                .by_name("project.json")
                .context("project.json not found in archive")?;
            project_file
                .read_to_string(&mut json_data)
                .context("Failed to read project.json")?;
        }

        // Verify CRC32
        {
            let mut checksum_data = String::new();
            let mut checksum_file = archive
                .by_name("checksum.crc32")
                .context("checksum.crc32 not found in archive")?;
            checksum_file
                .read_to_string(&mut checksum_data)
                .context("Failed to read checksum.crc32")?;

            let stored_crc: u32 = checksum_data
                .trim()
                .parse::<u32>()
                .context("Invalid CRC32 value in checksum file")?;

            let mut hasher = Hasher::new();
            hasher.update(json_data.as_bytes());
            let computed_crc = hasher.finalize();

            if stored_crc != computed_crc {
                anyhow::bail!(
                    "CRC32 mismatch: stored={:08X}, computed={:08X}. File may be corrupted.",
                    stored_crc,
                    computed_crc
                );
            }

            debug!("CRC32 verified: {:08X}", computed_crc);
        }

        let project: Project = serde_json::from_str(&json_data)
            .context("Failed to deserialize project JSON")?;

        info!("Project loaded from {}: {}", path, project.name);
        Ok(project)
    }

    /// Export the project as a basic FCPXML string.
    pub fn export_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<fcpxml version=\"1.9\">\n");
        xml.push_str("  <resources>\n");
        xml.push_str(&format!(
            "    <format id=\"r1\" name=\"FFVideoFormat{}p{}\" frameDuration=\"{}/{:.0}s\" width=\"{}\" height=\"{}\"/>\n",
            self.height,
            self.fps,
            100,
            100.0 / self.fps,
            self.width,
            self.height
        ));
        xml.push_str("  </resources>\n");
        xml.push_str("  <library>\n");
        xml.push_str(&format!("    <event name=\"{}\">\n", self.name));
        xml.push_str("      <project name=\"\">\n");
        xml.push_str("        <sequence format=\"r1\">\n");
        xml.push_str("          <spine>\n");

        for (track_idx, track) in self.timeline.tracks.iter().enumerate() {
            xml.push_str(&format!(
                "            <lane offset=\"{}\">\n",
                track_idx
            ));
            for clip in &track.clips {
                xml.push_str(&format!(
                    "              <asset-clip ref=\"{}\" offset=\"{}s\" duration=\"{}s\" start=\"{}s\"/>\n",
                    clip.source_path,
                    0.0, // offset on timeline
                    clip.get_duration(),
                    clip.trim_in
                ));
            }
            xml.push_str("            </lane>\n");
        }

        xml.push_str("          </spine>\n");
        xml.push_str("        </sequence>\n");
        xml.push_str("      </project>\n");
        xml.push_str("    </event>\n");
        xml.push_str("  </library>\n");
        xml.push_str("</fcpxml>\n");
        xml
    }

    /// Export as EDL (Edit Decision List) format.
    pub fn export_edl(&self) -> String {
        let mut edl = String::new();
        edl.push_str("TITLE: ");
        edl.push_str(&self.name);
        edl.push('\n');

        let mut event_num = 1;
        for track in &self.timeline.tracks {
            for clip in &track.clips {
                edl.push_str(&format!(
                    "{:03}  AX       V     C        {:08} {:08} {:08} {:08}\n",
                    event_num,
                    Self::frames_from_time(clip.trim_in, self.fps),
                    Self::frames_from_time(clip.trim_out, self.fps),
                    Self::frames_from_time(0.0, self.fps),
                    Self::frames_from_time(clip.get_duration(), self.fps),
                ));
                edl.push_str(&format!(
                    "* FROM CLIP NAME: {}\n",
                    clip.source_path
                ));
                event_num += 1;
            }
        }
        edl
    }

    fn frames_from_time(time: f64, fps: f64) -> u32 {
        (time * fps).round() as u32
    }

    /// Update the modified timestamp.
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("Test Project");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.width, 1920);
        assert_eq!(project.height, 1080);
        assert_eq!(project.fps, 30.0);
    }

    #[test]
    fn test_project_with_settings() {
        let project = Project::with_settings("4K Project", 3840, 2160, 60.0);
        assert_eq!(project.width, 3840);
        assert_eq!(project.height, 2160);
        assert_eq!(project.fps, 60.0);
    }

    #[test]
    fn test_project_save_and_load() {
        let dir = std::env::temp_dir().join("editors_pro_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_project.epp");

        let project = Project::new("Save Test");
        let save_result = project.save(path.to_str().unwrap());
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result.err());

        let loaded = Project::load(path.to_str().unwrap());
        assert!(loaded.is_ok(), "Load failed: {:?}", loaded.err());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "Save Test");
        assert_eq!(loaded.id, project.id);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_project_load_corrupted() {
        let dir = std::env::temp_dir().join("editors_pro_corrupt_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("corrupted.epp");

        // Create a project, save it, then corrupt the data
        let project = Project::new("Corrupt Test");
        project.save(path.to_str().unwrap()).unwrap();

        // Modify the ZIP to corrupt the project.json
        let file = fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut json_data = String::new();
        archive.by_name("project.json").unwrap().read_to_string(&mut json_data).unwrap();
        // Corrupt: change the name
        let corrupted = json_data.replace("Corrupt Test", "CORRUPTED");
        // Rebuild the ZIP with wrong CRC
        let out_file = fs::File::create(&path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(out_file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip_writer.start_file("project.json", options).unwrap();
        zip_writer.write_all(corrupted.as_bytes()).unwrap();
        zip_writer.start_file("checksum.crc32", options).unwrap();
        // Write the old CRC (which won't match the new data)
        let mut hasher = Hasher::new();
        hasher.update(json_data.as_bytes());
        let old_crc = hasher.finalize();
        zip_writer.write_all(format!("{}", old_crc).as_bytes()).unwrap();
        zip_writer.finish().unwrap();

        let result = Project::load(path.to_str().unwrap());
        assert!(result.is_err(), "Should fail CRC check");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_project_metadata_default() {
        let meta = ProjectMetadata::default();
        assert!(meta.author.is_empty());
        assert!(meta.description.is_empty());
        assert!(meta.tags.is_empty());
    }

    #[test]
    fn test_project_export_xml() {
        let project = Project::new("XML Export Test");
        let xml = project.export_xml();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<fcpxml"));
        assert!(xml.contains("XML Export Test"));
    }

    #[test]
    fn test_project_export_edl() {
        let project = Project::new("EDL Export Test");
        let edl = project.export_edl();
        assert!(edl.starts_with("TITLE:"));
        assert!(edl.contains("EDL Export Test"));
    }

    #[test]
    fn test_project_touch() {
        let mut project = Project::new("Touch Test");
        let old_modified = project.modified_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        project.touch();
        assert!(project.modified_at > old_modified);
    }

    #[test]
    fn test_project_unique_ids() {
        let p1 = Project::new("P1");
        let p2 = Project::new("P2");
        assert_ne!(p1.id, p2.id);
    }

    #[test]
    fn test_project_load_nonexistent() {
        let result = Project::load("/nonexistent/path/project.epp");
        assert!(result.is_err());
    }
}
