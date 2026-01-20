//! Infrastructure services for spec-tui.
//!
//! This module contains:
//! - GitService: Git and worktree operations
//! - SpecService: Specification discovery and management
//! - McpService: MCP client for AI agent communication
//! - ProcessService: Process execution and streaming

mod git;
mod spec;
pub mod mcp;
pub mod process;

pub use git::GitService;
pub use spec::SpecService;
pub use mcp::McpClient;
pub use process::{ProcessService, ProcessHandle, ProcessOutput, WorkflowRunner};
