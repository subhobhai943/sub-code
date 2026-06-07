// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use crate::config::Config;
use crate::error::SubcodeError;
use crate::llm::{ChatMessage, LlmRouter};
use crate::tools::{Tool, ToolResult};
use crate::context::ProjectContext;
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Agent operating mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AgentMode {
    /// Ask before executing shell commands and applying edits.
    Interactive,
    /// Execute without prompting the user.
    Yolo,
}

impl AgentMode {
    pub fn from_config(cfg: &Config, yolo_flag: bool) -> Self {
        if yolo_flag || cfg.agent.mode.eq_ignore_ascii_case("yolo") {
            AgentMode::Yolo
        } else {
            AgentMode::Interactive
        }
    }
}

/// Minimal UI abstraction so the core agent is UI-agnostic.
#[async_trait]
pub trait AgentUi: Send {
    /// Stream a token to the user interface.
    async fn on_token(&mut self, token: &str) -> Result<(), SubcodeError>;

    /// Show a status line or log message.
    async fn on_status(&mut self, status: &str) -> Result<(), SubcodeError>;

    /// Ask the user to confirm a potentially destructive action.
    async fn confirm(&mut self, prompt: &str) -> Result<bool, SubcodeError>;
}

/// High-level agent entrypoint used by the CLI and TUI.
pub struct AgentRunner {
    config: Config,
    ctx:    Arc<ProjectContext>,
    llm:    Arc<LlmRouter>,
    mode:   AgentMode,
}

impl AgentRunner {
    /// Create a new agent runner.
    pub fn new(config: Config, ctx: ProjectContext, llm: LlmRouter, yolo: bool) -> Self {
        let mode = AgentMode::from_config(&config, yolo);
        Self {
            config,
            ctx: Arc::new(ctx),
            llm: Arc::new(llm),
            mode,
        }
    }

    /// One-shot question handler used by `subcode ask` and `summary`.
    pub async fn one_shot(&self, question: &str) -> Result<(), SubcodeError> {
        let mut messages = Vec::new();

        // System prompt with lightweight project context.
        let ctx_summary = self.ctx.to_llm_string(self.config.context.max_tokens);
        let system = format!(
            "You are SUB CODE, a terminal-native Rust coding assistant.\\n\
             Project context:\\n{}",
            ctx_summary
        );
        messages.push(ChatMessage { role: "system".into(), content: system });
        messages.push(ChatMessage { role: "user".into(), content: question.to_string() });

        // Stream tokens directly to stdout for now; TUI implements its own Adapter.
        let mut stdout_ui = StdoutUi;
        self.stream_chat(messages, &mut stdout_ui).await
    }

    /// Run a longer task (currently identical to one_shot, but wired for future ReAct steps).
    pub async fn run_task(&self, task: &str) -> Result<(), SubcodeError> {
        self.one_shot(task).await
    }

    async fn stream_chat<U: AgentUi + ?Sized>(
        &self,
        messages: Vec<ChatMessage>,
        ui: &mut U,
    ) -> Result<(), SubcodeError> {
        ui.on_status("Connecting to LLM backend...").await?;
        let stream = self
            .llm
            .chat_stream(messages)
            .await
            .map_err(|e| SubcodeError::Llm(e.to_string()))?;

        tokio::pin!(stream);
        while let Some(evt) = stream
            .next()
            .await
        {
            let evt = evt.map_err(|e| SubcodeError::Llm(e.to_string()))?;
            if !evt.token.is_empty() {
                ui.on_token(&evt.token).await?;
            }
            if evt.done {
                break;
            }
        }
        Ok(())
    }
}

struct StdoutUi;

#[async_trait]
impl AgentUi for StdoutUi {
    async fn on_token(&mut self, token: &str) -> Result<(), SubcodeError> {
        use tokio::io::AsyncWriteExt;
        let mut stdout = tokio::io::stdout();
        stdout.write_all(token.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    }

    async fn on_status(&mut self, _status: &str) -> Result<(), SubcodeError> {
        // For non-TUI mode we keep status minimal to avoid noisy output.
        Ok(())
    }

    async fn confirm(&mut self, _prompt: &str) -> Result<bool, SubcodeError> {
        // Non-interactive stdout UI cannot confirm; deny by default.
        Ok(false)
    }
}
