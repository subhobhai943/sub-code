// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT
// Handles: Ollama OpenAI-compat, LM Studio, llama.cpp server, OpenAI, OpenAI-compatible APIs.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::config::LlmConfig;
use super::{ChatMessage, LlmBackend, TokenEvent, TokenStream};

pub struct OpenAiBackend {
    client:   Client,
    base_url: String,
    model:    String,
    temp:     f32,
}

impl OpenAiBackend {
    pub fn new(cfg: &LlmConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        if let Some(key) = &cfg.api_key {
            let val = header::HeaderValue::from_str(&format!("Bearer {key}"))
                .map_err(|e| anyhow!("Invalid API key header: {e}"))?;
            headers.insert(header::AUTHORIZATION, val);
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(180))
            .default_headers(headers)
            .build()?;
        let base = match cfg.backend.as_str() {
            "ollama"   => format!("{}/v1", cfg.endpoint.trim_end_matches('/')),
            "lmstudio" => format!("{}/v1", cfg.endpoint.trim_end_matches('/')),
            "llamacpp" => format!("{}/v1", cfg.endpoint.trim_end_matches('/')),
            _          => cfg.endpoint.trim_end_matches('/').to_owned(),
        };
        Ok(Self { client, base_url: base, model: cfg.model.clone(), temp: cfg.temperature })
    }
}

#[derive(Serialize)]
struct Request<'a> {
    model:    &'a str,
    messages: &'a [ChatMessage],
    stream:   bool,
    temperature: f32,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}
#[derive(Deserialize)]
struct StreamChoice {
    delta: Delta,
    #[serde(default)]
    finish_reason: Option<String>,
}
#[derive(Deserialize, Default)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct CompletionResp {
    choices: Vec<CompletionChoice>,
}
#[derive(Deserialize)]
struct CompletionChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ModelsResp {
    data: Vec<ModelEntry>,
}
#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

#[async_trait]
impl LlmBackend for OpenAiBackend {
    async fn chat_stream(&self, messages: Vec<ChatMessage>) -> Result<TokenStream> {
        let url  = format!("{}/chat/completions", self.base_url);
        let body = Request { model: &self.model, messages: &messages, stream: true, temperature: self.temp };
        let resp = self.client.post(&url).json(&body).send().await
            .map_err(|e| anyhow!("OpenAI stream request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text   = resp.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI returned {status}: {text}"));
        }

        let s = resp.bytes_stream().map(|chunk| -> Result<TokenEvent> {
            let chunk = chunk.map_err(|e| anyhow!("Stream error: {e}"))?;
            let text  = std::str::from_utf8(&chunk)
                .map_err(|e| anyhow!("UTF-8 error: {e}"))?;
            let mut out = TokenEvent { token: String::new(), done: false };
            for line in text.lines() {
                let line = line.trim();
                if line == "data: [DONE]" { out.done = true; continue; }
                let line = line.strip_prefix("data: ").unwrap_or(line);
                if line.is_empty() { continue; }
                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(line) {
                    for choice in chunk.choices {
                        if let Some(c) = choice.delta.content { out.token.push_str(&c); }
                        if choice.finish_reason.is_some() { out.done = true; }
                    }
                }
            }
            Ok(out)
        });
        Ok(Box::pin(s))
    }

    async fn complete(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let url  = format!("{}/chat/completions", self.base_url);
        let body = Request { model: &self.model, messages: &messages, stream: false, temperature: self.temp };
        let resp: CompletionResp = self.client.post(&url).json(&body).send().await
            .map_err(|e| anyhow!("OpenAI request failed: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parsing OpenAI response: {e}"))?;
        resp.choices.into_iter().next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow!("Empty response from OpenAI"))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url  = format!("{}/models", self.base_url);
        let resp: ModelsResp = self.client.get(&url).send().await
            .map_err(|e| anyhow!("List models failed: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parsing models: {e}"))?;
        Ok(resp.data.into_iter().map(|m| m.id).collect())
    }
}
