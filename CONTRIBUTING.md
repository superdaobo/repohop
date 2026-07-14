# Contributing to RepoHop

Thank you for contributing.

## Development setup

- Rust stable (MSVC on Windows)
- `cargo fmt`, `clippy`, `test` before every PR

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

## Commit style

[Conventional Commits](https://www.conventionalcommits.org/):

- `chore:`, `docs:`, `feat:`, `fix:`, `test:`, `refactor:`, `ci:`

Prefer small commits scoped to one issue.

## Architecture rules

1. **Provider Adapter only** — no scattered `if provider == "codex"` in business code.
2. **Never modify** agent session files; index only.
3. **No destructive git** (`reset --hard`, `clean -fd`, auto-stash, force delete) without explicit user confirmation flows (and Stage 2 has no delete yet).
4. Paths are `PathBuf` / `std::path` — never string-join Windows paths.
5. New heavy dependencies need a note in `docs/ARCHITECTURE.md`.

## Issues

Work from GitHub Issues on [superdaobo/repohop](https://github.com/superdaobo/repohop). One issue ≈ one PR when possible.

## License

By contributing you agree that your contributions are licensed under the MIT License.
