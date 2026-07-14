use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::discover::DiscoveredProject;
use crate::error::Result;
use crate::provider::ProviderId;

/// Max JSONL files to inspect (newest-first by mtime).
const MAX_FILES: usize = 400;
/// Lines to scan per file for session_meta.
const MAX_LINES: usize = 8;

pub fn discover_projects() -> Result<Vec<DiscoveredProject>> {
    let root = sessions_root();
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = list_jsonl_files(&root);
    files.sort_by_key(|b| std::cmp::Reverse(b.1));
    files.truncate(MAX_FILES);

    let mut out = Vec::new();
    for (path, mtime) in files {
        if let Some(mut d) = extract_from_jsonl(&path) {
            if d.last_activity.is_none() {
                d.last_activity = system_time_to_utc(mtime);
            }
            out.push(d);
        }
    }
    Ok(out)
}

fn sessions_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().join(".codex").join("sessions"))
        .unwrap_or_else(|| PathBuf::from(".codex/sessions"))
}

fn list_jsonl_files(root: &Path) -> Vec<(PathBuf, SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
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

fn extract_from_jsonl(path: &Path) -> Option<DiscoveredProject> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        if i >= MAX_LINES {
            break;
        }
        let line = line.ok()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line).ok()?;
        if let Some(d) = project_from_json_value(&v, path) {
            return Some(d);
        }
    }
    None
}

fn project_from_json_value(v: &Value, source: &Path) -> Option<DiscoveredProject> {
    // session_meta: { "type":"session_meta", "payload": { "cwd": "...", "id": "..." } }
    let typ = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let payload = v.get("payload");

    let cwd = if typ == "session_meta" {
        payload.and_then(|p| p.get("cwd")).and_then(|c| c.as_str())
    } else {
        v.get("cwd").and_then(|c| c.as_str())
    }?;

    let path = PathBuf::from(cwd);
    if cwd.trim().is_empty() {
        return None;
    }

    let id = payload
        .and_then(|p| p.get("id"))
        .and_then(|i| i.as_str())
        .or_else(|| v.get("id").and_then(|i| i.as_str()))
        .map(|s| s.to_string());

    let ts = payload
        .and_then(|p| p.get("timestamp"))
        .and_then(|t| t.as_str())
        .or_else(|| v.get("timestamp").and_then(|t| t.as_str()))
        .and_then(parse_rfc3339);

    Some(DiscoveredProject {
        path,
        provider: ProviderId::Codex,
        last_activity: ts,
        session_hint: id.or_else(|| source.file_name().map(|n| n.to_string_lossy().to_string())),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn parses_session_meta_cwd() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("rollout.jsonl");
        let mut f = File::create(&file).unwrap();
        writeln!(
            f,
            r#"{{"timestamp":"2026-01-30T14:14:50.891Z","type":"session_meta","payload":{{"id":"abc","timestamp":"2026-01-30T14:14:43.987Z","cwd":"D:\\Documents\\C_learn\\demo"}}}}"#
        )
        .unwrap();
        let d = extract_from_jsonl(&file).unwrap();
        assert!(d.path.to_string_lossy().contains("demo"));
        assert_eq!(d.provider, ProviderId::Codex);
    }
}
