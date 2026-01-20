# Data Model: Spec-Driven Development TUI

**Generated**: 2026-01-20
**Source**: Feature specification entities and functional requirements

## Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                              Project                                 │
│  (root entity - one per repository)                                 │
├─────────────────────────────────────────────────────────────────────┤
│  - root_path: PathBuf                                               │
│  - specs_directory: PathBuf                                         │
│  - main_branch: String                                              │
│  - config: ProjectConfig                                            │
└─────────────────────────────────────────────────────────────────────┘
          │                           │
          │ 1:*                       │ 1:*
          ▼                           ▼
┌──────────────────────┐    ┌─────────────────────────┐
│    Specification     │    │       Worktree          │
├──────────────────────┤    ├─────────────────────────┤
│  - id: SpecId        │◄──►│  - path: PathBuf        │
│  - number: u32       │    │  - branch: String       │
│  - name: String      │    │  - status: WorktreeStatus│
│  - branch: String    │    │  - spec_id: Option<SpecId>│
│  - phase: Phase      │    └─────────────────────────┘
│  - artifacts: Artifacts│
└──────────────────────┘
          │
          │ 1:*
          ▼
┌──────────────────────┐
│   WorkflowCommand    │
├──────────────────────┤
│  - command_type: Type│
│  - state: ExecState  │
│  - output: Vec<String>│
│  - started_at: Option│
│  - cancelled: bool   │
└──────────────────────┘
```

## Entities

### Project

Represents the overall git repository context.

```rust
struct Project {
    root_path: PathBuf,           // Repository root
    specs_directory: PathBuf,     // Default: {root}/specs/
    worktree_directory: PathBuf,  // Default: {root}/.worktrees/
    main_branch: String,          // Default: "main"
    config: ProjectConfig,        // Loaded configuration
}
```

**Validation Rules**:
- `root_path` must exist and be a git repository
- `specs_directory` must exist or be creatable
- `worktree_directory` parent must be writable

**Derivation**:
- Discovered by walking up from current directory to find `.git`
- Config loaded from hierarchy: defaults → project → user → env

---

### Specification

Represents a feature being developed.

```rust
struct SpecId(String);  // e.g., "001-feature-name"

struct Specification {
    id: SpecId,
    number: u32,                  // Parsed from directory name (e.g., 001)
    name: String,                 // Short name (e.g., "feature-name")
    branch: String,               // Git branch (e.g., "001-feature-name")
    phase: WorkflowPhase,         // Derived from artifacts
    artifacts: SpecArtifacts,
    directory: PathBuf,           // Full path to spec directory
}

struct SpecArtifacts {
    has_spec: bool,               // spec.md exists
    has_plan: bool,               // plan.md exists
    has_tasks: bool,              // tasks.md exists
    has_research: bool,           // research.md exists
    spec_path: Option<PathBuf>,
    plan_path: Option<PathBuf>,
    tasks_path: Option<PathBuf>,
}
```

**Validation Rules**:
- `number` must be unique across all specifications
- Directory name must match pattern `{NNN}-{name}` where NNN is zero-padded
- `branch` defaults to directory name if not explicitly set

**State Transitions (WorkflowPhase)**:

```
┌─────────────────────────────────────────────────────────────────┐
│                    WorkflowPhase Derivation                     │
├─────────────────────────────────────────────────────────────────┤
│  No artifacts         → Specify   (initial state)              │
│  spec.md only         → Clarify   (can also Plan)              │
│  spec.md + plan.md    → Tasks     (ready for task generation)  │
│  spec.md + plan.md + tasks.md → Implement                      │
└─────────────────────────────────────────────────────────────────┘
```

```rust
enum WorkflowPhase {
    Specify,    // No spec.md exists
    Clarify,    // spec.md exists, no plan.md
    Plan,       // Alias for Clarify (either command valid)
    Tasks,      // spec.md + plan.md exist, no tasks.md
    Implement,  // All artifacts exist
}

impl WorkflowPhase {
    fn from_artifacts(artifacts: &SpecArtifacts) -> Self {
        match (artifacts.has_spec, artifacts.has_plan, artifacts.has_tasks) {
            (false, _, _) => Self::Specify,
            (true, false, _) => Self::Clarify,
            (true, true, false) => Self::Tasks,
            (true, true, true) => Self::Implement,
        }
    }

    fn available_commands(&self) -> Vec<WorkflowCommandType> {
        match self {
            Self::Specify => vec![WorkflowCommandType::Specify],
            Self::Clarify => vec![WorkflowCommandType::Clarify, WorkflowCommandType::Plan],
            Self::Plan => vec![WorkflowCommandType::Plan],
            Self::Tasks => vec![WorkflowCommandType::Tasks],
            Self::Implement => vec![WorkflowCommandType::Implement],
        }
    }
}
```

---

### Worktree

Represents a git worktree instance.

```rust
struct Worktree {
    path: PathBuf,                // Full path to worktree directory
    branch: String,               // Associated branch name
    status: WorktreeStatus,       // Working tree state
    spec_id: Option<SpecId>,      // Associated spec (if matches pattern)
    is_main: bool,                // Is this the main worktree?
}

enum WorktreeStatus {
    Clean,                        // No uncommitted changes
    Dirty {                       // Has uncommitted changes
        modified: u32,
        staged: u32,
        untracked: u32,
    },
    Detached,                     // HEAD is detached
    Unknown,                      // Status cannot be determined
}

