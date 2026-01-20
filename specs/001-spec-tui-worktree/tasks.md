# Tasks: Spec-Driven Development TUI with Git Worktree Management

**Input**: Design documents from `/specs/001-spec-tui-worktree/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are NOT explicitly requested in the feature specification. Test tasks are omitted.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure (Single Rust CLI project):
- **Source**: `src/` at repository root
- **Tests**: `tests/` at repository root
- **Domain**: `src/domain/` for business entities
- **Services**: `src/services/` for infrastructure
- **UI**: `src/ui/` for TUI components

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and Rust project structure

- [x] T001 Initialize Rust project with `cargo new spec-tui --name spec-tui`
- [x] T002 Configure Cargo.toml with all dependencies from research.md in Cargo.toml
- [x] T003 [P] Create directory structure: `src/{ui/widgets,domain,services}` and `tests/{integration,unit,snapshots}`
- [x] T004 [P] Configure rustfmt.toml for code formatting
- [x] T005 [P] Configure clippy.toml for linting
- [x] T006 Create src/error.rs with unified error types (AppError, GitError, SpecError, McpError)
- [x] T007 Create src/lib.rs exposing all public modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### Configuration System (FR-020, FR-021, FR-023)

- [x] T008 Create src/config.rs with ProjectConfig struct per data-model.md
- [x] T009 Implement config loading hierarchy (defaults → project → user → env) using config-rs in src/config.rs
- [x] T010 Add WorktreeConfig, McpConfig, UiConfig, GitConfig structs in src/config.rs

### Domain Entities (from data-model.md)

- [x] T011 [P] Create src/domain/mod.rs exporting all domain modules
- [x] T012 [P] Create src/domain/spec.rs with SpecId, Specification, SpecArtifacts, WorkflowPhase per data-model.md
- [x] T013 [P] Create src/domain/worktree.rs with Worktree, WorktreeStatus, WorktreeSyncStatus per data-model.md
- [x] T014 [P] Create src/domain/workflow.rs with WorkflowCommandType, ExecutionState, WorkflowCommand, OutputLine per data-model.md
- [x] T015 [P] Create src/domain/project.rs with Project struct per data-model.md

### Core Services Infrastructure

- [x] T016 Create src/services/mod.rs exporting all service modules
- [x] T017 Implement GitService trait in src/services/git.rs per contracts/git-service.md
- [x] T018 Implement SpecService trait in src/services/spec.rs per contracts/spec-service.md (discover_specs, load_spec, get_phase)

### Application Shell

- [x] T019 Create src/app.rs with AppState struct per data-model.md (UI State Model section)
- [x] T020 Implement basic async event loop skeleton in src/app.rs using tokio::select!
- [x] T021 Create src/main.rs with terminal setup/restore and CLI args parsing
- [x] T022 Implement terminal setup (enable raw mode, alternate screen) in src/main.rs
- [x] T023 Implement terminal restore (disable raw mode, leave alternate screen) in src/main.rs

### UI Foundation

- [x] T024 Create src/ui/mod.rs exporting all UI modules
- [x] T025 Create src/ui/input.rs with InputMode enum (Normal, Insert, Command) and KeyBindings struct
- [x] T026 Implement vim-style key handling (j/k/h/l navigation) in src/ui/input.rs per FR-003
- [x] T027 Create src/ui/layout.rs with basic layout rendering function

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Specs Overview (Priority: P1) MVP

**Goal**: Display a navigable list of all specifications with status indicators

**Independent Test**: Launch TUI in a project with specs/ directory, verify all specs listed with correct phases

### Implementation for User Story 1

- [x] T028 [US1] Implement SpecService.discover_specs() in src/services/spec.rs (scan specs/ directory)
- [x] T029 [US1] Implement WorkflowPhase.from_artifacts() logic in src/domain/spec.rs per FR-002
- [x] T030 [US1] Create src/ui/widgets/mod.rs exporting all widgets
- [x] T031 [US1] Create src/ui/widgets/spec_list.rs with SpecListWidget struct
- [x] T032 [US1] Implement spec list rendering with name, branch, phase badge in src/ui/widgets/spec_list.rs per FR-001
- [x] T033 [US1] Add worktree active indicator (visual distinction) in src/ui/widgets/spec_list.rs
- [x] T034 [US1] Integrate SpecListWidget into main layout in src/ui/layout.rs
- [x] T035 [US1] Implement keyboard navigation (up/down, j/k) for spec list in src/app.rs per FR-003
- [x] T036 [US1] Implement spec selection state tracking in src/app.rs (selected_spec_index)
- [x] T037 [US1] Add loading indicator when discovering specs in src/ui/layout.rs per FR-004
- [x] T038 [US1] Wire discover_specs to app startup in src/app.rs

**Checkpoint**: User Story 1 complete - can view and navigate specs list

---

## Phase 4: User Story 2 - Switch Active Spec via Worktree (Priority: P1)

**Goal**: Switch between specs using git worktrees for parallel development

**Independent Test**: Select a spec, activate switch, verify worktree created/navigated

### Implementation for User Story 2

- [x] T039 [US2] Implement GitService.list_worktrees() in src/services/git.rs using gitoxide
- [x] T040 [US2] Implement GitService.branch_exists() in src/services/git.rs
- [x] T041 [US2] Implement GitService.create_worktree() in src/services/git.rs per FR-006, FR-007
- [x] T042 [US2] Implement duplicate worktree prevention in src/services/git.rs per FR-009
- [x] T043 [US2] Implement GitService.worktree_status() in src/services/git.rs per FR-008
- [x] T044 [US2] Add spawn_blocking wrapper for all git operations in src/services/git.rs
- [x] T045 [US2] Add 'w' keybinding to trigger worktree switch action in src/ui/input.rs
- [x] T046 [US2] Implement switch_to_spec() function in src/app.rs (create worktree if needed, navigate)
- [x] T047 [US2] Display worktree status (clean/dirty) in spec list in src/ui/widgets/spec_list.rs
- [x] T048 [US2] Add error display for failed worktree operations in src/ui/layout.rs
- [x] T049 [US2] Implement async worktree creation with loading state in src/app.rs per FR-005

**Checkpoint**: User Story 2 complete - can switch between specs via worktrees

---

## Phase 5: User Story 3 - Trigger Spec-Kit Workflow Commands (Priority: P1)

**Goal**: Execute workflow commands (specify, clarify, plan, tasks, implement) via MCP

**Independent Test**: Select a spec, trigger workflow command, see output streaming in TUI

### Implementation for User Story 3

- [x] T050 [US3] Create src/services/mcp.rs with McpClient struct
- [x] T051 [US3] Implement JSON-RPC 2.0 message types in src/services/mcp.rs per contracts/mcp-client.md
- [x] T052 [US3] Implement MCP initialize handshake in src/services/mcp.rs
- [x] T053 [US3] Implement tools/list discovery in src/services/mcp.rs
- [x] T054 [US3] Implement tools/call for workflow commands in src/services/mcp.rs per FR-011
- [x] T055 [US3] Implement progress notification handling in src/services/mcp.rs
- [x] T056 [US3] Implement $/cancelRequest for command cancellation in src/services/mcp.rs per FR-013
- [x] T057 [US3] Create src/ui/widgets/output_panel.rs for streaming output display
- [x] T058 [US3] Implement real-time output streaming to OutputPanel in src/ui/widgets/output_panel.rs per FR-012
- [x] T059 [US3] Integrate OutputPanel into layout (bottom panel) in src/ui/layout.rs
- [x] T060 [US3] Add 'r' keybinding to trigger workflow command in src/ui/input.rs
- [x] T061 [US3] Implement workflow command selection menu in src/app.rs (show available commands per phase)
- [x] T062 [US3] Validate prerequisites before running commands in src/app.rs per FR-015
- [x] T063 [US3] Create src/services/process.rs for process execution and stdio handling
- [x] T064 [US3] Implement log file persistence for command output in src/services/process.rs per FR-014
- [x] T065 [US3] Add 'c' and Ctrl+C keybinding to cancel running command in src/ui/input.rs

**Checkpoint**: User Story 3 complete - can trigger and monitor workflow commands

---

## Phase 6: User Story 4 - View and Edit Specification Documents (Priority: P2)

**Goal**: View and edit spec.md, plan.md, tasks.md directly in TUI

**Independent Test**: Open document viewer, make edit, save, verify file updated

### Implementation for User Story 4

- [x] T066 [US4] Implement SpecService.read_artifact() in src/services/spec.rs
- [x] T067 [US4] Implement SpecService.write_artifact() in src/services/spec.rs per FR-019
- [x] T068 [US4] Create src/ui/widgets/spec_detail.rs for document viewing
- [x] T069 [US4] Implement markdown syntax highlighting using syntect in src/ui/widgets/spec_detail.rs per FR-016
- [x] T070 [US4] Implement scrollable document view in src/ui/widgets/spec_detail.rs per FR-017
- [x] T071 [US4] Create src/ui/widgets/editor.rs using tui-textarea
- [x] T072 [US4] Implement insert mode (text editing) in src/ui/widgets/editor.rs per FR-018
- [x] T073 [US4] Implement save functionality (Ctrl+S or :w) in src/ui/widgets/editor.rs
- [x] T074 [US4] Add 'v' keybinding to view document in src/ui/input.rs
- [x] T075 [US4] Add 'e' keybinding to edit document in src/ui/input.rs
- [x] T076 [US4] Implement DocumentView and DocumentEdit app views in src/app.rs
- [x] T077 [US4] Add modified indicator in editor status line in src/ui/widgets/editor.rs

**Checkpoint**: User Story 4 complete - can view and edit specification documents

---

## Phase 7: User Story 5 - Manage Worktrees (Priority: P2)

**Goal**: View, manage, and cleanup git worktrees

**Independent Test**: View worktree list, delete an old worktree, verify removed from git

### Implementation for User Story 5

- [x] T078 [US5] Create src/ui/widgets/worktree_list.rs for worktree management view
- [x] T079 [US5] Implement worktree list rendering (path, branch, status) in src/ui/widgets/worktree_list.rs
- [x] T080 [US5] Implement GitService.delete_worktree() in src/services/git.rs per FR-010
- [x] T081 [US5] Add confirmation prompt widget in src/ui/widgets/worktree_list.rs
- [x] T082 [US5] Prevent deletion of current worktree in src/services/git.rs
- [x] T083 [US5] Implement GitService.sync_status() for ahead/behind display in src/services/git.rs
- [x] T084 [US5] Add WorktreeManagement app view in src/app.rs
- [x] T085 [US5] Add keybinding to access worktree management view in src/ui/input.rs
- [x] T086 [US5] Add 'd' keybinding for worktree deletion (with confirm) in src/ui/input.rs

**Checkpoint**: User Story 5 complete - can manage worktrees

---

## Phase 8: User Story 6 - Create New Feature Specification (Priority: P2)

**Goal**: Create new specs from within the TUI

**Independent Test**: Trigger new spec, enter description, verify spec directory and branch created

### Implementation for User Story 6

- [x] T087 [US6] Implement SpecService.next_number() in src/services/spec.rs
- [x] T088 [US6] Implement SpecService.create_spec() in src/services/spec.rs
- [x] T089 [US6] Implement GitService.create_branch() for new spec branch in src/services/git.rs
- [x] T090 [US6] Create text input widget for feature description in src/ui/widgets/
- [x] T091 [US6] Add 'n' keybinding to trigger new spec creation in src/ui/input.rs
- [x] T092 [US6] Implement new spec creation flow in src/app.rs (prompt → create → switch)
- [x] T093 [US6] Auto-switch to new spec worktree after creation in src/app.rs
- [x] T094 [US6] Refresh spec list after creation in src/app.rs

**Checkpoint**: User Story 6 complete - can create new specifications

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T095 [P] Add help view ('?' key) showing all keybindings in src/ui/widgets/help.rs
- [x] T096 [P] Implement terminal resize handling in src/app.rs per edge case
- [x] T097 Implement error message display toast/overlay in src/ui/layout.rs
- [x] T098 Add graceful shutdown on Ctrl+C (cleanup worktree locks) in src/main.rs
- [x] T099 [P] Add RUST_LOG environment variable support for debug logging in src/main.rs
- [x] T100 Performance optimization: cache spec discovery results in src/app.rs (specs cached on startup)
- [x] T101 [P] Create sample config file at project root: .spec-tui.toml.example
- [x] T102 Run quickstart.md validation (build and run TUI successfully)

**Checkpoint**: All phases complete - spec-tui implementation finished

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 (View Specs) → Foundation for all other stories
  - US2 (Worktrees) → Can start after Foundational, may use US1 components
  - US3 (Workflow) → Can start after Foundational, uses US1 for spec context
  - US4 (Editing) → Can start after Foundational
  - US5 (Manage Worktrees) → Uses US2 git components
  - US6 (New Spec) → Uses US2 git components
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: FOUNDATION - Must complete first after Foundational
- **User Story 2 (P1)**: Can start after US1 (needs spec list for context)
- **User Story 3 (P1)**: Can start after US1 (needs spec context for commands)
- **User Story 4 (P2)**: Can start after US1 (needs spec selection)
- **User Story 5 (P2)**: Depends on US2 (git service must exist)
- **User Story 6 (P2)**: Depends on US2 (needs git branch creation)

### Within Each User Story

- Services before UI
- UI widgets before integration into app
- Core implementation before keybindings

### Parallel Opportunities

**Setup (Phase 1)**:
- T003, T004, T005 can run in parallel (different files)

**Foundational (Phase 2)**:
- T011-T015 can run in parallel (different domain modules)

**User Stories**:
- US2, US3, US4 can potentially overlap once US1 is complete
- US5, US6 can overlap once US2 git service is complete

---

## Parallel Example: Foundational Domain Entities

```bash
# Launch all domain entity tasks in parallel:
Task: "Create src/domain/spec.rs with SpecId, Specification, SpecArtifacts, WorkflowPhase"
Task: "Create src/domain/worktree.rs with Worktree, WorktreeStatus, WorktreeSyncStatus"
Task: "Create src/domain/workflow.rs with WorkflowCommandType, ExecutionState, WorkflowCommand"
Task: "Create src/domain/project.rs with Project struct"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (View Specs)
4. **STOP and VALIDATE**: Can view and navigate specs list
5. Demo/validate core TUI functionality

### Core Workflow (P1 Stories)

1. Complete MVP (US1)
2. Add User Story 2 (Worktree switching) → Core parallel development capability
3. Add User Story 3 (Workflow commands) → Full spec-kit functionality in TUI
4. At this point, the TUI delivers its core value proposition

### Full Feature (All Stories)

1. Complete Core Workflow (US1 + US2 + US3)
2. Add User Story 4 (Document editing) → Enhanced workflow
3. Add User Story 5 (Worktree management) → Maintenance capability
4. Add User Story 6 (New spec creation) → Complete lifecycle
5. Polish phase for production readiness

---

## Notes

- [P] tasks = different files, no dependencies within same phase
- [US#] label maps task to specific user story for traceability
- Each user story should be independently completable after its dependencies
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All git operations must use spawn_blocking (blocking I/O)
- All MCP operations are async (stdio communication)
