# Feature Specification: Spec-Driven Development TUI with Git Worktree Management

**Feature Branch**: `001-spec-tui-worktree`
**Created**: 2026-01-20
**Status**: Draft
**Input**: User description: "Terminal UI for spec-driven development workflow similar to spec-kit, with git worktree integration for managing multiple feature specifications simultaneously"

## Clarifications

### Session 2026-01-20

- Q: How should the TUI invoke AI agents for workflow commands? → A: MCP (Model Context Protocol) client - implement the industry-standard protocol for vendor-agnostic agent communication via JSON-RPC 2.0.
- Q: How should multiple concurrent TUI instances behave? → A: Full concurrent access - all operations allowed from all instances; user responsible for managing conflicts.
- Q: How should the system determine a spec's current workflow phase? → A: File existence heuristic - derive phase from which artifact files exist (spec.md, plan.md, tasks.md).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Specs Overview (Priority: P1)

As a developer, I want to see a dashboard overview of all feature specifications in my project so that I can quickly understand the current state of all features being developed and navigate to any specific one.

**Why this priority**: This is the foundation of the TUI - without the ability to see and navigate specs, no other functionality can be used. Provides immediate value by giving developers visibility into their project's specification landscape.

**Independent Test**: Can be fully tested by launching the TUI in a project with multiple specs directories and verifying that all specs are listed with their current status.

**Acceptance Scenarios**:

1. **Given** a project with multiple specification directories (e.g., `specs/001-feature-a/`, `specs/002-feature-b/`), **When** the user launches the TUI, **Then** they see a list of all specifications with their names, branch status, and current workflow phase.
2. **Given** the specs overview is displayed, **When** a spec is currently being worked on in a worktree, **Then** that spec shows a visual indicator distinguishing it from inactive specs.
3. **Given** the overview displays multiple specs, **When** the user uses keyboard navigation (arrow keys or vim-style j/k), **Then** they can highlight and select different specs.

---

### User Story 2 - Switch Active Spec via Worktree (Priority: P1)

As a developer working on multiple features, I want to switch between active specifications using git worktrees so that I can context-switch between features without losing work or needing to stash/commit incomplete changes.

**Why this priority**: This is the core differentiating feature that enables parallel feature development. Critical for the workflow efficiency promise of the tool.

**Independent Test**: Can be tested by selecting a different spec from the overview and verifying that the TUI transitions to that spec's worktree directory, with the worktree created automatically if it doesn't exist.

**Acceptance Scenarios**:

