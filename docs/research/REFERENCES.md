# Research Notes — Reference Projects

Research date: 2026-07-14. **No source code was copied** into RepoHop from these projects.

## Agent Session Hub (vinzify/Agent-Session-Hub)

- **License:** MIT (GitHub API)
- **Ideas:** unified session picker across Codex/Claude/OpenCode; provider-aware storage; shell integration; fzf UX
- **RepoHop stance:** adopt provider-boundary idea; implement native Ratatui instead of fzf dependency; project-first hop rather than session-first only

## CCManager (kbwo/ccmanager)

- **License:** MIT
- **Ideas:** multi-project, worktree management, multi-agent command presets, session status monitoring (PTY-oriented)
- **RepoHop stance:** no PTY/session keepalive; no copying Claude session dirs between worktrees by default; safety-first worktrees later

## agentree (AryaLabsHQ/agentree)

- **License:** MIT
- **Ideas:** isolated worktrees for parallel agents
- **RepoHop stance:** similar product problem; independent implementation

## WorktreePilot

- **License:** not reported by GitHub API (`null`) — **treat as non-reusable code**
- **Ideas:** worktree + agent workflow product framing only

## zoxide

- **License:** MIT
- **Ideas:** frecent directory ranking as optional discovery source (future)

## fzf

- **License:** MIT
- **Ideas:** keyboard-driven fuzzy lists; RepoHop uses Ratatui for single-binary UX

## License conclusion for RepoHop

RepoHop can ship as **MIT** given independent implementation and MIT-compatible inspiration sources. Re-verify before vendoring any third-party code. WorktreePilot must not be copied without a clear license.
