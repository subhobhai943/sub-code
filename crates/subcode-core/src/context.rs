// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use crate::config::Config;
use crate::error::SubcodeError;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Lightweight summary of a single file in the project.
#[derive(Debug, Clone)]
pub struct FileSummary {
    pub path:     PathBuf,
    pub language: String,
    pub size:     u64,
}

/// In-memory index of the project codebase.
#[derive(Debug, Default)]
pub struct CodeIndex {
    pub files: HashMap<PathBuf, FileSummary>,
}

impl CodeIndex {
    pub fn new() -> Self {
        Self { files: HashMap::new() }
    }

    pub fn add_file(&mut self, path: PathBuf, language: String, size: u64) {
        self.files.insert(path.clone(), FileSummary { path, language, size });
    }

    pub fn to_summary_string(&self, max_entries: usize) -> String {
        let mut out = String::new();
        for (i, file) in self.files.values().take(max_entries).enumerate() {
            let _ = writeln!(
                &mut out,
                "{}: {} ({} bytes, {})",
                i + 1,
                file.path.display(),
                file.size,
                file.language,
            );
        }
        out
    }
}

/// Shared project context, including config and code index.
#[derive(Debug)]
pub struct ProjectContext {
    pub root:   PathBuf,
    pub config: Config,
    pub index:  Arc<RwLock<CodeIndex>>,    
}

impl ProjectContext {
    /// Build a new project context from the current working directory and config.
    pub async fn build(config: &Config) -> Result<Self, SubcodeError> {
        let root = std::env::current_dir().map_err(|e| SubcodeError::Context(e.to_string()))?;
        let index = Arc::new(RwLock::new(CodeIndex::new()));

        // Initial indexing.
        Self::index_tree(&root, &config.context.ignore_file, &index).await?;

        // Optionally start a background watcher.
        if config.context.watch {
            Self::spawn_watcher(root.clone(), config.context.ignore_file.clone(), index.clone())?;
        }

        Ok(Self { root, config: config.clone(), index })
    }

    /// Render a compact string that can be embedded into the system prompt.
    pub fn to_llm_string(&self, max_tokens: usize) -> String {
        // Very rough heuristic: assume ~4 chars per token.
        let approx_max_chars = max_tokens.saturating_mul(4);
        let index = self.index.blocking_read();
        let summary = index.to_summary_string(128);
        if summary.len() > approx_max_chars {
            summary[..approx_max_chars].to_string()
        } else {
            summary
        }
    }

    async fn index_tree(root: &Path, ignore_file: &str, index: &Arc<RwLock<CodeIndex>>) -> Result<(), SubcodeError> {
        let ignore_path = root.join(ignore_file);
        let ignore_patterns = if ignore_path.exists() {
            let raw = tokio::fs::read_to_string(&ignore_path).await?;
            raw.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let mut entries = tokio::fs::read_dir(root).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            if ignore_patterns.iter().any(|p| file_name.contains(p)) {
                continue;
            }

            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                Self::index_tree(&path, ignore_file, index).await?;
            } else if metadata.is_file() {
                if Self::is_binary(&path) {
                    continue;
                }

                let language = Self::detect_language(&path);
                let mut guard = index.write().await;
                guard.add_file(path.strip_prefix(root).unwrap_or(&path).to_path_buf(), language, metadata.len());
            }
        }
        Ok(())
    }

    fn spawn_watcher(root: PathBuf, ignore_file: String, index: Arc<RwLock<CodeIndex>>) -> Result<(), SubcodeError> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // notify v6 uses an event-handler closure.
        let sender = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = sender.send(event);
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| SubcodeError::Context(format!("file watcher init: {e}")))?;

        let watch_root = root.clone();
        watcher
            .watch(&watch_root, RecursiveMode::Recursive)
            .map_err(|e| SubcodeError::Context(format!("file watcher start: {e}")))?;

        // Keep the watcher alive in a dedicated task.
        tokio::spawn(async move {
            // Hold onto the watcher so it is not dropped.
            let _watcher = watcher;
            while let Some(_event) = rx.recv().await {
                // For now we re-scan everything; later we can do targeted updates.
                if let Err(err) = Self::index_tree(&root, &ignore_file, &index).await {
                    tracing::warn!("Context reindex failed: {err}");
                }
            }
        });

        Ok(())
    }

    fn is_binary(path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "exe" | "dll") => true,
            _ => false,
        }
    }

    fn detect_language(path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust".into(),
            Some("ts") => "typescript".into(),
            Some("js") => "javascript".into(),
            Some("py") => "python".into(),
            Some("cpp" | "cc" | "cxx" | "hpp" | "hh") => "cpp".into(),
            Some("md") => "markdown".into(),
            _ => "text".into(),
        }
    }
}
