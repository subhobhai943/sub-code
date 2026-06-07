// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct GitLogTool;

#[async_trait]
impl Tool for GitLogTool {
    fn name(&self) -> &str {
        "git_log"
    }

    fn description(&self) -> &str {
        "View recent git commits"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "count": { "type": "number", "description": "Number of commits to show (default: 10)" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let count = params.get("count").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let git = match GitManager::open(&ctx.root) {
            Ok(g) => g,
            Err(e) => return Ok(ToolResult::err(format!("Failed to open git repo: {}", e))),
        };

        match git.log(count) {
            Ok(log_output) => {
                if log_output.trim().is_empty() {
                    Ok(ToolResult::ok("No commits found."))
                } else {
                    Ok(ToolResult::ok(log_output))
                }
            }
            Err(e) => Ok(ToolResult::err(format!("Failed to get git log: {}", e))),
        }
    }
}
