pub mod claude;
pub mod codex;
pub mod command_spec;
pub mod detect;
pub mod opencode;
pub mod registry;
pub mod traits;

pub use command_spec::CommandSpec;
pub use registry::{all_providers, detect_installed, provider_by_id};
pub use traits::{
    AgentProvider, DetectedAgent, LaunchContext, ProviderCapabilities, ProviderId, SessionSummary,
};
