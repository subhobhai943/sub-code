// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct DeleteFileTool;

#[async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file from the filesystem"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to delete" }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or_default();

        if path.is_empty() {
            return Ok(ToolResult::err("Missing 'path' parameter"));
        }

        match fs::remove_file(path).await {
            Ok(_) => Ok(ToolResult::ok("File deleted successfully.")),
            Err(e) => Ok(ToolResult::err(format!("Failed to delete file: {}", e))),
        }
    }
}
