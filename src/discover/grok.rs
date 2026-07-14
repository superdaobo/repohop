//! Read-only discovery from Grok Build CLI session directories.
//!
//! Layout: `~/.grok/sessions/<percent-encoded-absolute-path>/...`
//! Never modifies agent stores.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};

use crate::discover::DiscoveredProject;
use crate::error::Result;
use crate::provider::grok::percent_decode_path_name;
use crate::provider::ProviderId;

pub fn discover_projects() -> Result<Vec<DiscoveredProject>> {
    let root = sessions_root();
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        // Skip non-encoded names (e.g. internal dirs).
        if !name.contains('%') {
            continue;
        }
        let Some(project_path) = percent_decode_path_name(&name) else {
            tracing::debug!(dir = %name, "skip undecodable grok session dir");
            continue;
        };
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(system_time_to_utc)
            .or_else(|| dir_newest_mtime(&path));

        out.push(DiscoveredProject {
            path: project_path,
            provider: ProviderId::Grok,
            last_activity: mtime,
            session_hint: Some(format!("grok:{}", name)),
        });
    }
    Ok(out)
}

fn sessions_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().join(".grok").join("sessions"))
        .unwrap_or_else(|| PathBuf::from(".grok/sessions"))
}

fn dir_newest_mtime(dir: &Path) -> Option<DateTime<Utc>> {
    let mut best: Option<SystemTime> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if let Ok(m) = meta.modified() {
                best = Some(match best {
                    Some(b) if b > m => b,
                    _ => m,
                });
            }
        }
    }
    best.and_then(system_time_to_utc)
}

fn system_time_to_utc(t: SystemTime) -> Option<DateTime<Utc>> {
    let dur = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    DateTime::from_timestamp(dur.as_secs() as i64, dur.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_root_ends_with_sessions() {
        let r = sessions_root();
        assert!(r.ends_with("sessions") || r.to_string_lossy().contains("sessions"));
    }
}
