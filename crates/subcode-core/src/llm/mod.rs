// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

mod ollama;
mod openai;
mod anthropic;

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use crate::config::Config;

/// A token event streamed from the LLM.
#[derive(Debug, Clone)]
pub struct TokenEvent {
    pub token: String,
    pub done:  bool,
}

/// A single message in the conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role:    String,
    pub content: String,
}

/// Boxed token stream.
pub type TokenStream = Pin<Box<dyn Stream<Item = Result<TokenEvent>> + Send>>;

/// Trait every LLM backend must implement.
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Send messages and receive a streaming token response.
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<TokenStream>;

    /// Blocking (non-streaming) completion helper.
    async fn complete(&self, messages: Vec<ChatMessage>) -> Result<String>;

    /// List models available on this backend.
    async fn list_models(&self) -> Result<Vec<String>>;
}

/// Routes LLM requests to the configured backend.
pub struct LlmRouter {
    backend: Box<dyn LlmBackend>,
}

impl LlmRouter {
    pub async fn from_config(config: &Config) -> Result<Self> {
        let backend: Box<dyn LlmBackend> = match config.llm.backend.as_str() {
            "ollama" | "lmstudio" | "llamacpp" | "openai" => {
                Box::new(openai::OpenAiBackend::new(&config.llm)?)
            }
            "anthropic" => {
                Box::new(anthropic::AnthropicBackend::new(&config.llm)?)
            }
            other => {
                // Fall back to Ollama-compatible API.
                tracing::warn!("Unknown backend '{}', falling back to ollama", other);
                Box::new(ollama::OllamaBackend::new(&config.llm)?)
            }
        };
        Ok(Self { backend })
    }

    /// Stream tokens from the backend.
    pub async fn chat_stream(&self, messages: Vec<ChatMessage>) -> Result<TokenStream> {
        self.backend.chat_stream(messages).await
    }

    /// Convenience: complete without streaming (used for commit messages etc.).
    pub async fn complete_simple(&self, prompt: &str) -> Result<String> {
        self.backend.complete(vec![
            ChatMessage { role: "user".into(), content: prompt.into() },
        ]).await
    }

    /// List models on the backend.
    pub async fn list_models(&self) -> Result<Vec<String>> {
        self.backend.list_models().await
    }
}
