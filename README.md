# RepoHop

**Hop into the right repo, agent, and session.**

RepoHop (`rhop`) is a **local-first workspace router** for AI coding CLIs. It helps you pick a recent project, choose Codex / Claude Code / OpenCode, and start a session in the right directory—optionally in an isolated Git worktree (roadmap).

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
- At least one of: [Codex CLI](https://github.com/openai/codex), [Claude Code](https://code.claude.com), [OpenCode](https://opencode.ai)

## Install (development)

```powershell
git clone https://github.com/superdaobo/repohop.git
cd repohop
cargo build --release
# binary: target\release\rhop.exe
```

## Quick start

1. Edit config (created on first run):

   `%APPDATA%\RepoHop\config.toml`

   ```toml
   project_roots = ["D:\\Documents\\Projects"]
   ```

2. Scan and check agents:

   ```powershell
   rhop doctor
   rhop scan
   ```

3. Hop:

   ```powershell
   rhop        # pick project + agent
   rhop .      # use current directory as project
   ```

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
