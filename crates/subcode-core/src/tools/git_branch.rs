// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct GitBranchTool;

#[async_trait]
impl Tool for GitBranchTool {
    fn name(&self) -> &str {
        "git_branch"
    }

    fn description(&self) -> &str {
        "List or create branches"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Optional branch name to create" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let git = match GitManager::open(&ctx.root) {
            Ok(g) => g,
            Err(e) => return Ok(ToolResult::err(format!("Failed to open git repo: {}", e))),
        };

        if name.is_empty() {
            match git.list_branches() {
                Ok(branches) => Ok(ToolResult::ok(branches.join("\n"))),
                Err(e) => Ok(ToolResult::err(format!("Failed to list branches: {}", e))),
            }
        } else {
            match git.create_branch(name) {
                Ok(_) => Ok(ToolResult::ok(format!("Successfully created branch: {}", name))),
                Err(e) => Ok(ToolResult::err(format!("Failed to create branch: {}", e))),
            }
        }
    }
}
