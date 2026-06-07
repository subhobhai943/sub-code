// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

//! Plugin management for SUB CODE.
//!
//! Plugins live under `~/.subcode/plugins/<name>/`. Each plugin directory
//! contains a `plugin.toml` manifest describing the plugin metadata and
//! required permissions.
//!
//! This module handles discovery, installation (scaffold), listing, removal,
//! and manifest loading. Native and WASM plugin *execution* is handled
//! separately once the plugin SDK stabilises.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::SubcodeError;

/// Describes a single plugin, read from `plugin.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Human-readable plugin name (must match the directory name).
    pub name: String,
    /// Semantic version string (e.g. `"0.1.0"`).
    pub version: String,
    /// Plugin author.
    pub author: String,
    /// One-line description of what the plugin does.
    pub description: String,
    /// List of permission strings the plugin requires (e.g. `"fs:read"`, `"shell:exec"`).
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Manages the lifecycle of SUB CODE plugins.
pub struct PluginManager {
    /// Root directory for all plugins (`~/.subcode/plugins`).
    plugins_dir: PathBuf,
    /// Names of plugins that are explicitly enabled in config.
    enabled: Vec<String>,
}

impl PluginManager {
    /// Create a new plugin manager from the user's configuration.
    pub fn new(config: &Config) -> Self {
        let plugins_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".subcode")
            .join("plugins");
        Self {
            plugins_dir,
            enabled: config.plugins.enabled.clone(),
        }
    }

    /// Install a plugin by name.
    ///
    /// Currently this creates the plugin scaffold directory and writes a
    /// default `plugin.toml` manifest. Future versions will download from
    /// a registry or a git repository.
    pub async fn install(&self, name: &str) -> Result<(), SubcodeError> {
        let plugin_dir = self.plugins_dir.join(name);

        if plugin_dir.exists() {
            return Err(SubcodeError::Plugin(format!(
                "plugin '{name}' is already installed at {}",
                plugin_dir.display()
            )));
        }

        tokio::fs::create_dir_all(&plugin_dir)
            .await
            .map_err(|e| {
                SubcodeError::Plugin(format!(
                    "failed to create plugin directory {}: {e}",
                    plugin_dir.display()
                ))
            })?;

        let manifest = PluginManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            author: String::new(),
            description: format!("SUB CODE plugin: {name}"),
            permissions: Vec::new(),
        };

        let toml_string = toml::to_string_pretty(&manifest).map_err(|e| {
            SubcodeError::Plugin(format!("failed to serialise manifest: {e}"))
        })?;

        let manifest_path = plugin_dir.join("plugin.toml");
        tokio::fs::write(&manifest_path, toml_string)
            .await
            .map_err(|e| {
                SubcodeError::Plugin(format!(
                    "failed to write {}: {e}",
                    manifest_path.display()
                ))
            })?;

        tracing::info!("Installed plugin '{name}' → {}", plugin_dir.display());
        Ok(())
    }

    /// List all installed plugins by reading their manifests.
    pub fn list(&self) -> Result<Vec<PluginManifest>, SubcodeError> {
        if !self.plugins_dir.exists() {
            return Ok(Vec::new());
        }

        let mut manifests = Vec::new();
        let entries = std::fs::read_dir(&self.plugins_dir).map_err(|e| {
            SubcodeError::Plugin(format!(
                "failed to read plugins directory {}: {e}",
                self.plugins_dir.display()
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                SubcodeError::Plugin(format!("directory entry error: {e}"))
            })?;

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                tracing::warn!(
                    "Skipping plugin directory {} — no plugin.toml",
                    path.display()
                );
                continue;
            }

            match self.read_manifest(&manifest_path) {
                Ok(m) => manifests.push(m),
                Err(e) => {
                    tracing::warn!(
                        "Skipping plugin {}: {e}",
                        path.display()
                    );
                }
            }
        }

        Ok(manifests)
    }

    /// Remove an installed plugin by name.
    pub fn remove(&self, name: &str) -> Result<(), SubcodeError> {
        let plugin_dir = self.plugins_dir.join(name);

        if !plugin_dir.exists() {
            return Err(SubcodeError::Plugin(format!(
                "plugin '{name}' is not installed"
            )));
        }

        std::fs::remove_dir_all(&plugin_dir).map_err(|e| {
            SubcodeError::Plugin(format!(
                "failed to remove plugin directory {}: {e}",
                plugin_dir.display()
            ))
        })?;

        tracing::info!("Removed plugin '{name}'");
        Ok(())
    }

    /// Load all enabled plugins.
    ///
    /// This reads the manifests for every plugin listed in
    /// `config.plugins.enabled` and logs their metadata. Actual dynamic
    /// loading (native shared libraries via `libloading` or WASM modules
    /// via `wasmtime`) will be wired once the plugin SDK provides stable
    /// ABI definitions.
    pub fn load_all(&self) -> Result<Vec<PluginManifest>, SubcodeError> {
        let installed = self.list()?;
        let mut loaded = Vec::new();

        for manifest in installed {
            if !self.enabled.is_empty()
                && !self.enabled.iter().any(|e| e == &manifest.name)
            {
                tracing::debug!("Plugin '{}' not enabled — skipping", manifest.name);
                continue;
            }

            tracing::info!(
                "Loaded plugin: {} v{} — {}",
                manifest.name,
                manifest.version,
                manifest.description
            );
            loaded.push(manifest);
        }

        Ok(loaded)
    }

    // ── internal helpers ─────────────────────────────────────────────────

    /// Read and parse a `plugin.toml` manifest.
    fn read_manifest(&self, path: &PathBuf) -> Result<PluginManifest, SubcodeError> {
        let raw = std::fs::read_to_string(path).map_err(|e| {
            SubcodeError::Plugin(format!(
                "failed to read {}: {e}",
                path.display()
            ))
        })?;

        let manifest: PluginManifest = toml::from_str(&raw).map_err(|e| {
            SubcodeError::Plugin(format!(
                "invalid plugin.toml at {}: {e}",
                path.display()
            ))
        })?;

        Ok(manifest)
    }
}
