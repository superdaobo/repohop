//! List Codex sessions for a project (read-only JSONL under ~/.codex/sessions).

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::paths::normalize_path;
use crate::provider::traits::{ProviderId, SessionSummary};

const MAX_FILES: usize = 400;
const MAX_SESSIONS: usize = 80;
const META_LINES: usize = 12;

pub fn list_sessions_for_project(project: &Path) -> Vec<SessionSummary> {
    let root = sessions_root();
    if !root.is_dir() {
        return Vec::new();
    }
    let target_key = path_key(project);
    let mut files = list_jsonl_files(&root);
    files.sort_by_key(|b| std::cmp::Reverse(b.1));
    files.truncate(MAX_FILES);

    let mut out = Vec::new();
    for (path, mtime) in files {
        if let Some(mut s) = extract_session(&path, project, &target_key) {
            if s.updated_at.is_none() {
                s.updated_at = system_time_to_utc(mtime);
            }
            out.push(s);
        }
        if out.len() >= MAX_SESSIONS {
            break;
        }
    }
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    out
}

fn sessions_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().join(".codex").join("sessions"))
        .unwrap_or_else(|| PathBuf::from(".codex/sessions"))
}

fn path_key(p: &Path) -> String {
    normalize_path(p)
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn list_jsonl_files(root: &Path) -> Vec<(PathBuf, SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("jsonl"))
            {
                let mtime = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                out.push((path, mtime));
            }
        }
    }
    out
}

fn extract_session(path: &Path, project: &Path, target_key: &str) -> Option<SessionSummary> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut id: Option<String> = None;
    let mut cwd: Option<PathBuf> = None;
    let mut ts: Option<DateTime<Utc>> = None;

    for (i, line) in reader.lines().enumerate() {
        if i >= META_LINES {
            break;
        }
        let line = line.ok()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line).ok()?;
        let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if ty != "session_meta" && !line.contains("cwd") {
            continue;
        }
        let payload = v.get("payload");
        let cwd_s = payload
            .and_then(|p| p.get("cwd"))
            .and_then(|c| c.as_str())
            .or_else(|| v.get("cwd").and_then(|c| c.as_str()));
        if let Some(c) = cwd_s {
            cwd = Some(PathBuf::from(c));
        }
        let sid = payload
            .and_then(|p| p.get("id"))
            .and_then(|i| i.as_str())
            .or_else(|| v.get("id").and_then(|i| i.as_str()));
        if let Some(s) = sid {
            id = Some(s.to_string());
        }
        let t = payload
            .and_then(|p| p.get("timestamp"))
            .and_then(|t| t.as_str())
            .or_else(|| v.get("timestamp").and_then(|t| t.as_str()))
            .and_then(parse_rfc3339);
        if t.is_some() {
            ts = t;
        }
        if cwd.is_some() && id.is_some() {
            break;
        }
    }

    let cwd = cwd?;
    if path_key(&cwd) != *target_key {
        return None;
    }
    let id = id.unwrap_or_else(|| {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string())
    });
    let title = format!("Session {}", short_id(&id));
    Some(SessionSummary {
        id,
        provider: ProviderId::Codex,
        project_path: project.to_path_buf(),
        title: title.clone(),
        preview: title,
        created_at: ts,
        updated_at: ts,
        git_branch: None,
        source_path: Some(path.to_path_buf()),
    })
}

fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
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
