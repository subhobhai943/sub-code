// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};
use regex::Regex;

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct ExplainFunctionTool;

#[async_trait]
impl Tool for ExplainFunctionTool {
    fn name(&self) -> &str {
        "explain_function"
    }

    fn description(&self) -> &str {
        "Find and return a function definition for explanation"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file" },
                "function_name": { "type": "string", "description": "Name of the function to find" }
            },
            "required": ["path", "function_name"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let func_name = params.get("function_name").and_then(|v| v.as_str()).unwrap_or_default();

        if path.is_empty() || func_name.is_empty() {
            return Ok(ToolResult::err("Missing 'path' or 'function_name' parameter"));
        }

        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => return Ok(ToolResult::err(format!("Failed to read file: {}", e))),
        };

        // Very basic regex to find function. In reality, you'd use tree-sitter.
        let pattern = format!(r"fn\s+{}\s*\(", regex::escape(func_name));
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => return Ok(ToolResult::err("Invalid function name regex")),
        };

        let mut lines = content.lines().enumerate().peekable();
        let mut result = String::new();
        let mut inside = false;

        while let Some((idx, line)) = lines.next() {
            if !inside && re.is_match(line) {
                inside = true;
            }
            if inside {
                result.push_str(&format!("{}: {}\n", idx + 1, line));
                // Extremely naive block end detection (assumes no indent at end of func)
                if line.starts_with('}') {
                    break;
                }
            }
        }

        if result.is_empty() {
            Ok(ToolResult::err(format!("Function '{}' not found in file", func_name)))
        } else {
            Ok(ToolResult::ok(result))
        }
    }
}
