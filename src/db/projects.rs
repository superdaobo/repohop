use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{RepoHopError, Result};
use crate::paths::normalize_path;
use crate::project::model::Project;
use crate::provider::ProviderId;

pub fn upsert_scanned_project(conn: &Connection, path: &Path) -> Result<Project> {
    let path = normalize_path(path);
    let path_str = path.to_string_lossy().to_string();
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.clone());
    let now = Utc::now().to_rfc3339();

    if get_by_path(conn, &path)?.is_some() {
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE path = ?3",
            params![name, now, path_str],
        )
        .map_err(db_err)?;
        return get_by_path(conn, &path)?
            .ok_or_else(|| RepoHopError::Database("project missing after update".into()));
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO projects (id, path, name, is_favorite, last_launched_at, launch_count, last_git_activity_at, last_provider, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, NULL, 0, NULL, NULL, ?4, ?4)",
        params![id, path_str, name, now],
    )
    .map_err(db_err)?;
    get_by_path(conn, &path)?
        .ok_or_else(|| RepoHopError::Database("project missing after insert".into()))
}

pub fn ensure_project(conn: &Connection, path: &Path) -> Result<Project> {
    upsert_scanned_project(conn, path)
}

pub fn get_by_path(conn: &Connection, path: &Path) -> Result<Option<Project>> {
    let path_str = normalize_path(path).to_string_lossy().to_string();
    conn.query_row(
        "SELECT id, path, name, is_favorite, last_launched_at, launch_count, last_git_activity_at, last_provider, created_at, updated_at
         FROM projects WHERE path = ?1",
        params![path_str],
        row_to_project,
    )
    .optional()
    .map_err(db_err)
}

pub fn list_all(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, path, name, is_favorite, last_launched_at, launch_count, last_git_activity_at, last_provider, created_at, updated_at
             FROM projects",
        )
        .map_err(db_err)?;
    let rows = stmt
        .query_map([], row_to_project)
        .map_err(db_err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_err)?;
    Ok(rows)
}

pub fn record_launch(conn: &Connection, project_path: &Path, provider: ProviderId) -> Result<()> {
    let path_str = normalize_path(project_path).to_string_lossy().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET last_launched_at = ?1, launch_count = launch_count + 1, last_provider = ?2, updated_at = ?1 WHERE path = ?3",
        params![now, provider.as_str(), path_str],
    )
    .map_err(db_err)?;
    Ok(())
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let last_launched: Option<String> = row.get(4)?;
    let last_git: Option<String> = row.get(6)?;
    let created: String = row.get(8)?;
    let updated: String = row.get(9)?;
    Ok(Project {
        id: row.get(0)?,
        path: PathBuf::from(row.get::<_, String>(1)?),
        name: row.get(2)?,
        is_favorite: row.get::<_, i64>(3)? != 0,
        last_launched_at: parse_dt(last_launched),
        launch_count: row.get(5)?,
        last_git_activity_at: parse_dt(last_git),
        last_provider: row
            .get::<_, Option<String>>(7)?
            .and_then(|s| ProviderId::parse(&s)),
        created_at: parse_dt(Some(created)).unwrap_or_else(Utc::now),
        updated_at: parse_dt(Some(updated)).unwrap_or_else(Utc::now),
    })
}

fn parse_dt(s: Option<String>) -> Option<DateTime<Utc>> {
    s.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&Utc))
}

fn db_err(e: rusqlite::Error) -> RepoHopError {
    RepoHopError::Database(e.to_string())
}
