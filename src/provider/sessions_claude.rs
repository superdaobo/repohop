//! List Claude Code sessions for a project (read-only JSONL under ~/.claude/projects).

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::paths::normalize_path;
use crate::provider::traits::{ProviderId, SessionSummary};

const MAX_SESSIONS: usize = 80;
const PREVIEW_LINES: usize = 60;

pub fn list_sessions_for_project(project: &Path) -> Vec<SessionSummary> {
    let root = projects_root();
    if !root.is_dir() {
        return Vec::new();
    }
    let target = normalize_path(project);
    let target_key = path_key(&target);

    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return out;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        // Match project dir by scanning a few JSONL files for cwd, or dirname decode.
        if !dir_matches_project(&dir, &target_key) {
            continue;
        }
        out.extend(sessions_in_dir(&dir, &target));
    }
    out.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
    out.truncate(MAX_SESSIONS);
    out
}

fn projects_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().join(".claude").join("projects"))
        .unwrap_or_else(|| PathBuf::from(".claude/projects"))
}

fn path_key(p: &Path) -> String {
    normalize_path(p)
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn dir_matches_project(dir: &Path, target_key: &str) -> bool {
    // Quick path: any jsonl with matching cwd.
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten().take(8) {
            let p = e.path();
            if p.extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case("jsonl"))
            {
                if let Some(cwd) = first_cwd_in_jsonl(&p) {
                    if path_key(&cwd) == target_key {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn first_cwd_in_jsonl(path: &Path) -> Option<PathBuf> {
    let file = File::open(path).ok()?;
    for line in BufReader::new(file).lines().take(PREVIEW_LINES).flatten() {
        let line = line.trim();
        if line.is_empty() || !line.contains("cwd") {
            continue;
        }
        let v: Value = serde_json::from_str(line).ok()?;
        if let Some(cwd) = v.get("cwd").and_then(|c| c.as_str()) {
            if !cwd.trim().is_empty() {
                return Some(PathBuf::from(cwd));
            }
        }
    }
    None
}

fn sessions_in_dir(dir: &Path, project: &Path) -> Vec<SessionSummary> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in rd.flatten() {
        let path = e.path();
        if !path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("jsonl"))
        {
            continue;
        }
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        let mtime = e
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(system_time_to_utc);
        let (title, preview, created) = peek_session_jsonl(&path, &id);
        out.push(SessionSummary {
            id: id.clone(),
            provider: ProviderId::Claude,
            project_path: project.to_path_buf(),
            title,
            preview,
            created_at: created,
            updated_at: mtime.or(created),
            git_branch: None,
            source_path: Some(path),
        });
    }
    out
}

fn peek_session_jsonl(path: &Path, fallback_id: &str) -> (String, String, Option<DateTime<Utc>>) {
    let mut title = format!("Session {}", short_id(fallback_id));
    let mut preview = String::new();
    let mut created = None;
    let Ok(file) = File::open(path) else {
        return (title, preview, created);
    };
    for line in BufReader::new(file).lines().take(PREVIEW_LINES).flatten() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if created.is_none() {
            created = v
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(parse_rfc3339);
        }
        let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if ty == "user" || ty == "human" {
            if let Some(text) = extract_text(&v) {
                if !text.is_empty() {
                    title = truncate(&text, 60);
                    preview = truncate(&text, 120);
                    break;
                }
            }
        }
        if preview.is_empty() {
            if let Some(text) = extract_text(&v) {
                if !text.is_empty() {
                    preview = truncate(&text, 120);
                }
            }
        }
    }
    (title, preview, created)
}

fn extract_text(v: &Value) -> Option<String> {
    if let Some(s) = v.get("content").and_then(|c| c.as_str()) {
        return Some(s.to_string());
    }
    if let Some(arr) = v
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
    {
        let mut parts = Vec::new();
        for item in arr {
            if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                parts.push(t);
            } else if let Some(t) = item.as_str() {
                parts.push(t);
            }
        }
        if !parts.is_empty() {
            return Some(parts.join(" "));
        }
    }
    v.get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
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
