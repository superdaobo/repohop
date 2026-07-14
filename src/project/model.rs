use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::provider::ProviderId;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub is_favorite: bool,
    pub last_launched_at: Option<DateTime<Utc>>,
    pub launch_count: i64,
    pub last_git_activity_at: Option<DateTime<Utc>>,
    pub last_provider: Option<ProviderId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn exists(&self) -> bool {
        self.path.is_dir()
    }
}
