pub mod launches;
pub mod migrate;
pub mod projects;

use std::path::Path;

use rusqlite::Connection;

use crate::error::{RepoHopError, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .map_err(|e| RepoHopError::Database(format!("open {}: {e}", path.display())))?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|e| RepoHopError::Database(e.to_string()))?;
        migrate::migrate(&conn)?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
