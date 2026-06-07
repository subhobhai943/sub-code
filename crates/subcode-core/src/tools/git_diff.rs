// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct GitDiffTool;

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn description(&self) -> &str {
        "View unstaged git differences"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "staged": { "type": "boolean", "description": "If true, show staged diff instead" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let staged = params.get("staged").and_then(|v| v.as_bool()).unwrap_or(false);

        let git = match GitManager::open(&ctx.root) {
            Ok(g) => g,
            Err(e) => return Ok(ToolResult::err(format!("Failed to open git repo: {}", e))),
        };

        let result = if staged {
            git.diff_staged()
        } else {
            git.diff()
        };

        match result {
            Ok(diff) => {
                if diff.trim().is_empty() {
                    Ok(ToolResult::ok("No changes."))
                } else {
                    Ok(ToolResult::ok(diff))
                }
            }
            Err(e) => Ok(ToolResult::err(format!("Git diff failed: {}", e))),
        }
    }
}
