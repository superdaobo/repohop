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

## Verified versions

| Tool | Version string |
|------|----------------|
| codex | codex-cli 0.142.5 |
| claude | 2.1.183 (Claude Code) |
| opencode | 1.17.19 |

## Windows PATH / shims

On this machine, npm installs expose:

- `codex.ps1` / `codex.cmd`
- `claude.ps1` / `claude.cmd`
- `opencode.ps1` / `opencode.cmd`

**Assumption:** RepoHop resolves **`.cmd` / `.exe`** for `std::process::Command`. Spawning `.ps1` directly is **unsupported**.

## Storage paths observed

| Provider | Path | Notes |
|----------|------|-------|
| Codex | `%USERPROFILE%\.codex\sessions\` | Also `sqlite/`, `archived_sessions/` |
| Claude | `%USERPROFILE%\.claude\projects\<encoded-path>\` | Encoding uses path-like segments |
| OpenCode | `%USERPROFILE%\.local\share\opencode\opencode.db` | Large DB observed; prefer CLI for list when possible |

## Compatibility rules

1. Prefer official CLI / JSON / stable interfaces.
2. Parse JSONL/SQLite only if official listing is insufficient; **read-only**.
3. Never modify agent files; never convert sessions across providers.
4. All path/format assumptions live in `provider/*` and this document.

## Unknowns / risks

- Codex machine-readable session list API: **UNVERIFIED** (resume has picker).
- Claude session JSONL schema stability: **UNVERIFIED** for Stage 3.
- OpenCode SQLite schema: **UNVERIFIED**; use `session list` first.
- Future flag renames: re-run `--help` and update this file.
