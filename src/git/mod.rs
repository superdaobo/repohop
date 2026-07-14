use std::path::Path;

/// True if `path` is a Git working tree or bare repo root (`.git` file or directory).
pub fn is_git_repo(path: &Path) -> bool {
    let git = path.join(".git");
    git.is_dir() || git.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_git_dir() {
        let d = tempdir().unwrap();
        assert!(!is_git_repo(d.path()));
        fs::create_dir(d.path().join(".git")).unwrap();
        assert!(is_git_repo(d.path()));
    }
}
