// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use walkdir::WalkDir;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct DebugErrorTool;

#[async_trait]
impl Tool for DebugErrorTool {
    fn name(&self) -> &str {
        "debug_error"
    }

    fn description(&self) -> &str {
        "Search the codebase for an error message or identifier to debug it"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "error_message": { "type": "string", "description": "The exact error message or related keyword to search for" }
            },
            "required": ["error_message"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let err_msg = params.get("error_message").and_then(|v| v.as_str()).unwrap_or_default();
        if err_msg.is_empty() {
            return Ok(ToolResult::err("Missing 'error_message' parameter"));
        }

        let mut results = String::new();
        let root = &ctx.root;

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(path) {
                    let mut file_matches = false;
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.contains(err_msg) {
                            if !file_matches {
                                results.push_str(&format!("\n--- {}\n", path.display()));
                                file_matches = true;
                            }
                            results.push_str(&format!("{}: {}\n", line_idx + 1, line));
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(ToolResult::ok("Error message not found in codebase."))
        } else {
            Ok(ToolResult::ok(results))
        }
    }
}
