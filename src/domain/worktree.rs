//! Worktree entity and related types.

use super::SpecId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a git worktree instance
#[derive(Debug, Clone)]
pub struct Worktree {
    /// Full path to worktree directory
    pub path: PathBuf,
    /// Associated branch name
    pub branch: String,
    /// Working tree status
    pub status: WorktreeStatus,
    /// Associated spec ID (if matches pattern)
    pub spec_id: Option<SpecId>,
    /// Is this the main worktree?
    pub is_main: bool,
}

impl Worktree {
    /// Create a new worktree instance
    pub fn new(path: PathBuf, branch: String, is_main: bool) -> Self {
        // Try to derive spec_id from branch name
        let spec_id = SpecId::parse(&branch).ok();

        Self {
            path,
            branch,
            status: WorktreeStatus::Unknown,
            spec_id,
            is_main,
        }
    }

    /// Check if this worktree is associated with a spec
    pub fn has_spec(&self) -> bool {
        self.spec_id.is_some()
    }

    /// Get display name for the worktree
    pub fn display_name(&self) -> &str {
        &self.branch
    }
}

/// Working tree status
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorktreeStatus {
    /// No uncommitted changes
    Clean,
    /// Has uncommitted changes
    Dirty {
        modified: u32,
        staged: u32,
        untracked: u32,
    },
    /// HEAD is detached
    Detached,
    /// Status cannot be determined
    #[default]
    Unknown,
}

impl WorktreeStatus {
    /// Check if the worktree is clean
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean)
    }

    /// Check if the worktree is dirty
    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty { .. })
    }

    /// Get status indicator for UI
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Clean => "",
            Self::Dirty { .. } => "*",
            Self::Detached => "!",
            Self::Unknown => "?",
        }
    }

    /// Get status description
    pub fn description(&self) -> String {
        match self {
            Self::Clean => "Clean".to_string(),
            Self::Dirty {
                modified,
                staged,
                untracked,
            } => {
                let mut parts = Vec::new();
                if *modified > 0 {
                    parts.push(format!("{}M", modified));
                }
                if *staged > 0 {
                    parts.push(format!("{}S", staged));
                }
                if *untracked > 0 {
                    parts.push(format!("{}?", untracked));
                }
                parts.join(" ")
            }
            Self::Detached => "Detached HEAD".to_string(),
            Self::Unknown => "Unknown".to_string(),
        }
    }
}

impl std::fmt::Display for WorktreeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Sync status with remote tracking branch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeSyncStatus {
    /// Commits ahead of remote
    pub ahead: u32,
    /// Commits behind remote
    pub behind: u32,
    /// Remote branch exists
    pub remote_exists: bool,
}

impl WorktreeSyncStatus {
    /// Create a new sync status
    pub fn new(ahead: u32, behind: u32, remote_exists: bool) -> Self {
        Self {
            ahead,
            behind,
            remote_exists,
        }
    }

    /// Check if in sync with remote
    pub fn is_synced(&self) -> bool {
        self.remote_exists && self.ahead == 0 && self.behind == 0
    }

    /// Get sync indicator for UI
    pub fn indicator(&self) -> String {
        if !self.remote_exists {
            return "⊘".to_string(); // No remote
        }
        match (self.ahead, self.behind) {
            (0, 0) => "✓".to_string(),
            (a, 0) => format!("↑{}", a),
            (0, b) => format!("↓{}", b),
            (a, b) => format!("↑{}↓{}", a, b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_new() {
        let wt = Worktree::new(PathBuf::from("/tmp/wt"), "001-feature".to_string(), false);
        assert!(!wt.is_main);
        assert!(wt.has_spec());
        assert_eq!(wt.spec_id.unwrap().number(), 1);
    }

    #[test]
    fn test_worktree_status_indicator() {
        assert_eq!(WorktreeStatus::Clean.indicator(), "");
        assert_eq!(
            WorktreeStatus::Dirty {
                modified: 1,
                staged: 0,
                untracked: 0
            }
            .indicator(),
            "*"
        );
        assert_eq!(WorktreeStatus::Detached.indicator(), "!");
    }

    #[test]
    fn test_sync_status_indicator() {
        let synced = WorktreeSyncStatus::new(0, 0, true);
        assert_eq!(synced.indicator(), "✓");

        let ahead = WorktreeSyncStatus::new(2, 0, true);
        assert_eq!(ahead.indicator(), "↑2");

        let behind = WorktreeSyncStatus::new(0, 3, true);
        assert_eq!(behind.indicator(), "↓3");

        let both = WorktreeSyncStatus::new(1, 2, true);
        assert_eq!(both.indicator(), "↑1↓2");

        let no_remote = WorktreeSyncStatus::new(0, 0, false);
        assert_eq!(no_remote.indicator(), "⊘");
    }
}
