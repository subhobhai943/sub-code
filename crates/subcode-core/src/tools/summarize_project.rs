// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use super::{Tool, ToolResult};

pub struct SummarizeProjectTool;

#[async_trait]
impl Tool for SummarizeProjectTool {
    fn name(&self) -> &str {
        "summarize_project"
    }

    fn description(&self) -> &str {
        "Provide a summary of the codebase structure and files"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "max_tokens": { "type": "number", "description": "Maximum tokens (approx) to return" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let max_tokens = params.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(32000) as usize;
        let summary = ctx.to_llm_string(max_tokens);
        Ok(ToolResult::ok(summary))
    }
}
