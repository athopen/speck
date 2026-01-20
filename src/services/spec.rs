//! Specification service for discovering and managing specs.

use crate::domain::{ArtifactType, SpecArtifacts, SpecId, Specification, WorkflowPhase};
use crate::error::{SpecError, SpecResult};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

/// Service for specification discovery and management
pub struct SpecService {
    specs_directory: PathBuf,
}

impl SpecService {
    /// Create a new SpecService
    pub fn new(specs_directory: PathBuf) -> Self {
        Self { specs_directory }
    }

    /// Discover all specifications in the specs directory
    pub fn discover_specs(&self) -> SpecResult<Vec<Specification>> {
        if !self.specs_directory.exists() {
            return Err(SpecError::DirectoryNotFound(self.specs_directory.clone()));
        }

        let spec_pattern = Regex::new(r"^\d{3}-.+$").unwrap();
        let mut specs = Vec::new();

        let entries = fs::read_dir(&self.specs_directory).map_err(SpecError::Io)?;

        for entry in entries {
            let entry = entry.map_err(SpecError::Io)?;
            let path = entry.path();

            // Only process directories
            if !path.is_dir() {
                continue;
            }

            // Check if directory name matches spec pattern
            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) if spec_pattern.is_match(name) => name.to_string(),
                _ => continue,
            };

            // Scan for artifacts
            let artifacts = SpecArtifacts::scan(&path);

            // Create specification
            match Specification::from_directory(path, artifacts) {
                Ok(spec) => specs.push(spec),
                Err(e) => {
                    // Log error but continue with other specs
                    tracing::warn!("Failed to load spec {}: {}", dir_name, e);
                }
            }
        }

        // Sort by number
        specs.sort_by_key(|s| s.number);

        Ok(specs)
    }

    /// Load a specific specification by ID
    pub fn load_spec(&self, id: &SpecId) -> SpecResult<Specification> {
        let spec_dir = self.specs_directory.join(id.as_str());

        if !spec_dir.exists() {
            return Err(SpecError::NotFound(id.to_string()));
        }

        let artifacts = SpecArtifacts::scan(&spec_dir);
        Specification::from_directory(spec_dir, artifacts)
    }

    /// Get the current workflow phase for a specification
    pub fn get_phase(&self, id: &SpecId) -> SpecResult<WorkflowPhase> {
        let spec = self.load_spec(id)?;
        Ok(spec.phase)
    }

    /// Create a new specification directory structure
    pub fn create_spec(&self, number: u32, name: &str) -> SpecResult<Specification> {
        // Validate name
        if name.is_empty() || name.contains('/') || name.contains('\\') {
            return Err(SpecError::InvalidName(name.to_string()));
        }

        let id = SpecId::new(number, name);
        let spec_dir = self.specs_directory.join(id.as_str());

        // Check if already exists
        if spec_dir.exists() {
            return Err(SpecError::AlreadyExists(id.to_string()));
        }

        // Create directory
        fs::create_dir_all(&spec_dir).map_err(SpecError::Io)?;

        // Create initial spec.md with template
        let spec_content = format!(
            "# Feature Specification: {}\n\n\
            **Feature Branch**: `{}`\n\
            **Created**: {}\n\
            **Status**: Draft\n\n\
            ## Description\n\n\
            [Describe the feature here]\n\n\
            ## User Scenarios\n\n\
            [Add user scenarios]\n\n\
            ## Requirements\n\n\
            [Add requirements]\n",
            name.replace('-', " "),
            id.as_str(),
            chrono_lite::Utc::today()
                .split('T')
                .next()
                .unwrap_or_default(),
        );

        // For now, just create empty spec.md
        let spec_path = spec_dir.join("spec.md");
        fs::write(&spec_path, &spec_content).map_err(SpecError::Io)?;

        // Return the new specification
        let artifacts = SpecArtifacts::scan(&spec_dir);
        Specification::from_directory(spec_dir, artifacts)
    }

    /// Read a specification artifact
    pub fn read_artifact(&self, id: &SpecId, artifact: ArtifactType) -> SpecResult<String> {
        let spec_dir = self.specs_directory.join(id.as_str());
        let artifact_path = spec_dir.join(artifact.filename());

        if !artifact_path.exists() {
            return Err(SpecError::ArtifactNotFound(artifact.filename().to_string()));
        }

        fs::read_to_string(&artifact_path).map_err(SpecError::Io)
    }

    /// Write a specification artifact
    pub fn write_artifact(
        &self,
        id: &SpecId,
        artifact: ArtifactType,
        content: &str,
    ) -> SpecResult<()> {
        let spec_dir = self.specs_directory.join(id.as_str());

        if !spec_dir.exists() {
            return Err(SpecError::NotFound(id.to_string()));
        }

        let artifact_path = spec_dir.join(artifact.filename());
        fs::write(&artifact_path, content).map_err(SpecError::Io)
    }

    /// Get the next available spec number
    pub fn next_number(&self) -> SpecResult<u32> {
        let specs = self.discover_specs()?;
        let max_number = specs.iter().map(|s| s.number).max().unwrap_or(0);
        Ok(max_number + 1)
    }
}

