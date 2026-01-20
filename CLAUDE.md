# speck Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-01-20

## Active Technologies

- Rust 1.75+ (2021 edition) + Ratatui (TUI rendering), Tokio (async runtime), Crossterm (terminal I/O), gitoxide/gix (git operations), jsonrpc-core (MCP client), config-rs (configuration), tui-textarea (editing), syntect (syntax highlighting) (001-spec-tui-worktree)

## Project Structure

```text
src/
├── lib.rs          # Library exports
├── main.rs         # Entry point with terminal setup
├── app.rs          # Application state and event loop
├── config.rs       # Configuration system
├── error.rs        # Unified error types
├── domain/         # Business entities
│   ├── mod.rs
│   ├── spec.rs     # Specification, WorkflowPhase
│   ├── worktree.rs # Worktree, WorktreeStatus
│   ├── workflow.rs # WorkflowCommand, ExecutionState
│   └── project.rs  # Project context
├── services/       # Infrastructure services
│   ├── mod.rs
│   ├── git.rs      # Git worktree operations
│   └── spec.rs     # Spec discovery and management
└── ui/             # TUI components
    ├── mod.rs
    ├── input.rs    # Vim-style input handling
    ├── layout.rs   # Main layout rendering
    └── widgets/
        ├── mod.rs
        └── spec_list.rs  # Spec list widget
tests/
specs/              # Feature specifications
```

## Commands

```bash
cargo build         # Build the TUI
cargo test          # Run all tests
cargo clippy        # Run linter
cargo run           # Run the TUI (requires terminal)
```

## Code Style

Rust 1.75+ (2021 edition): Follow standard conventions

## Recent Changes

- 001-spec-tui-worktree: Added Rust 1.75+ (2021 edition) + Ratatui (TUI rendering), Tokio (async runtime), Crossterm (terminal I/O), gitoxide/gix (git operations), jsonrpc-core (MCP client), config-rs (configuration), tui-textarea (editing), syntect (syntax highlighting)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
