# RepoHop Product Requirements Document

## Positioning

**English:** A local-first workspace router for AI coding CLIs.  
**Slogan:** Hop into the right repo, agent, and session.  
**Chinese:** 面向 AI 编程 CLI 的本地工作区、会话与 Git 任务管理器。

## Problem

Developers juggle multiple repositories and multiple AI coding CLIs (Codex, Claude Code, OpenCode). Switching costs include: remembering paths, choosing the right agent, resuming the right session, and optionally isolating work in a Git worktree. Existing tools either manage multi-session PTYs, focus on one agent, or require Node runtimes.

## Goals

1. One command (`rhop`) to hop into the right project and agent.
2. Reliable detection of installed agent CLIs on Windows first.
3. Explicit, fast project discovery (no whole-disk scan by default).
4. Local launch history and future session index (read-only toward agents).
5. Safe Git worktree workflows (later stage).
6. Single-file native binary without Node/Python runtime.

## Non-goals

- Desktop GUI / Web UI
- Embedded terminal / PTY multiplexing
- Cloud sync, accounts, telemetry
- LLM inference or API proxying
- Multi-agent orchestration
- Migrating chat history across providers
- Auto commit / push / merge
- Replacing agent CLIs

## Primary user flow

1. User runs `rhop`.
2. See recent / configured projects; select one.
3. See installed agents; default to last used for that project.
4. Choose launch mode (Stage 2: **new session only**; later: resume / history / worktree).
5. RepoHop starts the agent with correct cwd and command.
6. Launch is recorded locally.

## Commands

| Command | Requirement |
|---------|-------------|
| `rhop` | Interactive hop |
| `rhop .` | Project = current directory |
| `rhop doctor` | Environment + agent report |
| `rhop scan` | Refresh project cache from `project_roots` |
| `rhop sessions` | Session browser (Stage 3) |
| `rhop worktree` | Worktree ops (Stage 4) |
| `rhop config` | Show config location and roots |
| `rhop version` | Version |

## Project discovery priority

1. RepoHop launch history  
2. Agent session metadata cwd (Stage 3+)  
3. Configured `project_roots`  
4. Git scan under roots  
5. Optional zoxide (later)  
6. Favorites  

Stage 2 implements (1), (3), (4), and favorites flag in schema.

## Sorting factors

- Favorite  
- Last RepoHop launch time  
- Agent last session time (later)  
- Launch count  
- Git last activity (optional / later)  

## Session model (unified, Stage 3+)

`id`, `provider`, `project_path`, `title`, `preview`, `created_at`, `updated_at`, `git_branch`, `source_path`

RepoHop DB stores **index/cache only**, never full chat bodies. Never modify agent session files.

## Worktree (Stage 4)

Create `ai/<slug>` branch and worktree under `%USERPROFILE%\.repohop\worktrees\<project>\<task>`. Safety checks required; no auto destructive git.

## Success metrics (qualitative)

- Time from `rhop` to agent TUI &lt; 10s for known projects  
- Doctor explains missing agents clearly  
- No accidental data loss from git operations  
