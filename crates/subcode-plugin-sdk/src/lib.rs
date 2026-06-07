// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

//! # SUB CODE Plugin SDK
//!
//! This crate provides the stable public API for building native SUB CODE
//! plugins as dynamic libraries.  Plugin authors implement the [`Plugin`]
//! trait and export a `subcode_plugin_create` function using the provided
//! macro or manually via `#[no_mangle] extern "C"`.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use subcode_plugin_sdk::{Plugin, PluginConfig, PluginError, PluginManifest};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn manifest(&self) -> PluginManifest {
//!         PluginManifest {
//!             name: "my-plugin".into(),
//!             version: "0.1.0".into(),
//!             author: "you".into(),
//!             description: "My awesome plugin".into(),
//!             permissions: vec![],
//!         }
//!     }
//!
//!     fn on_load(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
//!         Ok(())
//!     }
//!
//!     fn on_unload(&mut self) -> Result<(), PluginError> {
//!         Ok(())
//!     }
//! }
//!
//! // Export the plugin create function.
//! #[no_mangle]
//! pub extern "C" fn subcode_plugin_create() -> *mut dyn Plugin {
//!     let plugin = MyPlugin;
//!     Box::into_raw(Box::new(plugin))
//! }
//! ```

use std::collections::HashMap;
use thiserror::Error;
use serde::{Deserialize, Serialize};

// ── Re-exports ──────────────────────────────────────────────────────────────

/// Re-export the core `Tool` trait so plugins can register tools.
pub use subcode_core::tools::{Tool, ToolResult, ToolRegistry};

/// Re-export the core `ProjectContext` for tool execution.
pub use subcode_core::context::ProjectContext;

// ── Plugin error ────────────────────────────────────────────────────────────

/// Errors that a plugin can return.
#[derive(Debug, Error)]
pub enum PluginError {
    /// A generic failure during plugin lifecycle.
    #[error("Plugin error: {0}")]
    General(String),

    /// The plugin requires a permission that was not granted.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// A configuration value was missing or invalid.
    #[error("Config error: {0}")]
    Config(String),

    /// An I/O error within the plugin.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Plugin manifest ─────────────────────────────────────────────────────────

/// Describes a plugin's identity and requirements.
///
/// This mirrors `subcode_core::plugin::PluginManifest` but is owned by the
/// SDK crate so that plugin authors do not need to depend on `subcode-core`
/// directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Human-readable plugin name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Plugin author.
    pub author: String,
    /// One-line description.
    pub description: String,
    /// Permission strings the plugin requires.
    #[serde(default)]
    pub permissions: Vec<String>,
}

// ── Plugin config ───────────────────────────────────────────────────────────

/// Key-value configuration passed to a plugin at load time.
///
/// The host populates this from the `[plugins.<name>]` section in the
/// user's TOML config file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Arbitrary key-value pairs.
    pub values: HashMap<String, String>,
}

impl PluginConfig {
    /// Look up a configuration key, returning `None` if absent.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Look up a key or return a [`PluginError::Config`] with the given
    /// message.
    pub fn require(&self, key: &str) -> Result<&str, PluginError> {
        self.get(key).ok_or_else(|| {
            PluginError::Config(format!("required config key '{key}' is missing"))
        })
    }
}

// ── Plugin trait ────────────────────────────────────────────────────────────

/// The main trait that every SUB CODE plugin must implement.
///
/// The host loads the plugin's shared library, calls `subcode_plugin_create`
/// to obtain a `Box<dyn Plugin>`, and then drives the lifecycle:
///
/// 1. [`Plugin::manifest`] — read identity and requirements.
/// 2. [`Plugin::on_load`] — initialise with config; register tools.
/// 3. (plugin is active — tools are callable)
/// 4. [`Plugin::on_unload`] — clean up resources.
pub trait Plugin: Send {
    /// Return the plugin's manifest (identity, version, permissions).
    fn manifest(&self) -> PluginManifest;

    /// Called when the plugin is loaded by the host.
    ///
    /// Use this to initialise state, validate config, and register any
    /// custom tools with the host.
    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError>;

    /// Called when the plugin is about to be unloaded.
    ///
    /// Use this to flush buffers, close connections, and release
    /// resources.
    fn on_unload(&mut self) -> Result<(), PluginError>;

    /// Optionally return a list of tools this plugin provides.
    ///
    /// The default implementation returns an empty list.
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        Vec::new()
    }
}

// ── Plugin create function ──────────────────────────────────────────────────

/// Declaration pattern for plugin entry points.
///
/// Plugin authors should export a function with this exact signature:
///
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn subcode_plugin_create() -> *mut dyn Plugin {
///     let plugin = MyPlugin::default();
///     Box::into_raw(Box::new(plugin))
/// }
/// ```
///
/// The host will call this function after loading the shared library and
/// wrap the returned pointer in a `Box<dyn Plugin>` for lifecycle
/// management.
///
/// # Safety
///
/// The returned pointer must have been allocated with `Box::new` and
/// leaked with `Box::into_raw`.  The host takes ownership and will
/// drop the plugin via `Box::from_raw` when it is unloaded.
pub type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn Plugin;

/// The exact symbol name the host looks for when loading a plugin.
pub const PLUGIN_CREATE_SYMBOL: &str = "subcode_plugin_create";
