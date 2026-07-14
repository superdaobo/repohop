//! Read-only project discovery from agent session metadata.
//!
//! Never modifies agent stores. Prefer official metadata fields (`cwd` / `directory`).

mod claude;
mod codex;
mod grok;
mod opencode;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::paths::normalize_path;
use crate::provider::ProviderId;

/// A project path inferred from agent history (read-only).
#[derive(Debug, Clone)]
pub struct DiscoveredProject {
    pub path: PathBuf,
    pub provider: ProviderId,
    pub last_activity: Option<DateTime<Utc>>,
    pub session_hint: Option<String>,
}

/// Discover projects from all known agent local stores.
pub fn discover_from_agents() -> Result<Vec<DiscoveredProject>> {
    let mut all = Vec::new();
    match codex::discover_projects() {
        Ok(mut v) => {
            tracing::info!(count = v.len(), "codex discovery");
            all.append(&mut v);
        }
        Err(e) => tracing::warn!(error = %e, "codex discovery failed"),
    }
    match claude::discover_projects() {
        Ok(mut v) => {
            tracing::info!(count = v.len(), "claude discovery");
            all.append(&mut v);
        }
        Err(e) => tracing::warn!(error = %e, "claude discovery failed"),
    }
    match opencode::discover_projects() {
        Ok(mut v) => {
            tracing::info!(count = v.len(), "opencode discovery");
            all.append(&mut v);
        }
        Err(e) => tracing::warn!(error = %e, "opencode discovery failed"),
    }
    match grok::discover_projects() {
        Ok(mut v) => {
            tracing::info!(count = v.len(), "grok discovery");
            all.append(&mut v);
        }
        Err(e) => tracing::warn!(error = %e, "grok discovery failed"),
    }
    Ok(merge_discovered(all))
}

/// Merge by normalized path: keep newest activity and prefer any provider hint.
pub fn merge_discovered(items: Vec<DiscoveredProject>) -> Vec<DiscoveredProject> {
    let mut map: HashMap<String, DiscoveredProject> = HashMap::new();
    for item in items {
        if !is_plausible_project_path(&item.path) {
            continue;
        }
        let key = normalize_path(&item.path).to_string_lossy().to_lowercase();
        map.entry(key)
            .and_modify(|existing| {
                let newer = match (existing.last_activity, item.last_activity) {
                    (Some(a), Some(b)) => b > a,
                    (None, Some(_)) => true,
                    _ => false,
                };
                if newer {
                    existing.last_activity = item.last_activity;
                    existing.session_hint = item.session_hint.clone();
                    existing.provider = item.provider;
                }
            })
            .or_insert(item);
    }
    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by_key(|b| std::cmp::Reverse(b.last_activity));
    out
}

fn is_plausible_project_path(path: &Path) -> bool {
    use std::path::Component;
    let s = path.to_string_lossy();
    if s.is_empty() || s == "/" || s == "\\" {
        return false;
    }
    if !path.is_absolute() {
        return false;
    }
    // Need more than a drive root (e.g. C:\) or filesystem root (/).
    let meaningful = path.components().filter(|c| {
        !matches!(
            c,
            Component::Prefix(_) | Component::RootDir | Component::CurDir
        )
    });
    meaningful.count() >= 1
}

/// Home-relative agent data roots (for tests / doctor).
pub fn agent_data_hints() -> Vec<(ProviderId, PathBuf)> {
    let mut out = Vec::new();
    if let Some(h) = directories::BaseDirs::new() {
        let home = h.home_dir();
        out.push((ProviderId::Codex, home.join(".codex").join("sessions")));
        out.push((ProviderId::Claude, home.join(".claude").join("projects")));
        out.push((
            ProviderId::OpenCode,
            home.join(".local")
                .join("share")
                .join("opencode")
                .join("opencode.db"),
        ));
        out.push((ProviderId::Grok, home.join(".grok").join("sessions")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keeps_newest() {
        let a = DiscoveredProject {
            path: PathBuf::from(r"D:\code\app"),
            provider: ProviderId::Codex,
            last_activity: Some(Utc::now() - chrono::Duration::days(2)),
            session_hint: Some("old".into()),
        };
        let b = DiscoveredProject {
            path: PathBuf::from(r"D:\code\app"),
            provider: ProviderId::Claude,
            last_activity: Some(Utc::now()),
            session_hint: Some("new".into()),
        };
        let m = merge_discovered(vec![a, b]);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].provider, ProviderId::Claude);
        assert_eq!(m[0].session_hint.as_deref(), Some("new"));
    }

    #[test]
    fn rejects_root_paths() {
        assert!(!is_plausible_project_path(Path::new("/")));
        assert!(!is_plausible_project_path(Path::new(r"C:\")));
    }
}
