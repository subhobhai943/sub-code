// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use walkdir::WalkDir;
use regex::Regex;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct SearchCodebaseTool;

#[async_trait]
impl Tool for SearchCodebaseTool {
    fn name(&self) -> &str {
        "search_codebase"
    }

    fn description(&self) -> &str {
        "Search the entire codebase for a regex pattern"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Regex pattern to search for" },
                "file_extension": { "type": "string", "description": "Optional file extension to filter by (e.g., 'rs')" }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let pattern_str = params.get("pattern").and_then(|v| v.as_str()).unwrap_or_default();
        let ext_filter = params.get("file_extension").and_then(|v| v.as_str());

        if pattern_str.is_empty() {
            return Ok(ToolResult::err("Missing 'pattern' parameter"));
        }

        let re = match Regex::new(pattern_str) {
            Ok(r) => r,
            Err(e) => return Ok(ToolResult::err(format!("Invalid regex pattern: {}", e))),
        };

        let mut results = String::new();
        let root = &ctx.root;

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = ext_filter {
                    if path.extension().and_then(|s| s.to_str()) != Some(ext) {
                        continue;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(path) {
                    let mut file_matches = false;
                    for (line_idx, line) in content.lines().enumerate() {
                        if re.is_match(line) {
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
            Ok(ToolResult::ok("No matches found."))
        } else {
            Ok(ToolResult::ok(results))
        }
    }
}