struct WorktreeSyncStatus {
    ahead: u32,                   // Commits ahead of remote
    behind: u32,                  // Commits behind remote
    remote_exists: bool,          // Remote branch exists
}
```

**Validation Rules**:
- `path` must be a valid directory
- `branch` must exist in the repository
- Cannot create duplicate worktrees for the same branch (FR-009)
- Cannot delete the main worktree or currently active worktree

**Operations**:

| Operation | Preconditions | Postconditions |
|-----------|---------------|----------------|
| Create | Branch exists, no existing worktree for branch | Worktree directory exists, git worktree registered |
| Switch | Worktree exists | Current working directory is worktree path |
| Delete | Worktree is not current, user confirmed | Worktree directory removed, git worktree pruned |
| Refresh | Worktree exists | Status updated from git |

---

### WorkflowCommand

Represents an executable workflow action.

```rust
enum WorkflowCommandType {
    Specify,
    Clarify,
    Plan,
    Tasks,
    Implement,
}

enum ExecutionState {
    Pending,
    Running {
        started_at: Instant,
        pid: Option<u32>,
    },
    Completed {
        exit_code: i32,
        duration: Duration,
    },
    Cancelled,
    Failed {
        error: String,
    },
}

struct WorkflowCommand {
    command_type: WorkflowCommandType,
    spec_id: SpecId,
    state: ExecutionState,
    output: Vec<OutputLine>,
    log_path: Option<PathBuf>,
}

struct OutputLine {
    timestamp: Instant,
    content: String,
    stream: OutputStream,  // Stdout or Stderr
}

enum OutputStream {
    Stdout,
    Stderr,
}
```

**Validation Rules (Prerequisites - FR-015)**:

| Command | Prerequisites |
|---------|---------------|
| Specify | Spec directory exists |
| Clarify | spec.md exists |
| Plan | spec.md exists |
| Tasks | spec.md AND plan.md exist |
| Implement | spec.md AND plan.md AND tasks.md exist |

**State Transitions**:

```
Pending → Running → Completed
                  → Cancelled (user-initiated)
                  → Failed (error/timeout)
```

---

### ProjectConfig

Configuration loaded from file hierarchy.

```rust
struct ProjectConfig {
    worktree: WorktreeConfig,
    mcp: McpConfig,
    ui: UiConfig,
    git: GitConfig,
}

struct WorktreeConfig {
    directory: PathBuf,           // Default: ".worktrees"
}

struct McpConfig {
    transport: McpTransport,
    timeout_seconds: u64,         // Default: 60
}

enum McpTransport {
    Stdio,                        // Spawn process, communicate via stdin/stdout
    Http { endpoint: String },    // HTTP with SSE for responses
}

struct UiConfig {
    refresh_rate_ms: u64,         // Default: 100
    vim_navigation: bool,         // Default: true
}

struct GitConfig {
    specs_directory: String,      // Default: "specs"
    main_branch: String,          // Default: "main"
}
```

**Validation Rules**:
- `refresh_rate_ms` must be >= 16 (60fps) and <= 1000
- `timeout_seconds` must be > 0 and <= 600
- `specs_directory` must not contain path separators

---

## UI State Model

### Application State

```rust
struct AppState {
    project: Project,
    specs: Vec<Specification>,
    worktrees: Vec<Worktree>,
    active_command: Option<WorkflowCommand>,

    // UI State
    view: AppView,
    selected_spec_index: usize,
    scroll_offset: usize,
    editor_state: Option<EditorState>,

    // Async State
    pending_operations: Vec<PendingOperation>,
    error_message: Option<String>,
}

enum AppView {
    Overview,                     // Main spec list
    SpecDetail(SpecId),          // Viewing spec details
    WorktreeManagement,          // Worktree list and operations
    DocumentView(DocType),       // Viewing a document
    DocumentEdit(DocType),       // Editing a document
    CommandOutput,               // Workflow command output panel
}

enum DocType {
    Spec,
    Plan,
    Tasks,
    Research,
}

struct EditorState {
    doc_type: DocType,
    content: String,
    cursor_position: (usize, usize),
    modified: bool,
    original_content: String,
}
```

### Input Handling

```rust
enum InputMode {
    Normal,                       // Standard navigation
    Insert,                       // Text editing mode
    Command,                      // Command palette / search
}

struct KeyBindings {
    // Navigation (Normal mode)
    up: Vec<KeyCode>,             // [Up, 'k']
    down: Vec<KeyCode>,           // [Down, 'j']
    left: Vec<KeyCode>,           // [Left, 'h']
    right: Vec<KeyCode>,          // [Right, 'l']
    select: Vec<KeyCode>,         // [Enter, ' ']
    back: Vec<KeyCode>,           // [Esc, 'q']

    // Actions
    switch_worktree: KeyCode,     // 'w'
    trigger_workflow: KeyCode,    // 'r' (run)
    view_document: KeyCode,       // 'v'
    edit_document: KeyCode,       // 'e'
    new_spec: KeyCode,            // 'n'
    delete_worktree: KeyCode,     // 'd' (with confirmation)
    cancel_command: KeyCode,      // 'c' or Ctrl+C
    help: KeyCode,                // '?'
}
```

---

## Relationships Summary

| From | To | Cardinality | Description |
|------|------|-------------|-------------|
| Project | Specification | 1:* | Project contains multiple specs |
| Project | Worktree | 1:* | Project contains multiple worktrees |
| Specification | Worktree | 1:0..1 | Spec may have associated worktree |
| Specification | WorkflowCommand | 1:* | Spec can have command history |
| Worktree | WorkflowCommand | 0..1:* | Commands execute in worktree context |

---

## Index Requirements

For efficient lookup:

1. **Specs by ID**: HashMap<SpecId, Specification>
2. **Specs by number**: BTreeMap<u32, SpecId> (for sorted display)
3. **Worktrees by branch**: HashMap<String, Worktree>
4. **Worktrees by spec**: HashMap<SpecId, Worktree>