1. **Given** a spec is selected from the overview, **When** the user activates the "switch to spec" action and no worktree exists for that spec's branch, **Then** the system creates a new worktree in a designated location and navigates to it.
2. **Given** a worktree already exists for the selected spec, **When** the user activates the switch action, **Then** the system navigates to the existing worktree without creating a duplicate.
3. **Given** the user is in a worktree for spec A, **When** they switch to spec B, **Then** they can return to spec A later with all their work preserved exactly as they left it.
4. **Given** the user attempts to switch to a spec, **When** the worktree creation fails (e.g., branch doesn't exist remotely, disk space issue), **Then** the system displays a clear error message explaining the failure.

---

### User Story 3 - Trigger Spec-Kit Workflow Commands (Priority: P1)

As a developer, I want to trigger spec-driven development workflow commands (specify, clarify, plan, tasks, implement) from the TUI so that I can progress through the development workflow without leaving the terminal interface.

**Why this priority**: This delivers the core functionality of spec-kit in a TUI form, making it the primary interaction mechanism rather than slash commands in an AI coding assistant.

**Independent Test**: Can be tested by selecting a workflow command (e.g., "specify") and verifying that the appropriate AI agent interaction is initiated and results are displayed/saved.

**Acceptance Scenarios**:

1. **Given** a spec is active in the TUI, **When** the user triggers the "specify" command, **Then** the system initiates the specification workflow and shows progress/results in the TUI.
2. **Given** a spec has a completed specification, **When** the user triggers the "plan" command, **Then** the system generates an implementation plan and displays it.
3. **Given** a workflow command is running, **When** the AI agent produces output, **Then** the output is streamed to a dedicated panel in the TUI in real-time.
4. **Given** a workflow command is running, **When** the user presses a cancel key combination, **Then** the command is interrupted gracefully with partial results preserved.

---

### User Story 4 - View and Edit Specification Documents (Priority: P2)

As a developer, I want to view and make quick edits to specification documents (spec.md, plan.md, tasks.md) directly within the TUI so that I can review and refine specs without opening a separate editor.

**Why this priority**: Enhances workflow by keeping the user in the TUI, but users can fall back to external editors if needed.

**Independent Test**: Can be tested by opening a spec document in the TUI viewer, making an edit, and verifying the change persists to the file.

**Acceptance Scenarios**:

1. **Given** a spec is selected, **When** the user requests to view its specification document, **Then** the document content is displayed in a scrollable, syntax-highlighted panel.
2. **Given** a document is displayed, **When** the user enters edit mode, **Then** they can modify the text using standard text editing controls.
3. **Given** changes have been made in edit mode, **When** the user saves, **Then** the changes are written to the file and the view updates to reflect the saved content.

---

### User Story 5 - Manage Worktrees (Priority: P2)

As a developer, I want to see and manage all git worktrees from the TUI so that I can clean up old worktrees, see disk usage, and understand which worktrees are associated with which specs.

**Why this priority**: Important for maintenance and avoiding worktree sprawl, but not critical for basic workflow.

**Independent Test**: Can be tested by viewing the worktree list, selecting an old worktree, and removing it, then verifying it no longer appears in git worktree list.

**Acceptance Scenarios**:

1. **Given** the user navigates to the worktree management view, **When** there are active worktrees, **Then** they see a list showing worktree path, associated branch, and status.
2. **Given** a worktree is selected, **When** the user requests deletion, **Then** the system prompts for confirmation before removing the worktree.
3. **Given** a worktree is in use by the current session, **When** the user attempts to delete it, **Then** the system prevents deletion and explains why.

---

### User Story 6 - Create New Feature Specification (Priority: P2)

As a developer, I want to create a new feature specification from the TUI so that I can start new features without leaving the interface.

**Why this priority**: Essential for starting new work, but can be done via external commands initially.

**Independent Test**: Can be tested by triggering new spec creation, providing a feature description, and verifying a new spec directory and branch are created.

**Acceptance Scenarios**:

1. **Given** the user is in the specs overview, **When** they trigger "new spec" action, **Then** they are prompted to enter a feature description.
2. **Given** a feature description is provided, **When** confirmed, **Then** a new spec branch is created, a worktree is set up, and the spec directory is initialized with template files.
3. **Given** the new spec is created, **When** the user returns to the overview, **Then** the new spec appears in the list.

---

### Edge Cases

- What happens when the user tries to switch to a spec whose branch has been deleted remotely?
- How does the system handle specs with merge conflicts in their worktrees?
- What happens when disk space is insufficient to create a new worktree?
- How does the TUI behave when the terminal is resized during a long-running operation?
- What happens when multiple TUI instances are launched in the same project? → Full concurrent access is allowed; users are responsible for coordinating operations across instances.
- How does the system handle specs directories that exist but have no corresponding git branch?
- What happens when an AI workflow command times out or loses network connection?

## Requirements *(mandatory)*

### Functional Requirements

**Core TUI Requirements**

- **FR-001**: System MUST display a navigable list of all specification directories found in the project's `specs/` folder.
- **FR-002**: System MUST show the current status of each spec (branch exists, worktree active, workflow phase determined by file existence: no spec.md → specify, spec.md only → clarify/plan, spec.md + plan.md → tasks, spec.md + plan.md + tasks.md → implement).
- **FR-003**: System MUST support keyboard-based navigation using both arrow keys and vim-style bindings (h/j/k/l).
- **FR-004**: System MUST provide visual feedback when operations are in progress (loading indicators, progress bars).
- **FR-005**: System MUST remain responsive during long-running operations by executing them asynchronously.

**Git Worktree Requirements**

- **FR-006**: System MUST create git worktrees in a configurable directory location (default: `.worktrees/` in the project root).
- **FR-007**: System MUST associate each worktree with its corresponding spec branch using the pattern `{number}-{short-name}`.
- **FR-008**: System MUST detect and display existing worktrees and their status (clean, dirty, ahead/behind remote).
- **FR-009**: System MUST prevent duplicate worktrees for the same branch.
- **FR-010**: System MUST provide worktree cleanup functionality with confirmation prompts.

**Workflow Command Requirements**

- **FR-011**: System MUST support triggering the following workflow commands: specify, clarify, plan, tasks, implement.
- **FR-012**: System MUST display real-time output from workflow commands in a dedicated panel.
- **FR-013**: System MUST allow cancellation of running workflow commands.
- **FR-014**: System MUST persist workflow command output to log files for later review.
- **FR-015**: System MUST validate that prerequisites are met before running workflow commands (e.g., spec.md exists before running "plan").

**Document Viewing/Editing Requirements**

- **FR-016**: System MUST display markdown documents with syntax highlighting.
- **FR-017**: System MUST support scrolling through documents longer than the terminal height.
- **FR-018**: System MUST support basic text editing operations (insert, delete, cut, copy, paste).
- **FR-019**: System MUST save document changes to disk when the user explicitly saves.

**Configuration Requirements**

- **FR-020**: System MUST support configuration via a config file in the project root or user's config directory.
- **FR-021**: System MUST support configuration of the worktree directory location.
- **FR-022**: System MUST implement Model Context Protocol (MCP) client for AI agent communication, supporting configuration of MCP server endpoints.
- **FR-023**: System MUST use sensible defaults when no configuration file exists.

### Key Entities

- **Specification**: Represents a feature being developed; contains a unique identifier (number), short name, associated branch, workflow phase (specify/clarify/plan/tasks/implement), and file artifacts (spec.md, plan.md, tasks.md).
- **Worktree**: Represents a git worktree instance; contains a path, associated branch name, working tree status (clean/dirty), and relationship to a specification.
- **Workflow Command**: Represents an executable workflow action; contains a command type, prerequisites, current execution state, and output stream.
- **Project**: Represents the overall git repository; contains configuration, list of specifications, list of worktrees, and the main branch name.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view all project specifications and switch between them in under 5 seconds.
- **SC-002**: Users can trigger any workflow command within 3 keystrokes from the main overview.
- **SC-003**: Worktree operations (create, switch, delete) complete within 10 seconds for typical repositories.
- **SC-004**: TUI remains responsive (renders within 100ms) during long-running background operations.
- **SC-005**: Users can navigate the full interface using only keyboard controls (no mouse required).
- **SC-006**: 90% of users can successfully complete a full workflow cycle (new spec -> implement) on first attempt without external documentation.
- **SC-007**: System correctly identifies and displays status for 100% of valid specifications in the project.
- **SC-008**: Users report reduced context-switching overhead compared to using slash commands in an AI coding assistant.

## Assumptions

- Users have git installed and configured on their system.
- The project follows the spec-kit directory structure with `specs/` containing numbered specification directories.
- Users have terminal emulators that support standard ANSI escape codes and common capabilities.
- An MCP-compatible AI agent (such as Claude Code, Gemini CLI, or similar) is available and accessible via MCP server endpoint.
- Users are familiar with basic terminal navigation and keyboard-driven interfaces.
- The project uses a git workflow where feature branches correspond to specifications.
