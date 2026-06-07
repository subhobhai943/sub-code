// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::ProjectContext;
use crate::error::SubcodeError;
use crate::shell::Shell;
use super::{Tool, ToolResult};

pub struct GenerateDocsTool;

#[async_trait]
impl Tool for GenerateDocsTool {
    fn name(&self) -> &str {
        "generate_docs"
    }

    fn description(&self) -> &str {
        "Generate project documentation"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let shell = Shell::new(&ctx.config.shell);
        
        let args = vec!["doc", "--no-deps"];
        
        match shell.execute("cargo", &args, &ctx.root).await {
            Ok(out) => {
                let mut result_text = String::new();
                if let Some(code) = out.exit_code {
                    result_text.push_str(&format!("Exit Code: {}\n", code));
                }
                result_text.push_str(&format!("STDOUT:\n{}\nSTDERR:\n{}\n", out.stdout, out.stderr));
                Ok(ToolResult::ok(result_text))
            }
            Err(e) => Ok(ToolResult::err(format!("Doc generation failed: {}", e))),
        }
    }
}
