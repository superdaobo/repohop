use std::path::PathBuf;

/// Portable description of a process to launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub display_command: String,
}

impl CommandSpec {
    pub fn new(executable: PathBuf, args: Vec<String>, cwd: PathBuf) -> Self {
        let display_command = format_display(&executable, &args, &cwd);
        Self {
            executable,
            args,
            cwd,
            env: Vec::new(),
            display_command,
        }
    }

    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }
}

pub fn format_display(
    executable: &std::path::Path,
    args: &[String],
    cwd: &std::path::Path,
) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(quote_if_needed(&executable.display().to_string()));
    for a in args {
        parts.push(quote_if_needed(a));
    }
    format!("{}  (cwd: {})", parts.join(" "), cwd.display())
}

fn quote_if_needed(s: &str) -> String {
    if s.is_empty() || s.contains(char::is_whitespace) {
        format!("\"{s}\"")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn display_quotes_spaces() {
        let exe = PathBuf::from(r"C:\Program Files\codex.cmd");
        let cwd = PathBuf::from(r"D:\my project");
        let d = format_display(&exe, &[], &cwd);
        assert!(d.contains("\"C:\\Program Files\\codex.cmd\"") || d.contains("Program Files"));
        assert!(d.contains("my project"));
    }

    #[test]
    fn new_sets_display() {
        let spec = CommandSpec::new(
            PathBuf::from("codex.cmd"),
            vec![],
            PathBuf::from(r"D:\code"),
        );
        assert!(spec.display_command.contains("codex.cmd"));
        assert_eq!(spec.cwd.as_path(), Path::new(r"D:\code"));
    }
}
