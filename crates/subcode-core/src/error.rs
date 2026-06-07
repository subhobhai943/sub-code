// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubcodeError {
    #[error("LLM backend error: {0}")]
    Llm(String),

    #[error("Tool execution failed ({name}): {message}")]
    ToolExec { name: String, message: String },

    #[error("Shell command denied: {0}")]
    ShellDenied(String),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Context indexing error: {0}")]
    Context(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialisation error: {0}")]
    Json(#[from] serde_json::Error),
}
