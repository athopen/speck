//! Process execution service for running workflow commands.
//!
//! Handles spawning processes, streaming output, and log persistence.

use crate::domain::{SpecId, WorkflowCommand, WorkflowCommandType};
use crate::error::{AppError, Result};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Output event from a running process
#[derive(Debug, Clone)]
pub enum ProcessOutput {
    /// Line from stdout
    Stdout(String),
    /// Line from stderr
    Stderr(String),
    /// Process exited with code
    Exit(i32),
    /// Process was killed/terminated
    Terminated,
    /// Error occurred
    Error(String),
}

/// Process handle for a running command
pub struct ProcessHandle {
    /// Child process
    child: Child,
    /// Start time
    start_time: Instant,
    /// Output receiver
    output_rx: mpsc::UnboundedReceiver<ProcessOutput>,
    /// Log file path
    log_file: Option<PathBuf>,
    /// Is running
    running: Arc<Mutex<bool>>,
}

impl ProcessHandle {
    /// Check if the process is still running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get log file path
    pub fn log_file(&self) -> Option<&PathBuf> {
        self.log_file.as_ref()
    }

    /// Try to receive next output (non-blocking)
    pub fn try_recv(&mut self) -> Option<ProcessOutput> {
        self.output_rx.try_recv().ok()
    }

    /// Kill the process
    pub fn kill(&mut self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        self.child
            .kill()
            .map_err(|e| AppError::Process(e.to_string()))?;
        Ok(())
    }

    /// Wait for the process to complete
    pub fn wait(&mut self) -> Result<i32> {
        let status = self
            .child
            .wait()
            .map_err(|e| AppError::Process(e.to_string()))?;
        *self.running.lock().unwrap() = false;
        Ok(status.code().unwrap_or(-1))
    }
}

/// Process execution service
pub struct ProcessService {
    /// Log directory
    log_dir: PathBuf,
}

impl ProcessService {
    /// Create a new process service
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    /// Ensure log directory exists
    fn ensure_log_dir(&self) -> Result<()> {
        if !self.log_dir.exists() {
            fs::create_dir_all(&self.log_dir).map_err(|e| AppError::io(e.to_string()))?;
        }
        Ok(())
    }

    /// Generate log file path for a command
    fn log_file_path(&self, command_type: WorkflowCommandType, spec_id: &str) -> PathBuf {
        let timestamp = chrono_lite_timestamp();
        let filename = format!("{}-{}-{}.log", spec_id, command_type.tool_name(), timestamp);
        self.log_dir.join(filename)
    }

    /// Spawn a workflow command process
    pub fn spawn_workflow(
        &self,
        command_type: WorkflowCommandType,
        spec_id: &str,
        spec_directory: &PathBuf,
        mcp_command: &str,
        mcp_args: &[String],
    ) -> Result<ProcessHandle> {
        self.ensure_log_dir()?;

        let log_file_path = self.log_file_path(command_type, spec_id);

        // Create log file
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)
            .map_err(|e| AppError::io(e.to_string()))?;

        // Write header to log
        writeln!(
            log_file,
            "# Workflow: {} for {}",
            command_type.tool_name(),
            spec_id
        )
        .map_err(|e| AppError::io(e.to_string()))?;
        writeln!(log_file, "# Started: {}", chrono_lite_timestamp())
            .map_err(|e| AppError::io(e.to_string()))?;
        writeln!(log_file, "# Directory: {}", spec_directory.display())
            .map_err(|e| AppError::io(e.to_string()))?;
        writeln!(log_file, "---").map_err(|e| AppError::io(e.to_string()))?;

        // Build command arguments
        let mut args = mcp_args.to_vec();
        args.push(command_type.tool_name().to_string());

        // Spawn the process
        let mut child = Command::new(mcp_command)
            .args(&args)
            .current_dir(spec_directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Process(format!("Failed to spawn process: {}", e)))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Create channel for output
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let running = Arc::new(Mutex::new(true));

