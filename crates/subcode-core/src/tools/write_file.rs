// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Overwrite a file with new content"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file" },
                "content": { "type": "string", "description": "The complete new content of the file" }
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

        match fs::write(path, content).await {
            Ok(_) => Ok(ToolResult::ok("File written successfully.")),
            Err(e) => Ok(ToolResult::err(format!("Failed to write file: {}", e))),
        }
    }
}
