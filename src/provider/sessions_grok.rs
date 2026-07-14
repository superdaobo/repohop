//! List Grok Build sessions for a project (read-only under ~/.grok/sessions).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::paths::normalize_path;
use crate::provider::grok::percent_decode_path_name;
use crate::provider::traits::{ProviderId, SessionSummary};

const MAX_SESSIONS: usize = 80;

pub fn list_sessions_for_project(project: &Path) -> Vec<SessionSummary> {
    let root = sessions_root();
    if !root.is_dir() {
        return Vec::new();
    }
    let target_key = path_key(project);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };

    let mut project_dir: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        if !name.contains('%') {
            continue;
        }
        let Some(decoded) = percent_decode_path_name(&name) else {
            continue;
        };
        if path_key(&decoded) == target_key {
            project_dir = Some(path);
            break;
        }
    }

    let Some(project_dir) = project_dir else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let Ok(sessions) = std::fs::read_dir(&project_dir) else {
        return out;
    };
    for entry in sessions.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        // UUID-like session folders
        if id.len() < 8 || id == "prompt_history.jsonl" {
            continue;
        }
        let summary_path = path.join("summary.json");
        let (title, preview, created, updated) = read_summary(&summary_path, &id);
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(system_time_to_utc);
        out.push(SessionSummary {
            id: id.clone(),
            provider: ProviderId::Grok,
            project_path: project.to_path_buf(),
            title,
            preview,
            created_at: created,
            updated_at: updated.or(mtime),
            git_branch: None,
            source_path: Some(path),
        });
    }
    out.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
    out.truncate(MAX_SESSIONS);
    out
}

fn sessions_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().join(".grok").join("sessions"))
        .unwrap_or_else(|| PathBuf::from(".grok/sessions"))
}

fn path_key(p: &Path) -> String {
    normalize_path(p)
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn read_summary(
    path: &Path,
    fallback_id: &str,
) -> (String, String, Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    let default_title = format!("Session {}", short_id(fallback_id));
    let Ok(text) = std::fs::read_to_string(path) else {
        return (default_title, String::new(), None, None);
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return (default_title, String::new(), None, None);
    };
    let title = v
        .get("generated_title")
        .or_else(|| v.get("session_summary"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_title);
    let preview = v
        .get("session_summary")
        .and_then(|t| t.as_str())
        .map(|s| truncate(s, 120))
        .unwrap_or_default();
    let created = v
        .get("created_at")
        .and_then(|t| t.as_str())
        .and_then(parse_rfc3339);
    let updated = v
        .get("last_active_at")
        .or_else(|| v.get("updated_at"))
        .and_then(|t| t.as_str())
        .and_then(parse_rfc3339);
    (title, preview, created, updated)
}

fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ").trim().to_string();
    if s.chars().count() <= max {
        s
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}

fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

fn system_time_to_utc(t: SystemTime) -> Option<DateTime<Utc>> {
    let dur = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    DateTime::from_timestamp(dur.as_secs() as i64, dur.subsec_nanos())
}
