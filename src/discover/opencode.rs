use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::discover::DiscoveredProject;
use crate::error::Result;
use crate::provider::ProviderId;

/// Max distinct directories to import from OpenCode DB.
const MAX_DIRS: usize = 500;

pub fn discover_projects() -> Result<Vec<DiscoveredProject>> {
    let db_path = db_path();
    if !db_path.is_file() {
        return Ok(Vec::new());
    }

    // Prefer read-only open so we never write agent DBs.
    let conn = Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .or_else(|_| Connection::open(&db_path))
        .map_err(|e| crate::error::RepoHopError::Database(format!("open opencode db: {e}")))?;

    // Prefer session.directory (verified schema on OpenCode 1.17).
    let mut stmt = conn
        .prepare(
            r#"
            SELECT directory, MAX(time_updated) as last_ts, title
            FROM session
            WHERE directory IS NOT NULL AND TRIM(directory) != '' AND directory != '/'
            GROUP BY directory
            ORDER BY last_ts DESC
            LIMIT ?1
            "#,
        )
        .map_err(|e| crate::error::RepoHopError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([MAX_DIRS as i64], |row| {
            let directory: String = row.get(0)?;
            let last_ts: Option<i64> = row.get(1)?;
            let title: Option<String> = row.get(2)?;
            Ok((directory, last_ts, title))
        })
        .map_err(|e| crate::error::RepoHopError::Database(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows.flatten() {
        let (directory, last_ts, title) = row;
        let path = PathBuf::from(&directory);
        // OpenCode may store forward slashes on Windows; PathBuf handles this.
        out.push(DiscoveredProject {
            path,
            provider: ProviderId::OpenCode,
            last_activity: last_ts.and_then(ms_or_secs_to_utc),
            session_hint: title,
        });
    }
    Ok(out)
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

/// OpenCode times appear as millisecond epoch integers.
fn ms_or_secs_to_utc(ts: i64) -> Option<DateTime<Utc>> {
    if ts > 1_000_000_000_000 {
        // milliseconds
        DateTime::from_timestamp(ts / 1000, ((ts % 1000) * 1_000_000) as u32)
    } else {
        DateTime::from_timestamp(ts, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_timestamp() {
        let dt = ms_or_secs_to_utc(1783964302707).unwrap();
        assert!(dt.timestamp() > 1_700_000_000);
    }
}
