# RepoHop Architecture

## Overview

RepoHop is a Rust CLI (`rhop`) that routes the user to an external AI coding agent process. Core design: **Provider Adapter** + **local SQLite index** + **foreground process launch** (inherit stdio). No PTY, no model APIs.

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│ CLI (clap)  │────▶│ App services │────▶│ Agent process   │
└─────────────┘     │ config/db/   │     │ (codex/claude/  │
       │            │ project/ui   │     │  opencode)      │
       ▼            └──────┬───────┘     └─────────────────┘
┌─────────────┐            │
│ Providers   │◀───────────┘
│ (adapters)  │
└─────────────┘
```

## Module boundaries

| Module | Owns | Must not own |
|--------|------|--------------|
| `cli` | Argument parsing, subcommand dispatch | Provider-specific flags |
| `config` / `paths` | TOML + OS paths | Agent session formats |
| `provider` | Detect, version, capabilities, command build, session I/O (later) | SQLite schema |
| `project` | Scan, rank, project model | Process spawn |
| `db` | Migrations, projects, launches | Provider CLI flags |
| `launch` | `CommandSpec` execution | Ranking policy |
| `ui` | Ratatui interaction | Persistence |
| `doctor` | Human-readable diagnostics | Mutations beyond config ensure |
| `git` | Repo detection; later worktree safety | Agent sessions |

## Key data structures

- **`CommandSpec`**: `executable`, `args`, `cwd`, `env`, `display_command`
- **`ProviderId`**: `Codex`, `Claude`, `OpenCode` (+ reserved extension points)
- **`DetectedAgent`**: provider, path, version, notes
- **`Project`**: path, name, favorite, launch stats
- **`LaunchRecord`**: project, provider, mode, time, display command
- **`SessionSummary`** (Stage 3): unified session row for UI
- **`AppConfig`**: `project_roots`, `scan.max_depth`, optional defaults

## Provider Adapter

```text
trait AgentProvider {
  detect, version, capabilities,
  list_sessions, build_new_command, build_resume_command, validate_session
}
```

Registry returns static providers. Business code selects by `ProviderId` via registry only.

**Windows detection:** search PATH for `name.exe`, `name.cmd`, `name.bat`, then `name`. Prefer non-`.ps1` for spawn.

Stage 2: `list_sessions` returns empty; `build_resume_command` returns not-implemented error.

## Project Discovery

1. Ensure config exists (empty `project_roots` allowed).
2. `rhop scan` walks each root to `max_depth`, records directories with `.git`.
3. Interactive list merges DB projects + optional cwd for `rhop .`.
4. Ranking: favorite → `last_launched_at` → `launch_count` → name.
5. Missing paths: show as missing; do not crash entire list.

No whole-disk scan. No zoxide in Stage 2.

## Session Index

Stage 2: only **launch history** in SQLite.  
Stage 3: read-only adapters over agent stores / official CLI list; cache metadata in RepoHop DB without full transcripts.

## Git Worktree

Stage 4 module responsibilities:

- Validate git repo, base branch, no merge/rebase/cherry-pick in progress
- Slugify task name → `ai/<slug>`
- Create worktree under managed root
- Safety on delete: dirty / unpushed checks
- Never `reset --hard`, `clean -fd`, auto-stash, force delete

## Process Launcher

- Build `CommandSpec` from provider + project cwd
- `std::process::Command` with inherited stdin/stdout/stderr
- Wait for process exit; record launch regardless of exit code when start succeeds
- Do not implement background daemons or terminal tabs

## SQLite

Location: `%LOCALAPPDATA%\RepoHop\repohop.db`  
Migrations versioned in `schema_migrations`.

Tables (Stage 2):

- `projects` — path unique, stats, favorite
- `launches` — history rows

Use `rusqlite` with `bundled` feature.

## Windows Shell Integration

Stage 2: plain executable on PATH (user adds or cargo run).  
Stage 5 / Issue 11: optional profile helpers, PATH install via `install.ps1`.

Path handling: `PathBuf`, `dunce` or careful canonicalize for dedup; support spaces, non-ASCII usernames, multi-drive. UNC: clear error if unsupported.

## Security boundaries

- Local-only data; no network required for core hop
- No telemetry
- Agent auth remains with agent tools
- Do not write agent session files
- Logs avoid secrets

## Failure and degradation

| Failure | Behavior |
|---------|----------|
| No agents installed | `doctor` exit 1; interactive explains install |
| One agent missing | doctor warns; others still usable |
| Empty project_roots | doctor/scan instruct user to edit config |
| Project path deleted | list marks missing; skip launch with error |
| Agent spawn fails | error with display_command and path |
| DB locked / corrupt | actionable error; do not delete user DB silently |
| Non-TTY interactive | error asking for terminal |

## Dependency policy

Prefer small crates listed in PRD/stack. Any new major dependency requires rationale + alternative in this document and THIRD_PARTY_NOTICES update.
