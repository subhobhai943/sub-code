// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::shell::Shell;
use super::{Tool, ToolResult};

pub struct RunTestsTool;

#[async_trait]
impl Tool for RunTestsTool {
    fn name(&self) -> &str {
        "run_tests"
    }

    fn description(&self) -> &str {
        "Run the project's test suite"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "test_name": { "type": "string", "description": "Optional specific test name to run" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let test_name = params.get("test_name").and_then(|v| v.as_str()).unwrap_or("");
        
        let mut args = vec!["test"];
        if !test_name.is_empty() {
            args.push(test_name);
        }

        let shell = Shell::new(&ctx.config.shell);
        
        match shell.execute("cargo", &args, &ctx.root).await {
            Ok(out) => {
                let mut result_text = String::new();
                if let Some(code) = out.exit_code {
                    result_text.push_str(&format!("Exit Code: {}\n", code));
                }
                result_text.push_str(&format!("STDOUT:\n{}\nSTDERR:\n{}\n", out.stdout, out.stderr));
                Ok(ToolResult::ok(result_text))
            }
            Err(e) => Ok(ToolResult::err(format!("Test execution failed: {}", e))),
        }
    }
}
