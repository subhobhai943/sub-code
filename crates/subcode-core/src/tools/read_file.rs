// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the complete contents of a file"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to read" }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        if path.is_empty() {
            return Ok(ToolResult::err("Missing 'path' parameter"));
        }

        match fs::read_to_string(path).await {
            Ok(content) => Ok(ToolResult::ok(content)),
            Err(e) => Ok(ToolResult::err(format!("Failed to read file: {}", e))),
        }
    }
}
