//! Application state and main event loop.

use crate::config::ProjectConfig;
use crate::domain::{
    ArtifactType, ExecutionState, Project, Specification, WorkflowCommand, WorkflowCommandType,
    Worktree, WorktreeStatus,
};
use crate::error::{AppError, Result};
use crate::services::{GitService, ProcessHandle, ProcessOutput, SpecService, WorkflowRunner};
use crate::ui::input::{Action, InputHandler, InputMode};
use crate::ui::widgets::editor::{EditorAction, EditorState};
use crate::ui::widgets::help::HelpViewState;
use crate::ui::widgets::output_panel::OutputBuffer;
use crate::ui::widgets::spec_detail::DocumentViewerState;
use crate::ui::widgets::text_input::{TextInputAction, TextInputState};
use crate::ui::widgets::worktree_list::WorktreeManagementState;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Application view state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppView {
    /// Main spec list overview
    Overview,
    /// Viewing spec details
    SpecDetail(String),
    /// Worktree management
    WorktreeManagement,
    /// Viewing a document
    DocumentView(DocType),
    /// Editing a document
    DocumentEdit(DocType),
    /// Workflow command output
    CommandOutput,
    /// Workflow command selection menu
    WorkflowMenu,
    /// Creating a new specification
    NewSpec,
    /// Help view showing keybindings
    Help,
}

impl Default for AppView {
    fn default() -> Self {
        Self::Overview
    }
}

/// Document type for viewing/editing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocType {
    Spec,
    Plan,
    Tasks,
    Research,
}

/// Main application state
pub struct App {
    /// Project context
    pub project: Project,
    /// Discovered specifications
    pub specs: Vec<Specification>,
    /// Known worktrees
    pub worktrees: Vec<Worktree>,
    /// Currently running workflow command
    pub active_command: Option<WorkflowCommand>,

    // UI State
    /// Current view
    pub view: AppView,
    /// Selected spec index in list
    pub selected_spec_index: usize,
    /// Scroll offset for long lists
    pub scroll_offset: usize,
    /// Current input mode
    pub input_mode: InputMode,

    // Async State
    /// Error message to display
    pub error_message: Option<String>,
    /// Loading state
    pub is_loading: bool,
    /// Loading message (what operation is in progress)
    pub loading_message: Option<String>,
    /// Worktree status cache (path -> status)
    pub worktree_statuses: std::collections::HashMap<PathBuf, WorktreeStatus>,

    // Services
    spec_service: SpecService,
    git_service: Option<GitService>,
    workflow_runner: Option<WorkflowRunner>,

    // Input handler
    input_handler: InputHandler,

    // Command execution state
    /// Process handle for running command
    process_handle: Option<ProcessHandle>,
    /// Output buffer for command output
    pub output_buffer: OutputBuffer,
    /// Selected workflow command index (for menu)
    pub selected_workflow_index: usize,
    /// Available workflow commands for selection
    pub available_workflows: Vec<WorkflowCommandType>,

    // Document viewing/editing state
    /// Current document content being viewed
    pub document_content: Option<String>,
    /// Document viewer state (scroll position etc.)
    pub document_viewer_state: DocumentViewerState,
    /// Editor state for document editing
    pub editor_state: EditorState,
    /// Currently selected document type for viewing
    pub current_doc_type: Option<DocType>,

    // Worktree management state
    /// State for worktree management view
    pub worktree_management_state: WorktreeManagementState,
    /// Sync status cache for worktrees
    pub worktree_sync_statuses:
        std::collections::HashMap<String, crate::domain::WorktreeSyncStatus>,

    // New spec creation state
    /// Text input state for new spec name
    pub new_spec_input: TextInputState,
    /// Error message for new spec creation
    pub new_spec_error: Option<String>,

    // Help view state
    /// State for help view (scroll position)
    pub help_view_state: HelpViewState,

    /// Should quit the application
    pub should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let config = ProjectConfig::load(Some(&project_root)).unwrap_or_default();
        let project = Project::new(project_root.clone(), config);

        let spec_service = SpecService::new(project.specs_directory.clone());

        let git_service =
            GitService::new(project_root.clone(), project.worktree_directory.clone()).ok();

        // Create workflow runner with log directory
        let log_dir = project_root.join(".spec-tui").join("logs");
        let workflow_runner = Some(WorkflowRunner::default_with_log_dir(log_dir));

