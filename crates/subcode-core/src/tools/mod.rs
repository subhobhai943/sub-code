// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use async_trait::async_trait;
use serde_json::Value;
use crate::context::ProjectContext;
use crate::error::SubcodeError;

/// Unified result type for tool execution.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output:  String,
    pub error:   Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, output: String::new(), error: Some(msg.into()) }
    }
}

/// Base trait implemented by every tool.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Machine-readable name (used in prompts and tool routing).
    fn name(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// JSON Schema describing expected parameters.
    fn parameters(&self) -> Value;
    /// Execute the tool with JSON parameters and shared project context.
    async fn execute(&self, params: Value, ctx: &ProjectContext) -> Result<ToolResult, SubcodeError>;
}

/// Placeholder registry for future tool set wiring.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,    
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.push(Box::new(tool));
    }

    pub fn all(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }
}

pub mod read_file;
pub mod write_file;
pub mod edit_file;
pub mod search_codebase;
pub mod run_command;
pub mod run_tests;
pub mod lint_fix;
pub mod explain_function;
pub mod refactor;
pub mod generate_docs;
pub mod review_code;
pub mod debug_error;
pub mod create_file_tool;
pub mod delete_file;
pub mod move_file;
pub mod git_status;
pub mod git_diff;
pub mod git_stage;
pub mod git_commit;
pub mod git_pr_summary;
pub mod git_branch;
pub mod git_log;
pub mod install_deps;
pub mod env_check;
pub mod summarize_project;

impl ToolRegistry {
    pub fn register_all(&mut self) {
        self.register(read_file::ReadFileTool);
        self.register(write_file::WriteFileTool);
        self.register(edit_file::EditFileTool);
        self.register(search_codebase::SearchCodebaseTool);
        self.register(run_command::RunCommandTool);
        self.register(run_tests::RunTestsTool);
        self.register(lint_fix::LintFixTool);
        self.register(explain_function::ExplainFunctionTool);
        self.register(refactor::RefactorTool);
        self.register(generate_docs::GenerateDocsTool);
        self.register(review_code::ReviewCodeTool);
        self.register(debug_error::DebugErrorTool);
        self.register(create_file_tool::CreateFileTool);
        self.register(delete_file::DeleteFileTool);
        self.register(move_file::MoveFileTool);
        self.register(git_status::GitStatusTool);
        self.register(git_diff::GitDiffTool);
        self.register(git_stage::GitStageTool);
        self.register(git_commit::GitCommitTool);
        self.register(git_pr_summary::GitPrSummaryTool);
        self.register(git_branch::GitBranchTool);
        self.register(git_log::GitLogTool);
        self.register(install_deps::InstallDepsTool);
        self.register(env_check::EnvCheckTool);
        self.register(summarize_project::SummarizeProjectTool);
    }
}


pub use read_file::ReadFileTool;
pub use write_file::WriteFileTool;
pub use edit_file::EditFileTool;
pub use search_codebase::SearchCodebaseTool;
pub use run_command::RunCommandTool;
pub use run_tests::RunTestsTool;
pub use lint_fix::LintFixTool;
pub use explain_function::ExplainFunctionTool;
pub use refactor::RefactorTool;
pub use generate_docs::GenerateDocsTool;
pub use review_code::ReviewCodeTool;
pub use debug_error::DebugErrorTool;
pub use create_file_tool::CreateFileTool;
pub use delete_file::DeleteFileTool;
pub use move_file::MoveFileTool;
pub use git_status::GitStatusTool;
pub use git_diff::GitDiffTool;
pub use git_stage::GitStageTool;
pub use git_commit::GitCommitTool;
pub use git_pr_summary::GitPrSummaryTool;
pub use git_branch::GitBranchTool;
pub use git_log::GitLogTool;
pub use install_deps::InstallDepsTool;
pub use env_check::EnvCheckTool;
pub use summarize_project::SummarizeProjectTool;

