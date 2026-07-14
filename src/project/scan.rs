use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::git;

/// Walk `root` up to `max_depth` looking for Git repositories.
pub fn scan_git_projects(root: &Path, max_depth: u32) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    if !root.exists() {
        tracing::warn!(path = %root.display(), "project root does not exist, skipping");
        return Ok(found);
    }
    walk(root, 0, max_depth, &mut found)?;
    Ok(found)
}

fn walk(dir: &Path, depth: u32, max_depth: u32, out: &mut Vec<PathBuf>) -> Result<()> {
    if git::is_git_repo(dir) {
        out.push(dir.to_path_buf());
        // Do not descend into nested repos by default once found.
        return Ok(());
    }
    if depth >= max_depth {
        return Ok(());
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!(path = %dir.display(), error = %e, "skip unreadable dir");
            return Ok(());
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip common heavy / irrelevant dirs
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if matches!(
                    name,
                    "node_modules" | "target" | ".git" | "vendor" | "dist" | ".repo"
                ) {
                    continue;
                }
            }
            walk(&path, depth + 1, max_depth, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_nested_git() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("apps").join("demo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        let found = scan_git_projects(dir.path(), 4).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("demo"));
    }
}
