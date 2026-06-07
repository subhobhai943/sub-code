// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Loaded from ~/.subcode/config.toml, merged with .subcode.toml in cwd.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub backend: String,
    pub endpoint: String,
    pub model: String,
    pub context_window: usize,
    pub temperature: f32,
    pub stream: bool,
    pub api_key: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend:        "ollama".into(),
            endpoint:       "http://localhost:11434".into(),
            model:          "llama3.2:8b".into(),
            context_window: 128_000,
            temperature:    0.2,
            stream:         true,
            api_key:        None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub mode: String,
    pub approval_timeout_secs: u64,
    pub max_steps: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            mode:                    "interactive".into(),
            approval_timeout_secs:  30,
            max_steps:              25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_tokens:  usize,
    pub embeddings:  bool,
    pub watch:       bool,
    pub ignore_file: String,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens:  32_000,
            embeddings:  false,
            watch:       true,
            ignore_file: ".subcodeignore".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellConfig {
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub denylist:  Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme:            String,
    pub show_reasoning:   bool,
    pub syntax_highlight: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme:            "dark".into(),
            show_reasoning:   true,
            syntax_highlight: true,
        }
    }
}

impl Config {
    /// Load global config, then overlay project-local config if present.
    pub async fn load() -> Result<Self> {
        let global = Self::load_global().await.unwrap_or_default();
        let local  = Self::load_local().await.unwrap_or_default();
        Ok(Self::merge(global, local))
    }

    async fn load_global() -> Result<Self> {
        let path = global_config_path()?;
        if path.exists() {
            let raw = tokio::fs::read_to_string(&path).await
                .with_context(|| format!("Reading {}", path.display()))?;
            let cfg: Self = toml::from_str(&raw)
                .with_context(|| format!("Parsing {}", path.display()))?;
            Ok(cfg)
        } else {
            Ok(Self::default())
        }
    }

    async fn load_local() -> Result<Self> {
        let path = PathBuf::from(".subcode.toml");
        if path.exists() {
            let raw = tokio::fs::read_to_string(&path).await
                .with_context(|| "Reading .subcode.toml".to_string())?;
            let cfg: Self = toml::from_str(&raw)
                .with_context(|| "Parsing .subcode.toml".to_string())?;
            Ok(cfg)
        } else {
            Ok(Self::default())
        }
    }

    /// Local config values override global config values.
    fn merge(global: Self, local: Self) -> Self {
        // Simple field-level merge — local wins for non-default values.
        Self {
            project: if local.project.name.is_some() { local.project } else { global.project },
            llm:     local.llm,
            agent:   local.agent,
            context: local.context,
            shell:   if !local.shell.allowlist.is_empty() { local.shell } else { global.shell },
            plugins: if !local.plugins.enabled.is_empty() { local.plugins } else { global.plugins },
            ui:      local.ui,
        }
    }
}

pub fn global_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve home directory"))?;
    Ok(home.join(".subcode").join("config.toml"))
}
