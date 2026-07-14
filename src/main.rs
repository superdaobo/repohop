use std::process::ExitCode;

use clap::Parser;

use repohop::cli::{Cli, Commands};
use repohop::config::AppConfig;
use repohop::db::Database;
use repohop::doctor::{print_report, run_doctor};
use repohop::error::RepoHopError;
use repohop::logging;
use repohop::paths::AppPaths;
use repohop::project::scan_and_upsert;
use repohop::ui::{run_interactive, HopOptions};

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(RepoHopError::Cancelled) => {
            eprintln!("Cancelled.");
            ExitCode::SUCCESS
        }
        Err(RepoHopError::NoAgents) => ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn real_main() -> repohop::Result<()> {
    let cli = Cli::parse();
    let paths = AppPaths::resolve()?;
    paths.ensure_dirs()?;
    let _ = logging::init(&paths.log_dir);
    let config = AppConfig::load_or_init(&paths)?;
    let db = Database::open(&paths.database_file)?;

    match cli.command {
        Some(Commands::Version) => {
            println!("rhop {}", env!("CARGO_PKG_VERSION"));
            println!("RepoHop — local-first workspace router for AI coding CLIs");
            Ok(())
        }
        Some(Commands::Doctor) => {
            let report = run_doctor(&paths, &config)?;
            print_report(&report)
        }
        Some(Commands::Scan) => {
            let report = scan_and_upsert(&db, &config)?;
            println!("Scan complete.");
            println!(
                "  from agent sessions: {} ({})",
                report.from_agents,
                if report.agent_sources.is_empty() {
                    "none".into()
                } else {
                    report.agent_sources.join(", ")
                }
            );
            println!("  from project_roots:  {}", report.from_roots);
            println!("  upserted total:     {}", report.total_upserted);
            println!("Database: {}", paths.database_file.display());
            if report.total_upserted == 0 {
                println!("Tip: open any project in Codex / Claude / OpenCode once, then re-run `rhop scan`.");
            } else {
                println!("Run `rhop` to pick a project and start an agent.");
            }
            Ok(())
        }
        Some(Commands::Config) => {
            println!("Config file: {}", paths.config_file.display());
            println!("Database:    {}", paths.database_file.display());
            println!("Log dir:     {}", paths.log_dir.display());
            println!("Worktrees:   {}", paths.worktree_root.display());
            println!();
            println!("Auto-discovery: ON (Codex ~/.codex/sessions, Claude ~/.claude/projects, OpenCode opencode.db)");
            println!("project_roots ({}):", config.project_roots.len());
            if config.project_roots.is_empty() {
                println!("  (empty — optional; agent history is enough for zero-config)");
            } else {
                for r in &config.project_roots {
                    println!("  - {}", r.display());
                }
            }
            println!("scan.max_depth = {}", config.max_depth());
            Ok(())
        }
        Some(Commands::Sessions) => {
            eprintln!(
                "rhop sessions is not implemented yet (Stage 3).\nSee GitHub issues for Codex/Claude/OpenCode session adapters."
            );
            Err(RepoHopError::NotImplemented("rhop sessions"))
        }
        Some(Commands::Worktree) => {
            eprintln!(
                "rhop worktree is not implemented yet (Stage 4).\nSee the Safe Git Worktree management issue."
            );
            Err(RepoHopError::NotImplemented("rhop worktree"))
        }
        None => {
            let project = cli.project.map(|p| {
                if p.as_os_str() == "." {
                    std::env::current_dir().unwrap_or(p)
                } else {
                    p
                }
            });
            run_interactive(&db, &config, HopOptions { project })
        }
    }
}
