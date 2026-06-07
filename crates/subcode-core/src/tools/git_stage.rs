// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::git::GitManager;
use super::{Tool, ToolResult};

pub struct GitStageTool;

#[async_trait]
impl Tool for GitStageTool {
    fn name(&self) -> &str {
        "git_stage"
    }

    fn description(&self) -> &str {
        "Stage files for commit"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "all": { "type": "boolean", "description": "If true, stage all changes" },
                "paths": { "type": "array", "items": { "type": "string" }, "description": "Specific files to stage" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let all = params.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
        
        let git = match GitManager::open(&ctx.root) {
            Ok(g) => g,
            Err(e) => return Ok(ToolResult::err(format!("Failed to open git repo: {}", e))),
        };

        if all {
            match git.stage_all() {
                Ok(_) => return Ok(ToolResult::ok("All changes staged successfully.")),
                Err(e) => return Ok(ToolResult::err(format!("Failed to stage all: {}", e))),
            }
        }

        let mut paths = Vec::new();
        if let Some(arr) = params.get("paths").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    paths.push(std::path::PathBuf::from(s));
                }
            }
        }

        if paths.is_empty() {
            return Ok(ToolResult::err("Must specify either 'all: true' or a list of 'paths'."));
        }

        match git.stage_files(&paths) {
            Ok(_) => Ok(ToolResult::ok("Specified files staged successfully.")),
            Err(e) => Ok(ToolResult::err(format!("Failed to stage files: {}", e))),
        }
    }
}
