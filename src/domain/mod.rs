//! Domain entities for speck.
//!
//! This module contains the core business entities:
//! - Specification: A feature being developed
//! - Worktree: A git worktree instance
//! - Workflow: Workflow command execution
//! - Project: The overall repository context

mod project;
mod spec;
mod workflow;
mod worktree;

pub use project::Project;
pub use spec::{ArtifactType, SpecArtifacts, SpecId, Specification, WorkflowPhase};
pub use workflow::{
    ExecutionState, OutputLine, OutputStream, WorkflowCommand, WorkflowCommandType,
};
pub use worktree::{Worktree, WorktreeStatus, WorktreeSyncStatus};
