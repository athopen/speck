//! Infrastructure services for speck.
//!
//! This module contains:
//! - GitService: Git and worktree operations
//! - SpecService: Specification discovery and management
//! - McpService: MCP client for AI agent communication
//! - ProcessService: Process execution and streaming

mod git;
pub mod mcp;
pub mod process;
mod spec;

pub use git::GitService;
pub use mcp::McpClient;
pub use process::{ProcessHandle, ProcessOutput, ProcessService, WorkflowRunner};
pub use spec::SpecService;
