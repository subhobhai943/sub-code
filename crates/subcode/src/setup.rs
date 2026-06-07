// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use anyhow::Result;
use std::io::{self, Write};

pub async fn run_setup() -> Result<()> {
    println!("\n╔═════════════════════════════════════════╗");
    println!("\u2551    SUB CODE — First-Time Setup         \u2551");
    println!("\u255a═════════════════════════════════════════╝\n");

    let endpoint = prompt("Ollama endpoint", "http://localhost:11434")?;
    let model    = prompt("Default model",   "llama3.2:8b")?;

    // Check Ollama connectivity
    println!("\nChecking Ollama connectivity at {endpoint}...");
    match reqwest::get(format!("{endpoint}/api/tags")).await {
        Ok(r) if r.status().is_success() => {
            println!("\u2705 Ollama is running.");
        }
        _ => {
            println!("\u26a0\ufe0f  Could not reach Ollama at {endpoint}.");
            println!("   Make sure Ollama is running: https://ollama.com");
            println!("   Then pull a model: ollama pull {model}");
        }
    }

    let config_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve home directory"))?
        .join(".subcode");
    std::fs::create_dir_all(&config_dir)?;

    let config_content = format!(
        r#"[llm]
backend  = "ollama"
endpoint = "{endpoint}"
model    = "{model}"
context_window = 128000
temperature = 0.2
stream   = true

[agent]
mode = "interactive"
max_steps = 25

[context]
max_tokens = 32000
watch = true

[ui]
theme = "dark"
show_reasoning = true

[profiles]
"#
    );

    let config_path = config_dir.join("config.toml");
    std::fs::write(&config_path, config_content)?;
    println!("\n✅ Config written to {}", config_path.display());
    println!("\nRun `subcode` to start your first session!");
    Ok(())
}

pub async fn run_uninstall() -> Result<()> {
    println!("Removing SUB CODE...");
    // Remove binary from PATH locations.
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve home directory"))?;
    let candidates = vec![
        home.join(".local").join("bin").join("subcode"),
        home.join(".subcode").join("bin").join("subcode.exe"),
    ];
    for path in candidates {
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!("Removed: {}", path.display());
        }
    }
    println!("Config and sessions remain at ~/.subcode/ — delete manually if desired.");
    Ok(())
}

fn prompt(label: &str, default: &str) -> Result<String> {
    print!("{label} [default: {default}]: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        Ok(default.to_owned())
    } else {
        Ok(trimmed.to_owned())
    }
}
