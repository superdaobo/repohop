# Session & Agent CLI Compatibility

**Verified on:** 2026-07-14 (developer machine)  
**Hosts:** Windows 11, x86_64  

When behavior is unknown, it is marked **UNVERIFIED**. Do not invent flags.

## Provider matrix (Stage 2 vs later)

| Provider | Binary names | New session (Stage 2) | Resume (Stage 3) | List sessions |
|----------|--------------|------------------------|------------------|---------------|
| Codex CLI | `codex` | `codex` with `cwd=project` | `codex resume [SESSION_ID]` or `--last` | Prefer official resume picker / future CLI; local `~/.codex/sessions` **read-only if needed** |
| Claude Code | `claude` | `claude` with `cwd=project` | `-c` / `--continue`; `-r` / `--resume [id]` | Project dirs under `~/.claude/projects/` |
| OpenCode | `opencode` | `opencode` with `cwd=project` | `-c` / `--continue`; `-s` / `--session` | `opencode session list` (verified text table) |
| Grok Build CLI | `grok` | `grok` with `cwd=project` (also supports `--cwd`) | `-c` / `--continue`; `-r` / `--resume [id]` | Project dirs under `~/.grok/sessions/` (percent-encoded path names) |

## Verified versions

| Tool | Version string |
|------|----------------|
| codex | codex-cli 0.142.5 |
| claude | 2.1.183 (Claude Code) |
| opencode | 1.17.19 |
| grok | grok 0.2.101 |

## Windows PATH / shims

On this machine, npm installs expose:

- `codex.ps1` / `codex.cmd`
- `claude.ps1` / `claude.cmd`
- `opencode.ps1` / `opencode.cmd`
- `grok.exe` under `%USERPROFILE%\.grok\bin\` (often on user PATH)

**Assumption:** RepoHop resolves **`.cmd` / `.exe`** for `std::process::Command`. Spawning `.ps1` directly is **unsupported**.

## Storage paths observed

| Provider | Path | Notes |
|----------|------|-------|
| Codex | `%USERPROFILE%\.codex\sessions\` | Also `sqlite/`, `archived_sessions/` |
| Claude | `%USERPROFILE%\.claude\projects\<encoded-path>\` | Encoding uses path-like segments |
| OpenCode | `%USERPROFILE%\.local\share\opencode\opencode.db` | Large DB observed; prefer CLI for list when possible |
| Grok | `%USERPROFILE%\.grok\sessions\<percent-encoded-path>\` | Dir name is URL-encoded absolute path (e.g. `D%3A%5C...`) |

## Project auto-discovery (v0.1.1+; Grok in v0.1.2+)

RepoHop indexes **project paths only** (not full chat bodies):

| Provider | Source | Field | Notes |
|----------|--------|-------|-------|
| Codex | `~/.codex/sessions/**/*.jsonl` | `session_meta.payload.cwd` | First ~8 lines; cap ~400 newest files |
| Claude | `~/.claude/projects/*/**.jsonl` | `cwd` | Prefer JSONL over encoded folder names |
| OpenCode | `~/.local/share/opencode/opencode.db` | `session.directory` | Read-only SQLite; group by directory |
| Grok | `~/.grok/sessions/*/` | directory name | Percent-decode folder name → project path; mtime for activity |

`project_roots` remains optional for extra Git scans.

## Compatibility rules

1. Prefer official CLI / JSON / stable interfaces.
2. Parse JSONL/SQLite only if official listing is insufficient; **read-only**.
3. Never modify agent files; never convert sessions across providers.
4. All path/format assumptions live in `provider/*`, `discover/*`, and this document.

## Unknowns / risks

- Codex machine-readable session list API: **UNVERIFIED** (resume has picker).
- Claude session JSONL schema stability: **UNVERIFIED** for Stage 3.
- OpenCode SQLite schema: **UNVERIFIED**; use `session list` first.
- Grok nested session UUID layout / stable list API: **UNVERIFIED** for Stage 3 resume UI.
- Future flag renames: re-run `--help` and update this file.
