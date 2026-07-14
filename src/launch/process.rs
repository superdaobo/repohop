use std::process::{Command, ExitStatus, Stdio};

use crate::error::{RepoHopError, Result};
use crate::provider::CommandSpec;

/// Run agent in the foreground, inheriting stdin/stdout/stderr.
pub fn run_foreground(spec: &CommandSpec) -> Result<ExitStatus> {
    if !spec.cwd.is_dir() {
        return Err(RepoHopError::ProjectMissing(spec.cwd.clone()));
    }
    if !spec.executable.is_file() {
        return Err(RepoHopError::Launch(format!(
            "executable not found: {}",
            spec.executable.display()
        )));
    }

    tracing::info!(cmd = %spec.display_command, "launching agent");

    let mut cmd = Command::new(&spec.executable);
    cmd.args(&spec.args)
        .current_dir(&spec.cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }

    let status = cmd
        .status()
        .map_err(|e| RepoHopError::Launch(format!("{}: {e}", spec.display_command)))?;
    Ok(status)
}
