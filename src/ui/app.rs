use std::path::PathBuf;

use crate::db::Database;
use crate::error::{RepoHopError, Result};
use crate::launch;
use crate::paths::display_path;
use crate::project::{list_ranked_projects, Project};
use crate::provider::{detect_installed, provider_by_id, DetectedAgent, ProviderId};
use crate::ui::picker::{pick_list, PickItem};

pub struct HopOptions {
    /// If set, skip project picker and use this path.
    pub project: Option<PathBuf>,
}

pub fn run_interactive(db: &Database, opts: HopOptions) -> Result<()> {
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
    if projects.is_empty() {
        return Err(RepoHopError::NoProjects {
            config: crate::paths::AppPaths::resolve()
                .map(|p| p.config_file)
                .unwrap_or_else(|_| PathBuf::from("config.toml")),
        });
    }

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
