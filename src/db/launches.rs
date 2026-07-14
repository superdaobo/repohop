use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{RepoHopError, Result};
use crate::paths::normalize_path;
use crate::provider::ProviderId;

pub fn insert_launch(
    conn: &Connection,
    project_path: &Path,
    provider: ProviderId,
    mode: &str,
    display_command: &str,
    worktree_path: Option<&Path>,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let path_str = normalize_path(project_path).to_string_lossy().to_string();
    let wt = worktree_path.map(|p| normalize_path(p).to_string_lossy().to_string());
    conn.execute(
        "INSERT INTO launches (id, project_path, provider, mode, started_at, display_command, worktree_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            path_str,
            provider.as_str(),
            mode,
            Utc::now().to_rfc3339(),
            display_command,
            wt
        ],
    )
    .map_err(|e| RepoHopError::Database(e.to_string()))?;
    Ok(id)
}
