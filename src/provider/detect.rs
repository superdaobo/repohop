use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{RepoHopError, Result};

/// Find an executable on PATH with Windows-friendly extensions.
/// Prefers `.exe` / `.cmd` / `.bat` over bare names / `.ps1`.
pub fn find_on_path(names: &[&str]) -> Result<PathBuf> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    let dirs: Vec<PathBuf> = env::split_paths(&path_var).collect();

    // Preference order for each base name.
    let suffixes: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };

    for name in names {
        for dir in &dirs {
            for suffix in suffixes {
                let candidate = if suffix.is_empty() || name.ends_with(suffix) {
                    dir.join(name)
                } else {
                    dir.join(format!("{name}{suffix}"))
                };
                if is_runnable(&candidate) {
                    return Ok(candidate);
                }
            }
        }
    }

    Err(RepoHopError::ExecutableNotFound {
        provider: names.first().unwrap_or(&"agent").to_string(),
        names: names.join(", "),
    })
}

fn is_runnable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // Skip PowerShell scripts — Command::new cannot run them like the shell.
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ps1"))
    {
        return false;
    }
    true
}

/// Run `exe --version` (or args) and return trimmed stdout/stderr.
pub fn run_version(exe: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(exe)
        .args(args)
        .output()
        .map_err(|e| RepoHopError::Launch(format!("failed to run {}: {e}", exe.display())))?;

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&output.stderr).to_string();
    }
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("(unknown version)")
        .to_string();
    Ok(line)
}

/// Find on a custom PATH string (for tests).
pub fn find_on_path_dirs(names: &[&str], dirs: &[PathBuf]) -> Result<PathBuf> {
    let suffixes: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };

    for name in names {
        for dir in dirs {
            for suffix in suffixes {
                let candidate = if suffix.is_empty() {
                    dir.join(name)
                } else {
                    dir.join(format!("{name}{suffix}"))
                };
                if is_runnable(&candidate) {
                    return Ok(candidate);
                }
            }
        }
    }

    Err(RepoHopError::ExecutableNotFound {
        provider: names.first().unwrap_or(&"agent").to_string(),
        names: names.join(", "),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn prefers_cmd_over_ps1() {
        let dir = tempdir().unwrap();
        let ps1 = dir.path().join("fakeagent.ps1");
        let cmd = dir.path().join("fakeagent.cmd");
        fs::write(&ps1, "write-host hi").unwrap();
        fs::write(&cmd, "@echo off").unwrap();
        let found = find_on_path_dirs(&["fakeagent"], &[dir.path().to_path_buf()]).unwrap();
        assert_eq!(found.extension().and_then(|e| e.to_str()), Some("cmd"));
    }

    #[test]
    fn missing_agent_errors() {
        let dir = tempdir().unwrap();
        let err = find_on_path_dirs(&["definitely-missing-xyz"], &[dir.path().to_path_buf()]);
        assert!(err.is_err());
    }
}
