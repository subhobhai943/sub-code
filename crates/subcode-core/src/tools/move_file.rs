// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct MoveFileTool;

#[async_trait]
impl Tool for MoveFileTool {
    fn name(&self) -> &str {
        "move_file"
    }

    fn description(&self) -> &str {
        "Move or rename a file"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "source": { "type": "string", "description": "Current path of the file" },
                "destination": { "type": "string", "description": "New path for the file" }
            },
            "required": ["source", "destination"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let source = params.get("source").and_then(|v| v.as_str()).unwrap_or_default();
        let destination = params.get("destination").and_then(|v| v.as_str()).unwrap_or_default();

        if source.is_empty() || destination.is_empty() {
            return Ok(ToolResult::err("Missing 'source' or 'destination' parameter"));
        }

        if let Some(parent) = std::path::Path::new(destination).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(ToolResult::err(format!("Failed to create parent directories for destination: {}", e)));
            }
        }

        match fs::rename(source, destination).await {
            Ok(_) => Ok(ToolResult::ok("File moved successfully.")),
            Err(e) => Ok(ToolResult::err(format!("Failed to move file: {}", e))),
        }
    }
}
