pub mod claude;
pub mod codex;
pub mod command_spec;
pub mod detect;
pub mod grok;
pub mod opencode;
pub mod registry;
pub mod sessions_claude;
pub mod sessions_codex;
pub mod sessions_grok;
pub mod sessions_opencode;
pub mod traits;

pub use command_spec::CommandSpec;
pub use registry::{all_providers, detect_installed, provider_by_id};
pub use traits::{
    AgentProvider, DetectedAgent, LaunchContext, ProviderCapabilities, ProviderId, SessionSummary,
};
