# Security Policy

## Supported versions

RepoHop is early-stage. Security fixes target the latest `main` branch.

## What RepoHop does and does not do

- **Does:** read local config, scan configured roots, detect agent binaries, spawn agent processes with inherited stdio, store local SQLite indexes.
- **Does not:** send telemetry, upload chat logs, proxy model APIs, or modify agent session stores.

## Reporting a vulnerability

Please open a **private** security advisory on GitHub if available, or contact the repository owner via GitHub. Do not file public issues for exploitable secrets-handling bugs.

Include:

- RepoHop version / commit
- OS and shell
- Reproduction steps
- Impact assessment

## Hardening expectations

- Treat agent process launch as user-trusted (same trust as running `codex` yourself).
- Do not store API keys in RepoHop config; agents manage their own auth.
- Logs must not intentionally record secret tokens.
