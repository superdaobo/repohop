use std::path::PathBuf;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{RepoHopError, Result};
use crate::launch;
use crate::paths::display_path;
use crate::project::{ensure_projects_indexed, list_ranked_projects, Project};
use crate::provider::{detect_installed, provider_by_id, DetectedAgent, ProviderId};
use crate::ui::picker::{pick_list, PickItem};

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
                // Fall through — select_project will surface a clear error,
                // or user can still use rhop .
            }
            Err(e) => return Err(e),
        }
    }

    let project = if let Some(p) = opts.project {
        if !p.is_dir() {
            return Err(RepoHopError::ProjectMissing(p));
        }
        crate::project::ensure_cwd_project(db, &p)?
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
    let projects = list_ranked_projects(db)?;
    // Prefer existing paths in the list, but still show missing.
    let projects: Vec<Project> = {
        let mut alive: Vec<_> = projects.iter().filter(|p| p.exists()).cloned().collect();
        let missing: Vec<_> = projects.into_iter().filter(|p| !p.exists()).collect();
        if alive.is_empty() && missing.is_empty() {
            return Err(RepoHopError::NoProjects {
                config: crate::paths::AppPaths::resolve()
                    .map(|p| p.config_file)
                    .unwrap_or_else(|_| PathBuf::from("config.toml")),
            });
        }
        // Put existing first (rank already applied within each group via prior sort)
        alive.extend(missing);
        alive
    };

    let items: Vec<PickItem> = projects
        .iter()
        .map(|p| {
            let mut detail = display_path(&p.path);
            if !p.exists() {
                detail = format!("[missing] {detail}");
            }
            if p.is_favorite {
                detail = format!("★ {detail}");
            }
            if let Some(prov) = p.last_provider {
                detail = format!("{detail}  · {prov}");
            }
            PickItem {
                label: p.name.clone(),
                detail,
            }
        })
        .collect();

    let idx = pick_list("Select project", &items, 0)?;
    let project = projects[idx].clone();
    if !project.exists() {
        return Err(RepoHopError::ProjectMissing(project.path));
    }
    Ok(project)
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
