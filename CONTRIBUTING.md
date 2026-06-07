# Contributing to SUB CODE

Thank you for your interest in contributing to **SUB CODE — AI Coding Assistant**.

SUB CODE is a terminal-native, compiled Rust coding agent focused on local-first LLM integration, low latency, and deep Git/codebase awareness. Contributions that improve reliability, performance, ergonomics, and extensibility are especially welcome.

---

## Code of Conduct

Be respectful, constructive, and professional. Treat maintainers and other contributors with kindness and assume good intent. Harassment, discrimination, or abuse of any kind will not be tolerated.

---

## Project structure

The repository is a Rust workspace:

- `crates/subcode` – CLI binary (`subcode`), argument parsing, TUI integration, setup/uninstall flows.
- `crates/subcode-core` – core engine: config, errors, LLM router, agent, context, git, plugin manager, tools.
- `crates/subcode-tui` – Ratatui-based terminal UI (chat view, agent steps, status bar, diff viewer; in progress).
- `crates/subcode-plugin-sdk` – shared types and traits for external plugins (native + WASM; in progress).

Support crates and scripts (to be fleshed out):

- `install/install.sh` – installer for Linux/macOS.
- `install/install.ps1` – installer for Windows.
- `docs/*` – architecture and plugin API documentation.

---

## Development setup

### 1. Prerequisites

- Rust (stable) toolchain.
- Git.
- A local LLM backend (recommended: Ollama on `http://localhost:11434`).

### 2. Clone and build

```bash
git clone https://github.com/subhobhai943/sub-code.git
cd sub-code
cargo build --workspace
```

### 3. Run tests

```bash
cargo test --workspace
```

If you add new crates or modules, ensure they compile and are covered by at least minimal tests.

---

## Coding guidelines

To keep the codebase consistent and robust:

- **Rust edition:** 2021.
- **Error handling:**
  - Core/library code (especially in `subcode-core`) should avoid `.unwrap()` or `.expect()` on user-controlled data paths.
  - Prefer `Result<T, SubcodeError>` (or `anyhow::Result` in binaries/tests) and use `?` for propagation.
- **Async:**
  - Use `tokio` for all async I/O.
  - Avoid blocking calls in async contexts (no `std::thread::sleep` inside async flows).
- **LLM integration:**
  - Route all model calls through `LlmRouter` and backend implementations.
  - Prefer streaming where possible.
- **Context & indexing:**
  - Work through `ProjectContext` and `CodeIndex` for file tree/state.
  - Respect `.subcodeignore` and `.gitignore` semantics as they evolve.
- **Git operations:**
  - Use the `git2`-backed Git manager in `subcode-core` instead of shelling out directly to `git`.
- **Plugins:**
  - When extending plugin functionality, go through the plugin manager and `subcode-plugin-sdk` (once stabilised).

Formatting and linting:

- Run `cargo fmt` before opening a PR.
- Run `cargo clippy --workspace --all-targets` and address warnings where reasonable.

---

## Making changes

1. **Fork** the repository and create a feature branch:

   ```bash
   git checkout -b feat/your-change
   ```

2. **Implement** your changes following the guidelines above.

3. **Add tests** for new behavior where feasible.

4. **Run checks**:

   ```bash
   cargo fmt
   cargo clippy --workspace --all-targets
   cargo test --workspace
   ```

5. **Commit** using clear messages (Conventional Commits style is preferred):

   - `feat: add X`
   - `fix: handle Y`
   - `refactor: simplify Z`

6. **Open a pull request** against `main` with:

   - A clear summary of what changed and why.
   - Notes on any API changes or breaking behavior.
   - Screenshots or terminal recordings for UX changes (TUI, CLI output).

---

## Feature areas & priorities

You can have high impact by working in these areas:

- **Tooling:**
  - Implement built-in tools (file read/write/edit, search, refactor, tests, lint, docs, debug, benchmarks).
  - Improve tool ergonomics and safety (dry-run modes, richer diffs, better error messages).
- **TUI:**
  - Chat layout, scrolling, agent steps panel, diff viewer, status bar, theming.
- **LLM backends:**
  - Additional backends and configuration ergonomics.
  - Better context window management and summarisation.
- **Plugins:**
  - Docker, GitHub CLI, Jira, Linear, Slack, and other integrations.
  - Permissions model and sandboxing for native/WASM plugins.
- **Installers & packaging:**
  - Robust `install.sh` / `install.ps1` scripts.
  - `cargo install`, Homebrew, winget, distro packages.

If you are unsure where to start, open an issue or discussion and we can help scope something that matches your interests.

---

## Reporting bugs and requesting features

- Use GitHub Issues to report bugs, request features, or ask questions.
- When reporting a bug, include:
  - OS and architecture.
  - Rust version.
  - `subcode` version/commit.
  - Reproduction steps and expected vs actual behavior.

---

## Thank you

Your time and contributions help make SUB CODE a powerful, open, and extensible coding assistant. Thank you for helping push terminal-native AI tooling forward.