        // Spawn thread to read stdout
        if let Some(stdout) = stdout {
            let tx = output_tx.clone();
            let running_clone = running.clone();
            let log_path = log_file_path.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if !*running_clone.lock().unwrap() {
                        break;
                    }
                    match line {
                        Ok(text) => {
                            // Append to log file
                            if let Ok(mut f) = OpenOptions::new().append(true).open(&log_path) {
                                let _ = writeln!(f, "[OUT] {}", text);
                            }
                            let _ = tx.send(ProcessOutput::Stdout(text));
                        }
                        Err(e) => {
                            let _ = tx.send(ProcessOutput::Error(e.to_string()));
                            break;
                        }
                    }
                }
            });
        }

        // Spawn thread to read stderr
        if let Some(stderr) = stderr {
            let tx = output_tx.clone();
            let running_clone = running.clone();
            let log_path = log_file_path.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if !*running_clone.lock().unwrap() {
                        break;
                    }
                    match line {
                        Ok(text) => {
                            // Append to log file
                            if let Ok(mut f) = OpenOptions::new().append(true).open(&log_path) {
                                let _ = writeln!(f, "[ERR] {}", text);
                            }
                            let _ = tx.send(ProcessOutput::Stderr(text));
                        }
                        Err(e) => {
                            let _ = tx.send(ProcessOutput::Error(e.to_string()));
                            break;
                        }
                    }
                }
            });
        }

        Ok(ProcessHandle {
            child,
            start_time: Instant::now(),
            output_rx,
            log_file: Some(log_file_path),
            running,
        })
    }

    /// Spawn a simple command
    pub fn spawn_command(
        &self,
        command: &str,
        args: &[String],
        working_dir: &PathBuf,
    ) -> Result<ProcessHandle> {
        let mut child = Command::new(command)
            .args(args)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Process(format!("Failed to spawn process: {}", e)))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let running = Arc::new(Mutex::new(true));

        // Spawn thread to read stdout
        if let Some(stdout) = stdout {
            let tx = output_tx.clone();
            let running_clone = running.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if !*running_clone.lock().unwrap() {
                        break;
                    }
                    match line {
                        Ok(text) => {
                            let _ = tx.send(ProcessOutput::Stdout(text));
                        }
                        Err(e) => {
                            let _ = tx.send(ProcessOutput::Error(e.to_string()));
                            break;
                        }
                    }
                }
            });
        }

        // Spawn thread to read stderr
        if let Some(stderr) = stderr {
            let tx = output_tx.clone();
            let running_clone = running.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if !*running_clone.lock().unwrap() {
                        break;
                    }
                    match line {
                        Ok(text) => {
                            let _ = tx.send(ProcessOutput::Stderr(text));
                        }
                        Err(e) => {
                            let _ = tx.send(ProcessOutput::Error(e.to_string()));
                            break;
                        }
                    }
                }
            });
        }

        Ok(ProcessHandle {
            child,
            start_time: Instant::now(),
            output_rx,
            log_file: None,
            running,
        })
    }
}

/// Generate a simple timestamp without chrono dependency
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{}", duration.as_secs())
}

/// Workflow command runner that integrates with the domain model
pub struct WorkflowRunner {
    process_service: ProcessService,
    mcp_command: String,
    mcp_args: Vec<String>,
}

impl WorkflowRunner {
    /// Create a new workflow runner
    pub fn new(log_dir: PathBuf, mcp_command: String, mcp_args: Vec<String>) -> Self {
        Self {
            process_service: ProcessService::new(log_dir),
            mcp_command,
            mcp_args,
        }
    }

    /// Create with default settings
    pub fn default_with_log_dir(log_dir: PathBuf) -> Self {
        Self::new(log_dir, "claude".to_string(), vec!["--mcp".to_string()])
    }

    /// Start a workflow command
    pub fn start_command(
        &self,
        command_type: WorkflowCommandType,
        spec_id: &str,
        spec_directory: &PathBuf,
    ) -> Result<(WorkflowCommand, ProcessHandle)> {
        let handle = self.process_service.spawn_workflow(
            command_type,
            spec_id,
            spec_directory,
            &self.mcp_command,
            &self.mcp_args,
        )?;

        // Parse spec_id into SpecId
        let parsed_spec_id = SpecId::parse(spec_id).unwrap_or_else(|_| SpecId::new(0, spec_id));

        let mut command = WorkflowCommand::new(command_type, parsed_spec_id);
        command.start();
        if let Some(log_path) = handle.log_file() {
            command.log_path = Some(log_path.clone());
        }

        Ok((command, handle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_file_path() {
        let temp = TempDir::new().unwrap();
        let service = ProcessService::new(temp.path().to_path_buf());

        let path = service.log_file_path(WorkflowCommandType::Specify, "001-test");
        assert!(path.to_string_lossy().contains("001-test"));
        assert!(path.to_string_lossy().contains("speckit.specify"));
        assert!(path.extension().unwrap() == "log");
    }

    #[test]
    fn test_chrono_lite_timestamp() {
        let ts = chrono_lite_timestamp();
        assert!(!ts.is_empty());
        // Should be a number (unix timestamp)
        assert!(ts.parse::<u64>().is_ok());
    }
}
