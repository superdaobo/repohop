use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::provider::command_spec::CommandSpec;

/// Stable provider identifier used in DB and config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderId {
    Codex,
    Claude,
    OpenCode,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::OpenCode => "opencode",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" | "claude-code" => Some(Self::Claude),
            "opencode" | "open-code" => Some(Self::OpenCode),
            _ => None,
        }
    }

    pub fn all() -> &'static [ProviderId] {
        &[Self::Codex, Self::Claude, Self::OpenCode]
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct DetectedAgent {
    pub provider: ProviderId,
    pub executable: PathBuf,
    pub version: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    pub new_session: bool,
    pub resume_session: bool,
    pub list_sessions: bool,
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub provider: ProviderId,
    pub project_path: PathBuf,
    pub title: String,
    pub preview: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub git_branch: Option<String>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct LaunchContext {
    pub project_path: PathBuf,
}

/// Adapter interface for AI coding CLIs.
pub trait AgentProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn display_name(&self) -> &str;
    fn binary_names(&self) -> &[&str];

    fn detect(&self) -> Result<DetectedAgent>;
    fn version(&self, exe: &Path) -> Result<String>;
    fn capabilities(&self) -> ProviderCapabilities;

    fn list_sessions(&self, project: &Path) -> Result<Vec<SessionSummary>>;
    fn build_new_command(&self, ctx: &LaunchContext) -> Result<CommandSpec>;
    fn build_resume_command(
        &self,
        ctx: &LaunchContext,
        session: &SessionSummary,
    ) -> Result<CommandSpec>;
    fn validate_session(&self, session: &SessionSummary) -> Result<()>;
}