/// Minimal date helper (to avoid heavy chrono dependency)
mod chrono_lite {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct Utc;

    impl Utc {
        pub fn today() -> String {
            let duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let days = duration.as_secs() / 86400;

            // Simple date calculation (not accounting for all edge cases)
            let mut year = 1970i32;
            let mut remaining_days = days as i32;

            loop {
                let days_in_year = if is_leap_year(year) { 366 } else { 365 };
                if remaining_days < days_in_year {
                    break;
                }
                remaining_days -= days_in_year;
                year += 1;
            }

            let mut month = 1;
            let days_in_months = if is_leap_year(year) {
                [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            } else {
                [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            };

            for days_in_month in days_in_months.iter() {
                if remaining_days < *days_in_month {
                    break;
                }
                remaining_days -= days_in_month;
                month += 1;
            }

            let day = remaining_days + 1;

            format!("{:04}-{:02}-{:02}", year, month, day)
        }
    }

    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_specs_dir() -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create some test specs
        let spec1 = specs_dir.join("001-feature-a");
        fs::create_dir_all(&spec1).unwrap();
        fs::write(spec1.join("spec.md"), "# Feature A").unwrap();

        let spec2 = specs_dir.join("002-feature-b");
        fs::create_dir_all(&spec2).unwrap();
        fs::write(spec2.join("spec.md"), "# Feature B").unwrap();
        fs::write(spec2.join("plan.md"), "# Plan B").unwrap();

        (temp, specs_dir)
    }

    #[test]
    fn test_discover_specs() {
        let (_temp, specs_dir) = create_test_specs_dir();
        let service = SpecService::new(specs_dir);

        let specs = service.discover_specs().unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].number, 1);
        assert_eq!(specs[1].number, 2);
    }

    #[test]
    fn test_load_spec() {
        let (_temp, specs_dir) = create_test_specs_dir();
        let service = SpecService::new(specs_dir);

        let id = SpecId::new(1, "feature-a");
        let spec = service.load_spec(&id).unwrap();
        assert_eq!(spec.phase, WorkflowPhase::Clarify);
    }

    #[test]
    fn test_get_phase() {
        let (_temp, specs_dir) = create_test_specs_dir();
        let service = SpecService::new(specs_dir);

        let id1 = SpecId::new(1, "feature-a");
        assert_eq!(service.get_phase(&id1).unwrap(), WorkflowPhase::Clarify);

        let id2 = SpecId::new(2, "feature-b");
        assert_eq!(service.get_phase(&id2).unwrap(), WorkflowPhase::Tasks);
    }

    #[test]
    fn test_next_number() {
        let (_temp, specs_dir) = create_test_specs_dir();
        let service = SpecService::new(specs_dir);

        assert_eq!(service.next_number().unwrap(), 3);
    }

    #[test]
    fn test_read_artifact() {
        let (_temp, specs_dir) = create_test_specs_dir();
        let service = SpecService::new(specs_dir);

        let id = SpecId::new(1, "feature-a");
        let content = service.read_artifact(&id, ArtifactType::Spec).unwrap();
        assert_eq!(content, "# Feature A");
    }
}
