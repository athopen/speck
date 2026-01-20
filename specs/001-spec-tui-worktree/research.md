# Research: Spec-Driven Development TUI with Git Worktree Management

**Generated**: 2026-01-20
**Status**: Complete

## 1. TUI Framework Selection

### Decision: Rust + Ratatui

### Rationale
- **Performance**: Ratatui uses 30-40% less memory and 15% lower CPU than Go's Bubbletea in benchmarks with 1,000 data points/second rendering
- **Responsiveness**: Zero-cost abstractions guarantee sub-millisecond rendering (meets SC-004: <100ms)
- **Async Support**: Excellent Tokio integration for non-blocking UI during long operations
- **Production Adoption**: Used by Netflix, OpenAI, AWS, Vercel; 17.3k GitHub stars, 14.9M crates.io downloads
- **Ecosystem**: Rich widget ecosystem (tui-textarea, tui-markdown, syntect integration)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Go + Bubbletea | 30-40% higher memory/CPU; GC pauses can violate responsiveness requirements |
| Python + Textual | Performance unsuitable; harder to distribute as standalone binary |
| Node.js + Ink | Not suitable for complex TUIs; weaker terminal compatibility |

## 2. Git Operations

### Decision: gitoxide (gix crate)

### Rationale
- **Pure Rust**: No C dependencies (unlike libgit2), simpler build and distribution
- **Performance**: Memory-mapped I/O, optimized for large repositories
- **Worktree Support**: Dedicated `gix-worktree` crate (v0.45.0+) with full worktree lifecycle management
- **Type Safety**: Rust's type system prevents common git operation errors
- **Async Ready**: Designed to work with spawn_blocking for blocking operations

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| git2 (libgit2) | C dependency complicates builds; less idiomatic Rust |
| Command-line git | Parsing text output is fragile; no type safety |
| go-git | Wrong language ecosystem |

## 3. MCP Client Implementation

### Decision: JSON-RPC 2.0 via jsonrpc-core + stdio transport

### Rationale
- **Standard Protocol**: MCP specification (2025-11-25) uses JSON-RPC 2.0
- **Transport**: stdio recommended for local AI agents (Claude Code, Gemini CLI)
- **Streaming**: Native support for notifications and progress updates
- **Cancellation**: Protocol supports `$/cancelRequest` for graceful abort

### Implementation Pattern
```
Client → Initialize Request (protocol version, capabilities)
Server ← Initialize Response (version, capabilities)
Client → Initialized Notification
→ Active Protocol Phase (requests/responses with streaming)
```

### Key Features Required
- Request timeouts (configurable, default 60s)
- Cancellation support (user can abort running commands)
- Real-time output streaming to dedicated UI panel
- Error handling with user-friendly messages

## 4. Async Architecture

### Decision: Tokio with channel-based communication

### Rationale
- **Separation of Concerns**: Event handling, background tasks, and render loop run concurrently
- **Responsiveness**: UI never blocks on git operations or MCP calls
- **Proven Pattern**: Standard Ratatui async architecture from official tutorials

### Architecture Pattern
```
┌─────────────────────────────────────────────────────┐
│ Event Handler Task                                  │
│ ├─ Reads keyboard input async via EventStream      │
│ ├─ Sends events through mpsc channel               │
│ └─ Runs concurrently with render loop              │
├─────────────────────────────────────────────────────┤
│ Background Tasks                                    │
│ ├─ Git operations (spawn_blocking for blocking IO) │
│ ├─ MCP client calls to AI agents                   │
│ ├─ Process streaming (real-time output)            │
│ └─ File I/O (config, specs)                        │
├─────────────────────────────────────────────────────┤
│ Render Loop                                         │
│ ├─ Draws UI from current state                     │
│ ├─ Non-blocking (300-500μs per frame typical)      │
│ └─ Decoupled from input handling                   │
└─────────────────────────────────────────────────────┘
```

### Implementation Details
- Use `tokio::select!` with tick_interval (100ms) and render_interval (50-100ms)
- EventStream from crossterm for async keyboard input
- MPSC channels for task-to-UI communication
- spawn_blocking for git operations (memory-mapped IO is blocking)

## 5. Text Editing & Markdown Rendering

