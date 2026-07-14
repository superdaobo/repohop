use std::path::Path;

use crate::config::AppConfig;
use crate::db::{projects, Database};
use crate::discover::{discover_from_agents, DiscoveredProject};
use crate::error::{RepoHopError, Result};
use crate::project::model::Project;
use crate::project::rank::{dedup_projects, rank_projects};
use crate::project::scan::scan_git_projects;

/// Result of a scan / auto-discovery pass.
#[derive(Debug, Default)]
pub struct ScanReport {
    pub from_roots: usize,
    pub from_agents: usize,
    pub total_upserted: usize,
    pub agent_sources: Vec<String>,
}

/// Full scan: optional `project_roots` + always agent session discovery.
pub fn scan_and_upsert(db: &Database, config: &AppConfig) -> Result<ScanReport> {
    let mut report = ScanReport::default();

    // 1) Explicit roots (optional)
    for root in &config.project_roots {
        let found = scan_git_projects(root, config.max_depth())?;
        for path in found {
            projects::upsert_scanned_project(db.conn(), &path)?;
            report.from_roots += 1;
            report.total_upserted += 1;
        }
    }

    // 2) Agent session metadata (always — zero-config path)
    let discovered = discover_from_agents()?;
    let mut sources = std::collections::BTreeSet::new();
    for d in discovered {
        sources.insert(d.provider.as_str().to_string());
        // Prefer existing dirs; still index missing so UI can show [missing]
        upsert_discovered(db, &d)?;
        report.from_agents += 1;
        report.total_upserted += 1;
    }
    report.agent_sources = sources.into_iter().collect();
    Ok(report)
}

fn upsert_discovered(db: &Database, d: &DiscoveredProject) -> Result<()> {
    projects::upsert_discovered_project(db.conn(), &d.path, d.provider, d.last_activity)?;
    Ok(())
}

/// Ensure the project index is non-empty by auto-discovering agent history.
pub fn ensure_projects_indexed(db: &Database, config: &AppConfig) -> Result<ScanReport> {
    let existing = projects::list_all(db.conn())?;
    let existing_alive: Vec<_> = existing.into_iter().filter(|p| p.path.is_dir()).collect();
    if !existing_alive.is_empty() {
        return Ok(ScanReport::default());
    }
    tracing::info!("no indexed projects; auto-discovering from agent sessions");
    let report = scan_and_upsert(db, config)?;
    if report.total_upserted == 0 {
        return Err(RepoHopError::NoProjects {
            config: crate::paths::AppPaths::resolve()
                .map(|p| p.config_file)
                .unwrap_or_else(|_| std::path::PathBuf::from("config.toml")),
        });
    }
    Ok(report)
}

pub fn list_ranked_projects(db: &Database) -> Result<Vec<Project>> {
    let list = projects::list_all(db.conn())?;
    let list = dedup_projects(list);
    Ok(rank_projects(list))
}

pub fn ensure_cwd_project(db: &Database, cwd: &Path) -> Result<Project> {
    projects::ensure_project(db.conn(), cwd)
}
