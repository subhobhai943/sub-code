// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;
use std::time::Duration;
use crate::config::LlmConfig;
use super::{ChatMessage, LlmBackend, TokenEvent, TokenStream};

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model:    &'a str,
    messages: &'a [ChatMessage],
    stream:   bool,
    options:  OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature:  f32,
    num_ctx:      usize,
}

#[derive(Deserialize, Debug)]
struct OllamaStreamChunk {
    message: Option<OllamaMsg>,
    done:    bool,
}

#[derive(Deserialize, Debug)]
struct OllamaMsg {
    content: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

pub struct OllamaBackend {
    client:   Client,
    endpoint: String,
    model:    String,
    temp:     f32,
    ctx:      usize,
}

impl OllamaBackend {
    pub fn new(cfg: &LlmConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self {
            client,
            endpoint: cfg.endpoint.trim_end_matches('/').to_owned(),
            model:    cfg.model.clone(),
            temp:     cfg.temperature,
            ctx:      cfg.context_window,
        })
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    async fn chat_stream(&self, messages: Vec<ChatMessage>) -> Result<TokenStream> {
        let url = format!("{}/api/chat", self.endpoint);
        let body = OllamaRequest {
            model:    &self.model,
            messages: &messages,
            stream:   true,
            options:  OllamaOptions { temperature: self.temp, num_ctx: self.ctx },
        };

        let resp = self.client.post(&url).json(&body).send().await
            .map_err(|e| anyhow!("Ollama connection failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text   = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama returned {status}: {text}"));
        }

        let byte_stream = resp.bytes_stream();
        let s = byte_stream.map(move |chunk| -> Result<TokenEvent> {
            let chunk = chunk.map_err(|e| anyhow!("Stream error: {e}"))?;
            let text  = std::str::from_utf8(&chunk)
                .map_err(|e| anyhow!("UTF-8 decode error: {e}"))?;

            // Each NDJSON line is one chunk.
            let mut last = TokenEvent { token: String::new(), done: false };
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                match serde_json::from_str::<OllamaStreamChunk>(line) {
                    Ok(ev) => {
                        last.token.push_str(ev.message.map(|m| m.content).unwrap_or_default().as_str());
                        last.done = ev.done;
                    }
                    Err(_) => {} // skip malformed lines
                }
            }
            Ok(last)
        });

        Ok(Box::pin(s))
    }

    async fn complete(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let url = format!("{}/api/chat", self.endpoint);
        let body = OllamaRequest {
            model:    &self.model,
            messages: &messages,
            stream:   false,
            options:  OllamaOptions { temperature: self.temp, num_ctx: self.ctx },
        };

        #[derive(Deserialize)]
        struct NonStreamResp {
            message: OllamaMsg,
        }

        let resp = self.client.post(&url).json(&body).send().await
            .map_err(|e| anyhow!("Ollama connection failed: {e}"))?;
        let parsed: NonStreamResp = resp.json().await
            .map_err(|e| anyhow!("Parsing Ollama response: {e}"))?;
        Ok(parsed.message.content)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url  = format!("{}/api/tags", self.endpoint);
        let resp: OllamaTagsResponse = self.client.get(&url).send().await
            .map_err(|e| anyhow!("Ollama list_models: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parsing models: {e}"))?;
        Ok(resp.models.into_iter().map(|m| m.name).collect())
    }
}
