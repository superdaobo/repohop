use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{RepoHopError, Result};
use crate::paths::normalize_path;
use crate::provider::ProviderId;

/// Per-provider launch stats for a project (for the agent table).
#[derive(Debug, Clone)]
pub struct ProviderLaunchStats {
    pub provider: ProviderId,
    pub launch_count: i64,
    pub last_launched_at: Option<DateTime<Utc>>,
}

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

/// Aggregate launch counts and last-used time per provider for a project.
pub fn stats_for_project(
    conn: &Connection,
    project_path: &Path,
) -> Result<Vec<ProviderLaunchStats>> {
    let path_str = normalize_path(project_path).to_string_lossy().to_string();
    // Match case-insensitively on Windows-style stored paths.
    let mut stmt = conn
        .prepare(
            r#"
            SELECT provider, COUNT(*) as cnt, MAX(started_at) as last_at
            FROM launches
            WHERE lower(project_path) = lower(?1)
            GROUP BY provider
            "#,
        )
        .map_err(|e| RepoHopError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![path_str], |row| {
            let provider_s: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let last: Option<String> = row.get(2)?;
            Ok((provider_s, count, last))
        })
        .map_err(|e| RepoHopError::Database(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows.flatten() {
        let (provider_s, count, last) = row;
        let Some(provider) = ProviderId::parse(&provider_s) else {
            continue;
        };
        out.push(ProviderLaunchStats {
            provider,
            launch_count: count,
            last_launched_at: last
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
        });
    }
    Ok(out)
}
