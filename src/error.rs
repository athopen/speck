//! Unified error types for the spec-tui application.

use std::path::PathBuf;
use thiserror::Error;

/// Main application error type
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("Specification error: {0}")]
    Spec(#[from] SpecError),

    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("IO error: {0}")]
    IoString(String),

    #[error("Task cancelled")]
    Cancelled,
}

impl AppError {
    /// Create an IO error from a string
    pub fn io(msg: impl Into<String>) -> Self {
        Self::IoString(msg.into())
    }
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Failed to parse configuration: {0}")]
    Parse(String),

    #[error("IO error reading config: {0}")]
    Io(#[from] std::io::Error),
}

/// Git/worktree operation errors
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Not a git repository")]
    NotARepository,

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Worktree already exists for branch: {0}")]
    WorktreeExists(String),

    #[error("Worktree not found: {0}")]
    WorktreeNotFound(PathBuf),

    #[error("Cannot delete main worktree")]
    CannotDeleteMain,

    #[error("Worktree has uncommitted changes")]
    WorktreeDirty,

    #[error("Path already exists: {0}")]
    PathExists(PathBuf),

    #[error("No remote configured")]
    NoRemote,

    #[error("Git operation failed: {0}")]
    Operation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Specification-related errors
#[derive(Debug, Error)]
pub enum SpecError {
    #[error("Specification not found: {0}")]
    NotFound(String),

    #[error("Specification already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid specification ID: {0}")]
    InvalidId(String),

    #[error("Invalid specification name: {0}")]
    InvalidName(String),

    #[error("Specs directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// MCP client errors
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Already connected")]
    AlreadyConnected,

    #[error("Not connected")]
    NotConnected,

    #[error("Not initialized")]
    NotInitialized,

    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("RPC error (code {code}): {message}")]
    RpcError { code: i32, message: String },

    #[error("Request timeout")]
    Timeout,

    #[error("Request cancelled")]
    Cancelled,

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;

/// Result type alias for Git operations
pub type GitResult<T> = std::result::Result<T, GitError>;

/// Result type alias for Spec operations
pub type SpecResult<T> = std::result::Result<T, SpecError>;

/// Result type alias for MCP operations
pub type McpResult<T> = std::result::Result<T, McpError>;
