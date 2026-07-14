# Third-Party Notices

RepoHop is MIT-licensed. This file records **inspiration sources** and **Rust crate** dependencies.

## Inspiration (no source code copied)

| Project | URL | License (as of research 2026-07-14) | How used |
|---------|-----|-------------------------------------|----------|
| Agent Session Hub | https://github.com/vinzify/Agent-Session-Hub | MIT | Architecture ideas: multi-provider session discovery, shell integration patterns |
| CCManager | https://github.com/kbwo/ccmanager | MIT | Multi-project / worktree UX ideas |
| agentree | https://github.com/AryaLabsHQ/agentree | MIT | Worktree isolation product ideas |
| WorktreePilot | https://github.com/WorktreePilot/worktree-pilot | **Unclear / not listed on GitHub API** | Product ideas only; **no code reuse** |
| zoxide | https://github.com/ajeetdsouza/zoxide | MIT | Directory ranking concepts (future data source) |
| fzf | https://github.com/junegunn/fzf | MIT | Fuzzy picker interaction ideas (RepoHop uses Ratatui, not fzf) |

If any algorithm or substantial snippet is ever taken from an upstream project, it must:

1. Have a verified compatible license
2. Be recorded here with original path
3. Preserve required copyright notices

## Rust dependencies

See `Cargo.toml` / `Cargo.lock`. Major direct dependencies:

| Crate | Purpose |
|-------|---------|
| clap | CLI parsing |
| ratatui / crossterm | TUI |
| serde / toml / serde_json | Config and data |
| rusqlite (bundled) | Local index DB |
| anyhow / thiserror | Errors |
| tracing / tracing-subscriber | Logging |
| chrono | Timestamps |
| uuid | IDs |
| directories | Path helpers |

License texts for crates ship with crates.io packages; regenerate a full NOTICE with `cargo about` or similar in a future release task.
