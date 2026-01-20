# Git Service Contract

**Implementation**: gitoxide (gix crate)

## Overview

The Git service provides worktree management and repository operations. All operations are blocking (filesystem I/O) and must be executed via `spawn_blocking` to avoid blocking the async runtime.

## Service Interface

```rust
/// Git service for repository and worktree operations
pub trait GitService {
    /// List all worktrees in the repository
    fn list_worktrees(&self) -> Result<Vec<Worktree>, GitError>;

    /// Create a new worktree for a branch
    fn create_worktree(&self, branch: &str, path: &Path) -> Result<Worktree, GitError>;

    /// Delete a worktree (removes directory and prunes git metadata)
    fn delete_worktree(&self, path: &Path, force: bool) -> Result<(), GitError>;

    /// Get the status of a worktree (clean/dirty/detached)
    fn worktree_status(&self, path: &Path) -> Result<WorktreeStatus, GitError>;

    /// Get sync status with remote (ahead/behind counts)
    fn sync_status(&self, branch: &str) -> Result<WorktreeSyncStatus, GitError>;

    /// Check if a branch exists (local or remote)
    fn branch_exists(&self, branch: &str) -> Result<bool, GitError>;

    /// Get the current branch of a worktree
    fn current_branch(&self, path: &Path) -> Result<String, GitError>;

    /// Get the main worktree path
    fn main_worktree(&self) -> Result<PathBuf, GitError>;
}
```

## Operations

### list_worktrees

Returns all worktrees associated with the repository.

**Input**: None

**Output**:
```rust
struct Worktree {
    path: PathBuf,
    branch: String,
    is_main: bool,
    is_bare: bool,
    locked: Option<String>,  // Lock reason if locked
}
```

**Errors**:
- `NotARepository`: Current directory is not a git repository
- `GitError`: gitoxide operation failed

**Postconditions**:
- Returns at least one worktree (main worktree)
- All returned paths exist on filesystem

---

### create_worktree

Creates a new worktree for a specified branch.

**Input**:
```rust
struct CreateWorktreeParams {
    branch: String,     // Branch to check out
    path: PathBuf,      // Where to create worktree
}
```

**Preconditions**:
- Branch must exist (local or remote)
- Path must not exist or be empty
- No existing worktree for this branch (FR-009)

**Output**: Created `Worktree` struct

**Errors**:
- `BranchNotFound`: Branch doesn't exist
- `WorktreeExists`: Worktree already exists for branch
- `PathExists`: Target path already exists and is not empty
- `IoError`: Filesystem operation failed

**Postconditions**:
- Worktree directory exists at path
- Branch is checked out in worktree
- Worktree is registered with git

---

### delete_worktree

Removes a worktree and its directory.

**Input**:
```rust
struct DeleteWorktreeParams {
    path: PathBuf,
    force: bool,  // Delete even if dirty
}
```

**Preconditions**:
- Worktree must exist
- Worktree must not be main worktree
- If not force, worktree must be clean

**Output**: None

**Errors**:
- `WorktreeNotFound`: No worktree at path
- `CannotDeleteMain`: Attempted to delete main worktree
- `WorktreeDirty`: Worktree has uncommitted changes and force=false
- `IoError`: Filesystem operation failed

**Postconditions**:
- Worktree directory removed
- Worktree unregistered from git
- Associated files cleaned up

---

### worktree_status

Gets the working tree status (uncommitted changes).

**Input**: Path to worktree

**Output**:
```rust
enum WorktreeStatus {
    Clean,
    Dirty {
        modified: u32,
        staged: u32,
        untracked: u32,
    },
    Detached,
    Unknown,
}
```

**Errors**:
- `WorktreeNotFound`: Path is not a worktree
- `GitError`: Status operation failed

---

### sync_status

Gets ahead/behind counts relative to remote tracking branch.

**Input**: Branch name

**Output**:
```rust
struct WorktreeSyncStatus {
    ahead: u32,
    behind: u32,
    remote_exists: bool,
}
```

**Errors**:
- `BranchNotFound`: Branch doesn't exist
- `NoRemote`: No remote configured

---

### branch_exists

Checks if a branch exists locally or on remote.

**Input**: Branch name

**Output**: `bool`

**Errors**:
- `GitError`: Operation failed

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("not a git repository")]
    NotARepository,

    #[error("branch not found: {0}")]
    BranchNotFound(String),

    #[error("worktree already exists for branch: {0}")]
    WorktreeExists(String),

    #[error("worktree not found: {0}")]
    WorktreeNotFound(PathBuf),

    #[error("cannot delete main worktree")]
    CannotDeleteMain,

    #[error("worktree has uncommitted changes")]
    WorktreeDirty,

    #[error("path already exists: {0}")]
    PathExists(PathBuf),

    #[error("no remote configured")]
    NoRemote,

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("git error: {0}")]
    Git(#[from] gix::Error),
}
```

## Async Integration

All operations are blocking and must be wrapped for async contexts:

```rust
impl GitService for GitServiceImpl {
    async fn list_worktrees(&self) -> Result<Vec<Worktree>, GitError> {
        let repo = self.repo.clone();
        tokio::task::spawn_blocking(move || {
            repo.worktrees()?.collect()
        }).await?
    }
}
```

## Event Notifications

The service can emit events for UI updates:

```rust
pub enum GitEvent {
    WorktreeCreated { path: PathBuf, branch: String },
    WorktreeDeleted { path: PathBuf },
    StatusChanged { path: PathBuf, status: WorktreeStatus },
}
```

## Configuration

```rust
pub struct GitServiceConfig {
    /// Default directory for new worktrees
    pub worktree_base: PathBuf,  // Default: ".worktrees"

    /// Pattern for worktree directory names
    pub worktree_pattern: String,  // Default: "{branch}"

    /// Auto-fetch before status checks
    pub auto_fetch: bool,  // Default: false
}
```

## Usage Examples

### Creating a worktree for a spec

```rust
let git = GitServiceImpl::new(repo_path)?;

// Check if branch exists
if !git.branch_exists("001-feature")? {
    return Err(Error::BranchNotFound("001-feature"));
}

// Check if worktree already exists
let worktrees = git.list_worktrees()?;
if worktrees.iter().any(|w| w.branch == "001-feature") {
    return Err(Error::WorktreeExists("001-feature"));
}

// Create worktree
let worktree_path = worktree_base.join("001-feature");
let worktree = git.create_worktree("001-feature", &worktree_path)?;
```

### Checking worktree health

```rust
let worktrees = git.list_worktrees()?;
for worktree in worktrees {
    let status = git.worktree_status(&worktree.path)?;
    let sync = git.sync_status(&worktree.branch)?;

    println!(
        "{}: {:?}, ahead={}, behind={}",
        worktree.branch, status, sync.ahead, sync.behind
    );
}
```
