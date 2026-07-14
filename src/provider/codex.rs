use std::path::Path;

use crate::error::Result;
use crate::provider::command_spec::CommandSpec;
use crate::provider::detect::{find_on_path, run_version};
use crate::provider::sessions_codex;
use crate::provider::traits::{
    AgentProvider, DetectedAgent, LaunchContext, ProviderCapabilities, ProviderId, SessionSummary,
};

pub struct CodexProvider;

impl AgentProvider for CodexProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Codex
    }

    fn display_name(&self) -> &str {
        "Codex CLI"
    }

    fn binary_names(&self) -> &[&str] {
        &["codex"]
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
            resume_session: true,
            list_sessions: true,
        }
    }

    fn list_sessions(&self, project: &Path) -> Result<Vec<SessionSummary>> {
        Ok(sessions_codex::list_sessions_for_project(project))
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
        ctx: &LaunchContext,
        session: &SessionSummary,
    ) -> Result<CommandSpec> {
        let detected = self.detect()?;
        Ok(CommandSpec::new(
            detected.executable,
            vec!["resume".into(), session.id.clone()],
            ctx.project_path.clone(),
        ))
    }

    fn validate_session(&self, _session: &SessionSummary) -> Result<()> {
        Ok(())
    }
}
