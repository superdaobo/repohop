# Roadmap

## Phase 0 — Repository review & research

- [x] Inspect local/remote repo state  
- [x] Survey agent CLIs and reference projects/licenses  
- [x] Gap analysis  

## Phase 1 — Architecture & foundation

- [x] Product docs (PRD, ARCHITECTURE, ROADMAP, …)  
- [x] Rust crate + module boundaries  
- [x] fmt / clippy / test / CI  
- [x] Errors + logging  

## Phase 2 — Minimal runnable loop

- [x] `rhop doctor` + provider detection  
- [x] `project_roots` + `rhop scan`  
- [x] Project selection (Ratatui)  
- [x] Agent selection  
- [x] Launch **new** session  
- [x] Persist launch history  

## Phase 3 — Session resume

- [ ] Unified session model  
- [ ] Codex / Claude / OpenCode adapters  
- [ ] Continue last + history picker  

## Phase 4 — Worktree

- [ ] Create / list / safety checks  
- [ ] Launch agent in worktree  
- [ ] Safe cleanup  

## Phase 5 — Release

- [ ] Windows x64 release artifacts + SHA-256  
- [ ] `install.ps1` / `uninstall.ps1`  
- [ ] GitHub Release  
- [ ] Install verification  

## Explicit later / out of scope

Linux/macOS/ARM packaging, zoxide source, fzf dependency, GUI, cloud, multi-agent, telemetry.
