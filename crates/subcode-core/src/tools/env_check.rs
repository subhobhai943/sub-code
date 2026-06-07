// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::shell::Shell;
use super::{Tool, ToolResult};

pub struct EnvCheckTool;

#[async_trait]
impl Tool for EnvCheckTool {
    fn name(&self) -> &str {
        "env_check"
    }

    fn description(&self) -> &str {
        "Check environment tools (cargo version, rustc version, etc)"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let shell = Shell::new(&ctx.config.shell);
        
        let mut result_text = String::new();
        
        // Check rustc
        match shell.execute("rustc", &["--version"], &ctx.root).await {
            Ok(out) => result_text.push_str(&format!("rustc:\n{}\n", out.stdout)),
            Err(e) => result_text.push_str(&format!("rustc not found: {}\n", e)),
        }

        // Check cargo
        match shell.execute("cargo", &["--version"], &ctx.root).await {
            Ok(out) => result_text.push_str(&format!("cargo:\n{}\n", out.stdout)),
            Err(e) => result_text.push_str(&format!("cargo not found: {}\n", e)),
        }

        Ok(ToolResult::ok(result_text))
    }
}
