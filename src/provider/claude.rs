use std::path::Path;

use crate::error::{RepoHopError, Result};
use crate::provider::command_spec::CommandSpec;
use crate::provider::detect::{find_on_path, run_version};
use crate::provider::traits::{
    AgentProvider, DetectedAgent, LaunchContext, ProviderCapabilities, ProviderId, SessionSummary,
};

pub struct ClaudeProvider;

impl AgentProvider for ClaudeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn binary_names(&self) -> &[&str] {
        &["claude"]
    }

    fn detect(&self) -> Result<DetectedAgent> {
        let executable = find_on_path(self.binary_names())?;
        let version = self.version(&executable).ok();
        let mut notes = Vec::new();
        if executable
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("cmd"))
        {
            notes.push("npm/cmd shim".into());
        }
        Ok(DetectedAgent {
            provider: self.id(),
            executable,
            version,
            notes,
        })
    }

    fn version(&self, exe: &Path) -> Result<String> {
        run_version(exe, &["--version"])
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            new_session: true,
            resume_session: false,
            list_sessions: false,
        }
    }

    fn list_sessions(&self, _project: &Path) -> Result<Vec<SessionSummary>> {
        Ok(Vec::new())
    }

    fn build_new_command(&self, ctx: &LaunchContext) -> Result<CommandSpec> {
        let detected = self.detect()?;
        Ok(CommandSpec::new(
            detected.executable,
            Vec::new(),
            ctx.project_path.clone(),
        ))
    }

    fn build_resume_command(
        &self,
        _ctx: &LaunchContext,
        _session: &SessionSummary,
    ) -> Result<CommandSpec> {
        Err(RepoHopError::NotImplemented(
            "Claude Code session resume (Stage 3)",
        ))
    }

    fn validate_session(&self, _session: &SessionSummary) -> Result<()> {
        Ok(())
    }
}
