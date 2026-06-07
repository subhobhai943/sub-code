# SUB CODE — AI Coding Assistant

> Terminal-native, compiled Rust coding agent with local-first LLM integration.

SUB CODE is a high-performance, terminal-native AI coding assistant designed to work directly inside your existing projects, with **local LLMs first** (Ollama, LM Studio, llama.cpp) and opt-in support for cloud backends (OpenAI-compatible, Anthropic). It is built in **Rust 2021**, with a focus on low latency, streaming responses, and deep Git/codebase integration.

- **Repo:** https://github.com/subhobhai943/sub-code
- **Binary:** `subcode`
- **Author:** `subhobhai`
- **License:** MIT

---

## Features (current & planned)

- **Compiled Rust binary** with async I/O (tokio) and low startup overhead.
- **Local LLM first** via a pluggable `LlmRouter` (Ollama, LM Studio, llama.cpp, OpenAI-compatible, Anthropic).[cite:14]
- **Project-aware context** via `ProjectContext` and `CodeIndex` (file tree, language detection, sizes, ignore rules).[cite:19]
- **Streaming chat** using a ReAct-ready `AgentRunner` that integrates config, context, and LLM backends.[cite:18]
- **CLI-first UX** with subcommands:
  - `subcode` – interactive session (TUI placeholder today).
  - `subcode ask "…"` – one-shot question for the current project.[cite:16]
  - `subcode run "…"` – task-oriented run mode.[cite:16]
  - `subcode summary` – quick project summary using the agent.[cite:16]
  - `subcode git …` – semantic commit, status, diff, branches, log, PR description.[cite:16]
  - `subcode plugin …` – plugin manager (install/list/remove).[cite:16]
  - `subcode model …` – list/switch models.
  - `subcode profile …` – profile hints.
  - `subcode --setup` / `subcode uninstall` – install/uninstall helpers.[cite:16][cite:15]
- **Git integration** via `GitManager` (status, diff, log, branches, staged commit, PR summary – backed by `git2`).[cite:16]
- **Plugin system (in progress)** with a dedicated `subcode-plugin-sdk` crate and `PluginManager` in `subcode-core`.[cite:22][cite:16]
- **TUI (in progress)** using Ratatui and crossterm via a `subcode-tui` crate (currently a minimal REPL wrapper around the agent).[cite:22]

> ⚠️ SUB CODE is under active development. Expect breaking changes while core architecture, tools, and plugin APIs stabilize.

---

## Quick start

### Prerequisites

- Rust toolchain (stable) installed.
- Git installed.
- A local LLM backend:
  - Recommended: [Ollama](https://ollama.com) running on `http://localhost:11434`.
  - A model available, e.g. `llama3.2:8b` (the default in config).

### Install via GitHub install script (planned)

Once GitHub releases are published, you will be able to install with:

```bash
curl -fsSL https://raw.githubusercontent.com/subhobhai943/sub-code/main/install/install.sh | bash
```

On Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/subhobhai943/sub-code/main/install/install.ps1 | iex
```

These scripts will:

- Detect OS/arch and download the appropriate prebuilt binary.
- Install it to `~/.local/bin` (Linux/macOS) or `%USERPROFILE%\.subcode\bin` (Windows).
- Add it to your PATH.
- Run `subcode --setup` to configure your LLM endpoint and model.

> Until releases are available, use the **from source** instructions below.

### From source

Clone and build the workspace:

```bash
git clone https://github.com/subhobhai943/sub-code.git
cd sub-code
cargo build --workspace
```

Run the CLI binary from your project directory:

```bash
# From inside a project you want SUB CODE to work on
subcode ask "explain the overall structure of this codebase"
```

### First-time setup

On first run, or explicitly via `subcode --setup`, SUB CODE will:

- Load `~/.subcode/config.toml` and `.subcode.toml` (if present) and merge them.[cite:13]
- Check for a configured LLM backend and endpoint.
- Recommend verifying that Ollama is running and that `llama3.2:8b` (or your chosen model) is pulled.

---

## Configuration

Configuration is layered:

1. **Global config:** `~/.subcode/config.toml`.
2. **Project config:** `.subcode.toml` at your repository root.[cite:13]
3. **Environment variables:** `SUBCODE_MODEL`, `SUBCODE_ENDPOINT`, `SUBCODE_API_KEY` (planned), etc.

### Config model

`subcode-core` exposes a `Config` struct with these sections:[cite:13]

- `project`: name, language hints.
- `llm`: backend, endpoint, model, context window, temperature, stream, API key.
- `agent`: mode (`interactive`/`yolo`), approval timeout, max steps.
- `context`: max tokens, embeddings flag (future), watch flag, ignore file name.
- `shell`: allowlist/denylist for commands.
- `plugins`: enabled plugin list.
- `ui`: theme, whether to show reasoning, syntax highlighting.

See `.subcode.toml.example` for a concrete template.[cite:2]

### Ignore file

Use `.subcodeignore` (by default) in your repo root to exclude files and directories from the project index.[cite:13][cite:19]

---

## Architecture

The project is structured as a Rust workspace:[cite:22]

- `crates/subcode-core` – core engine (config, errors, LLM router, agent, context, git, plugin, tools).
- `crates/subcode` – CLI binary (`subcode`) wiring config, context, LLM, agent, plugins, and TUI.[cite:16][cite:15]
- `crates/subcode-tui` – terminal UI crate (Ratatu2f-based TUI; currently a minimal REPL loop around the agent; will evolve).
- `crates/subcode-plugin-sdk` – stable SDK types for external plugins (native + WASM).

Key pieces:

- **Config loader**: async `Config::load()` merging global + local TOML.[cite:13]
- **LLM router**: `LlmRouter::from_config(&Config)` picks a backend and offers `chat_stream`, `complete_simple`, and `list_models`.[cite:14]
- **Context**: `ProjectContext::build(&Config)` indexes the codebase and can render a compact context string for prompts.[cite:19]
- **Agent**: `AgentRunner::new(config, ctx, llm, yolo)` powers `subcode ask`, `subcode run`, and `subcode summary` using streaming chat.[cite:18][cite:16]
- **Git manager**: `GitManager` wraps `git2` to provide high-level operations used by `subcode git` subcommands.[cite:16]
- **Plugin manager**: `PluginManager` handles discovery/installation of plugins, reading `plugin.toml` manifests and enforcing permissions (in progress).[cite:16]

For more detail, see `docs/ARCHITECTURE.md` and `PLUGIN_API.md`.

---

## Roadmap (high level)

- Full **ReAct tool loop** with 25+ built-in tools (read/edit files, tests, linters, refactors, docs, debug, benchmarks, env checks, etc.).
- Rich Ratatui-based TUI with:
  - Chat pane, agent steps pane, status bar (model/context/project), diff viewer, syntax highlighting.
- Plugin system:
  - Native + WASM plugins with manifest-declared permissions.
  - Example plugins for Docker, GitHub CLI, Jira, Linear, Slack.
- Installers and package managers:
  - GitHub install scripts (Linux/macOS/Windows).
  - `cargo install`, `brew`, `winget`, and distro packages via CI.
- PR-ready CI workflow and release automation.

---

## Contributing

Contributions, bug reports, and feature requests are welcome.

- See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
- Please open an issue before starting large refactors or new feature areas.

---

## License

This project is licensed under the [MIT License](LICENSE).
