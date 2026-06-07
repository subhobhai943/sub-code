// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::config::LlmConfig;
use super::{ChatMessage, LlmBackend, TokenEvent, TokenStream};

pub struct AnthropicBackend {
    client: Client,
    model:  String,
    temp:   f32,
}

impl AnthropicBackend {
    pub fn new(cfg: &LlmConfig) -> Result<Self> {
        let api_key = cfg.api_key.as_deref().unwrap_or("");
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(api_key)
                .map_err(|e| anyhow!("Invalid Anthropic API key: {e}"))?,
        );
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        let client = Client::builder()
            .timeout(Duration::from_secs(180))
            .default_headers(headers)
            .build()?;
        Ok(Self { client, model: cfg.model.clone(), temp: cfg.temperature })
    }
}

#[derive(Serialize)]
struct Request<'a> {
    model:      &'a str,
    max_tokens: usize,
    messages:   Vec<AnthropicMsg<'a>>,
    stream:     bool,
    temperature: f32,
}

#[derive(Serialize)]
struct AnthropicMsg<'a> {
    role:    &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    ev_type: String,
    delta:   Option<AnthropicDelta>,
}

#[derive(Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text:        Option<String>,
}

#[derive(Deserialize)]
struct CompletionResp {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[async_trait]
impl LlmBackend for AnthropicBackend {
    async fn chat_stream(&self, messages: Vec<ChatMessage>) -> Result<TokenStream> {
        const BASE: &str = "https://api.anthropic.com/v1";
        let url  = format!("{BASE}/messages");
        let msgs: Vec<AnthropicMsg> = messages.iter()
            .map(|m| AnthropicMsg { role: &m.role, content: &m.content })
            .collect();
        let body = Request {
            model:       &self.model,
            max_tokens:  4096,
            messages:    msgs,
            stream:      true,
            temperature: self.temp,
        };
        let resp = self.client.post(&url).json(&body).send().await
            .map_err(|e| anyhow!("Anthropic request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text   = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic returned {status}: {text}"));
        }

        let s = resp.bytes_stream().map(|chunk| -> Result<TokenEvent> {
            let chunk = chunk.map_err(|e| anyhow!("Stream error: {e}"))?;
            let text  = std::str::from_utf8(&chunk).map_err(|e| anyhow!("{e}"))?;
            let mut ev = TokenEvent { token: String::new(), done: false };
            for line in text.lines() {
                let line = line.trim();
                let line = line.strip_prefix("data: ").unwrap_or(line);
                if line.is_empty() { continue; }
                if let Ok(se) = serde_json::from_str::<StreamEvent>(line) {
                    if se.ev_type == "content_block_delta" {
                        if let Some(d) = se.delta {
                            if d.delta_type == "text_delta" {
                                ev.token.push_str(d.text.as_deref().unwrap_or(""));
                            }
                        }
                    } else if se.ev_type == "message_stop" {
                        ev.done = true;
                    }
                }
            }
            Ok(ev)
        });
        Ok(Box::pin(s))
    }

    async fn complete(&self, messages: Vec<ChatMessage>) -> Result<String> {
        const BASE: &str = "https://api.anthropic.com/v1";
        let url  = format!("{BASE}/messages");
        let msgs: Vec<AnthropicMsg> = messages.iter()
            .map(|m| AnthropicMsg { role: &m.role, content: &m.content })
            .collect();
        let body = Request {
            model:       &self.model,
            max_tokens:  4096,
            messages:    msgs,
            stream:      false,
            temperature: self.temp,
        };
        let resp: CompletionResp = self.client.post(&url).json(&body).send().await
            .map_err(|e| anyhow!("Anthropic request: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parsing Anthropic: {e}"))?;
        resp.content.into_iter().next()
            .map(|b| b.text)
            .ok_or_else(|| anyhow!("Empty Anthropic response"))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "claude-opus-4-5".into(),
            "claude-sonnet-4-5".into(),
            "claude-haiku-3-5".into(),
        ])
    }
}
