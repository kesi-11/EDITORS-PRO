use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, info};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::project::project::{Project, ProjectMetadata};

/// A stored project record for listing/searching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub author: String,
    pub description: String,
    pub tags: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub created_at: String,
    pub modified_at: String,
    pub thumbnail: Option<Vec<u8>>,
}

/// SQLite-based project storage.
pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    /// Create or open a project store at the given database path.
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open project database")?;

        let mut store = Self { conn };
        store.initialize()?;
        info!("ProjectStore opened at {}", db_path);
        Ok(store)
    }

    /// Initialize the database schema.
    fn initialize(&mut self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS projects (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    author TEXT DEFAULT '',
                    description TEXT DEFAULT '',
                    tags TEXT DEFAULT '',
                    width INTEGER DEFAULT 1920,
                    height INTEGER DEFAULT 1080,
                    fps REAL DEFAULT 30.0,
                    created_at TEXT NOT NULL,
                    modified_at TEXT NOT NULL,
                    thumbnail BLOB
                );

                CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
                CREATE INDEX IF NOT EXISTS idx_projects_modified ON projects(modified_at);
                ",
            )
            .context("Failed to initialize database schema")?;
        Ok(())
    }

    /// Save a project record to the database.
    pub fn save_project(&self, project: &Project, path: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO projects (id, name, path, author, description, tags, width, height, fps, created_at, modified_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    project.id.to_string(),
                    project.name,
                    path,
                    project.project_metadata.author,
                    project.project_metadata.description,
                    project.project_metadata.tags.join(","),
                    project.width,
                    project.height,
                    project.fps,
                    project.created_at.to_rfc3339(),
                    project.modified_at.to_rfc3339(),
                ],
            )
            .context("Failed to save project")?;
        debug!("Saved project {} ({}) to store", project.name, project.id);
        Ok(())
    }

    /// Load a project record from the database by ID.
    pub fn load_project(&self, id: &str) -> Result<StoredProject> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, path, author, description, tags, width, height, fps, created_at, modified_at FROM projects WHERE id = ?1")
            .context("Failed to prepare query")?;

        let result = stmt
            .query_row(params![id], |row| {
                Ok(StoredProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    author: row.get(3)?,
                    description: row.get(4)?,
                    tags: row.get(5)?,
                    width: row.get(6)?,
                    height: row.get(7)?,
                    fps: row.get(8)?,
                    created_at: row.get(9)?,
                    modified_at: row.get(10)?,
                    thumbnail: None,
                })
            })
            .context("Project not found")?;

        Ok(result)
    }

    /// Delete a project record by ID.
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])
            .context("Failed to delete project")?;
        debug!("Deleted project {} from store", id);
        Ok(())
    }

    /// List all projects, ordered by most recently modified.
    pub fn list_projects(&self) -> Result<Vec<StoredProject>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, path, author, description, tags, width, height, fps, created_at, modified_at FROM projects ORDER BY modified_at DESC")
            .context("Failed to prepare query")?;

        let projects = stmt
            .query_map([], |row| {
                Ok(StoredProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    author: row.get(3)?,
                    description: row.get(4)?,
                    tags: row.get(5)?,
                    width: row.get(6)?,
                    height: row.get(7)?,
                    fps: row.get(8)?,
                    created_at: row.get(9)?,
                    modified_at: row.get(10)?,
                    thumbnail: None,
                })
            })
            .context("Failed to query projects")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(projects)
    }

    /// Search projects by name, author, or description.
    pub fn search_projects(&self, query: &str) -> Result<Vec<StoredProject>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, path, author, description, tags, width, height, fps, created_at, modified_at FROM projects WHERE name LIKE ?1 OR author LIKE ?1 OR description LIKE ?1 ORDER BY modified_at DESC")
            .context("Failed to prepare search query")?;

        let projects = stmt
            .query_map(params![pattern], |row| {
                Ok(StoredProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    author: row.get(3)?,
                    description: row.get(4)?,
                    tags: row.get(5)?,
                    width: row.get(6)?,
                    height: row.get(7)?,
                    fps: row.get(8)?,
                    created_at: row.get(9)?,
                    modified_at: row.get(10)?,
                    thumbnail: None,
                })
            })
            .context("Failed to search projects")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(projects)
    }

    /// Update the thumbnail for a project.
    pub fn update_thumbnail(&self, id: &str, thumbnail: &[u8]) -> Result<()> {
        self.conn
            .execute(
                "UPDATE projects SET thumbnail = ?1 WHERE id = ?2",
                params![thumbnail, id],
            )
            .context("Failed to update thumbnail")?;
        debug!("Updated thumbnail for project {}", id);
        Ok(())
    }

    /// Get recently modified projects (limited count).
    pub fn get_recent_projects(&self, limit: usize) -> Result<Vec<StoredProject>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, path, author, description, tags, width, height, fps, created_at, modified_at FROM projects ORDER BY modified_at DESC LIMIT ?1")
            .context("Failed to prepare query")?;

        let projects = stmt
            .query_map(params![limit as i64], |row| {
                Ok(StoredProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    author: row.get(3)?,
                    description: row.get(4)?,
                    tags: row.get(5)?,
                    width: row.get(6)?,
                    height: row.get(7)?,
                    fps: row.get(8)?,
                    created_at: row.get(9)?,
                    modified_at: row.get(10)?,
                    thumbnail: None,
                })
            })
            .context("Failed to query recent projects")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(projects)
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db_path() -> String {
        let dir = std::env::temp_dir().join("editors_pro_store_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("test_{}.db", Uuid::new_v4()));
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_project_store_new() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path);
        assert!(store.is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_project_store_save_and_load() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path).unwrap();
        let project = Project::new("Test Store Project");
        store.save_project(&project, "/tmp/test.epp").unwrap();

        let loaded = store.load_project(&project.id.to_string());
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "Test Store Project");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_project_store_delete() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path).unwrap();
        let project = Project::new("To Delete");
        store.save_project(&project, "/tmp/del.epp").unwrap();
        store.delete_project(&project.id.to_string()).unwrap();

        let loaded = store.load_project(&project.id.to_string());
        assert!(loaded.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_project_store_list() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path).unwrap();
        let p1 = Project::new("Project A");
        let p2 = Project::new("Project B");
        store.save_project(&p1, "/tmp/a.epp").unwrap();
        store.save_project(&p2, "/tmp/b.epp").unwrap();

        let list = store.list_projects().unwrap();
        assert_eq!(list.len(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_project_store_search() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path).unwrap();
        let p1 = Project::new("My Video Edit");
        let p2 = Project::new("Another Project");
        store.save_project(&p1, "/tmp/a.epp").unwrap();
        store.save_project(&p2, "/tmp/b.epp").unwrap();

        let results = store.search_projects("Video").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "My Video Edit");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_project_store_update_thumbnail() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path).unwrap();
        let project = Project::new("Thumb Test");
        store.save_project(&project, "/tmp/thumb.epp").unwrap();
        let thumb = vec![255u8; 100];
        let result = store.update_thumbnail(&project.id.to_string(), &thumb);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_project_store_recent() {
        let path = temp_db_path();
        let store = ProjectStore::new(&path).unwrap();
        let p1 = Project::new("Old");
        let p2 = Project::new("New");
        store.save_project(&p1, "/tmp/old.epp").unwrap();
        store.save_project(&p2, "/tmp/new.epp").unwrap();

        let recent = store.get_recent_projects(1).unwrap();
        assert_eq!(recent.len(), 1);
        let _ = std::fs::remove_file(&path);
    }
}
