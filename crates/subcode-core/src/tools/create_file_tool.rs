// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct CreateFileTool;

#[async_trait]
impl Tool for CreateFileTool {
    fn name(&self) -> &str {
        "create_file_tool"
    }

    fn description(&self) -> &str {
        "Create a new file with the specified content"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the new file" },
                "content": { "type": "string", "description": "Initial content for the file" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or_default();

        if path.is_empty() {
            return Ok(ToolResult::err("Missing 'path' parameter"));
        }

        if std::path::Path::new(path).exists() {
            return Ok(ToolResult::err(format!("File already exists at {}", path)));
        }

        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(ToolResult::err(format!("Failed to create parent directories: {}", e)));
            }
        }

        match fs::write(path, content).await {
            Ok(_) => Ok(ToolResult::ok("File created successfully.")),
            Err(e) => Ok(ToolResult::err(format!("Failed to write file: {}", e))),
        }
    }
}
