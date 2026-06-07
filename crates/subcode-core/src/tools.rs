// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::Value;
use crate::context::ProjectContext;
use crate::error::SubcodeError;

/// Unified result type for tool execution.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output:  String,
    pub error:   Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, output: String::new(), error: Some(msg.into()) }
    }
}

/// Base trait implemented by every tool.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Machine-readable name (used in prompts and tool routing).
    fn name(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// JSON Schema describing expected parameters.
    fn parameters(&self) -> Value;
    /// Execute the tool with JSON parameters and shared project context.
    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError>;
}

/// Placeholder registry for future tool set wiring.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,    
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.push(Box::new(tool));
    }

    pub fn all(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }
}
