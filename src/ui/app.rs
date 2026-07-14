use std::path::PathBuf;

use chrono::Utc;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{RepoHopError, Result};
use crate::launch;
use crate::paths::display_path;
use crate::project::{ensure_cwd_project, ensure_projects_indexed, list_ranked_projects, Project};
use crate::provider::{detect_installed, provider_by_id, DetectedAgent, ProviderId};
use crate::ui::picker::{
    pick_list, pick_project_table, resolve_user_path, PickItem, PickOutcome, ProjectRow,
};
use crate::ui::timefmt::format_relative_time;

pub struct HopOptions {
    /// If set, skip project picker and use this path.
    pub project: Option<PathBuf>,
}

pub fn run_interactive(db: &Database, config: &AppConfig, opts: HopOptions) -> Result<()> {
    // First run / empty index: pull projects from agent session metadata.
    if opts.project.is_none() {
        match ensure_projects_indexed(db, config) {
            Ok(report) if report.total_upserted > 0 => {
                eprintln!(
                    "Discovered {} project(s) from agent history ({})",
                    report.from_agents,
                    if report.agent_sources.is_empty() {
                        "agents".into()
                    } else {
                        report.agent_sources.join(", ")
                    }
                );
            }
            Ok(_) => {}
            Err(RepoHopError::NoProjects { .. }) => {
                // Fall through — select_project allows . / n even when empty.
            }
            Err(e) => return Err(e),
        }
    }

    let project = if let Some(p) = opts.project {
        if !p.is_dir() {
            return Err(RepoHopError::ProjectMissing(p));
        }
        ensure_cwd_project(db, &p)?
    } else {
        select_project(db)?
    };

    let agents = detect_installed();
    if agents.is_empty() {
        return Err(RepoHopError::NoAgents);
    }

    let provider_id = select_agent(&project, &agents)?;
    let provider = provider_by_id(provider_id);

    println!(
        "Launching {} in {}",
        provider.display_name(),
        display_path(&project.path)
    );

    let status = launch::launch_new_session(db, provider.as_ref(), &project.path)?;
    if !status.success() {
        eprintln!(
            "Agent exited with status {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".into())
        );
    }
    Ok(())
}

fn select_project(db: &Database) -> Result<Project> {
    let projects = list_ranked_projects(db).unwrap_or_default();
    let projects: Vec<Project> = {
        let mut alive: Vec<_> = projects.iter().filter(|p| p.exists()).cloned().collect();
        let missing: Vec<_> = projects.into_iter().filter(|p| !p.exists()).collect();
        // Existing paths first; rank already applied within the full list.
        alive.extend(missing);
        alive
    };

    let now = Utc::now();
    let rows: Vec<ProjectRow> = projects
        .iter()
        .map(|p| {
            let mut name = p.name.clone();
            if p.is_favorite {
                name = format!("★ {name}");
            }
            if !p.exists() {
                name = format!("[?] {name}");
            }
            ProjectRow {
                name,
                path: display_path(&p.path),
                last_used: format_relative_time(now, p.last_launched_at),
            }
        })
        .collect();

    let outcome = pick_project_table("Select project", &rows, 0)?;
    match outcome {
        PickOutcome::Index(idx) => {
            let project = projects
                .get(idx)
                .cloned()
                .ok_or_else(|| RepoHopError::Config("invalid project index".into()))?;
            if !project.exists() {
                return Err(RepoHopError::ProjectMissing(project.path));
            }
            Ok(project)
        }
        PickOutcome::Cwd => {
            let cwd = std::env::current_dir().map_err(RepoHopError::Io)?;
            if !cwd.is_dir() {
                return Err(RepoHopError::ProjectMissing(cwd));
            }
            ensure_cwd_project(db, &cwd)
        }
        PickOutcome::NewPath(raw) => {
            let path = resolve_user_path(&raw)?;
            if !path.is_dir() {
                return Err(RepoHopError::ProjectMissing(path));
            }
            ensure_cwd_project(db, &path)
        }
    }
}

fn select_agent(project: &Project, agents: &[DetectedAgent]) -> Result<ProviderId> {
    let default_idx = project
        .last_provider
        .and_then(|lp| agents.iter().position(|a| a.provider == lp))
        .unwrap_or(0);

    let items: Vec<PickItem> = agents
        .iter()
        .map(|a| {
            let ver = a.version.clone().unwrap_or_default();
            PickItem {
                label: a.provider.as_str().to_string(),
                detail: format!("{}  {}", ver, a.executable.display()),
            }
        })
        .collect();

    let idx = pick_list("Select agent (new session)", &items, default_idx)?;
    Ok(agents[idx].provider)
}
