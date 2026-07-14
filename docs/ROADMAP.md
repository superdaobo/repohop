# Roadmap

## Phase 0 — Repository review & research

- [x] Inspect local/remote repo state  
- [x] Survey agent CLIs and reference projects/licenses  
- [x] Gap analysis  

## Phase 1 — Architecture & foundation

- [x] Product docs (PRD, ARCHITECTURE, ROADMAP, …)  
- [ ] Rust crate + module boundaries  
- [ ] fmt / clippy / test / CI  
- [ ] Errors + logging  

## Phase 2 — Minimal runnable loop

- [ ] `rhop doctor` + provider detection  
- [ ] `project_roots` + `rhop scan`  
- [ ] Project selection (Ratatui)  
- [ ] Agent selection  
- [ ] Launch **new** session  
- [ ] Persist launch history  

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
