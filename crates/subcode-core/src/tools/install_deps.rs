// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::shell::Shell;
use super::{Tool, ToolResult};

pub struct InstallDepsTool;

#[async_trait]
impl Tool for InstallDepsTool {
    fn name(&self) -> &str {
        "install_deps"
    }

    fn description(&self) -> &str {
        "Install project dependencies (e.g., cargo add / cargo build)"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "package": { "type": "string", "description": "Optional package to add" }
            }
        })
    }

    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let pkg = params.get("package").and_then(|v| v.as_str()).unwrap_or("");
        let shell = Shell::new(&ctx.config.shell);
        
        let mut args = Vec::new();
        if pkg.is_empty() {
            args.push("build");
        } else {
            args.push("add");
            args.push(pkg);
        }
        
        match shell.execute("cargo", &args, &ctx.root).await {
            Ok(out) => {
                let mut result_text = String::new();
                if let Some(code) = out.exit_code {
                    result_text.push_str(&format!("Exit Code: {}\n", code));
                }
                result_text.push_str(&format!("STDOUT:\n{}\nSTDERR:\n{}\n", out.stdout, out.stderr));
                Ok(ToolResult::ok(result_text))
            }
            Err(e) => Ok(ToolResult::err(format!("Failed to install deps: {}", e))),
        }
    }
}
