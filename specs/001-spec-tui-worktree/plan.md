# Implementation Plan: Spec-Driven Development TUI with Git Worktree Management

**Branch**: `001-spec-tui-worktree` | **Date**: 2026-01-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-spec-tui-worktree/spec.md`

## Summary

Build a terminal UI for spec-driven development workflow with git worktree integration for parallel feature development. The TUI will use MCP (Model Context Protocol) for vendor-agnostic AI agent communication, enabling workflow commands (specify, clarify, plan, tasks, implement) to be executed from a responsive, keyboard-driven interface.

**Technical Approach**: Rust with Ratatui framework for high-performance TUI rendering, Tokio for async operations, gitoxide for native git worktree management, and JSON-RPC 2.0 for MCP client implementation.

## Technical Context

**Language/Version**: Rust 1.75+ (2021 edition)
**Primary Dependencies**: Ratatui (TUI rendering), Tokio (async runtime), Crossterm (terminal I/O), gitoxide/gix (git operations), jsonrpc-core (MCP client), config-rs (configuration), tui-textarea (editing), syntect (syntax highlighting)
**Storage**: File-based (specs/*.md, config files - TOML format)
**Testing**: cargo test, ratatui-testlib (TUI integration), insta (snapshot testing)
**Target Platform**: Linux, macOS, Windows (cross-platform via crossterm)
**Project Type**: Single CLI application
**Performance Goals**: UI renders <100ms (SC-004), spec navigation <5s (SC-001), worktree ops <10s (SC-003)
**Constraints**: Keyboard-only navigation required (SC-005), memory-efficient for long-running sessions
**Scale/Scope**: Handle 100+ specs, multiple concurrent worktrees, real-time process output streaming

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Note**: The project constitution (`.specify/memory/constitution.md`) contains placeholder values and has not been configured with specific principles. The following gates are derived from industry best practices and the feature spec requirements:

### Pre-Phase 0 Gates

| Gate | Status | Evidence |
|------|--------|----------|
| Single-purpose focus | ✅ PASS | TUI application with clear scope: spec management + worktree operations |
| Testability | ✅ PASS | Rust testing ecosystem (cargo test, ratatui-testlib, insta) planned |
| Security boundaries | ✅ PASS | Local tool only, no network services, MCP client uses standard protocol |
| Dependency justification | ✅ PASS | All dependencies serve specific purposes documented in Technical Context |

### Complexity Assessment

- **Project Structure**: Single project (not multi-project/monorepo) - appropriate for CLI tool
- **Abstraction Level**: Direct implementation without unnecessary patterns
- **External Dependencies**: Minimal, well-maintained Rust crates from established ecosystem

### Post-Phase 1 Re-evaluation

| Gate | Status | Evidence |
|------|--------|----------|
| Data model simplicity | ✅ PASS | 4 core entities (Project, Spec, Worktree, Workflow) map directly to domain |
| Service contracts clarity | ✅ PASS | 3 service contracts (MCP, Git, Spec) with clear boundaries |
| No over-engineering | ✅ PASS | No repository pattern, no dependency injection framework, no ORM |
| Performance achievable | ✅ PASS | Ratatui benchmarks show <5ms render time, gitoxide is production-tested |
| Test strategy feasible | ✅ PASS | Standard Rust testing + ratatui-testlib for TUI, insta for snapshots |

**Conclusion**: Design passes all gates. No complexity violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point, CLI args, app initialization
├── app.rs               # Application state, main event loop
├── config.rs            # Configuration loading (config-rs)
├── ui/
│   ├── mod.rs           # UI module exports
│   ├── layout.rs        # Panel layouts (overview, detail, output)
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── spec_list.rs      # Spec listing widget
│   │   ├── spec_detail.rs    # Spec detail/preview widget
│   │   ├── output_panel.rs   # Streaming output panel
│   │   ├── editor.rs         # Document editor widget
│   │   └── worktree_list.rs  # Worktree management widget
│   └── input.rs         # Keyboard input handling (vim-style)
├── domain/
│   ├── mod.rs           # Domain module exports
│   ├── spec.rs          # Specification entity and operations
│   ├── worktree.rs      # Worktree entity and git operations
│   ├── workflow.rs      # Workflow command definitions
│   └── project.rs       # Project-level operations
├── services/
│   ├── mod.rs           # Service module exports
│   ├── git.rs           # Git/worktree operations (gitoxide)
│   ├── mcp.rs           # MCP client (JSON-RPC 2.0)
│   ├── file_watcher.rs  # File system monitoring
│   └── process.rs       # Process execution and streaming
└── error.rs             # Error types and handling

tests/
├── integration/
│   ├── spec_navigation.rs    # E2E spec list and navigation
│   ├── worktree_ops.rs       # Git worktree create/switch/delete
│   └── workflow_commands.rs  # MCP workflow execution
├── unit/
│   ├── domain/               # Domain logic tests
│   └── services/             # Service tests (mocked)
└── snapshots/                # insta snapshot baselines
```

**Structure Decision**: Single Rust project with standard Cargo layout. The `ui/` module contains Ratatui widgets and layout logic. The `domain/` module contains business entities (Spec, Worktree, Workflow). The `services/` module contains infrastructure (git, MCP client, file I/O). This separation enables independent unit testing of each layer.

## Complexity Tracking

> **No violations to justify** - Design passes all constitution gates.

## Generated Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Research | [research.md](./research.md) | ✅ Complete |
| Data Model | [data-model.md](./data-model.md) | ✅ Complete |
| MCP Contract | [contracts/mcp-client.md](./contracts/mcp-client.md) | ✅ Complete |
| Git Service Contract | [contracts/git-service.md](./contracts/git-service.md) | ✅ Complete |
| Spec Service Contract | [contracts/spec-service.md](./contracts/spec-service.md) | ✅ Complete |
| Quickstart | [quickstart.md](./quickstart.md) | ✅ Complete |

## Next Steps

Run `/speckit.tasks` to generate the task breakdown from this implementation plan.
