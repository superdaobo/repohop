# RepoHop

**Hop into the right repo, agent, and session.**

RepoHop (`rhop`) is a **local-first workspace router** for AI coding CLIs. It helps you pick a recent project, choose Codex / Claude Code / OpenCode / Grok Build CLI, and start a session in the right directory—optionally in an isolated Git worktree (roadmap).

RepoHop does **not** run models, proxy APIs, or replace agent CLIs. It only handles project discovery, agent detection, session indexing, safe worktree management, launch command construction, and local history.

## Status

Stage 2 minimal loop (in progress / early):

- `rhop doctor` — detect installed agent CLIs
- `rhop scan` — scan configured `project_roots` for Git repos
- Interactive project + agent selection → **new session** launch
- SQLite launch history

Session resume, full worktree UX, and install scripts are planned (see [docs/ROADMAP.md](docs/ROADMAP.md)).

## Requirements

- Windows 11 x86_64 (primary)
- PowerShell 7 or Windows PowerShell 5.1
- Windows Terminal recommended
- At least one of: [Codex CLI](https://github.com/openai/codex), [Claude Code](https://code.claude.com), [OpenCode](https://opencode.ai), [Grok Build CLI](https://grok.x.ai)

## Install (Windows)

One-liner (PowerShell 5.1 / 7, no admin):

```powershell
irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex
```

This downloads the latest GitHub Release binary into `%LOCALAPPDATA%\RepoHop\bin` and adds it to your user `PATH`. Open a new terminal, then run `rhop version`.

Pin a version:

```powershell
$env:REPOPHOP_VERSION = 'v0.1.0'
irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex
```

Uninstall:

```powershell
irm https://raw.githubusercontent.com/superdaobo/repohop/main/uninstall.ps1 | iex
```

### Install (development)

```powershell
git clone https://github.com/superdaobo/repohop.git
cd repohop
cargo build --release
# binary: target\release\rhop.exe
```

## Quick start

**Zero config.** After install, just run:

```powershell
rhop doctor   # see agents + auto-discovered projects
rhop scan     # refresh project list from agent session history
rhop          # multi-level hop: project → tool → chat
rhop .        # use current directory as project
rhop update   # check GitHub for a newer release
rhop update --apply  # download and install
```

Interactive flow stays in **one** screen:

1. **Project** — Name | Path | Last used · `.` = cwd · `n`/`a` = path  
2. **Tool** — Tool | Last used | Uses  
3. **Chat** — resume a session or **＋ New chat** (`n`)

Updates: on startup RepoHop rate-limits a GitHub check and **auto-downloads** a newer release when found. Disable with `REPOPHOP_NO_UPDATE=1`, or banner-only with `REPOPHOP_UPDATE_CHECK_ONLY=1`. Manual: `rhop update` / `rhop update --apply`.

RepoHop **automatically** finds projects by reading (read-only) local metadata from:

- Codex: `~/.codex/sessions/**/*.jsonl` (`cwd` in `session_meta`)
- Claude Code: `~/.claude/projects/**` (`cwd` in session JSONL)
- OpenCode: `~/.local/share/opencode/opencode.db` (`session.directory`)
- Grok Build CLI: `~/.grok/sessions/<percent-encoded-path>/`

Optional: add extra folders under `project_roots` in `%APPDATA%\RepoHop\config.toml` if you want Git-tree scanning beyond agent history.

## Commands

| Command | Description |
|---------|-------------|
| `rhop` | Interactive hop |
| `rhop .` | Hop with cwd as project |
| `rhop doctor` | Detect agents and environment |
| `rhop scan` | Update project cache from `project_roots` |
| `rhop sessions` | Session browser (Stage 3) |
| `rhop worktree` | Worktree management (Stage 4) |
| `rhop config` | Show config path and roots |
| `rhop version` | Version info |

## Data locations (Windows)

| Kind | Path |
|------|------|
| Config | `%APPDATA%\RepoHop\config.toml` |
| Database | `%LOCALAPPDATA%\RepoHop\repohop.db` |
| Logs | `%LOCALAPPDATA%\RepoHop\logs` |
| Worktrees (future) | `%USERPROFILE%\.repohop\worktrees` |

## Documentation

- [Product requirements](docs/PRD.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Session compatibility](docs/SESSION_COMPATIBILITY.md)
- [Release process](docs/RELEASE.md)
- [中文说明](README.zh-CN.md)

## Non-goals (Stage 1–2)

Desktop/Web UI, embedded terminal, cloud sync, agent API proxy, multi-agent orchestration, chat migration between providers, telemetry, automatic git commit/push/merge.

## License

MIT — see [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md).
