# Specification Service Contract

## Overview

The Specification service discovers, loads, and manages feature specifications. It provides the mapping between spec directories, branches, and workflow phases.

## Service Interface

```rust
/// Specification discovery and management service
pub trait SpecService {
    /// Discover all specifications in the specs directory
    fn discover_specs(&self) -> Result<Vec<Specification>, SpecError>;

    /// Load a specific specification by ID
    fn load_spec(&self, id: &SpecId) -> Result<Specification, SpecError>;

    /// Get the current workflow phase for a specification
    fn get_phase(&self, id: &SpecId) -> Result<WorkflowPhase, SpecError>;

    /// Create a new specification directory structure
    fn create_spec(&self, number: u32, name: &str) -> Result<Specification, SpecError>;

    /// Read a specification artifact (spec.md, plan.md, etc.)
    fn read_artifact(&self, id: &SpecId, artifact: ArtifactType) -> Result<String, SpecError>;

    /// Write a specification artifact
    fn write_artifact(&self, id: &SpecId, artifact: ArtifactType, content: &str) -> Result<(), SpecError>;

    /// Get the next available spec number
    fn next_number(&self) -> Result<u32, SpecError>;
}
```

## Data Types

### SpecId

```rust
/// Unique identifier for a specification
/// Format: "{NNN}-{name}" (e.g., "001-feature-auth")
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecId(String);

impl SpecId {
    pub fn new(number: u32, name: &str) -> Self {
        Self(format!("{:03}-{}", number, name))
    }

    pub fn parse(s: &str) -> Result<Self, SpecError> {
        // Validate format: NNN-name
        let re = regex::Regex::new(r"^(\d{3})-(.+)$").unwrap();
        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(SpecError::InvalidId(s.to_string()))
        }
    }

    pub fn number(&self) -> u32 {
        self.0[..3].parse().unwrap()
    }

    pub fn name(&self) -> &str {
        &self.0[4..]
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Specification

```rust
pub struct Specification {
    pub id: SpecId,
    pub directory: PathBuf,
    pub branch: String,
    pub phase: WorkflowPhase,
    pub artifacts: SpecArtifacts,
}

pub struct SpecArtifacts {
    pub spec: Option<PathBuf>,      // spec.md
    pub plan: Option<PathBuf>,      // plan.md
    pub tasks: Option<PathBuf>,     // tasks.md
    pub research: Option<PathBuf>,  // research.md
}

impl SpecArtifacts {
    pub fn has_spec(&self) -> bool { self.spec.is_some() }
    pub fn has_plan(&self) -> bool { self.plan.is_some() }
    pub fn has_tasks(&self) -> bool { self.tasks.is_some() }
}
```

### WorkflowPhase

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowPhase {
    Specify,    // No spec.md
    Clarify,    // spec.md exists, no plan.md
    Plan,       // Alias state (same as Clarify)
    Tasks,      // spec.md + plan.md, no tasks.md
    Implement,  // All artifacts present
}

impl WorkflowPhase {
    /// Determine phase from existing artifacts (FR-002)
    pub fn from_artifacts(artifacts: &SpecArtifacts) -> Self {
        match (artifacts.has_spec(), artifacts.has_plan(), artifacts.has_tasks()) {
            (false, _, _) => Self::Specify,
            (true, false, _) => Self::Clarify,
            (true, true, false) => Self::Tasks,
            (true, true, true) => Self::Implement,
        }
    }

    /// Commands available in this phase
    pub fn available_commands(&self) -> Vec<WorkflowCommandType> {
        match self {
            Self::Specify => vec![WorkflowCommandType::Specify],
            Self::Clarify | Self::Plan => vec![
                WorkflowCommandType::Clarify,
                WorkflowCommandType::Plan,
            ],
            Self::Tasks => vec![WorkflowCommandType::Tasks],
            Self::Implement => vec![WorkflowCommandType::Implement],
        }
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Specify => "Specify",
            Self::Clarify => "Clarify",
            Self::Plan => "Plan",
            Self::Tasks => "Tasks",
            Self::Implement => "Implement",
        }
    }

    /// Badge/indicator for spec list
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Specify => "[SPEC]",
            Self::Clarify => "[CLARIFY]",
            Self::Plan => "[PLAN]",
            Self::Tasks => "[TASKS]",
            Self::Implement => "[IMPL]",
        }
    }
}
```

