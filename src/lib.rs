//! RepoHop library — workspace router for AI coding CLIs.

pub mod cli;
pub mod config;
pub mod db;
pub mod discover;
pub mod doctor;
pub mod error;
pub mod git;
pub mod launch;
pub mod logging;
pub mod paths;
pub mod project;
pub mod provider;
pub mod ui;
pub mod update;

pub use error::{RepoHopError, Result};
