use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rhop",
    version,
    about = "A local-first workspace router for AI coding CLIs",
    long_about = "Hop into the right repo, agent, and session.\nRepoHop does not run models — it routes you to Codex, Claude Code, or OpenCode."
)]
pub struct Cli {
    /// Project path (use `.` for current directory). Omit for interactive project picker.
    #[arg(value_name = "PROJECT")]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Diagnose environment and detect agent CLIs
    Doctor,
    /// Scan project_roots for Git repositories
    Scan,
    /// Session browser (Stage 3 — not implemented yet)
    Sessions,
    /// Git worktree management (Stage 4 — not implemented yet)
    Worktree,
    /// Show configuration path and project_roots
    Config,
    /// Print version
    Version,
}
