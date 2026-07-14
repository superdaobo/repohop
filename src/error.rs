use std::path::PathBuf;

use thiserror::Error;

/// Domain-level errors for RepoHop.
#[derive(Debug, Error)]
pub enum RepoHopError {
    #[error("agent provider not found: {0}")]
    ProviderNotFound(String),

    #[error("agent executable not found for {provider}: searched PATH for {names}")]
    ExecutableNotFound { provider: String, names: String },

    #[error("project path does not exist: {0}")]
    ProjectMissing(PathBuf),

    #[error("project path is not a directory: {0}")]
    ProjectNotDir(PathBuf),

    #[error(
        "no projects found from agent history or config.\n  \
         RepoHop looks for Codex/Claude/OpenCode session folders automatically.\n  \
         Optional: add project_roots in {config} and run `rhop scan`\n  \
         Or: `rhop .` from inside a project directory"
    )]
    NoProjects { config: PathBuf },

    #[error("no AI coding agents found on PATH; install Codex, Claude Code, or OpenCode")]
    NoAgents,

    #[error("feature not implemented yet: {0}")]
    NotImplemented(&'static str),

    #[error("interactive UI requires a TTY (try Windows Terminal)")]
    NotTty,

    #[error("cancelled by user")]
    Cancelled,

    #[error("config error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("failed to launch agent: {0}")]
    Launch(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, RepoHopError>;
