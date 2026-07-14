use std::path::{Path, PathBuf};

use directories::BaseDirs;

use crate::error::{RepoHopError, Result};

const APP_NAME: &str = "RepoHop";

/// Resolved filesystem locations for RepoHop data.
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub data_dir: PathBuf,
    pub database_file: PathBuf,
    pub log_dir: PathBuf,
    pub worktree_root: PathBuf,
}

impl AppPaths {
    /// Resolve default paths from the current user profile.
    pub fn resolve() -> Result<Self> {
        let base = BaseDirs::new().ok_or_else(|| {
            RepoHopError::Config("could not resolve user home directories".into())
        })?;

        // Config: %APPDATA%\RepoHop on Windows (Roaming)
        let config_dir = base.config_dir().join(APP_NAME);
        let config_file = config_dir.join("config.toml");

        // Data: %LOCALAPPDATA%\RepoHop on Windows
        let data_dir = base.data_local_dir().join(APP_NAME);
        let database_file = data_dir.join("repohop.db");
        let log_dir = data_dir.join("logs");

        // Worktrees: %USERPROFILE%\.repohop\worktrees
        let worktree_root = base.home_dir().join(".repohop").join("worktrees");

        Ok(Self {
            config_dir,
            config_file,
            data_dir,
            database_file,
            log_dir,
            worktree_root,
        })
    }

    /// Ensure config/data/log directories exist.
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }
}

/// Normalize a path for stable identity (dedup). Best-effort on Windows.
pub fn normalize_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    // Prefer canonicalize when the path exists; strip Windows \\?\ prefix.
    match absolute.canonicalize() {
        Ok(c) => strip_verbatim_prefix(c),
        Err(_) => absolute,
    }
}

fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        // \\?\UNC\server\share → \\server\share
        if let Some(unc) = rest.strip_prefix("UNC\\") {
            return PathBuf::from(format!(r"\\{unc}"));
        }
        return PathBuf::from(rest);
    }
    path
}

/// Display a path for UI (lossy UTF-8).
pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn normalize_path_with_spaces() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("my project");
        fs::create_dir_all(&nested).unwrap();
        let n = normalize_path(&nested);
        assert!(n.exists());
        assert!(n.to_string_lossy().contains("my project"));
    }

    #[test]
    fn strip_verbatim_is_idempotent_on_normal_paths() {
        let p = PathBuf::from(r"C:\Users\test");
        assert_eq!(strip_verbatim_prefix(p.clone()), p);
    }
}
