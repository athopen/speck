//! Specification entity and related types.

use crate::error::SpecError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Unique identifier for a specification.
/// Format: "{NNN}-{name}" (e.g., "001-feature-auth")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpecId(String);

impl SpecId {
    /// Create a new SpecId from number and name
    pub fn new(number: u32, name: &str) -> Self {
        Self(format!("{:03}-{}", number, name))
    }

    /// Parse a SpecId from a string
    pub fn parse(s: &str) -> Result<Self, SpecError> {
        let re = Regex::new(r"^(\d{3})-(.+)$").unwrap();
        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(SpecError::InvalidId(s.to_string()))
        }
    }

    /// Get the numeric portion of the ID
    pub fn number(&self) -> u32 {
        self.0[..3].parse().unwrap_or(0)
    }

    /// Get the name portion of the ID
    pub fn name(&self) -> &str {
        if self.0.len() > 4 {
            &self.0[4..]
        } else {
            ""
        }
    }

    /// Get the full ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SpecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SpecId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Represents a feature specification being developed
#[derive(Debug, Clone)]
pub struct Specification {
    /// Unique identifier
    pub id: SpecId,
    /// Numeric portion (e.g., 001)
    pub number: u32,
    /// Short name (e.g., "feature-name")
    pub name: String,
    /// Associated git branch (defaults to directory name)
    pub branch: String,
    /// Current workflow phase (derived from artifacts)
    pub phase: WorkflowPhase,
    /// File artifacts present in the spec directory
    pub artifacts: SpecArtifacts,
    /// Full path to the spec directory
    pub directory: PathBuf,
}

impl Specification {
    /// Create a new Specification from directory info
    pub fn from_directory(directory: PathBuf, artifacts: SpecArtifacts) -> Result<Self, SpecError> {
        let dir_name = directory
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SpecError::InvalidId("Invalid directory name".to_string()))?;

        let id = SpecId::parse(dir_name)?;
        let number = id.number();
        let name = id.name().to_string();
        let branch = dir_name.to_string();
        let phase = WorkflowPhase::from_artifacts(&artifacts);

        Ok(Self {
            id,
            number,
            name,
            branch,
            phase,
            artifacts,
            directory,
        })
    }
}

/// Artifacts present in a specification directory
#[derive(Debug, Clone, Default)]
pub struct SpecArtifacts {
    /// spec.md exists
    pub has_spec: bool,
    /// plan.md exists
    pub has_plan: bool,
    /// tasks.md exists
    pub has_tasks: bool,
    /// research.md exists
    pub has_research: bool,
    /// Path to spec.md if it exists
    pub spec_path: Option<PathBuf>,
    /// Path to plan.md if it exists
    pub plan_path: Option<PathBuf>,
    /// Path to tasks.md if it exists
    pub tasks_path: Option<PathBuf>,
    /// Path to research.md if it exists
    pub research_path: Option<PathBuf>,
}

impl SpecArtifacts {
    /// Scan a directory for artifacts
    pub fn scan(directory: &Path) -> Self {
        let spec_path = directory.join("spec.md");
        let plan_path = directory.join("plan.md");
        let tasks_path = directory.join("tasks.md");
        let research_path = directory.join("research.md");

        Self {
            has_spec: spec_path.exists(),
            has_plan: plan_path.exists(),
            has_tasks: tasks_path.exists(),
            has_research: research_path.exists(),
            spec_path: if spec_path.exists() {
                Some(spec_path)
            } else {
                None
            },
            plan_path: if plan_path.exists() {
                Some(plan_path)
            } else {
                None
            },
            tasks_path: if tasks_path.exists() {
                Some(tasks_path)
            } else {
                None
            },
            research_path: if research_path.exists() {
                Some(research_path)
            } else {
                None
            },
        }
    }
}

/// Current workflow phase of a specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowPhase {
    /// No spec.md exists - needs specification
    Specify,
    /// spec.md exists, no plan.md - needs clarification or planning
    Clarify,
    /// spec.md + plan.md exist, no tasks.md - needs task generation
    Tasks,
    /// All artifacts exist - ready for implementation
    Implement,
}

impl WorkflowPhase {
    /// Determine phase from existing artifacts (FR-002)
    pub fn from_artifacts(artifacts: &SpecArtifacts) -> Self {
        match (artifacts.has_spec, artifacts.has_plan, artifacts.has_tasks) {
            (false, _, _) => Self::Specify,
            (true, false, _) => Self::Clarify,
            (true, true, false) => Self::Tasks,
            (true, true, true) => Self::Implement,
        }
    }

    /// Get available workflow commands for this phase
    pub fn available_commands(&self) -> Vec<super::WorkflowCommandType> {
        use super::WorkflowCommandType;
        match self {
            Self::Specify => vec![WorkflowCommandType::Specify],
            Self::Clarify => vec![WorkflowCommandType::Clarify, WorkflowCommandType::Plan],
            Self::Tasks => vec![WorkflowCommandType::Tasks],
            Self::Implement => vec![WorkflowCommandType::Implement],
        }
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Specify => "Specify",
            Self::Clarify => "Clarify",
            Self::Tasks => "Tasks",
            Self::Implement => "Implement",
        }
    }

    /// Badge/indicator for spec list
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Specify => "[SPEC]",
            Self::Clarify => "[CLARIFY]",
            Self::Tasks => "[TASKS]",
            Self::Implement => "[IMPL]",
        }
    }
}

impl std::fmt::Display for WorkflowPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Type of artifact in a specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    Spec,
    Plan,
    Tasks,
    Research,
    DataModel,
}

impl ArtifactType {
    /// Get the filename for this artifact type
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_id_new() {
        let id = SpecId::new(1, "feature-auth");
        assert_eq!(id.as_str(), "001-feature-auth");
        assert_eq!(id.number(), 1);
        assert_eq!(id.name(), "feature-auth");
    }

    #[test]
    fn test_spec_id_parse() {
        let id = SpecId::parse("042-my-feature").unwrap();
        assert_eq!(id.number(), 42);
        assert_eq!(id.name(), "my-feature");
    }

    #[test]
    fn test_spec_id_parse_invalid() {
        assert!(SpecId::parse("invalid").is_err());
        assert!(SpecId::parse("42-feature").is_err()); // not zero-padded
    }

    #[test]
    fn test_workflow_phase_from_artifacts() {
        let no_artifacts = SpecArtifacts::default();
        assert_eq!(
            WorkflowPhase::from_artifacts(&no_artifacts),
            WorkflowPhase::Specify
        );

        let spec_only = SpecArtifacts {
            has_spec: true,
            ..Default::default()
        };
        assert_eq!(
            WorkflowPhase::from_artifacts(&spec_only),
            WorkflowPhase::Clarify
        );

        let spec_and_plan = SpecArtifacts {
            has_spec: true,
            has_plan: true,
            ..Default::default()
        };
        assert_eq!(
            WorkflowPhase::from_artifacts(&spec_and_plan),
            WorkflowPhase::Tasks
        );

        let all_artifacts = SpecArtifacts {
            has_spec: true,
            has_plan: true,
            has_tasks: true,
            ..Default::default()
        };
        assert_eq!(
            WorkflowPhase::from_artifacts(&all_artifacts),
            WorkflowPhase::Implement
        );
    }
}
