// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct RefactorTool;

#[async_trait]
impl Tool for RefactorTool {
    fn name(&self) -> &str {
        "refactor"
    }

    fn description(&self) -> &str {
        "Refactor a block of code within a file"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file" },
                "search": { "type": "string", "description": "Original code block to replace" },
                "replace": { "type": "string", "description": "New refactored code block" }
            },
            "required": ["path", "search", "replace"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let search = params.get("search").and_then(|v| v.as_str()).unwrap_or_default();
        let replace = params.get("replace").and_then(|v| v.as_str()).unwrap_or_default();

        if path.is_empty() || search.is_empty() {
            return Ok(ToolResult::err("Missing 'path' or 'search' parameter"));
        }

        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::err(format!("Failed to read file: {}", e))),
        };

        if !content.contains(search) {
            return Ok(ToolResult::err("Code block to refactor not found in file."));
        }

        let new_content = content.replace(search, replace);

        match tokio::fs::write(path, new_content).await {
            Ok(_) => Ok(ToolResult::ok("File refactored successfully.")),
            Err(e) => Ok(ToolResult::err(format!("Failed to write file: {}", e))),
        }
    }
}
