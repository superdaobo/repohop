use std::io::{self, Write};

use crate::config::AppConfig;
use crate::discover::{agent_data_hints, discover_from_agents};
use crate::error::{RepoHopError, Result};
use crate::paths::AppPaths;
use crate::provider::{all_providers, DetectedAgent};

pub struct DoctorReport {
    pub agents: Vec<AgentStatus>,
    pub config_path: std::path::PathBuf,
    pub project_roots: usize,
    pub db_path: std::path::PathBuf,
    pub discovered_projects: usize,
    pub discovery_notes: Vec<String>,
}

pub struct AgentStatus {
    pub name: String,
    pub id: String,
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

pub fn run_doctor(paths: &AppPaths, config: &AppConfig) -> Result<DoctorReport> {
    let mut agents = Vec::new();
    for p in all_providers() {
        match p.detect() {
            Ok(d) => agents.push(status_from_detected(p.display_name(), d)),
            Err(e) => agents.push(AgentStatus {
                name: p.display_name().to_string(),
                id: p.id().as_str().to_string(),
                found: false,
                path: None,
                version: None,
                notes: Vec::new(),
                error: Some(e.to_string()),
            }),
        }
    }

    let mut discovery_notes = Vec::new();
    for (id, path) in agent_data_hints() {
        let status = if path.exists() { "found" } else { "missing" };
        discovery_notes.push(format!("{id}: {status} — {}", path.display()));
    }

    let discovered_projects = match discover_from_agents() {
        Ok(list) => {
            discovery_notes.push(format!(
                "unique project paths from sessions: {}",
                list.len()
            ));
            list.len()
        }
        Err(e) => {
            discovery_notes.push(format!("discovery error: {e}"));
            0
        }
    };

    Ok(DoctorReport {
        agents,
        config_path: paths.config_file.clone(),
        project_roots: config.project_roots.len(),
        db_path: paths.database_file.clone(),
        discovered_projects,
        discovery_notes,
    })
}

fn status_from_detected(name: &str, d: DetectedAgent) -> AgentStatus {
    AgentStatus {
        name: name.to_string(),
        id: d.provider.as_str().to_string(),
        found: true,
        path: Some(d.executable.display().to_string()),
        version: d.version,
        notes: d.notes,
        error: None,
    }
}

pub fn print_report(report: &DoctorReport) -> Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "RepoHop doctor")?;
    writeln!(out, "==============")?;
    writeln!(out)?;
    writeln!(out, "Config: {}", report.config_path.display())?;
    writeln!(out, "Database: {}", report.db_path.display())?;
    writeln!(out, "project_roots entries: {}", report.project_roots)?;
    writeln!(
        out,
        "auto-discovered projects (from agent sessions): {}",
        report.discovered_projects
    )?;
    if report.project_roots == 0 {
        writeln!(
            out,
            "  note: project_roots is optional — `rhop` uses agent session history by default"
        )?;
    }
    for note in &report.discovery_notes {
        writeln!(out, "  · {note}")?;
    }
    writeln!(out)?;
    writeln!(
        out,
        "{:<14} {:<8} {:<40} Path / notes",
        "Provider", "Found", "Version"
    )?;
    writeln!(out, "{}", "-".repeat(90))?;

    let mut any = false;
    for a in &report.agents {
        let found = if a.found { "yes" } else { "no" };
        if a.found {
            any = true;
        }
        let ver = a.version.as_deref().unwrap_or("-");
        let path = a.path.as_deref().unwrap_or("-");
        let notes = if a.notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", a.notes.join(", "))
        };
        writeln!(
            out,
            "{:<14} {:<8} {:<40} {}{}",
            a.name, found, ver, path, notes
        )?;
        if let Some(err) = &a.error {
            writeln!(out, "               error: {err}")?;
        }
    }

    writeln!(out)?;
    if !any {
        writeln!(
            out,
            "No AI coding agents found. Install Codex, Claude Code, or OpenCode and ensure they are on PATH."
        )?;
        return Err(RepoHopError::NoAgents);
    }
    writeln!(out, "OK: at least one agent is available.")?;
    if report.discovered_projects > 0 {
        writeln!(
            out,
            "OK: run `rhop` or `rhop scan` to use {n} auto-discovered project(s).",
            n = report.discovered_projects
        )?;
    }
    Ok(())
}
