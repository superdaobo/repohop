use std::path::PathBuf;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{RepoHopError, Result};
use crate::launch;
use crate::paths::{display_path, AppPaths};
use crate::project::{ensure_cwd_project, ensure_projects_indexed, list_ranked_projects, Project};
use crate::provider::provider_by_id;
use crate::ui::hop::{run_hop_ui, HopChoice};
use crate::update;

pub struct HopOptions {
    /// If set, skip project picker and use this path.
    pub project: Option<PathBuf>,
}

pub fn run_interactive(db: &Database, config: &AppConfig, opts: HopOptions) -> Result<()> {
    // Soft auto-update check (network, rate-limited).
    let update_banner = check_update_banner();

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
            Err(RepoHopError::NoProjects { .. }) => {}
            Err(e) => return Err(e),
        }
    }

    let (projects, start_at_agents) = if let Some(p) = opts.project {
        if !p.is_dir() {
            return Err(RepoHopError::ProjectMissing(p));
        }
        (vec![ensure_cwd_project(db, &p)?], true)
    } else {
        (load_projects(db), false)
    };

    let choice = run_hop_ui(db, projects, update_banner, start_at_agents)?;
    launch_choice(db, choice)
}

fn load_projects(db: &Database) -> Vec<Project> {
    let projects = list_ranked_projects(db).unwrap_or_default();
    let mut alive: Vec<_> = projects.iter().filter(|p| p.exists()).cloned().collect();
    let missing: Vec<_> = projects.into_iter().filter(|p| !p.exists()).collect();
    alive.extend(missing);
    alive
}

fn check_update_banner() -> Option<String> {
    let paths = AppPaths::resolve().ok()?;
    let info = update::maybe_auto_check(&paths)?;
    if !info.update_available {
        return None;
    }
    // Auto-download + install when a newer GitHub release exists (rate-limited).
    // Set REPOPHOP_NO_UPDATE=1 to skip; REPOPHOP_UPDATE_CHECK_ONLY=1 for banner only.
    if std::env::var_os("REPOPHOP_UPDATE_CHECK_ONLY").is_some() {
        return Some(format!(
            "Update available: {} → {}  ·  run `rhop update --apply`",
            info.current, info.latest_version
        ));
    }
    match update::apply_update(&info) {
        Ok(path) => Some(format!(
            "Updated {} → {} at {} — restart rhop to use it",
            info.current,
            info.latest_version,
            path.display()
        )),
        Err(e) => {
            tracing::warn!(error = %e, "auto-update apply failed");
            Some(format!(
                "Update {} → {} failed ({e})  ·  try `rhop update --apply`",
                info.current, info.latest_version
            ))
        }
    }
}

fn launch_choice(db: &Database, choice: HopChoice) -> Result<()> {
    match choice {
        HopChoice::New { project, provider } => {
            let p = provider_by_id(provider);
            println!(
                "Launching {} (new) in {}",
                p.display_name(),
                display_path(&project.path)
            );
            let status = launch::launch_new_session(db, p.as_ref(), &project.path)?;
            report_status(status);
        }
        HopChoice::Resume {
            project,
            provider,
            session,
        } => {
            let p = provider_by_id(provider);
            println!(
                "Resuming {} session «{}» in {}",
                p.display_name(),
                session.title,
                display_path(&project.path)
            );
            let status = launch::launch_resume_session(db, p.as_ref(), &project.path, &session)?;
            report_status(status);
        }
    }
    Ok(())
}

fn report_status(status: std::process::ExitStatus) {
    if !status.success() {
        eprintln!(
            "Agent exited with status {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".into())
        );
    }
}
