//! Project entity representing the overall repository context.

use crate::config::ProjectConfig;
use std::path::PathBuf;

/// Represents the overall git repository context
#[derive(Debug, Clone)]
pub struct Project {
    /// Repository root path
    pub root_path: PathBuf,
    /// Directory containing specs (absolute path)
    pub specs_directory: PathBuf,
    /// Directory for worktrees (absolute path)
    pub worktree_directory: PathBuf,
    /// Main branch name
    pub main_branch: String,
    /// Loaded configuration
    pub config: ProjectConfig,
}

impl Project {
    /// Create a new Project from a root path and configuration
    pub fn new(root_path: PathBuf, config: ProjectConfig) -> Self {
        let specs_directory = root_path.join(&config.git.specs_directory);
        let worktree_directory = root_path.join(&config.worktree.directory);

        Self {
            root_path,
            specs_directory,
            worktree_directory,
            main_branch: config.git.main_branch.clone(),
            config,
        }
    }

    /// Discover the project root by walking up from current directory
    pub fn discover(start_path: Option<PathBuf>) -> Option<PathBuf> {
        let start = start_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let mut current = start.as_path();
        loop {
            // Check for .git directory or file (worktree)
            let git_path = current.join(".git");
            if git_path.exists() {
                return Some(current.to_path_buf());
            }

            // Move up to parent
            match current.parent() {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }

    /// Check if the specs directory exists
    pub fn has_specs_directory(&self) -> bool {
        self.specs_directory.exists() && self.specs_directory.is_dir()
    }

    /// Check if the worktree directory exists
    pub fn has_worktree_directory(&self) -> bool {
        self.worktree_directory.exists() && self.worktree_directory.is_dir()
    }

    /// Ensure the worktree directory exists
    pub fn ensure_worktree_directory(&self) -> std::io::Result<()> {
        if !self.worktree_directory.exists() {
            std::fs::create_dir_all(&self.worktree_directory)?;
        }
        Ok(())
    }

    /// Get the path for a new worktree for a given branch
    pub fn worktree_path_for_branch(&self, branch: &str) -> PathBuf {
        self.worktree_directory.join(branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let config = ProjectConfig::default();
        let project = Project::new(PathBuf::from("/tmp/test-project"), config);

        assert_eq!(project.root_path, PathBuf::from("/tmp/test-project"));
        assert_eq!(
            project.specs_directory,
            PathBuf::from("/tmp/test-project/specs")
        );
        assert_eq!(
            project.worktree_directory,
            PathBuf::from("/tmp/test-project/.worktrees")
        );
        assert_eq!(project.main_branch, "main");
    }

    #[test]
    fn test_worktree_path_for_branch() {
        let config = ProjectConfig::default();
        let project = Project::new(PathBuf::from("/tmp/test-project"), config);

        assert_eq!(
            project.worktree_path_for_branch("001-feature"),
            PathBuf::from("/tmp/test-project/.worktrees/001-feature")
        );
    }
}
