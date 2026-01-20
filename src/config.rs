//! Configuration management for spec-tui.
//!
//! Supports layered configuration: defaults → project → user → env

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub worktree: WorktreeConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub git: GitConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            worktree: WorktreeConfig::default(),
            mcp: McpConfig::default(),
            ui: UiConfig::default(),
            git: GitConfig::default(),
        }
    }
}

impl ProjectConfig {
    /// Load configuration with hierarchy: defaults → project → user → env
    pub fn load(project_root: Option<&PathBuf>) -> Result<Self, ConfigError> {
        use config::{Config, Environment, File};

        let mut builder = Config::builder();

        // 1. Start with defaults
        builder = builder.add_source(
            config::File::from_str(
                include_str!("../default_config.toml"),
                config::FileFormat::Toml,
            )
            .required(false),
        );

        // 2. Project-specific config (.spec-tui.toml in project root)
        if let Some(root) = project_root {
            let project_config = root.join(".spec-tui.toml");
            if project_config.exists() {
                builder = builder.add_source(File::from(project_config).required(false));
            }
        }

        // 3. User config (~/.config/spec-tui/config.toml)
        if let Some(config_dir) = directories::ProjectDirs::from("com", "spec-tui", "spec-tui") {
            let user_config = config_dir.config_dir().join("config.toml");
            if user_config.exists() {
                builder = builder.add_source(File::from(user_config).required(false));
            }
        }

        // 4. Environment variables (SPEC_TUI_*)
        builder = builder.add_source(
            Environment::with_prefix("SPEC_TUI")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Load configuration with default settings only
    pub fn load_defaults() -> Self {
        Self::default()
    }
}

/// Worktree-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// Directory where worktrees are created (relative to project root)
    #[serde(default = "default_worktree_directory")]
    pub directory: PathBuf,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            directory: default_worktree_directory(),
        }
    }
}

fn default_worktree_directory() -> PathBuf {
    PathBuf::from(".worktrees")
}

/// MCP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Transport type for MCP communication
    #[serde(default)]
    pub transport: McpTransport,
    /// Request timeout in seconds
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            transport: McpTransport::default(),
            timeout_seconds: default_timeout_seconds(),
        }
    }
}

fn default_timeout_seconds() -> u64 {
    60
}

/// MCP transport type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransport {
    /// Standard I/O transport (spawn process)
    #[default]
    Stdio,
    /// HTTP transport with SSE for responses
    Http { endpoint: String },
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// UI refresh rate in milliseconds
    #[serde(default = "default_refresh_rate_ms")]
    pub refresh_rate_ms: u64,
    /// Enable vim-style navigation (j/k/h/l)
    #[serde(default = "default_vim_navigation")]
    pub vim_navigation: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: default_refresh_rate_ms(),
            vim_navigation: default_vim_navigation(),
        }
    }
}

fn default_refresh_rate_ms() -> u64 {
    100
}

fn default_vim_navigation() -> bool {
    true
}

/// Git-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Directory containing specs (relative to project root)
    #[serde(default = "default_specs_directory")]
    pub specs_directory: String,
    /// Main branch name
    #[serde(default = "default_main_branch")]
    pub main_branch: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            specs_directory: default_specs_directory(),
            main_branch: default_main_branch(),
        }
    }
}

fn default_specs_directory() -> String {
    "specs".to_string()
}

fn default_main_branch() -> String {
    "main".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert_eq!(config.worktree.directory, PathBuf::from(".worktrees"));
        assert_eq!(config.mcp.timeout_seconds, 60);
        assert_eq!(config.ui.refresh_rate_ms, 100);
        assert!(config.ui.vim_navigation);
        assert_eq!(config.git.specs_directory, "specs");
        assert_eq!(config.git.main_branch, "main");
    }
}
