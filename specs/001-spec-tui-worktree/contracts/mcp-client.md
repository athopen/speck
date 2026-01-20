# MCP Client Contract

**Protocol Version**: MCP 2025-11-25
**Transport**: JSON-RPC 2.0 over stdio

## Overview

The TUI implements an MCP client to communicate with AI agents for workflow commands (specify, clarify, plan, tasks, implement). The client uses stdio transport by default, spawning the agent process and communicating via stdin/stdout.

## Message Types

### JSON-RPC 2.0 Base

All messages conform to JSON-RPC 2.0 specification.

```json
// Request
{
  "jsonrpc": "2.0",
  "id": "<unique-integer-or-string>",
  "method": "<method-name>",
  "params": { ... }
}

// Response (success)
{
  "jsonrpc": "2.0",
  "id": "<matching-request-id>",
  "result": { ... }
}

// Response (error)
{
  "jsonrpc": "2.0",
  "id": "<matching-request-id>",
  "error": {
    "code": <integer>,
    "message": "<error-message>",
    "data": { ... }  // optional
  }
}

// Notification (no response expected)
{
  "jsonrpc": "2.0",
  "method": "<method-name>",
  "params": { ... }
}
```

## Lifecycle

### 1. Initialization

```json
// Client → Server
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "sampling": {}
    },
    "clientInfo": {
      "name": "spec-tui",
      "version": "0.1.0"
    }
  }
}

// Server → Client
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "tools": { "listChanged": true }
    },
    "serverInfo": {
      "name": "claude-code",
      "version": "1.0.0"
    }
  }
}

// Client → Server (notification, no id)
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

### 2. Tool Discovery

```json
// Client → Server
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}

// Server → Client
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "speckit.specify",
        "description": "Create or update feature specification",
        "inputSchema": { ... }
      },
      {
        "name": "speckit.plan",
        "description": "Generate implementation plan",
        "inputSchema": { ... }
      }
      // ... other workflow tools
    ]
  }
}
```

### 3. Tool Invocation (Workflow Commands)

```json
// Client → Server
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "speckit.specify",
    "arguments": {
      "feature_description": "User-provided feature description",
      "spec_directory": "/path/to/specs/001-feature"
    }
  }
}

// Server → Client (progress notification)
{
  "jsonrpc": "2.0",
  "method": "notifications/progress",
  "params": {
    "progressToken": "tool-3",
    "progress": 50,
    "total": 100,
    "message": "Analyzing requirements..."
  }
}

// Server → Client (completion)
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Specification created successfully at /path/to/specs/001-feature/spec.md"
      }
    ],
    "isError": false
  }
}
```

### 4. Cancellation

```json
// Client → Server (notification)
{
  "jsonrpc": "2.0",
  "method": "$/cancelRequest",
  "params": {
    "id": 3
  }
}
```

### 5. Shutdown

```json
// Client → Server
{
  "jsonrpc": "2.0",
  "id": 99,
  "method": "shutdown"
}

// Server → Client
{
  "jsonrpc": "2.0",
  "id": 99,
  "result": null
}

// Client → Server (notification)
{
  "jsonrpc": "2.0",
  "method": "exit"
}
```

## Workflow Tool Schemas

### speckit.specify

```json
{
  "name": "speckit.specify",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_description": {
        "type": "string",
        "description": "Natural language description of the feature"
      },
      "spec_directory": {
        "type": "string",
        "description": "Absolute path to specification directory"
      }
    },
    "required": ["feature_description", "spec_directory"]
  }
}
```

### speckit.clarify

```json
{
  "name": "speckit.clarify",
  "inputSchema": {
    "type": "object",
    "properties": {
      "spec_directory": {
        "type": "string",
        "description": "Absolute path to specification directory"
      },
      "questions": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Specific questions to clarify (optional)"
      }
    },
    "required": ["spec_directory"]
  }
}
```

### speckit.plan

```json
{
  "name": "speckit.plan",
  "inputSchema": {
    "type": "object",
    "properties": {
      "spec_directory": {
        "type": "string",
        "description": "Absolute path to specification directory"
      }
    },
    "required": ["spec_directory"]
  }
}
```

### speckit.tasks

```json
{
  "name": "speckit.tasks",
  "inputSchema": {
    "type": "object",
    "properties": {
      "spec_directory": {
        "type": "string",
        "description": "Absolute path to specification directory"
      }
    },
    "required": ["spec_directory"]
  }
}
```

### speckit.implement

```json
{
  "name": "speckit.implement",
  "inputSchema": {
    "type": "object",
    "properties": {
      "spec_directory": {
        "type": "string",
        "description": "Absolute path to specification directory"
      },
      "task_filter": {
        "type": "string",
        "description": "Optional filter for specific tasks to implement"
      }
    },
    "required": ["spec_directory"]
  }
}
```

## Error Codes

Standard JSON-RPC 2.0 error codes plus MCP-specific:

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Invalid JSON-RPC request |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Server internal error |
| -32000 | Tool execution error | Tool failed to execute |
| -32001 | Timeout | Operation timed out |
| -32002 | Cancelled | Operation was cancelled |

## Client Implementation Requirements

### Timeout Handling

- Default timeout: 60 seconds (configurable)
- Send `$/cancelRequest` on timeout
- Display timeout error to user

### Progress Display

- Subscribe to `notifications/progress` for long-running operations
- Display progress bar in output panel
- Show progress message text

### Output Streaming

- Buffer output lines for display
- Handle both stdout and stderr from tool results
- Persist output to log file (FR-014)

### Error Handling

- Display user-friendly error messages
- Log full error details for debugging
- Allow retry for recoverable errors

## Transport Configuration

### stdio (default)

```toml
[mcp]
transport = "stdio"
command = "claude"  # Command to spawn MCP server
args = ["--mcp"]    # Arguments for MCP mode
```

### HTTP (alternative)

```toml
[mcp]
transport = "http"
endpoint = "http://localhost:8080/mcp"
```

## Sequence Diagram

```
TUI                          MCP Server (AI Agent)
 │                                    │
 │──── initialize ───────────────────►│
 │◄─── capabilities ──────────────────│
 │──── initialized ──────────────────►│
 │                                    │
 │──── tools/list ───────────────────►│
 │◄─── available tools ───────────────│
 │                                    │
 │==== User triggers workflow ========│
 │                                    │
 │──── tools/call (speckit.plan) ────►│
 │                                    │
 │◄─── progress notification ─────────│ (display in output panel)
 │◄─── progress notification ─────────│
 │◄─── progress notification ─────────│
 │                                    │
 │◄─── result ────────────────────────│ (command complete)
 │                                    │
 │==== User cancels ==================│
 │                                    │
 │──── $/cancelRequest ──────────────►│
 │                                    │
 │==== Shutdown ======================│
 │                                    │
 │──── shutdown ─────────────────────►│
 │◄─── ack ───────────────────────────│
 │──── exit ─────────────────────────►│
 │                                    │
```