### ArtifactType

```rust
#[derive(Clone, Copy, Debug)]
pub enum ArtifactType {
    Spec,
    Plan,
    Tasks,
    Research,
    DataModel,
}

impl ArtifactType {
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Spec => "spec.md",
            Self::Plan => "plan.md",
            Self::Tasks => "tasks.md",
            Self::Research => "research.md",
            Self::DataModel => "data-model.md",
        }
    }
}
```

## Operations

### discover_specs

Scans the specs directory for all specification subdirectories.

**Input**: None (uses configured specs_directory)

**Output**: `Vec<Specification>` sorted by number

**Discovery Rules**:
1. Scan `{project_root}/specs/` for directories
2. Each directory name must match pattern `{NNN}-{name}`
3. Extract number and name from directory
4. Determine phase from existing files
5. Default branch name to directory name

**Errors**:
- `DirectoryNotFound`: specs directory doesn't exist
- `IoError`: Filesystem error during scan

**Example Discovery**:
```
specs/
├── 001-user-auth/         → SpecId("001-user-auth"), phase based on files
│   ├── spec.md
│   └── plan.md
├── 002-dashboard/         → SpecId("002-dashboard")
│   └── spec.md
├── 003-reporting/         → SpecId("003-reporting")
│   ├── spec.md
│   ├── plan.md
│   └── tasks.md
└── notes.txt              → Ignored (not a directory)
```

---

### load_spec

Loads full details for a single specification.

**Input**: `SpecId`

**Output**: `Specification`

**Errors**:
- `NotFound`: Specification doesn't exist
- `IoError`: Cannot read directory

---

### create_spec

Creates a new specification directory with initial structure.

**Input**:
- `number`: Spec number (or auto-assign via `next_number()`)
- `name`: Short name (slugified)

**Output**: Created `Specification`

**Actions**:
1. Create directory `specs/{NNN}-{name}/`
2. Create empty `spec.md` with template header
3. Return new Specification in `Specify` phase

**Errors**:
- `AlreadyExists`: Directory already exists
- `InvalidName`: Name contains invalid characters
- `IoError`: Cannot create directory

---

### read_artifact / write_artifact

Read or write specification document contents.

**Input**:
- `id`: Specification ID
- `artifact`: Type of artifact
- `content`: (write only) New content

**Errors**:
- `NotFound`: Specification or artifact doesn't exist
- `IoError`: Filesystem error

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("specification not found: {0}")]
    NotFound(SpecId),

    #[error("specification already exists: {0}")]
    AlreadyExists(SpecId),

    #[error("invalid specification id: {0}")]
    InvalidId(String),

    #[error("invalid specification name: {0}")]
    InvalidName(String),

    #[error("specs directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("artifact not found: {0:?}")]
    ArtifactNotFound(ArtifactType),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Configuration

```rust
pub struct SpecServiceConfig {
    /// Root directory containing specs
    pub specs_directory: PathBuf,  // Default: "specs"

    /// Template for new spec.md files
    pub spec_template: Option<PathBuf>,
}
```

## Usage Examples

### Listing specs with phases

```rust
let spec_service = SpecServiceImpl::new(config)?;
let specs = spec_service.discover_specs()?;

for spec in specs {
    println!(
        "{} {} - Phase: {}",
        spec.id.as_str(),
        spec.phase.badge(),
        spec.phase.display_name()
    );
}
```

### Creating a new spec

```rust
let number = spec_service.next_number()?;
let spec = spec_service.create_spec(number, "my-feature")?;
// Creates: specs/004-my-feature/spec.md
```

### Reading and writing artifacts

```rust
// Read spec content
let content = spec_service.read_artifact(&spec_id, ArtifactType::Spec)?;

// Write updated content
spec_service.write_artifact(&spec_id, ArtifactType::Spec, &new_content)?;
```
