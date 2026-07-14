//! List OpenCode sessions for a project (read-only opencode.db).

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::paths::normalize_path;
use crate::provider::traits::{ProviderId, SessionSummary};

const MAX_SESSIONS: usize = 80;

pub fn list_sessions_for_project(project: &Path) -> Vec<SessionSummary> {
    let db_path = db_path();
    if !db_path.is_file() {
        return Vec::new();
    }
    let conn = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .or_else(|_| rusqlite::Connection::open(&db_path))
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let target_key = path_key(project);
    let mut stmt = match conn.prepare(
        r#"
        SELECT id, directory, title, time_created, time_updated
        FROM session
        WHERE directory IS NOT NULL AND TRIM(directory) != ''
        ORDER BY time_updated DESC
        LIMIT 500
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = match stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let directory: String = row.get(1)?;
        let title: Option<String> = row.get(2)?;
        let created: Option<i64> = row.get(3)?;
        let updated: Option<i64> = row.get(4)?;
        Ok((id, directory, title, created, updated))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();
    for row in rows.flatten() {
        let (id, directory, title, created, updated) = row;
        let dir_path = PathBuf::from(&directory);
        if path_key(&dir_path) != target_key {
            // Also try slash-normalized compare for mixed separators.
            let alt = directory.replace('/', "\\").to_ascii_lowercase();
            let tgt = target_key.replace('/', "\\");
            if alt != tgt {
                continue;
            }
        }
        let title = title
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| format!("Session {}", short_id(&id)));
        out.push(SessionSummary {
            id,
            provider: ProviderId::OpenCode,
            project_path: project.to_path_buf(),
            title: title.clone(),
            preview: title,
            created_at: created.and_then(ms_or_secs_to_utc),
            updated_at: updated.and_then(ms_or_secs_to_utc),
            git_branch: None,
            source_path: None,
        });
        if out.len() >= MAX_SESSIONS {
            break;
        }
    }
    out
}

fn db_path() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| {
            b.home_dir()
                .join(".local")
                .join("share")
                .join("opencode")
                .join("opencode.db")
        })
        .unwrap_or_else(|| PathBuf::from("opencode.db"))
}

fn path_key(p: &Path) -> String {
    normalize_path(p)
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn short_id(id: &str) -> String {
    if id.len() > 12 {
        id[..12].to_string()
    } else {
        id.to_string()
    }
}

fn ms_or_secs_to_utc(ts: i64) -> Option<DateTime<Utc>> {
    if ts > 1_000_000_000_000 {
        DateTime::from_timestamp(ts / 1000, ((ts % 1000) * 1_000_000) as u32)
    } else {
        DateTime::from_timestamp(ts, 0)
    }
}
