# AGENTS.md — working on RepoHop

This file is for coding agents and humans implementing RepoHop.

## Product one-liner

Local-first workspace router for AI coding CLIs. Binary: **`rhop`**. Does not implement LLMs.

## Commands

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
cargo run -- doctor
cargo run -- scan
cargo run -- version
```

## Module map

| Module | Responsibility |
|--------|----------------|
| `cli` | clap surface |
| `config` / `paths` | TOML config + Windows paths |
| `provider` | `AgentProvider` trait + Codex/Claude/OpenCode |
| `project` | scan, rank, models |
| `db` | SQLite migrations + projects/launches |
| `launch` | `CommandSpec` process spawn (inherit stdio) |
| `ui` | Ratatui pickers |
| `doctor` | environment report |
| `git` | minimal repo detection (Stage 2) |

## Hard rules

1. Provider logic lives in `provider/*` only.
2. Never write/modify agent session stores.
3. No destructive git in Stage 2; later stages require safety checks.
4. Use `PathBuf`; no Windows path string concatenation.
5. Prefer unit-testable services over TUI-only logic.
6. Document agent CLI assumptions in `docs/SESSION_COMPATIBILITY.md` when flags change.
7. Small Conventional Commits; do not mix unrelated issues.

## Stage boundaries

- **Stage 2 (current):** doctor, scan, pick project/agent, **new session**, launch history.
- **Stage 3:** session list/resume adapters.
- **Stage 4:** safe worktrees.
- **Stage 5:** release + install.ps1.

## Windows notes

npm-installed agents often appear as `.ps1` + `.cmd` shims. Prefer spawning `.cmd` / `.exe` from Rust (`std::process::Command` does not run PowerShell scripts the way the shell does).