        Ok(Self {
            project,
            specs: Vec::new(),
            worktrees: Vec::new(),
            active_command: None,
            view: AppView::Overview,
            selected_spec_index: 0,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            error_message: None,
            is_loading: false,
            loading_message: None,
            worktree_statuses: std::collections::HashMap::new(),
            spec_service,
            git_service,
            workflow_runner,
            input_handler: InputHandler::new(true), // vim navigation enabled
            process_handle: None,
            output_buffer: OutputBuffer::new(),
            selected_workflow_index: 0,
            available_workflows: Vec::new(),
            document_content: None,
            document_viewer_state: DocumentViewerState::new(),
            editor_state: EditorState::new(),
            current_doc_type: None,
            worktree_management_state: WorktreeManagementState::new(),
            worktree_sync_statuses: std::collections::HashMap::new(),
            new_spec_input: TextInputState::new(),
            new_spec_error: None,
            help_view_state: HelpViewState::new(),
            should_quit: false,
        })
    }

    /// Initialize the application (load initial data)
    pub fn init(&mut self) -> Result<()> {
        self.refresh_specs()?;
        self.refresh_worktrees();
        Ok(())
    }

    /// Refresh the specifications list
    pub fn refresh_specs(&mut self) -> Result<()> {
        self.is_loading = true;
        match self.spec_service.discover_specs() {
            Ok(specs) => {
                self.specs = specs;
                self.is_loading = false;
                // Adjust selection if needed
                if self.selected_spec_index >= self.specs.len() && !self.specs.is_empty() {
                    self.selected_spec_index = self.specs.len() - 1;
                }
            }
            Err(e) => {
                self.is_loading = false;
                self.error_message = Some(format!("Failed to load specs: {}", e));
            }
        }
        Ok(())
    }

    /// Refresh the worktrees list
    pub fn refresh_worktrees(&mut self) {
        if let Some(ref git) = self.git_service {
            match git.list_worktrees() {
                Ok(worktrees) => {
                    self.worktrees = worktrees;
                    self.refresh_worktree_statuses();
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load worktrees: {}", e));
                }
            }
        }
    }

    /// Refresh worktree status cache
    pub fn refresh_worktree_statuses(&mut self) {
        self.worktree_statuses.clear();
        if let Some(ref git) = self.git_service {
            for wt in &self.worktrees {
                if let Ok(status) = git.worktree_status(&wt.path) {
                    self.worktree_statuses.insert(wt.path.clone(), status);
                }
            }
        }
    }

    /// Get worktree status for a path
    pub fn get_worktree_status(&self, path: &PathBuf) -> Option<&WorktreeStatus> {
        self.worktree_statuses.get(path)
    }

    /// Find worktree for a spec (by branch name)
    pub fn find_worktree_for_spec(&self, spec: &Specification) -> Option<&Worktree> {
        self.worktrees.iter().find(|w| {
            w.branch == spec.branch
                || w.branch.contains(&spec.branch)
                || spec.branch.contains(&w.branch)
        })
    }

    /// Switch to a spec's worktree, creating it if necessary
    pub fn switch_to_spec(&mut self) -> std::result::Result<Option<PathBuf>, String> {
        let spec = match self.selected_spec() {
            Some(s) => s.clone(),
            None => return Ok(None),
        };

        let git = match &self.git_service {
            Some(g) => g,
            None => return Err("Git service not available".to_string()),
        };

        // Check if worktree already exists for this spec
        if let Some(wt) = self.find_worktree_for_spec(&spec) {
            // Worktree exists, return the path
            return Ok(Some(wt.path.clone()));
        }

        // Need to create a worktree
        // First check if the branch exists
        let branch = &spec.branch;
        match git.branch_exists(branch) {
            Ok(true) => {}
            Ok(false) => {
                // Try to create the branch
                if let Err(e) = git.create_branch(branch, None) {
                    return Err(format!(
                        "Branch '{}' doesn't exist and couldn't create it: {}",
                        branch, e
                    ));
                }
            }
            Err(e) => return Err(format!("Failed to check branch: {}", e)),
        }

        // Create the worktree path
        let worktree_path = self.project.worktree_path_for_branch(branch);

        // Create the worktree
        self.is_loading = true;
        self.loading_message = Some(format!("Creating worktree for {}...", branch));

        match git.create_worktree(branch, &worktree_path) {
            Ok(_wt) => {
                self.is_loading = false;
                self.loading_message = None;
                // Refresh worktrees list
                self.refresh_worktrees();
                Ok(Some(worktree_path))
            }
            Err(e) => {
                self.is_loading = false;
                self.loading_message = None;
                Err(format!("Failed to create worktree: {}", e))
            }
        }
    }

    /// Get the currently selected specification
    pub fn selected_spec(&self) -> Option<&Specification> {
        self.specs.get(self.selected_spec_index)
    }

    /// Get available workflow commands for the selected spec
    pub fn get_available_workflows(&self) -> Vec<WorkflowCommandType> {
        match self.selected_spec() {
            Some(spec) => spec.phase.available_commands(),
            None => Vec::new(),
        }
    }

    /// Open the workflow command menu
    pub fn open_workflow_menu(&mut self) {
        self.available_workflows = self.get_available_workflows();
        if self.available_workflows.is_empty() {
            self.error_message = Some("No workflow commands available for this phase".to_string());
            return;
        }
        self.selected_workflow_index = 0;
        self.view = AppView::WorkflowMenu;
    }

    /// Run the selected workflow command
    pub fn run_selected_workflow(&mut self) -> std::result::Result<(), String> {
        let spec = match self.selected_spec() {
            Some(s) => s.clone(),
            None => return Err("No spec selected".to_string()),
        };

        let command_type = match self.available_workflows.get(self.selected_workflow_index) {
            Some(t) => *t,
            None => return Err("No workflow command selected".to_string()),
        };

        self.run_workflow(command_type, &spec)
    }

    /// Run a workflow command
    pub fn run_workflow(
        &mut self,
        command_type: WorkflowCommandType,
        spec: &Specification,
    ) -> std::result::Result<(), String> {
        // Check if a command is already running
        if self.is_command_running() {
            return Err("A command is already running".to_string());
        }

        let runner = match &self.workflow_runner {
            Some(r) => r,
            None => return Err("Workflow runner not available".to_string()),
        };

        // Start the command
        let (command, handle) = runner
            .start_command(command_type, spec.id.as_str(), &spec.directory)
            .map_err(|e| format!("Failed to start command: {}", e))?;

        self.active_command = Some(command);
        self.process_handle = Some(handle);
        self.output_buffer.start();
        self.view = AppView::CommandOutput;

        Ok(())
    }

    /// Check if a command is currently running
    pub fn is_command_running(&self) -> bool {
        self.process_handle
            .as_ref()
            .map_or(false, |h| h.is_running())
    }

    /// Cancel the running command
    pub fn cancel_command(&mut self) {
        if let Some(ref mut handle) = self.process_handle {
            let _ = handle.kill();
        }

        if let Some(ref mut cmd) = self.active_command {
            cmd.state = ExecutionState::Cancelled;
        }

        self.output_buffer
            .push_stderr("Command cancelled by user".to_string());
    }

    /// Poll process output (call this in the event loop)
    pub fn poll_process_output(&mut self) {
        if let Some(ref mut handle) = self.process_handle {
            // Drain all available output
            while let Some(output) = handle.try_recv() {
                match output {
                    ProcessOutput::Stdout(line) => {
                        self.output_buffer.push_stdout(line);
                    }
                    ProcessOutput::Stderr(line) => {
                        self.output_buffer.push_stderr(line);
                    }
                    ProcessOutput::Exit(code) => {
                        if let Some(ref mut cmd) = self.active_command {
                            cmd.complete(code);
                        }
                        self.output_buffer
                            .push_stdout(format!("Process exited with code {}", code));
                    }
                    ProcessOutput::Terminated => {
                        if let Some(ref mut cmd) = self.active_command {
                            cmd.cancel();
                        }
                    }
                    ProcessOutput::Error(e) => {
                        self.output_buffer.push_stderr(format!("Error: {}", e));
                        if let Some(ref mut cmd) = self.active_command {
                            cmd.fail(e);
                        }
                    }
                }
            }
        }
    }

    /// Open a document for viewing
    pub fn open_document_view(&mut self, doc_type: DocType) -> std::result::Result<(), String> {
        let spec = match self.selected_spec() {
            Some(s) => s.clone(),
            None => return Err("No spec selected".to_string()),
        };

        let artifact_type = match doc_type {
            DocType::Spec => ArtifactType::Spec,
            DocType::Plan => ArtifactType::Plan,
            DocType::Tasks => ArtifactType::Tasks,
            DocType::Research => ArtifactType::Research,
        };

        // Check if artifact exists
        let has_artifact = match doc_type {
            DocType::Spec => spec.artifacts.has_spec,
            DocType::Plan => spec.artifacts.has_plan,
            DocType::Tasks => spec.artifacts.has_tasks,
            DocType::Research => spec.artifacts.has_research,
        };

        if !has_artifact {
            return Err(format!(
                "{} not found for {}",
                artifact_type.filename(),
                spec.id
            ));
        }

        // Read the artifact
        match self.spec_service.read_artifact(&spec.id, artifact_type) {
            Ok(content) => {
                self.document_content = Some(content.clone());
                self.document_viewer_state = DocumentViewerState::new();
                self.document_viewer_state
                    .set_total_lines(content.lines().count());
                self.current_doc_type = Some(doc_type);
                self.view = AppView::DocumentView(doc_type);
                Ok(())
            }
            Err(e) => Err(format!("Failed to read document: {}", e)),
        }
    }

    /// Open a document for editing
    pub fn open_document_edit(&mut self, doc_type: DocType) -> std::result::Result<(), String> {
        let spec = match self.selected_spec() {
            Some(s) => s.clone(),
            None => return Err("No spec selected".to_string()),
        };

        let artifact_type = match doc_type {
            DocType::Spec => ArtifactType::Spec,
            DocType::Plan => ArtifactType::Plan,
            DocType::Tasks => ArtifactType::Tasks,
            DocType::Research => ArtifactType::Research,
        };

        // Read the artifact (or create empty if it doesn't exist)
        let content = match self.spec_service.read_artifact(&spec.id, artifact_type) {
            Ok(content) => content,
            Err(_) => String::new(),
        };

        let file_path = spec.directory.join(artifact_type.filename());
        let title = format!("{} - {}", artifact_type.filename(), spec.id);

        self.editor_state.open(content, title, file_path);
        self.current_doc_type = Some(doc_type);
        self.input_mode = InputMode::Insert;
        self.view = AppView::DocumentEdit(doc_type);
        Ok(())
    }

    /// Save the current document being edited
    pub fn save_document(&mut self) -> std::result::Result<(), String> {
        let spec = match self.selected_spec() {
            Some(s) => s.clone(),
            None => return Err("No spec selected".to_string()),
        };

        let doc_type = match self.current_doc_type {
            Some(dt) => dt,
            None => return Err("No document type set".to_string()),
        };

        let content = match self.editor_state.content() {
            Some(c) => c,
            None => return Err("No content to save".to_string()),
        };

        let artifact_type = match doc_type {
            DocType::Spec => ArtifactType::Spec,
            DocType::Plan => ArtifactType::Plan,
            DocType::Tasks => ArtifactType::Tasks,
            DocType::Research => ArtifactType::Research,
        };

        match self
            .spec_service
            .write_artifact(&spec.id, artifact_type, &content)
        {
            Ok(()) => {
                self.editor_state.mark_saved();
                self.loading_message = Some("Document saved".to_string());
                // Refresh specs to update artifact status
                let _ = self.refresh_specs();
                Ok(())
            }
            Err(e) => Err(format!("Failed to save: {}", e)),
        }
    }

    /// Close the document viewer
    pub fn close_document_view(&mut self) {
        self.document_content = None;
        self.current_doc_type = None;
        self.view = AppView::Overview;
    }

    /// Close the document editor
    pub fn close_document_edit(&mut self) {
        self.editor_state.close();
        self.current_doc_type = None;
        self.input_mode = InputMode::Normal;
        self.view = AppView::Overview;
    }

    /// Open worktree management view
    pub fn open_worktree_management(&mut self) {
        self.worktree_management_state = WorktreeManagementState::new();
        self.refresh_worktree_sync_statuses();
        self.view = AppView::WorktreeManagement;
    }

    /// Refresh worktree sync statuses
    pub fn refresh_worktree_sync_statuses(&mut self) {
        self.worktree_sync_statuses.clear();
        if let Some(ref git) = self.git_service {
            for wt in &self.worktrees {
                if let Ok(sync_status) = git.sync_status(&wt.branch) {
                    self.worktree_sync_statuses
                        .insert(wt.branch.clone(), sync_status);
                }
            }
        }
    }

    /// Get the selected worktree in management view
    pub fn selected_worktree(&self) -> Option<&Worktree> {
        self.worktrees
            .get(self.worktree_management_state.selected_index)
    }

    /// Request deletion of the selected worktree
    pub fn request_worktree_delete(&mut self) {
        if let Some(wt) = self.selected_worktree() {
            if wt.is_main {
                self.error_message = Some("Cannot delete the main worktree".to_string());
                return;
            }
            self.worktree_management_state
                .request_delete(wt.path.clone());
        }
    }

    /// Perform worktree deletion (called after confirmation)
    pub fn delete_worktree(&mut self, force: bool) -> std::result::Result<(), String> {
        let path = match self.worktree_management_state.pending_delete.take() {
            Some(p) => p,
            None => return Err("No worktree pending deletion".to_string()),
        };

        let git = match &self.git_service {
            Some(g) => g,
            None => return Err("Git service not available".to_string()),
        };

        match git.delete_worktree(&path, force) {
            Ok(()) => {
                self.loading_message = Some("Worktree deleted".to_string());
                self.refresh_worktrees();
                // Adjust selection if needed
                if self.worktree_management_state.selected_index >= self.worktrees.len()
                    && !self.worktrees.is_empty()
                {
                    self.worktree_management_state.selected_index = self.worktrees.len() - 1;
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to delete worktree: {}", e)),
        }
    }

    /// Close worktree management view
    pub fn close_worktree_management(&mut self) {
        self.worktree_management_state = WorktreeManagementState::new();
        self.view = AppView::Overview;
    }

    /// Open help view
    pub fn open_help(&mut self) {
        self.help_view_state = HelpViewState::new();
        self.view = AppView::Help;
    }

    /// Close help view
    pub fn close_help(&mut self) {
        self.view = AppView::Overview;
    }

    /// Get available document types for the selected spec
    pub fn get_available_documents(&self) -> Vec<DocType> {
        match self.selected_spec() {
            Some(spec) => {
                let mut docs = Vec::new();
                if spec.artifacts.has_spec {
                    docs.push(DocType::Spec);
                }
                if spec.artifacts.has_plan {
                    docs.push(DocType::Plan);
                }
                if spec.artifacts.has_tasks {
                    docs.push(DocType::Tasks);
                }
                if spec.artifacts.has_research {
                    docs.push(DocType::Research);
                }
                docs
            }
            None => Vec::new(),
        }
    }

    /// Open the new spec creation dialog
    pub fn open_new_spec_dialog(&mut self) {
        self.new_spec_input = TextInputState::new();
        self.new_spec_error = None;
        self.input_mode = InputMode::Insert;
        self.view = AppView::NewSpec;
    }

    /// Cancel new spec creation
    pub fn cancel_new_spec(&mut self) {
        self.new_spec_input.clear();
        self.new_spec_error = None;
        self.input_mode = InputMode::Normal;
        self.view = AppView::Overview;
    }

    /// Create a new specification
    pub fn create_new_spec(&mut self) -> std::result::Result<(), String> {
        let name = self.new_spec_input.value().trim().to_string();

        // Validate name
        if name.is_empty() {
            self.new_spec_error = Some("Name cannot be empty".to_string());
            return Err("Name cannot be empty".to_string());
        }

        // Convert to kebab-case (lowercase, replace spaces with hyphens)
        let kebab_name: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if kebab_name.is_empty() {
            self.new_spec_error = Some("Invalid name".to_string());
            return Err("Invalid name".to_string());
        }

        // Get the next spec number
        let number = match self.spec_service.next_number() {
            Ok(n) => n,
            Err(e) => {
                self.new_spec_error = Some(format!("Failed to get next number: {}", e));
                return Err(format!("Failed to get next number: {}", e));
            }
        };

        // Create the spec
        let spec = match self.spec_service.create_spec(number, &kebab_name) {
            Ok(s) => s,
            Err(e) => {
                self.new_spec_error = Some(format!("Failed to create spec: {}", e));
                return Err(format!("Failed to create spec: {}", e));
            }
        };

        // Create the branch if git service is available
        if let Some(ref git) = self.git_service {
            let branch_name = spec.branch.clone();
            if let Err(e) = git.create_branch(&branch_name, None) {
                // Branch might already exist, which is OK
                tracing::warn!("Could not create branch {}: {}", branch_name, e);
            }
        }

        // Refresh specs list
        let _ = self.refresh_specs();

        // Find and select the new spec
        if let Some(idx) = self
            .specs
            .iter()
            .position(|s| s.id.as_str() == spec.id.as_str())
        {
            self.selected_spec_index = idx;
        }

        // Close the dialog
        self.new_spec_input.clear();
        self.new_spec_error = None;
        self.input_mode = InputMode::Normal;
        self.view = AppView::Overview;

        // Show success message
        self.loading_message = Some(format!("Created spec: {}", spec.id));

        // Optionally auto-switch to the new worktree
        if self.git_service.is_some() {
            match self.switch_to_spec() {
                Ok(Some(path)) => {
                    self.loading_message =
                        Some(format!("Created and switched to: {}", path.display()));
                }
                Ok(None) => {}
                Err(e) => {
                    // Don't fail the whole operation, just warn
                    tracing::warn!("Could not auto-switch to new spec: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Select previous workflow command in menu
    pub fn select_previous_workflow(&mut self) {
        if self.selected_workflow_index > 0 {
            self.selected_workflow_index -= 1;
        }
    }

    /// Select next workflow command in menu
    pub fn select_next_workflow(&mut self) {
        if self.selected_workflow_index < self.available_workflows.len().saturating_sub(1) {
            self.selected_workflow_index += 1;
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_spec_index > 0 {
            self.selected_spec_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_spec_index < self.specs.len().saturating_sub(1) {
            self.selected_spec_index += 1;
        }
    }

    /// Handle keyboard input and return true if should quit
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Clear error on any key press
        self.error_message = None;

        // Handle view-specific keys first
        match &self.view {
            AppView::Overview => {
                // Handle quit keys in overview
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    return true;
                }
            }
            AppView::WorkflowMenu => {
                return self.handle_workflow_menu_key(key);
            }
            AppView::CommandOutput => {
                return self.handle_command_output_key(key);
            }
            AppView::DocumentView(_) => {
                return self.handle_document_view_key(key);
            }
            AppView::DocumentEdit(_) => {
                return self.handle_document_edit_key(key);
            }
            AppView::WorktreeManagement => {
                return self.handle_worktree_management_key(key);
            }
            AppView::NewSpec => {
                return self.handle_new_spec_key(key);
            }
            AppView::Help => {
                return self.handle_help_key(key);
            }
            _ => {}
        }

        // Process action from input handler
        if let Some(action) = self.input_handler.handle_key(key, self.input_mode) {
            match action {
                Action::MoveUp => self.select_previous(),
                Action::MoveDown => self.select_next(),
                Action::Select => {
                    // Enter spec detail view
                    if let Some(spec) = self.selected_spec() {
                        self.view = AppView::SpecDetail(spec.id.as_str().to_string());
                    }
                }
                Action::Back => {
                    if self.view != AppView::Overview {
                        self.view = AppView::Overview;
                    } else {
                        return true; // Quit
                    }
                }
                Action::SwitchWorktree => match self.switch_to_spec() {
                    Ok(Some(path)) => {
                        self.error_message = None;
                        self.loading_message = Some(format!("Switched to: {}", path.display()));
                    }
                    Ok(None) => {}
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                },
                Action::ManageWorktrees => {
                    self.open_worktree_management();
                }
                Action::RunWorkflow => {
                    self.open_workflow_menu();
                }
                Action::ViewDocument => {
                    // Open document selection or default to spec.md
                    let docs = self.get_available_documents();
                    if docs.is_empty() {
                        self.error_message =
                            Some("No documents available for this spec".to_string());
                    } else {
                        // Default to first available document
                        if let Err(e) = self.open_document_view(docs[0]) {
                            self.error_message = Some(e);
                        }
                    }
                }
                Action::EditDocument => {
                    // Open editor for spec.md by default
                    if let Err(e) = self.open_document_edit(DocType::Spec) {
                        self.error_message = Some(e);
                    }
                }
                Action::NewSpec => {
                    self.open_new_spec_dialog();
                }
                Action::CancelCommand => {
                    if self.is_command_running() {
                        self.cancel_command();
                    }
                }
                Action::Refresh => {
                    let _ = self.refresh_specs();
                    self.refresh_worktrees();
                }
                Action::Help => {
                    self.open_help();
                }
                Action::Quit => return true,
                _ => {}
            }
        }

        false
    }

    /// Handle keys in help view
    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.close_help();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.help_view_state.scroll_up(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.help_view_state.scroll_down(1);
            }
            KeyCode::PageUp | KeyCode::Char('b') => {
                self.help_view_state.page_up();
            }
            KeyCode::PageDown | KeyCode::Char('f') => {
                self.help_view_state.page_down();
            }
            _ => {}
        }
        false
    }

    /// Handle keys in new spec dialog
    fn handle_new_spec_key(&mut self, key: KeyEvent) -> bool {
        match self.new_spec_input.handle_key(key) {
            TextInputAction::Submit => {
                // Try to create the spec
                match self.create_new_spec() {
                    Ok(()) => {}
                    Err(_) => {
                        // Error is stored in new_spec_error
                    }
                }
            }
            TextInputAction::Cancel => {
                self.cancel_new_spec();
            }
            TextInputAction::Changed | TextInputAction::None => {
                // Clear error on any change
                if matches!(
                    key.code,
                    KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
                ) {
                    self.new_spec_error = None;
                }
            }
        }
        false
    }

    /// Handle keys in document view
    fn handle_document_view_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close_document_view();
            }
            KeyCode::Char('e') => {
                // Switch to edit mode
                if let Some(doc_type) = self.current_doc_type {
                    if let Err(e) = self.open_document_edit(doc_type) {
                        self.error_message = Some(e);
                    }
                }
            }
            // Scroll navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.document_viewer_state.scroll_up(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.document_viewer_state.scroll_down(1);
            }
            KeyCode::PageUp | KeyCode::Char('b') => {
                self.document_viewer_state.page_up();
            }
            KeyCode::PageDown | KeyCode::Char('f') => {
                self.document_viewer_state.page_down();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.document_viewer_state.scroll_to_top();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.document_viewer_state.scroll_to_bottom();
            }
            // Switch between documents
            KeyCode::Char('1') => {
                if let Err(e) = self.open_document_view(DocType::Spec) {
                    self.error_message = Some(e);
                }
            }
            KeyCode::Char('2') => {
                if let Err(e) = self.open_document_view(DocType::Plan) {
                    self.error_message = Some(e);
                }
            }
            KeyCode::Char('3') => {
                if let Err(e) = self.open_document_view(DocType::Tasks) {
                    self.error_message = Some(e);
                }
            }
            KeyCode::Char('4') => {
                if let Err(e) = self.open_document_view(DocType::Research) {
                    self.error_message = Some(e);
                }
            }
            _ => {}
        }
        false
    }

    /// Handle keys in worktree management view
    fn handle_worktree_management_key(&mut self, key: KeyEvent) -> bool {
        // Handle confirmation dialog if showing
        if self.worktree_management_state.showing_confirm {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    self.worktree_management_state.confirm_yes_selected = true;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.worktree_management_state.confirm_yes_selected = false;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(_path) = self.worktree_management_state.confirm_delete() {
                        match self.delete_worktree(false) {
                            Ok(()) => {}
                            Err(e) => {
                                self.error_message = Some(e);
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.worktree_management_state.cancel_confirm();
                }
                KeyCode::Char('y') => {
                    self.worktree_management_state.confirm_yes_selected = true;
                    if let Some(_path) = self.worktree_management_state.confirm_delete() {
                        match self.delete_worktree(false) {
                            Ok(()) => {}
                            Err(e) => {
                                self.error_message = Some(e);
                            }
                        }
                    }
                }
                _ => {}
            }
            return false;
        }

        // Normal worktree management navigation
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close_worktree_management();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.worktree_management_state
                    .select_previous(self.worktrees.len());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.worktree_management_state
                    .select_next(self.worktrees.len());
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                self.request_worktree_delete();
            }
            KeyCode::Char('D') => {
                // Force delete (with shift)
                if let Some(wt) = self.selected_worktree() {
                    if wt.is_main {
                        self.error_message = Some("Cannot delete the main worktree".to_string());
                    } else {
                        self.worktree_management_state.pending_delete = Some(wt.path.clone());
                        match self.delete_worktree(true) {
                            Ok(()) => {}
                            Err(e) => {
                                self.error_message = Some(e);
                            }
                        }
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Switch to selected worktree
                if let Some(wt) = self.selected_worktree() {
                    let path = wt.path.clone();
                    self.loading_message = Some(format!("Active worktree: {}", path.display()));
                    self.close_worktree_management();
                }
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                // Refresh worktrees
                self.refresh_worktrees();
                self.refresh_worktree_sync_statuses();
            }
            _ => {}
        }
        false
    }

    /// Handle keys in document edit mode
    fn handle_document_edit_key(&mut self, key: KeyEvent) -> bool {
        // Check for save shortcut (Ctrl+S)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            match self.save_document() {
                Ok(()) => {}
                Err(e) => {
                    self.error_message = Some(e);
                }
            }
            return false;
        }

        // Check for quit/cancel (Esc)
        if key.code == KeyCode::Esc {
            // Check if modified
            if self.editor_state.is_modified() {
                // For now, just close without saving
                // Could add confirmation dialog later
                self.close_document_edit();
            } else {
                self.close_document_edit();
            }
            return false;
        }

        // Forward other keys to the editor
        if let Some(ref mut editor) = self.editor_state.editor_mut() {
            match editor.handle_key(key) {
                EditorAction::Save => match self.save_document() {
                    Ok(()) => {}
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                },
                EditorAction::Quit => {
                    self.close_document_edit();
                }
                EditorAction::None => {}
            }
        }

        false
    }

    /// Handle keys in workflow menu
    fn handle_workflow_menu_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.view = AppView::Overview;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous_workflow();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next_workflow();
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.run_selected_workflow() {
                Ok(()) => {}
                Err(e) => {
                    self.error_message = Some(e);
                    self.view = AppView::Overview;
                }
            },
            _ => {}
        }
        false
    }

    /// Handle keys in command output view
    fn handle_command_output_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Only allow leaving if command is not running
                if !self.is_command_running() {
                    self.view = AppView::Overview;
                }
            }
            KeyCode::Char('c') => {
                // Cancel running command
                if self.is_command_running() {
                    self.cancel_command();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.output_buffer.scroll_up(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.output_buffer.scroll_down(1, 20); // Assume 20 visible lines
            }
            KeyCode::PageUp => {
                self.output_buffer.scroll_up(10);
            }
            KeyCode::PageDown => {
                self.output_buffer.scroll_down(10, 20);
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.output_buffer.scroll_to_bottom();
            }
            _ => {}
        }
        false
    }

    /// Main event loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let tick_rate = Duration::from_millis(self.project.config.ui.refresh_rate_ms);
        let mut last_tick = Instant::now();

        // Initial load
        self.init()?;

        loop {
            // Poll process output if a command is running
            self.poll_process_output();

            // Draw UI
            terminal.draw(|f| crate::ui::layout::draw(f, self))?;

            // Calculate timeout - use shorter timeout when command is running for responsiveness
            let timeout = if self.is_command_running() {
                Duration::from_millis(50)
            } else {
                tick_rate.saturating_sub(last_tick.elapsed())
            };

            // Wait for event with timeout
            if event::poll(timeout).map_err(|e| AppError::Terminal(e.to_string()))? {
                match event::read().map_err(|e| AppError::Terminal(e.to_string()))? {
                    Event::Key(key) => {
                        if self.handle_key(key) {
                            break;
                        }
                    }
                    Event::Resize(_width, _height) => {
                        // Terminal was resized - the next draw will automatically
                        // use the new dimensions from frame.area()
                        // We could update any cached dimensions here if needed
                        tracing::debug!("Terminal resized to {}x{}", _width, _height);
                    }
                    Event::FocusGained | Event::FocusLost => {
                        // Ignore focus events
                    }
                    Event::Mouse(_) | Event::Paste(_) => {
                        // Ignore mouse and paste events for now
                    }
                }
            }

            // Tick
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
                // Could do periodic updates here
            }
        }

        // Cleanup: cancel any running command
        if self.is_command_running() {
            self.cancel_command();
        }

        Ok(())
    }
}
