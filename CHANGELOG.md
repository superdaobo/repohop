# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-07-15

### Added

- **Mouse support** in multi-level hop UI (project / tool / session tables):
  - Left-click selects a row
  - Double-click activates (same as Enter)
  - Mouse wheel moves selection up/down

## [0.1.3] - 2026-07-14

### Added

- **Unified multi-level TUI** (single alternate screen): project → tool → chats (no tear-down between levels)
- Agent table columns: **Tool | Last used | Uses** (per-project launch stats)
- Session / chat picker with **＋ New chat** (key `n`) and resume for Codex / Claude / OpenCode / Grok
- `rhop update` / `rhop update --apply` — check and install from GitHub Releases
- Soft auto-update banner on startup (rate-limited; disable with `REPOPHOP_NO_UPDATE`)

### Changed

- Interactive hop no longer opens a second TUI after project pick
- Provider adapters expose list/resume sessions (Stage 3 UI embedded in hop)

## [0.1.2] - 2026-07-14

### Added

- **Grok Build CLI** provider (`grok` / `grok-build` / `grok-cli`) for new-session launch
- Read-only project discovery from `~/.grok/sessions/<percent-encoded-path>/`
- Interactive project **table**: columns **Name | Path | Last used** (aligned)
- Picker shortcuts: `.` = current directory, `n`/`a` = type a new project path

### Changed

- Project sort is **recency-first** (`last_launched_at` desc); favorites no longer jump the list (★ still shown)
- Relative last-used labels: `just now`, `5m ago`, `3h ago`, `2d ago`, `never`

## [0.1.1] - 2026-07-14

### Added

- **Zero-config project discovery** from agent session metadata (read-only):
  - Codex `session_meta.cwd` in `~/.codex/sessions`
  - Claude Code `cwd` in `~/.claude/projects`
  - OpenCode `session.directory` in `opencode.db`
- `rhop` auto-imports projects on first run when the local index is empty
- `rhop scan` / `rhop doctor` report discovery sources without requiring `project_roots`

### Changed

- `project_roots` is optional; empty config is valid

## [0.1.0] - 2026-07-14

### Added

- GitHub Actions **Release** workflow: Windows x64 `rhop.exe`, SHA-256, attach to GitHub Release.
- `install.ps1` / `uninstall.ps1` one-liner install from latest (or pinned) release.

### Added

- Project documentation (PRD, architecture, roadmap, session compatibility).
- Rust CLI scaffold for `rhop` with Stage 2 minimal loop:
  - Provider detection (Codex, Claude Code, OpenCode) and `rhop doctor`
  - `project_roots` config and `rhop scan`
  - SQLite project + launch history
  - Interactive project/agent selection and new-session launch
- GitHub Issues for Stages 3–5 (sessions, worktree, shell integration, release).
