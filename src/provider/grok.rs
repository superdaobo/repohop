use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::provider::command_spec::CommandSpec;
use crate::provider::detect::{find_on_path, run_version};
use crate::provider::sessions_grok;
use crate::provider::traits::{
    AgentProvider, DetectedAgent, LaunchContext, ProviderCapabilities, ProviderId, SessionSummary,
};

/// Grok Build CLI (`grok`) — xAI agentic coding TUI.
pub struct GrokProvider;

impl AgentProvider for GrokProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Grok
    }

    fn display_name(&self) -> &str {
        "Grok Build CLI"
    }

    fn binary_names(&self) -> &[&str] {
        &["grok"]
    }

    fn detect(&self) -> Result<DetectedAgent> {
        let executable = find_on_path(self.binary_names())?;
        let version = self.version(&executable).ok();
        let mut notes = Vec::new();
        if executable
            .components()
            .any(|c| c.as_os_str().eq_ignore_ascii_case(".grok"))
        {
            notes.push("user .grok install".into());
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
        Ok(sessions_grok::list_sessions_for_project(project))
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
            vec!["--resume".into(), session.id.clone()],
            ctx.project_path.clone(),
        ))
    }

    fn validate_session(&self, _session: &SessionSummary) -> Result<()> {
        Ok(())
    }
}

/// Percent-decode a Grok session directory name into a filesystem path string.
///
/// Grok stores sessions under `~/.grok/sessions/<percent-encoded-abs-path>/`.
pub fn percent_decode_path_name(encoded: &str) -> Option<PathBuf> {
    let bytes = percent_decode_to_bytes(encoded)?;
    let s = String::from_utf8(bytes).ok()?;
    if s.is_empty() {
        return None;
    }
    Some(PathBuf::from(s))
}

fn percent_decode_to_bytes(input: &str) -> Option<Vec<u8>> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let h1 = from_hex(bytes[i + 1])?;
                let h2 = from_hex(bytes[i + 2])?;
                out.push((h1 << 4) | h2);
                i += 3;
            }
            b'+' => {
                out.push(b'+');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    Some(out)
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_windows_encoded_path() {
        let p = percent_decode_path_name(r"D%3A%5CDocuments%5Cfoo").unwrap();
        assert_eq!(p, PathBuf::from(r"D:\Documents\foo"));
    }

    #[test]
    fn decode_rejects_bad_hex() {
        assert!(percent_decode_path_name("D%3G%5Cfoo").is_none());
    }
}
