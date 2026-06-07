# SUB CODE Plugin API

SUB CODE is designed to be highly extensible. While the `subcode-core` crate ships with 25 highly-optimized, built-in tools (for filesystem, shell, and git interactions), users and organizations can build their own custom tools using the `subcode-plugin-sdk`.

Plugins are dynamic libraries (WASM support planned) loaded at runtime from `~/.subcode/plugins/`.

---

## The Plugin SDK

The SDK (`subcode-plugin-sdk`) provides a stable interface allowing you to inject custom Rust structs that implement the `Tool` trait into SUB CODE's `ToolRegistry`.

To build a plugin, your crate must be configured as a `cdylib` in `Cargo.toml`:

```toml
[package]
name = "subcode-plugin-mytools"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
subcode-plugin-sdk = { path = "../subcode-plugin-sdk" }
async-trait = "0.1"
serde_json = "1.0"
```

---

## Writing a Custom Tool

Every tool must implement the `Tool` trait. Here is an example of a simple tool that calculates cryptographic hashes:

```rust
use subcode_plugin_sdk::{Tool, ToolResult, ProjectContext, SubcodeError};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct HashTool;

#[async_trait]
impl Tool for HashTool {
    fn name(&self) -> &str {
        "calculate_hash"
    }

    fn description(&self) -> &str {
        "Calculate a SHA-256 hash for a given string"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "The string to hash" }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, params: Value, _ctx: &ProjectContext) -> Result<ToolResult, SubcodeError> {
        let text = params.get("text").and_then(|v| v.as_str()).unwrap_or_default();
        
        if text.is_empty() {
            return Ok(ToolResult::err("Missing 'text' parameter"));
        }

        // Extremely naive mock hash logic for the example
        let hash = format!("{:x}", md5::compute(text)); // Pretend this is sha256
        
        Ok(ToolResult::ok(format!("Hash computed: {}", hash)))
    }
}
```

---

## Creating the Plugin Export

To expose your tools to SUB CODE, you must implement the `Plugin` trait and export it using the stable `subcode_plugin_create` C ABI.

```rust
use subcode_plugin_sdk::{Plugin, PluginManifest, PluginConfig, PluginError, Tool};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "My Tools".to_string(),
            version: "0.1.0".to_string(),
            author: "You".to_string(),
            description: "Adds custom hashing tools".to_string(),
            permissions: vec![],
        }
    }

    fn on_load(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
        // Run setup logic here
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn tools(&self) -> Vec<Box<dyn Tool>> {
        // Return boxed instances of your tools
        vec![Box::new(HashTool)]
    }
}

// -----------------------------------------------------------------------------
// FFI EXPORT — Required by SUB CODE plugin loader
// -----------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn subcode_plugin_create() -> *mut dyn Plugin {
    // We leak the box to hand ownership over the FFI boundary.
    // SUB CODE will properly reconstruct the box and drop it during unload.
    Box::into_raw(Box::new(MyPlugin))
}
```

## Installation

1. Compile your plugin using `cargo build --release`.
2. Find the shared object (`.so`, `.dylib`, or `.dll`) in your `target/release` directory.
3. Move the library into `~/.subcode/plugins/mytools/` alongside a `plugin.toml` manifest file.
4. Ensure `mytools` is added to the `enabled` list in your `~/.subcode/config.toml`.
