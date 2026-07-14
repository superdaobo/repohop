use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::discover::DiscoveredProject;
use crate::error::Result;
use crate::provider::ProviderId;

const MAX_LINES: usize = 40;
const MAX_PROJECT_DIRS: usize = 300;

pub fn discover_projects() -> Result<Vec<DiscoveredProject>> {
    let root = projects_root();
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .map_err(crate::error::RepoHopError::Io)?
        .flatten()
        .filter(|e| e.path().is_dir())
        .collect();
    dirs.sort_by_key(|e| {
        std::cmp::Reverse(
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH),
        )
    });
    dirs.truncate(MAX_PROJECT_DIRS);

    for entry in dirs {
        let dir = entry.path();
        if let Some(d) = discover_one_project_dir(&dir) {
            out.push(d);
        }
    }
    Ok(out)
}

fn projects_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|b| b.home_dir().join(".claude").join("projects"))
        .unwrap_or_else(|| PathBuf::from(".claude/projects"))
}

fn discover_one_project_dir(dir: &Path) -> Option<DiscoveredProject> {
    // Prefer cwd from session JSONL (accurate, handles Unicode).
    if let Some(d) = scan_dir_for_cwd(dir) {
        return Some(d);
    }
    // Fallback: best-effort decode of Claude's encoded folder name.
    let name = dir.file_name()?.to_string_lossy();
    let decoded = decode_claude_project_dirname(&name)?;
    let path = PathBuf::from(&decoded);
    if !path.is_absolute() {
        return None;
    }
    let mtime = std::fs::metadata(dir)
        .and_then(|m| m.modified())
        .ok()
        .and_then(system_time_to_utc);
    Some(DiscoveredProject {
        path,
        provider: ProviderId::Claude,
        last_activity: mtime,
        session_hint: Some(name.to_string()),
    })
}

fn scan_dir_for_cwd(dir: &Path) -> Option<DiscoveredProject> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("jsonl"))
        })
        .collect();
    files.sort_by_key(|p| {
        std::cmp::Reverse(
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH),
        )
    });

    for file in files.into_iter().take(5) {
        if let Some(d) = extract_cwd_from_jsonl(&file) {
            return Some(d);
        }
    }
    None
}

fn extract_cwd_from_jsonl(path: &Path) -> Option<DiscoveredProject> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut best: Option<DiscoveredProject> = None;
    for (i, line) in reader.lines().enumerate() {
        if i >= MAX_LINES {
            break;
        }
        let line = line.ok()?;
        let line = line.trim();
        if line.is_empty() || !line.contains("cwd") {
            continue;
        }
        let v: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(cwd) = v.get("cwd").and_then(|c| c.as_str()) {
            if cwd.trim().is_empty() {
                continue;
            }
            let ts = v
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(parse_rfc3339);
            let sid = v
                .get("sessionId")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let candidate = DiscoveredProject {
                path: PathBuf::from(cwd),
                provider: ProviderId::Claude,
                last_activity: ts,
                session_hint: sid,
            };
            let take = match (&best, ts) {
                (None, _) => true,
                (Some(b), Some(t)) => b.last_activity.map(|x| t > x).unwrap_or(true),
                _ => false,
            };
            if take {
                best = Some(candidate);
            }
        }
    }
    best
}

/// Best-effort reverse of Claude Code project directory encoding.
/// Observed form: `D--Documents-C-learn-foo` ≈ `D:\Documents\C_learn\foo`
/// Not perfect for all paths; JSONL `cwd` is preferred.
pub fn decode_claude_project_dirname(name: &str) -> Option<String> {
    if name.is_empty() || name == "-" {
        return None;
    }
    // Windows drive: starts with letter then `--` after missing colon.
    // e.g. D--Documents-... → D:\Documents\...
    let mut s = name.to_string();
    if s.len() >= 3 {
        let bytes = s.as_bytes();
        if bytes[0].is_ascii_alphabetic() && &s[1..3] == "--" {
            let drive = s.chars().next()?.to_ascii_uppercase();
            s = format!("{drive}:\\{}", s[3..].replace('-', "\\"));
            // Collapse accidental empty segments from double dashes in names
            while s.contains("\\\\") {
                s = s.replace("\\\\", "\\");
            }
            return Some(s);
        }
    }
    // Unix-like: leading - for root
    if s.starts_with('-') {
        s = s.replacen('-', "/", 1).replace('-', "/");
        while s.contains("//") {
            s = s.replace("//", "/");
        }
        return Some(s);
    }
    None
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
    fn decode_drive_path() {
        let d = decode_claude_project_dirname("D--Documents-C-learn-demo").unwrap();
        assert!(d.starts_with(r"D:\"));
        assert!(d.contains("Documents"));
    }

    #[test]
    fn parse_cwd_line() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("sess.jsonl");
        let mut f = File::create(&file).unwrap();
        writeln!(
            f,
            r#"{{"type":"user","cwd":"D:\\Documents\\demo","sessionId":"abc","timestamp":"2026-05-19T01:49:40.394Z"}}"#
        )
        .unwrap();
        let d = extract_cwd_from_jsonl(&file).unwrap();
        assert!(d.path.to_string_lossy().contains("demo"));
    }
}
