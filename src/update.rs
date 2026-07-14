//! Self-update from GitHub Releases (Windows primary).

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::{RepoHopError, Result};
use crate::paths::AppPaths;

const DEFAULT_REPO: &str = "superdaobo/repohop";
const ASSET_NAME: &str = "rhop-x86_64-pc-windows-msvc.exe";
const CHECKSUM_NAME: &str = "SHA256SUMS.txt";
const USER_AGENT: &str = "RepoHop-Updater";
/// Minimum interval between automatic background checks.
const AUTO_CHECK_INTERVAL_SECS: u64 = 6 * 60 * 60;

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current: String,
    pub latest_tag: String,
    pub latest_version: String,
    pub asset_url: String,
    pub checksum_url: Option<String>,
    pub update_available: bool,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn repo() -> String {
    env::var("REPOPHOP_REPO").unwrap_or_else(|_| DEFAULT_REPO.into())
}

fn github_token() -> Option<String> {
    env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| env::var("GH_TOKEN").ok())
        .filter(|s| !s.is_empty())
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(60))
        .user_agent(USER_AGENT)
        .build()
}

fn apply_auth(mut req: ureq::Request) -> ureq::Request {
    req = req.set("Accept", "application/vnd.github+json");
    if let Some(token) = github_token() {
        req = req.set("Authorization", &format!("Bearer {token}"));
    }
    req
}

/// Fetch latest release metadata and compare to the running binary.
pub fn check_for_update() -> Result<UpdateInfo> {
    let current = current_version().to_string();
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo());
    let body = apply_auth(agent().get(&url))
        .call()
        .map_err(|e| RepoHopError::Update(format!("GitHub API: {e}")))?
        .into_string()
        .map_err(|e| RepoHopError::Update(format!("read response: {e}")))?;
    let release: GhRelease = serde_json::from_str(&body)
        .map_err(|e| RepoHopError::Update(format!("parse release JSON: {e}")))?;

    let latest_tag = release.tag_name.clone();
    let latest_version = strip_v(&latest_tag);
    let asset_url = release
        .assets
        .iter()
        .find(|a| a.name == ASSET_NAME)
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| {
            RepoHopError::Update(format!("release {latest_tag} has no asset {ASSET_NAME}"))
        })?;
    let checksum_url = release
        .assets
        .iter()
        .find(|a| a.name == CHECKSUM_NAME)
        .map(|a| a.browser_download_url.clone());

    let update_available = is_newer(&latest_version, &current);
    Ok(UpdateInfo {
        current,
        latest_tag,
        latest_version,
        asset_url,
        checksum_url,
        update_available,
    })
}

/// Soft check used on interactive startup: only hits network if enough time passed.
///
/// Env:
/// - `REPOPHOP_NO_UPDATE=1` — skip entirely
/// - `REPOPHOP_UPDATE_CHECK_ONLY=1` — check but do not auto-install (banner only)
pub fn maybe_auto_check(paths: &AppPaths) -> Option<UpdateInfo> {
    if env::var_os("REPOPHOP_NO_UPDATE").is_some() {
        return None;
    }
    let stamp = paths.data_dir.join("last_update_check");
    if !should_check(&stamp) {
        return None;
    }
    let info = check_for_update().ok()?;
    let _ = write_stamp(&stamp);
    Some(info)
}

fn should_check(stamp: &Path) -> bool {
    match fs::metadata(stamp).and_then(|m| m.modified()) {
        Ok(modified) => match SystemTime::now().duration_since(modified) {
            Ok(d) => d.as_secs() >= AUTO_CHECK_INTERVAL_SECS,
            Err(_) => true,
        },
        Err(_) => true,
    }
}

fn write_stamp(stamp: &Path) -> Result<()> {
    if let Some(parent) = stamp.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(stamp, current_version())?;
    Ok(())
}

