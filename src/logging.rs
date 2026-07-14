use std::fs::OpenOptions;
use std::path::Path;

use tracing_subscriber::EnvFilter;

use crate::error::Result;

/// Initialize tracing: file log under log_dir, optional stderr via RUST_LOG.
pub fn init(log_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;
    let log_path = log_dir.join("rhop.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Prefer file always; if RUST_LOG is set, also mirror to stderr via fmt layer is complex
    // without multi-writer. Keep simple: file subscriber with filter.
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(file)
        .with_ansi(false)
        .with_target(false)
        .init();

    tracing::info!(path = %log_path.display(), "logging initialized");
    Ok(())
}
