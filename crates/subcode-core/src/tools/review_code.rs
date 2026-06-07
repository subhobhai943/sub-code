// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct ReviewCodeTool;

#[async_trait]
impl Tool for ReviewCodeTool {
    fn name(&self) -> &str {
        "review_code"
    }

    fn description(&self) -> &str {
        "Review the current changes (unstaged and staged diffs)"
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

        let mut diffs = String::new();
        
        if let Ok(staged) = git.diff_staged() {
            if !staged.trim().is_empty() {
                diffs.push_str("--- Staged Changes ---\n");
                diffs.push_str(&staged);
                diffs.push_str("\n\n");
            }
        }

        if let Ok(unstaged) = git.diff() {
            if !unstaged.trim().is_empty() {
                diffs.push_str("--- Unstaged Changes ---\n");
                diffs.push_str(&unstaged);
            }
        }

        if diffs.trim().is_empty() {
            Ok(ToolResult::ok("No changes to review."))
        } else {
            Ok(ToolResult::ok(diffs))
        }
    }
}
