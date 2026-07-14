pub mod model;
pub mod rank;
pub mod scan;
pub mod service;

pub use model::Project;
pub use service::{ensure_cwd_project, list_ranked_projects, scan_and_upsert};
