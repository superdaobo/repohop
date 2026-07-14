use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{RepoHopError, Result};
use crate::paths::AppPaths;

const DEFAULT_CONFIG_TEMPLATE: &str = r#"# RepoHop configuration
# Docs: https://github.com/superdaobo/repohop

# Directories to scan for Git repositories (used by `rhop scan`).
# Example (Windows):
# project_roots = ["D:\\Documents\\Projects", "C:\\Users\\you\\code"]
project_roots = []

[scan]
# Maximum directory depth under each project root.
max_depth = 4
"#;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub project_roots: Vec<PathBuf>,
    #[serde(default)]
    pub scan: ScanConfig,
    /// Optional default provider id: "codex" | "claude" | "opencode"
    #[serde(default)]
    pub default_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
}

fn default_max_depth() -> u32 {
    4
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: default_max_depth(),
        }
    }
}

impl AppConfig {
    pub fn load_or_init(paths: &AppPaths) -> Result<Self> {
        paths.ensure_dirs()?;
        if !paths.config_file.exists() {
            fs::write(&paths.config_file, DEFAULT_CONFIG_TEMPLATE)?;
            return Ok(Self::default());
        }
        Self::load_from(&paths.config_file)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .map_err(|e| RepoHopError::Config(format!("read {}: {e}", path.display())))?;
        let cfg: AppConfig = toml::from_str(&text)
            .map_err(|e| RepoHopError::Config(format!("parse {}: {e}", path.display())))?;
        Ok(cfg)
    }

    pub fn max_depth(&self) -> u32 {
        self.scan.max_depth.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_sample_config() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("config.toml");
        fs::write(
            &file,
            r#"
project_roots = ["D:\\code", "C:\\work with spaces"]
default_provider = "codex"
[scan]
max_depth = 3
"#,
        )
        .unwrap();
        let cfg = AppConfig::load_from(&file).unwrap();
        assert_eq!(cfg.project_roots.len(), 2);
        assert_eq!(cfg.max_depth(), 3);
        assert_eq!(cfg.default_provider.as_deref(), Some("codex"));
    }

    #[test]
    fn default_max_depth() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.max_depth(), 4);
    }
}
