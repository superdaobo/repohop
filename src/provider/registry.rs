use crate::error::{RepoHopError, Result};
use crate::provider::claude::ClaudeProvider;
use crate::provider::codex::CodexProvider;
use crate::provider::opencode::OpenCodeProvider;
use crate::provider::traits::{AgentProvider, DetectedAgent, ProviderId};

/// Returns all first-party providers.
pub fn all_providers() -> Vec<Box<dyn AgentProvider>> {
    vec![
        Box::new(CodexProvider),
        Box::new(ClaudeProvider),
        Box::new(OpenCodeProvider),
    ]
}

pub fn provider_by_id(id: ProviderId) -> Box<dyn AgentProvider> {
    match id {
        ProviderId::Codex => Box::new(CodexProvider),
        ProviderId::Claude => Box::new(ClaudeProvider),
        ProviderId::OpenCode => Box::new(OpenCodeProvider),
    }
}

/// Detect every known provider; missing ones are omitted.
pub fn detect_installed() -> Vec<DetectedAgent> {
    let mut out = Vec::new();
    for p in all_providers() {
        if let Ok(d) = p.detect() {
            out.push(d);
        }
    }
    out
}

pub fn require_provider(id: ProviderId) -> Result<Box<dyn AgentProvider>> {
    let p = provider_by_id(id);
    p.detect()?;
    Ok(p)
}

pub fn parse_provider_or_err(s: &str) -> Result<ProviderId> {
    ProviderId::parse(s).ok_or_else(|| RepoHopError::ProviderNotFound(s.to_string()))
}
