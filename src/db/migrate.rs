use rusqlite::Connection;

use crate::error::{RepoHopError, Result};

const MIGRATIONS: &[&str] = &[
    // v1
    r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    last_launched_at TEXT,
    launch_count INTEGER NOT NULL DEFAULT 0,
    last_git_activity_at TEXT,
    last_provider TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS launches (
    id TEXT PRIMARY KEY,
    project_path TEXT NOT NULL,
    provider TEXT NOT NULL,
    mode TEXT NOT NULL,
    started_at TEXT NOT NULL,
    display_command TEXT NOT NULL,
    worktree_path TEXT
);

CREATE INDEX IF NOT EXISTS idx_projects_last_launched ON projects(last_launched_at);
CREATE INDEX IF NOT EXISTS idx_launches_project ON launches(project_path);
"#,
];

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(db_err)?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )
        .map_err(db_err)?;

    for (idx, sql) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i64;
        if version <= current {
            continue;
        }
        conn.execute_batch(sql).map_err(db_err)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, chrono::Utc::now().to_rfc3339()],
        )
        .map_err(db_err)?;
    }
    Ok(())
}

fn db_err(e: rusqlite::Error) -> RepoHopError {
    RepoHopError::Database(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrate_creates_tables() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.db");
        let conn = Connection::open(&path).unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // idempotent
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
    }
}