### Decision: tui-textarea + pulldown-cmark + syntect

### Rationale
- **tui-textarea**: Mature, multi-line editing with yank/paste, backend-agnostic
- **pulldown-cmark**: Fast CommonMark parser for markdown AST
- **syntect**: Sublime Text syntax definitions for code highlighting
- **Integration**: tui-markdown or tui-syntax-highlight bridges these for Ratatui

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| EdTUI | More complex vim modal editing not required for basic document editing |
| Custom editor | Unnecessary complexity; tui-textarea is feature-complete |

## 6. Configuration Management

### Decision: config-rs with TOML format

### Rationale
- **Layered Configuration**: Supports defaults → project → user → env → CLI override hierarchy
- **Format**: TOML is standard for Rust ecosystem (Cargo.toml), familiar to users
- **Zero Boilerplate**: Automatic deserialization to typed structs

### Configuration Hierarchy (highest priority first)
1. Command-line arguments
2. Environment variables (SPEC_TUI_*)
3. User config: `~/.config/spec-tui/config.toml`
4. Project config: `.spec-tui.toml` in project root
5. Built-in defaults

### Default Configuration Schema
```toml
[worktree]
directory = ".worktrees"

[mcp]
transport = "stdio"  # or "http://localhost:8080"
timeout_seconds = 60

[ui]
refresh_rate_ms = 100
vim_navigation = true

[git]
specs_directory = "specs"
```

## 7. Testing Strategy

### Decision: Three-tier testing with ratatui-testlib + insta

### Rationale
- **Unit Tests**: Test domain logic and services independently with mocks
- **Integration Tests**: ratatui-testlib provides PTY-based TUI testing
- **Snapshot Tests**: insta captures terminal output baselines for regression detection

### Testing Approach by Component
| Component | Test Type | Tool |
|-----------|-----------|------|
| Domain entities (Spec, Worktree) | Unit | cargo test |
| Git service | Integration | Real git repos in temp dirs |
| MCP client | Unit + Integration | Mocked server + real stdio tests |
| UI widgets | Snapshot | ratatui-testlib + insta |
| Full workflows | E2E | PTY-based integration tests |

### Test Coverage Priority
1. Spec listing and status detection (FR-001, FR-002)
2. Worktree operations (FR-006 through FR-010)
3. Workflow command execution (FR-011 through FR-015)
4. Real-time output streaming

## 8. Performance Targets

| Metric | Requirement | Expected |
|--------|-------------|----------|
| UI render time | <100ms (SC-004) | 1-5ms |
| Spec list navigation | <5s (SC-001) | <100ms for 100+ specs |
| Worktree operations | <10s (SC-003) | 2-5s typical |
| Memory usage | Efficient | 30-40% less than Go alternatives |
| Terminal resize | Instant | <20ms |

## 9. Dependencies Summary

### Runtime Dependencies
| Crate | Purpose | Version |
|-------|---------|---------|
| ratatui | TUI rendering | 0.26+ |
| crossterm | Terminal I/O | 0.27+ |
| tokio | Async runtime | 1.35+ |
| gix | Git operations | 0.60+ |
| jsonrpc-core | MCP JSON-RPC | 18.0+ |
| config | Configuration | 0.14+ |
| tui-textarea | Text editing | 0.4+ |
| syntect | Syntax highlighting | 5.0+ |
| pulldown-cmark | Markdown parsing | 0.9+ |

### Development Dependencies
| Crate | Purpose |
|-------|---------|
| ratatui-testlib | TUI integration testing |
| insta | Snapshot testing |
| tempfile | Temporary directories for tests |
| tokio-test | Async test utilities |

## Sources

- [Ratatui Official Documentation](https://ratatui.rs/)
- [Ratatui Async Tutorial](https://ratatui.rs/tutorials/counter-async-app/)
- [Gitoxide Repository](https://github.com/GitoxideLabs/gitoxide)
- [MCP Specification (2025-11-25)](https://modelcontextprotocol.io/specification/2025-11-25)
- [config-rs Documentation](https://docs.rs/config/)
- [ratatui-testlib](https://lib.rs/crates/ratatui-testlib)
- [insta Snapshot Testing](https://insta.rs/)
