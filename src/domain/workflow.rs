//! Workflow command entity and related types.

use super::SpecId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Type of workflow command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowCommandType {
    Specify,
    Clarify,
    Plan,
    Tasks,
    Implement,
}

impl WorkflowCommandType {
    /// Get the MCP tool name for this command
    pub fn tool_name(&self) -> &'static str {
        match self {
            Self::Specify => "speckit.specify",
            Self::Clarify => "speckit.clarify",
            Self::Plan => "speckit.plan",
            Self::Tasks => "speckit.tasks",
            Self::Implement => "speckit.implement",
        }
    }

    /// Get the display name for this command
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Specify => "Specify",
            Self::Clarify => "Clarify",
            Self::Plan => "Plan",
            Self::Tasks => "Tasks",
            Self::Implement => "Implement",
        }
    }

    /// Get the keyboard shortcut hint
    pub fn shortcut_hint(&self) -> &'static str {
        match self {
            Self::Specify => "s",
            Self::Clarify => "c",
            Self::Plan => "p",
            Self::Tasks => "t",
            Self::Implement => "i",
        }
    }
}

impl std::fmt::Display for WorkflowCommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Execution state of a workflow command
#[derive(Debug, Clone)]
pub enum ExecutionState {
    /// Command is queued but not started
    Pending,
    /// Command is currently running
    Running {
        started_at: Instant,
        pid: Option<u32>,
    },
    /// Command completed successfully
    Completed { exit_code: i32, duration: Duration },
    /// Command was cancelled by user
    Cancelled,
    /// Command failed with error
    Failed { error: String },
}

impl ExecutionState {
    /// Check if the command is still running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    /// Check if the command is pending
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Check if the command has finished (completed, cancelled, or failed)
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Cancelled | Self::Failed { .. }
        )
    }

    /// Get status indicator for UI
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Pending => "⏳",
            Self::Running { .. } => "🔄",
            Self::Completed { exit_code, .. } if *exit_code == 0 => "✓",
            Self::Completed { .. } => "✗",
            Self::Cancelled => "⊘",
            Self::Failed { .. } => "✗",
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::Pending
    }
}

/// A workflow command being executed
#[derive(Debug, Clone)]
pub struct WorkflowCommand {
    /// Type of command
    pub command_type: WorkflowCommandType,
    /// Target specification
    pub spec_id: SpecId,
    /// Current execution state
    pub state: ExecutionState,
    /// Output lines from the command
    pub output: Vec<OutputLine>,
    /// Path to log file (if persisted)
    pub log_path: Option<PathBuf>,
}

impl WorkflowCommand {
    /// Create a new pending workflow command
    pub fn new(command_type: WorkflowCommandType, spec_id: SpecId) -> Self {
        Self {
            command_type,
            spec_id,
            state: ExecutionState::Pending,
            output: Vec::new(),
            log_path: None,
        }
    }

    /// Mark the command as running
    pub fn start(&mut self) {
        self.state = ExecutionState::Running {
            started_at: Instant::now(),
            pid: None,
        };
    }

    /// Mark the command as running with PID
    pub fn start_with_pid(&mut self, pid: u32) {
        self.state = ExecutionState::Running {
            started_at: Instant::now(),
            pid: Some(pid),
        };
    }

    /// Mark the command as completed
    pub fn complete(&mut self, exit_code: i32) {
        if let ExecutionState::Running { started_at, .. } = self.state {
            self.state = ExecutionState::Completed {
                exit_code,
                duration: started_at.elapsed(),
            };
        }
    }

    /// Mark the command as cancelled
    pub fn cancel(&mut self) {
        self.state = ExecutionState::Cancelled;
    }

    /// Mark the command as failed
    pub fn fail(&mut self, error: String) {
        self.state = ExecutionState::Failed { error };
    }

    /// Add an output line
    pub fn add_output(&mut self, content: String, stream: OutputStream) {
        self.output.push(OutputLine {
            timestamp: Instant::now(),
            content,
            stream,
        });
    }

    /// Get all output as a single string
    pub fn output_text(&self) -> String {
        self.output
            .iter()
            .map(|line| line.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// A line of output from a workflow command
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// When the line was received
    pub timestamp: Instant,
    /// Content of the line
    pub content: String,
    /// Which stream (stdout/stderr) it came from
    pub stream: OutputStream,
}

/// Output stream type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

impl OutputStream {
    /// Get indicator for UI
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Stdout => "",
            Self::Stderr => "!",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_tool_name() {
        assert_eq!(WorkflowCommandType::Specify.tool_name(), "speckit.specify");
        assert_eq!(WorkflowCommandType::Plan.tool_name(), "speckit.plan");
    }

    #[test]
    fn test_execution_state_transitions() {
        let mut cmd = WorkflowCommand::new(WorkflowCommandType::Plan, SpecId::new(1, "test"));

        assert!(cmd.state.is_pending());

        cmd.start();
        assert!(cmd.state.is_running());

        cmd.complete(0);
        assert!(cmd.state.is_finished());
    }

    #[test]
    fn test_output_collection() {
        let mut cmd = WorkflowCommand::new(WorkflowCommandType::Specify, SpecId::new(1, "test"));

        cmd.add_output("Line 1".to_string(), OutputStream::Stdout);
        cmd.add_output("Line 2".to_string(), OutputStream::Stdout);

        assert_eq!(cmd.output.len(), 2);
        assert_eq!(cmd.output_text(), "Line 1\nLine 2");
    }
}
