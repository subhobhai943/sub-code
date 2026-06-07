// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use thiserror::Error;

/// Unified error type used throughout the SUB CODE engine.
#[derive(Debug, Error)]
pub enum SubcodeError {
    /// An LLM backend returned an error or produced invalid output.
    #[error("LLM backend error: {0}")]
    Llm(String),

    /// A tool execution failed.
    #[error("Tool execution failed ({name}): {message}")]
    ToolExec { name: String, message: String },

    /// A shell command was blocked by the allowlist / denylist policy.
    #[error("Shell command denied: {0}")]
    ShellDenied(String),

    /// A shell command failed during execution.
    #[error("Shell error: {0}")]
    Shell(String),

    /// A git operation failed.
    #[error("Git error: {0}")]
    Git(String),

    /// A plugin operation failed.
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// A configuration error.
    #[error("Config error: {0}")]
    Config(String),

    /// Context indexing failed.
    #[error("Context indexing error: {0}")]
    Context(String),

    /// An I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialisation / deserialisation error.
    #[error("JSON serialisation error: {0}")]
    Json(#[from] serde_json::Error),
}
