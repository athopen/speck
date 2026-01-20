# Quickstart: Spec-Driven Development TUI

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Git 2.20+ with worktree support
- Terminal emulator with ANSI support
- MCP-compatible AI agent (Claude Code, Gemini CLI, etc.)

## Setup

### 1. Create Project Structure

```bash
# Initialize a new Rust project
cargo new spec-tui --name spec-tui
cd spec-tui

# Create directory structure
mkdir -p src/{ui/widgets,domain,services}
mkdir -p tests/{integration,unit,snapshots}
```

### 2. Add Dependencies

Add to `Cargo.toml`:

```toml
[package]
name = "spec-tui"
version = "0.1.0"
edition = "2021"

[dependencies]
# TUI Framework
ratatui = "0.26"
crossterm = "0.27"

# Async Runtime
tokio = { version = "1.35", features = ["full"] }

# Git Operations
gix = { version = "0.60", features = ["worktree"] }

# MCP Client
jsonrpc-core = "18.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = "0.14"
directories = "5.0"

# Text Editing
tui-textarea = "0.4"

# Markdown & Syntax
pulldown-cmark = "0.9"
syntect = "5.0"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
# Testing
insta = "1.34"
tempfile = "3.10"
tokio-test = "0.4"
```

### 3. Initialize Configuration

Create default config at `~/.config/spec-tui/config.toml`:

```toml
[worktree]
directory = ".worktrees"

[mcp]
transport = "stdio"
timeout_seconds = 60

[ui]
refresh_rate_ms = 100
vim_navigation = true

[git]
specs_directory = "specs"
main_branch = "main"
```

Or project-specific at `.spec-tui.toml`:

```toml
[worktree]
directory = ".worktrees"

[mcp]
transport = "stdio"
timeout_seconds = 120  # Longer timeout for complex specs
```

## Development Workflow

### Build and Run

```bash
# Build
cargo build

# Run in development
cargo run

# Run with verbose logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration

# Update snapshots
cargo insta review
```

### Code Structure

```
src/
├── main.rs         # Entry point
├── app.rs          # Application state and event loop
├── config.rs       # Configuration loading
├── error.rs        # Error types
├── ui/             # TUI components
│   ├── layout.rs   # Panel layouts
│   ├── input.rs    # Keyboard handling
│   └── widgets/    # UI widgets
├── domain/         # Business entities
│   ├── spec.rs     # Specification
│   ├── worktree.rs # Worktree
│   └── workflow.rs # Commands
└── services/       # Infrastructure
    ├── git.rs      # Git operations
    ├── mcp.rs      # MCP client
    └── process.rs  # Process execution
```

## Key Patterns

### 1. Async Event Loop

```rust
// main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let terminal = setup_terminal()?;
    let app = App::new()?;

    app.run(terminal).await?;

    restore_terminal()?;
    Ok(())
}

// app.rs
impl App {
    pub async fn run(&mut self, mut terminal: Terminal<impl Backend>) -> Result<()> {
        let tick_rate = Duration::from_millis(self.config.ui.refresh_rate_ms);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            tokio::select! {
                // Handle keyboard input
                event = crossterm::event::Event::read_async() => {
                    if let Event::Key(key) = event? {
                        if self.handle_key(key).await? {
                            break;
                        }
                    }
                }

                // Handle background task results
                result = self.task_rx.recv() => {
                    if let Some(result) = result {
                        self.handle_task_result(result)?;
                    }
                }

                // Periodic tick for UI updates
                _ = tokio::time::sleep(timeout) => {
                    last_tick = Instant::now();
                }
            }
        }
        Ok(())
    }
}
```

### 2. Git Operations (spawn_blocking)

```rust
// services/git.rs
impl GitService {
    pub async fn list_worktrees(&self) -> Result<Vec<Worktree>> {
        let repo = self.repo.clone();

        tokio::task::spawn_blocking(move || {
            let worktrees = repo.worktrees()?;
            worktrees.into_iter()
                .map(Worktree::from_gix)
                .collect()
        }).await?
    }
}
```

### 3. MCP Client Communication

```rust
// services/mcp.rs
impl McpClient {
    pub async fn call_tool(
        &self,
        tool: &str,
        args: serde_json::Value,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<ToolResult> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method: "tools/call",
            params: ToolCallParams { name: tool, arguments: args },
        };

        self.send(request).await?;

        // Handle responses and progress notifications
        loop {
            let msg = self.receive().await?;
            match msg {
                Message::Progress(p) => {
                    progress_tx.send(p.into()).await?;
                }
                Message::Response(r) if r.id == request.id => {
                    return Ok(r.result.into());
                }
                _ => continue,
            }
        }
    }
}
```

### 4. Vim-style Navigation

```rust
// ui/input.rs
impl InputHandler {
    pub fn handle_key(&self, key: KeyEvent, mode: InputMode) -> Option<Action> {
        match mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Insert => self.handle_insert_key(key),
            InputMode::Command => self.handle_command_key(key),
        }
    }

    fn handle_normal_key(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
            KeyCode::Char('h') | KeyCode::Left => Some(Action::MoveLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::MoveRight),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Select),
            KeyCode::Esc | KeyCode::Char('q') => Some(Action::Back),
            KeyCode::Char('w') => Some(Action::SwitchWorktree),
            KeyCode::Char('r') => Some(Action::RunWorkflow),
            KeyCode::Char('e') => Some(Action::EditDocument),
            KeyCode::Char('?') => Some(Action::ShowHelp),
            _ => None,
        }
    }
}
```

## Running the TUI

```bash
# Navigate to a project with specs/ directory
cd my-project

# Launch TUI
spec-tui

# Or with specific config
spec-tui --config ./custom-config.toml
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Select / Activate |
| `w` | Switch worktree |
| `r` | Run workflow command |
| `v` | View document |
| `e` | Edit document |
| `n` | New specification |
| `d` | Delete worktree (with confirm) |
| `c` / `Ctrl+C` | Cancel running command |
| `q` / `Esc` | Back / Quit |
| `?` | Help |

## Next Steps

1. Implement core entities in `src/domain/`
2. Build git service in `src/services/git.rs`
3. Create basic TUI layout in `src/ui/layout.rs`
4. Add spec list widget
5. Implement worktree switching
6. Build MCP client
7. Add workflow command execution
8. Add document viewing/editing
