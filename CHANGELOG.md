# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- GitHub Actions **Release** workflow: Windows x64 `rhop.exe`, SHA-256, attach to GitHub Release.
- `install.ps1` / `uninstall.ps1` one-liner install from latest (or pinned) release.

## [0.1.0] - 2026-07-14

### Added

- Project documentation (PRD, architecture, roadmap, session compatibility).
- Rust CLI scaffold for `rhop` with Stage 2 minimal loop:
  - Provider detection (Codex, Claude Code, OpenCode) and `rhop doctor`
  - `project_roots` config and `rhop scan`
  - SQLite project + launch history
  - Interactive project/agent selection and new-session launch
- GitHub Issues for Stages 3–5 (sessions, worktree, shell integration, release).
