//! MCP (Model Context Protocol) client for communicating with AI agents.
//!
//! Implements JSON-RPC 2.0 over stdio transport for workflow commands.

use crate::domain::WorkflowCommandType;
use crate::error::{McpError, McpResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// JSON-RPC 2.0 version string
const JSONRPC_VERSION: &str = "2.0";

/// MCP protocol version
const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

/// Client name and version
const CLIENT_NAME: &str = "spec-tui";
const CLIENT_VERSION: &str = "0.1.0";

/// JSON-RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new request with an ID (expects response)
    pub fn new(id: u64, method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id),
            method: method.to_string(),
            params,
        }
    }

    /// Create a notification (no response expected)
    pub fn notification(method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: None,
            method: method.to_string(),
            params,
        }
    }
}

/// JSON-RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the result or error
    pub fn into_result(self) -> McpResult<Value> {
        if let Some(error) = self.error {
            Err(McpError::RpcError {
                code: error.code,
                message: error.message,
            })
        } else {
            Ok(self.result.unwrap_or(Value::Null))
        }
    }
}

/// JSON-RPC Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const TOOL_EXECUTION_ERROR: i32 = -32000;
    pub const TIMEOUT: i32 = -32001;
    pub const CANCELLED: i32 = -32002;
}

/// Progress notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressParams {
    #[serde(rename = "progressToken")]
    pub progress_token: String,
    pub progress: u32,
    pub total: Option<u32>,
    pub message: Option<String>,
}

/// Tool definition from tools/list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

/// Tool call result content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolResultContent>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// MCP Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
}

/// Initialize request params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Option<ServerInfo>,
}

/// Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Tools list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDefinition>,
}

/// MCP event for async handling
#[derive(Debug, Clone)]
pub enum McpEvent {
    /// Progress notification received
    Progress(ProgressParams),
    /// Response received
    Response(JsonRpcResponse),
    /// Server message (stdout line)
    Output(String),
    /// Error occurred
    Error(String),
    /// Connection closed
    Closed,
}