/// Download latest release binary and replace the running executable (Windows-friendly).
pub fn apply_update(info: &UpdateInfo) -> Result<PathBuf> {
    if !info.update_available {
        return Err(RepoHopError::Update("already up to date".into()));
    }

    let current_exe = env::current_exe().map_err(RepoHopError::Io)?;
    let current_exe = fs::canonicalize(&current_exe).unwrap_or(current_exe);
    let install_dir = current_exe
        .parent()
        .ok_or_else(|| RepoHopError::Update("cannot resolve install directory".into()))?
        .to_path_buf();

    let tmp_dir = env::temp_dir().join(format!("rhop-update-{}", uuid::Uuid::new_v4().as_simple()));
    fs::create_dir_all(&tmp_dir)?;
    let tmp_exe = tmp_dir.join(ASSET_NAME);

    eprintln!("Downloading {} ...", info.latest_tag);
    download_file(&info.asset_url, &tmp_exe)?;

    if let Some(sum_url) = &info.checksum_url {
        match download_string(sum_url) {
            Ok(sums) => {
                if let Some(expected) = find_checksum(&sums, ASSET_NAME) {
                    let actual = sha256_file(&tmp_exe)?;
                    if !actual.eq_ignore_ascii_case(&expected) {
                        let _ = fs::remove_dir_all(&tmp_dir);
                        return Err(RepoHopError::Update(format!(
                            "SHA-256 mismatch (expected {expected}, got {actual})"
                        )));
                    }
                    eprintln!("Checksum OK.");
                }
            }
            Err(e) => tracing::warn!(error = %e, "checksum download failed; continuing"),
        }
    }

    // On Windows, replace in-use exe via rename dance + batch helper.
    let target = install_dir.join("rhop.exe");
    let bak = install_dir.join("rhop.exe.bak");
    let _ = fs::remove_file(&bak);
    if target.exists() {
        fs::rename(&target, &bak).map_err(|e| {
            RepoHopError::Update(format!(
                "could not backup current binary (is it locked?): {e}"
            ))
        })?;
    }
    fs::copy(&tmp_exe, &target).map_err(|e| {
        // Try restore
        let _ = fs::rename(&bak, &target);
        RepoHopError::Update(format!("install failed: {e}"))
    })?;
    // Also copy long asset name for install.ps1 compatibility.
    let long_name = install_dir.join(ASSET_NAME);
    let _ = fs::copy(&tmp_exe, &long_name);
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::remove_file(&bak);

    eprintln!(
        "Updated rhop {} → {} at {}",
        info.current,
        info.latest_version,
        target.display()
    );
    Ok(target)
}

/// CLI entry: check and optionally apply.
pub fn run_update_cli(apply: bool) -> Result<()> {
    println!("Current version: {}", current_version());
    println!("Checking GitHub releases ({}) ...", repo());
    let info = check_for_update()?;
    println!(
        "Latest release:  {} ({})",
        info.latest_tag, info.latest_version
    );
    if !info.update_available {
        println!("Already up to date.");
        return Ok(());
    }
    println!(
        "Update available: {} → {}",
        info.current, info.latest_version
    );
    if !apply {
        println!("Run `rhop update --apply` to download and install.");
        return Ok(());
    }
    let path = apply_update(&info)?;
    println!("Installed: {}", path.display());
    println!("Restart `rhop` to use the new version.");
    Ok(())
}

/// After a successful update from within the TUI, optionally re-exec is not needed;
/// we just print a message. Helper to spawn install.ps1 as fallback.
#[allow(dead_code)]
pub fn spawn_install_script() -> Result<()> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/main/install.ps1",
        repo()
    );
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!("irm '{url}' | iex"),
        ])
        .status()
        .map_err(|e| RepoHopError::Update(format!("powershell: {e}")))?;
    if !status.success() {
        return Err(RepoHopError::Update(format!(
            "install.ps1 exited with {status}"
        )));
    }
    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    let resp = apply_auth(agent().get(url))
        .call()
        .map_err(|e| RepoHopError::Update(format!("download: {e}")))?;
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(dest)?;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| RepoHopError::Update(format!("read body: {e}")))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
    }
    Ok(())
}

fn download_string(url: &str) -> Result<String> {
    apply_auth(agent().get(url))
        .call()
        .map_err(|e| RepoHopError::Update(format!("download: {e}")))?
        .into_string()
        .map_err(|e| RepoHopError::Update(format!("read: {e}")))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn find_checksum(sums: &str, asset: &str) -> Option<String> {
    for line in sums.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // "hash  filename" or "hash *filename"
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        if name == asset || name.ends_with(asset) {
            return Some(hash.to_string());
        }
    }
    None
}

fn strip_v(tag: &str) -> String {
    tag.trim()
        .strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag)
        .to_string()
}

/// Compare dotted semver-ish versions; true if `latest` is greater than `current`.
pub fn is_newer(latest: &str, current: &str) -> bool {
    let l = parse_semver(latest);
    let c = parse_semver(current);
    l > c
}

fn parse_semver(s: &str) -> (u64, u64, u64) {
    let mut parts = s.split(['.', '-', '+']);
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare() {
        assert!(is_newer("0.1.3", "0.1.2"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.2", "0.1.2"));
        assert!(!is_newer("0.1.1", "0.1.2"));
    }

    #[test]
    fn strip_v_prefix() {
        assert_eq!(strip_v("v0.1.2"), "0.1.2");
        assert_eq!(strip_v("0.1.2"), "0.1.2");
    }

    #[test]
    fn parse_checksum_line() {
        let sums = "abc123  rhop-x86_64-pc-windows-msvc.exe\ndef456 *other.exe\n";
        assert_eq!(find_checksum(sums, ASSET_NAME).as_deref(), Some("abc123"));
    }
}
