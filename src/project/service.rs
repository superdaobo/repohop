use std::path::Path;

use crate::config::AppConfig;
use crate::db::{projects, Database};
use crate::error::{RepoHopError, Result};
use crate::project::model::Project;
use crate::project::rank::{dedup_projects, rank_projects};
use crate::project::scan::scan_git_projects;

pub fn scan_and_upsert(db: &Database, config: &AppConfig) -> Result<usize> {
    if config.project_roots.is_empty() {
        return Err(RepoHopError::Config(
            "project_roots is empty; edit config and retry".into(),
        ));
    }
    let mut count = 0usize;
    for root in &config.project_roots {
        let found = scan_git_projects(root, config.max_depth())?;
        for path in found {
            projects::upsert_scanned_project(db.conn(), &path)?;
            count += 1;
        }
    }
    Ok(count)
}

pub fn list_ranked_projects(db: &Database) -> Result<Vec<Project>> {
    let list = projects::list_all(db.conn())?;
    let list = dedup_projects(list);
    Ok(rank_projects(list))
}

pub fn ensure_cwd_project(db: &Database, cwd: &Path) -> Result<Project> {
    projects::ensure_project(db.conn(), cwd)
}