/// MCP Client for communicating with AI agents
pub struct McpClient {
    /// Child process
    process: Option<Child>,
    /// Stdin for writing
    stdin: Option<ChildStdin>,
    /// Stdout reader
    stdout_reader: Option<BufReader<ChildStdout>>,
    /// Request ID counter
    next_id: AtomicU64,
    /// Pending requests (id -> sender)
    pending: Arc<Mutex<HashMap<u64, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
    /// Available tools
    tools: Vec<ToolDefinition>,
    /// Is initialized
    initialized: bool,
    /// Command to spawn MCP server
    command: String,
    /// Arguments for MCP server
    args: Vec<String>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            process: None,
            stdin: None,
            stdout_reader: None,
            next_id: AtomicU64::new(1),
            pending: Arc::new(Mutex::new(HashMap::new())),
            tools: Vec::new(),
            initialized: false,
            command,
            args,
        }
    }

    /// Create a client with default claude command
    pub fn default_claude() -> Self {
        Self::new("claude".to_string(), vec!["--mcp".to_string()])
    }

    /// Get the next request ID
    fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Connect to the MCP server (spawn process)
    pub fn connect(&mut self) -> McpResult<()> {
        if self.process.is_some() {
            return Err(McpError::AlreadyConnected);
        }

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| McpError::SpawnFailed(e.to_string()))?;

        self.stdin = child.stdin.take();
        self.stdout_reader = child.stdout.take().map(BufReader::new);
        self.process = Some(child);

        Ok(())
    }

    /// Send a JSON-RPC request and get response
    fn send_request(&mut self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let stdin = self.stdin.as_mut().ok_or(McpError::NotConnected)?;
        let stdout_reader = self.stdout_reader.as_mut().ok_or(McpError::NotConnected)?;

        // Serialize and send
        let msg = serde_json::to_string(request)
            .map_err(|e| McpError::SerializationError(e.to_string()))?;

        writeln!(stdin, "{}", msg).map_err(|e| McpError::IoError(e.to_string()))?;
        stdin
            .flush()
            .map_err(|e| McpError::IoError(e.to_string()))?;

        // If this is a notification (no id), don't wait for response
        if request.id.is_none() {
            return Ok(JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.to_string(),
                id: None,
                result: Some(Value::Null),
                error: None,
            });
        }

        // Read response (blocking)
        let mut line = String::new();
        stdout_reader
            .read_line(&mut line)
            .map_err(|e| McpError::IoError(e.to_string()))?;

        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| McpError::DeserializationError(e.to_string()))?;

        Ok(response)
    }

    /// Send a notification (no response expected)
    fn send_notification(&mut self, method: &str, params: Option<Value>) -> McpResult<()> {
        let request = JsonRpcRequest::notification(method, params);
        self.send_request(&request)?;
        Ok(())
    }

    /// Initialize the MCP connection
    pub fn initialize(&mut self) -> McpResult<InitializeResult> {
        let params = InitializeParams {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities {
                sampling: Some(serde_json::json!({})),
            },
            client_info: ClientInfo {
                name: CLIENT_NAME.to_string(),
                version: CLIENT_VERSION.to_string(),
            },
        };

        let request = JsonRpcRequest::new(
            self.next_request_id(),
            "initialize",
            Some(serde_json::to_value(params).unwrap()),
        );

        let response = self.send_request(&request)?;
        let result: InitializeResult = serde_json::from_value(response.into_result()?)
            .map_err(|e| McpError::DeserializationError(e.to_string()))?;

        // Send initialized notification
        self.send_notification("notifications/initialized", None)?;

        self.initialized = true;
        Ok(result)
    }

    /// List available tools
    pub fn list_tools(&mut self) -> McpResult<Vec<ToolDefinition>> {
        if !self.initialized {
            return Err(McpError::NotInitialized);
        }

        let request = JsonRpcRequest::new(self.next_request_id(), "tools/list", None);

        let response = self.send_request(&request)?;
        let result: ToolsListResult = serde_json::from_value(response.into_result()?)
            .map_err(|e| McpError::DeserializationError(e.to_string()))?;

        self.tools = result.tools.clone();
        Ok(result.tools)
    }

    /// Call a tool
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> McpResult<ToolResult> {
        if !self.initialized {
            return Err(McpError::NotInitialized);
        }

        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let request = JsonRpcRequest::new(self.next_request_id(), "tools/call", Some(params));

        let response = self.send_request(&request)?;
        let result: ToolResult = serde_json::from_value(response.into_result()?)
            .map_err(|e| McpError::DeserializationError(e.to_string()))?;

        Ok(result)
    }

    /// Call a workflow command
    pub fn call_workflow(
        &mut self,
        command_type: WorkflowCommandType,
        spec_directory: &PathBuf,
        extra_args: Option<Value>,
    ) -> McpResult<ToolResult> {
        let tool_name = command_type.tool_name();

        let mut arguments = serde_json::json!({
            "spec_directory": spec_directory.to_string_lossy()
        });

        // Merge extra args if provided
        if let Some(extra) = extra_args {
            if let (Value::Object(ref mut base), Value::Object(extra_obj)) = (&mut arguments, extra)
            {
                for (k, v) in extra_obj {
                    base.insert(k, v);
                }
            }
        }

        self.call_tool(tool_name, arguments)
    }

    /// Cancel a pending request
    pub fn cancel_request(&mut self, request_id: u64) -> McpResult<()> {
        let params = serde_json::json!({
            "id": request_id
        });

        self.send_notification("$/cancelRequest", Some(params))
    }

    /// Shutdown the connection
    pub fn shutdown(&mut self) -> McpResult<()> {
        if !self.initialized {
            return Ok(());
        }

        // Send shutdown request
        let request = JsonRpcRequest::new(self.next_request_id(), "shutdown", None);

        let _ = self.send_request(&request);

        // Send exit notification
        self.send_notification("exit", None)?;

        self.initialized = false;
        Ok(())
    }

    /// Close the connection and kill the process
    pub fn close(&mut self) {
        if let Some(ref mut process) = self.process {
            let _ = process.kill();
        }
        self.process = None;
        self.stdin = None;
        self.stdout_reader = None;
        self.initialized = false;
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.process.is_some()
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get available tools
    pub fn get_tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    /// Check if a tool is available
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request =
            JsonRpcRequest::new(1, "test/method", Some(serde_json::json!({"key": "value"})));
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"test/method\""));
    }

    #[test]
    fn test_notification_has_no_id() {
        let notification = JsonRpcRequest::notification("notifications/initialized", None);
        assert!(notification.id.is_none());
    }

    #[test]
    fn test_response_is_error() {
        let success = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: Some(Value::Null),
            error: None,
        };
        assert!(!success.is_error());

        let error = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
        };
        assert!(error.is_error());
    }
}
