# Speck

A keyboard-driven terminal UI for spec-driven development with integrated git worktree management.

## Overview

Speck enables developers to manage feature specifications, trigger AI-powered workflow commands, and seamlessly switch between multiple parallel feature developments using git worktrees—all without leaving the terminal.

## Features

### Specification Management
- **Automatic Discovery**: Scans `specs/` directory for feature specifications organized by numbered directories (e.g., `001-feature-name`)
- **Workflow Phases**: Tracks each spec's development phase based on existing artifacts:
  - ○ **Specify**: No spec.md exists yet
  - ◐ **Clarify**: spec.md exists, needs plan.md
  - ◑ **Tasks**: spec.md + plan.md exist, needs tasks.md
  - ● **Implement**: All artifacts present, ready for implementation
- **Document Viewer/Editor**: View and edit spec.md, plan.md, tasks.md, and research.md with syntax highlighting

### Git Worktree Integration
- **Automatic Worktree Creation**: Creates a git worktree when switching to a spec
- **Parallel Development**: Work on multiple features simultaneously without stashing or context switching
- **Status Tracking**: Shows clean/dirty/detached status and commits ahead/behind remote
- **Worktree Management**: Dedicated view for listing, selecting, and deleting worktrees

### Workflow Commands
Integrates with AI agents via MCP (Model Context Protocol) to run:
- **Specify**: Generate initial feature specification
- **Clarify**: Refine and clarify specification details
- **Plan**: Create implementation plan
- **Tasks**: Generate task breakdown
- **Implement**: Execute the implementation workflow

### Terminal UI
- Vim-style keyboard navigation
- Real-time command output streaming
- Syntax-highlighted document viewing
- In-TUI document editing

## Installation

### From Release

Download the latest release from the [releases page](https://github.com/athopen/speck/releases).

```bash
# Extract and install
tar -xzf speck-v0.1.0-linux-x86_64.tar.gz
sudo mv speck /usr/local/bin/
```

### From Source

Requires Rust 1.75+.

```bash
git clone https://github.com/athopen/speck.git
cd speck
cargo build --release
```

The binary will be at `target/release/speck`.

## Usage

Run `speck` in a git repository with a `specs/` directory:

```bash
cd your-project
speck
```

### Keybindings

| Key | Action |
|-----|--------|
| `j`/`↓` | Move down |
| `k`/`↑` | Move up |
| `Enter` | Select/confirm |
| `q`/`Esc` | Back/quit |
| `w` | Switch to spec's worktree |
| `W` | Open worktree management |
| `r` | Run workflow command |
| `v` | View spec document |
| `e` | Edit spec document |
| `n` | Create new spec |
| `d` | Delete worktree |
| `?` | Show help |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `Ctrl+C` | Cancel running command |

### Document Viewing/Editing

While viewing a document:
- `1` - View spec.md
- `2` - View plan.md
- `3` - View tasks.md
- `4` - View research.md
- `e` - Switch to edit mode

While editing:
- `Ctrl+S` - Save
- `Esc` - Exit without saving

## Configuration

Create `.speck.toml` in your project root:

```toml
[worktree]
directory = ".worktrees"    # Where to create worktrees

[mcp]
type = "stdio"              # MCP transport type
timeout_seconds = 60        # Command timeout

[ui]
refresh_rate_ms = 100       # UI refresh interval
vim_navigation = true       # Enable vim keybindings

[git]
specs_directory = "specs"   # Specs folder location
main_branch = "main"        # Primary branch name
```

Configuration is loaded from (lowest to highest precedence):
1. Compiled defaults
2. Project config (`.speck.toml`)
3. User config (`~/.config/speck/config.toml`)
4. Environment variables (`SPECK_*`)

## Project Structure

```
your-project/
├── specs/
│   ├── 001-feature-name/
│   │   ├── spec.md
│   │   ├── plan.md
│   │   ├── tasks.md
│   │   └── research.md
│   └── 002-another-feature/
│       └── spec.md
├── .worktrees/           # Created automatically
│   ├── 001-feature-name/
│   └── 002-another-feature/
└── .speck.toml           # Optional config
```

## Requirements

- Git 2.20+
- A terminal with Unicode support
- For workflow commands: An MCP-compatible AI agent (e.g., Claude)

## License

MIT
