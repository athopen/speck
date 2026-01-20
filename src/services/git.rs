//! Git service for repository and worktree operations.
//!
//! Uses gitoxide (gix) for native Rust git operations.
//! All operations are blocking and should be wrapped with spawn_blocking.

use crate::domain::{Worktree, WorktreeStatus, WorktreeSyncStatus};
use crate::error::{GitError, GitResult};
use std::path::{Path, PathBuf};

/// Git service for worktree management
pub struct GitService {
    repo_path: PathBuf,
    _worktree_base: PathBuf,
}

impl GitService {
    /// Create a new GitService for a repository
    pub fn new(repo_path: PathBuf, worktree_base: PathBuf) -> GitResult<Self> {
        // Verify it's a git repository
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return Err(GitError::NotARepository);
        }

        Ok(Self {
            repo_path,
            _worktree_base: worktree_base,
        })
    }

    /// Get the repository path
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// List all worktrees in the repository
    pub fn list_worktrees(&self) -> GitResult<Vec<Worktree>> {
        // Use git command to list worktrees (more reliable than gix for this)
        let output = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to list worktrees: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::Operation(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();
        let mut current_path: Option<PathBuf> = None;
        let mut current_branch: Option<String> = None;
        let mut is_bare = false;

        for line in output_str.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                // Save previous worktree if any
                if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                    let is_main = worktrees.is_empty(); // First worktree is main
                    worktrees.push(Worktree::new(path, branch, is_main));
                }
                current_path = Some(PathBuf::from(path));
                is_bare = false;
            } else if let Some(branch_ref) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(branch_ref.to_string());
            } else if line == "bare" {
                is_bare = true;
            } else if line == "detached" {
                current_branch = Some("(detached)".to_string());
            }
        }

        // Don't forget the last worktree
        if let (Some(path), Some(branch)) = (current_path, current_branch) {
            if !is_bare {
                let is_main = worktrees.is_empty();
                worktrees.push(Worktree::new(path, branch, is_main));
            }
        }

        Ok(worktrees)
    }

    /// Create a new worktree for a branch
    pub fn create_worktree(&self, branch: &str, path: &Path) -> GitResult<Worktree> {
        // Check if branch exists
        if !self.branch_exists(branch)? {
            return Err(GitError::BranchNotFound(branch.to_string()));
        }

        // Check if worktree already exists for this branch
        let existing = self.list_worktrees()?;
        if existing.iter().any(|w| w.branch == branch) {
            return Err(GitError::WorktreeExists(branch.to_string()));
        }

        // Check if path already exists
        if path.exists() {
            return Err(GitError::PathExists(path.to_path_buf()));
        }

        // Create the worktree
        let output = std::process::Command::new("git")
            .args(["worktree", "add", path.to_str().unwrap(), branch])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to create worktree: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::Operation(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(Worktree::new(path.to_path_buf(), branch.to_string(), false))
    }

    /// Delete a worktree
    pub fn delete_worktree(&self, path: &Path, force: bool) -> GitResult<()> {
        let worktrees = self.list_worktrees()?;

        // Find the worktree
        let wt = worktrees
            .iter()
            .find(|w| w.path == path)
            .ok_or_else(|| GitError::WorktreeNotFound(path.to_path_buf()))?;

        // Can't delete main worktree
        if wt.is_main {
            return Err(GitError::CannotDeleteMain);
        }

        // Check if dirty (unless force)
        if !force {
            let status = self.worktree_status(path)?;
            if status.is_dirty() {
                return Err(GitError::WorktreeDirty);
            }
        }

        // Remove the worktree
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(path.to_str().unwrap());

        let output = std::process::Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to remove worktree: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::Operation(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    /// Get the status of a worktree
    pub fn worktree_status(&self, path: &Path) -> GitResult<WorktreeStatus> {
        if !path.exists() {
            return Err(GitError::WorktreeNotFound(path.to_path_buf()));
        }

        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to get status: {}", e)))?;

        if !output.status.success() {
            return Ok(WorktreeStatus::Unknown);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.is_empty() {
            return Ok(WorktreeStatus::Clean);
        }

        let mut modified = 0u32;
        let mut staged = 0u32;
        let mut untracked = 0u32;

        for line in output_str.lines() {
            if line.len() >= 2 {
                let index = line.chars().next().unwrap_or(' ');
                let worktree = line.chars().nth(1).unwrap_or(' ');

                match (index, worktree) {
                    ('?', '?') => untracked += 1,
                    (i, w) if i != ' ' && w != ' ' => {
                        staged += 1;
                        modified += 1;
                    }
                    (i, _) if i != ' ' && i != '?' => staged += 1,
                    (_, w) if w != ' ' && w != '?' => modified += 1,
                    _ => {}
                }
            }
        }

        Ok(WorktreeStatus::Dirty {
            modified,
            staged,
            untracked,
        })
    }

    /// Get sync status with remote
    pub fn sync_status(&self, branch: &str) -> GitResult<WorktreeSyncStatus> {
        // Check if remote tracking branch exists
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--verify", &format!("origin/{}", branch)])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to check remote: {}", e)))?;

        if !output.status.success() {
            return Ok(WorktreeSyncStatus::new(0, 0, false));
        }

        // Get ahead/behind counts
        let output = std::process::Command::new("git")
            .args([
                "rev-list",
                "--left-right",
                "--count",
                &format!("{}...origin/{}", branch, branch),
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to get sync status: {}", e)))?;

        if !output.status.success() {
            return Ok(WorktreeSyncStatus::new(0, 0, true));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.trim().split('\t').collect();

        let ahead = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let behind = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        Ok(WorktreeSyncStatus::new(ahead, behind, true))
    }

    /// Check if a branch exists (local or remote)
    pub fn branch_exists(&self, branch: &str) -> GitResult<bool> {
        // Check local branch
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to check branch: {}", e)))?;

        if output.status.success() {
            return Ok(true);
        }

        // Check remote branch
        let output = std::process::Command::new("git")
            .args([
                "rev-parse",
                "--verify",
                &format!("refs/remotes/origin/{}", branch),
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to check remote branch: {}", e)))?;

        Ok(output.status.success())
    }

    /// Get the current branch of a worktree
    pub fn current_branch(&self, path: &Path) -> GitResult<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to get current branch: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::Operation(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the main worktree path
    pub fn main_worktree(&self) -> GitResult<PathBuf> {
        let worktrees = self.list_worktrees()?;
        worktrees
            .into_iter()
            .find(|w| w.is_main)
            .map(|w| w.path)
            .ok_or(GitError::NotARepository)
    }

    /// Create a new branch
    pub fn create_branch(&self, branch: &str, start_point: Option<&str>) -> GitResult<()> {
        let mut args = vec!["branch", branch];
        if let Some(start) = start_point {
            args.push(start);
        }

        let output = std::process::Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Operation(format!("Failed to create branch: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::Operation(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let path = temp.path().to_path_buf();

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&path)
            .output()
            .unwrap();

        // Create initial commit
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&path)
            .output()
            .unwrap();

        std::fs::write(path.join("README.md"), "# Test").unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&path)
            .output()
            .unwrap();

        (temp, path)
    }

    #[test]
    fn test_list_worktrees() {
        let (_temp, path) = create_test_repo();
        let git = GitService::new(path.clone(), path.join(".worktrees")).unwrap();

        let worktrees = git.list_worktrees().unwrap();
        assert_eq!(worktrees.len(), 1);
        assert!(worktrees[0].is_main);
    }

    #[test]
    fn test_branch_exists() {
        let (_temp, path) = create_test_repo();
        let git = GitService::new(path.clone(), path.join(".worktrees")).unwrap();

        // Main branch should exist (could be 'main' or 'master' depending on git config)
        let current = git.current_branch(&path).unwrap();
        assert!(git.branch_exists(&current).unwrap());

        // Non-existent branch
        assert!(!git.branch_exists("nonexistent-branch").unwrap());
    }
}
