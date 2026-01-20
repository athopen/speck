//! spec-tui: Terminal UI for spec-driven development workflow
//!
//! This crate provides a terminal-based user interface for managing feature
//! specifications with git worktree integration for parallel development.

pub mod app;
pub mod config;
pub mod domain;
pub mod error;
pub mod services;
pub mod ui;

pub use app::App;
pub use config::ProjectConfig;
pub use error::{AppError, Result};
