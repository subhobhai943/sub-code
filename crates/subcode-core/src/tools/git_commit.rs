// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct GitCommitTool;

#[async_trait]
impl Tool for GitCommitTool {
    fn name(&self) -> &str {
        "git_commit"
    }

    fn description(&self) -> &str {
        "Commit staged changes"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "The commit message" }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let msg = params.get("message").and_then(|v| v.as_str()).unwrap_or_default();
        if msg.is_empty() {
            return Ok(ToolResult::err("Missing 'message' parameter"));
        }

        let git = match GitManager::open(&ctx.root) {
            Ok(g) => g,
            Err(e) => return Ok(ToolResult::err(format!("Failed to open git repo: {}", e))),
        };

        match git.commit(msg) {
            Ok(oid) => Ok(ToolResult::ok(format!("Successfully committed with OID: {}", oid))),
            Err(e) => Ok(ToolResult::err(format!("Failed to commit: {}", e))),
        }
    }
}
