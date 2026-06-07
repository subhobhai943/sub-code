// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::shell::Shell;
use super::{Tool, ToolResult};

pub struct RunCommandTool;

#[async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Run a shell command within the project context"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The command executable to run" },
                "args": { 
                    "type": "array", 
                    "items": { "type": "string" },
                    "description": "Arguments to pass to the command" 
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let cmd = params.get("command").and_then(|v| v.as_str()).unwrap_or_default();
        
        if cmd.is_empty() {
            return Ok(ToolResult::err("Missing 'command' parameter"));
        }

        let mut args_vec = Vec::new();
        if let Some(arr) = params.get("args").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    args_vec.push(s);
                }
            }
        }

        let shell = Shell::new(&ctx.config.shell);
        
        match shell.execute(cmd, &args_vec, &ctx.root).await {
            Ok(out) => {
                let mut result_text = String::new();
                if let Some(code) = out.exit_code {
                    result_text.push_str(&format!("Exit Code: {}\n", code));
                }
                if !out.stdout.is_empty() {
                    result_text.push_str(&format!("STDOUT:\n{}\n", out.stdout));
                }
                if !out.stderr.is_empty() {
                    result_text.push_str(&format!("STDERR:\n{}\n", out.stderr));
                }
                Ok(ToolResult::ok(result_text))
            }
            Err(e) => Ok(ToolResult::err(format!("Execution failed: {}", e))),
        }
    }
}
