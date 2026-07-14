pub mod process;

use crate::db::{launches, projects, Database};
use crate::error::Result;
use crate::provider::{AgentProvider, CommandSpec, LaunchContext, ProviderId};

pub use process::run_foreground;

/// Build new-session command, record history, and run foreground.
pub fn launch_new_session(
    db: &Database,
    provider: &dyn AgentProvider,
    project_path: &std::path::Path,
) -> Result<std::process::ExitStatus> {
    let ctx = LaunchContext {
        project_path: project_path.to_path_buf(),
    };
    let spec = provider.build_new_command(&ctx)?;
    record_and_run(db, provider.id(), project_path, &spec, "new")
}

fn record_and_run(
    db: &Database,
    provider: ProviderId,
    project_path: &std::path::Path,
    spec: &CommandSpec,
    mode: &str,
) -> Result<std::process::ExitStatus> {
    projects::ensure_project(db.conn(), project_path)?;
    launches::insert_launch(
        db.conn(),
        project_path,
        provider,
        mode,
        &spec.display_command,
        None,
    )?;
    projects::record_launch(db.conn(), project_path, provider)?;
    run_foreground(spec)
}
