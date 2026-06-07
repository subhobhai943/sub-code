// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct GitPrSummaryTool;

#[async_trait]
impl Tool for GitPrSummaryTool {
    fn name(&self) -> &str {
        "git_pr_summary"
    }

    fn description(&self) -> &str {
        "Return the current diff suitable for generating a PR summary"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let git = match GitManager::open(&ctx.root) {
            Ok(g) => g,
            Err(e) => return Ok(ToolResult::err(format!("Failed to open git repo: {}", e))),
        };

        match git.diff() {
            Ok(diff) => {
                if diff.trim().is_empty() {
                    Ok(ToolResult::ok("No changes found for PR summary."))
                } else {
                    Ok(ToolResult::ok(diff))
                }
            }
            Err(e) => Ok(ToolResult::err(format!("Failed to get diff for PR summary: {}", e))),
        }
    }
}
