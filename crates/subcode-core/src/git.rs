// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

//! Git integration for SUB CODE.
//!
//! Wraps `git2` to provide status, diff, staging, committing, branch
//! management, and LLM-powered commit message generation.

use std::path::{Path, PathBuf};

use futures::StreamExt;
use git2::{DiffFormat, DiffOptions, Repository, Signature, StatusOptions};

use crate::{
    error::SubcodeError,
    llm::LlmRouter,
};

/// Manages all git operations for a project directory.
pub struct GitManager {
    repo: Repository,
}

impl GitManager {
    /// Open the git repository at `path` (or any parent that contains `.git`).
    pub fn open(path: &Path) -> Result<Self, SubcodeError> {
        let repo = Repository::discover(path).map_err(|e| {
            SubcodeError::Git(format!("failed to open repository at {}: {e}", path.display()))
        })?;
        Ok(Self { repo })
    }

    /// Return a human-readable summary of the working-tree status.
    pub fn status(&self) -> Result<String, SubcodeError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);

        let statuses = self
            .repo
            .statuses(Some(&mut opts))
            .map_err(|e| SubcodeError::Git(format!("status failed: {e}")))?;

        if statuses.is_empty() {
            return Ok("nothing to commit, working tree clean".to_string());
        }

        let mut out = String::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("<invalid utf-8>");
            let st = entry.status();
            let label = status_label(st);
            out.push_str(&format!("{label:>2} {path}\n"));
        }
        Ok(out)
    }

    /// Return the unstaged diff (working tree vs index).
    pub fn diff(&self) -> Result<String, SubcodeError> {
        let mut opts = DiffOptions::new();
        let diff = self
            .repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|e| SubcodeError::Git(format!("diff failed: {e}")))?;
        diff_to_string(&diff)
    }

    /// Return the staged diff (index vs HEAD).
    pub fn diff_staged(&self) -> Result<String, SubcodeError> {
        let head_tree = match self.repo.head() {
            Ok(r) => {
                let commit = r
                    .peel_to_commit()
                    .map_err(|e| SubcodeError::Git(format!("peel HEAD: {e}")))?;
                Some(
                    commit
                        .tree()
                        .map_err(|e| SubcodeError::Git(format!("HEAD tree: {e}")))?,
                )
            }
            Err(_) => None, // initial commit — no HEAD yet
        };

        let mut opts = DiffOptions::new();
        let diff = self
            .repo
            .diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))
            .map_err(|e| SubcodeError::Git(format!("diff --staged failed: {e}")))?;
        diff_to_string(&diff)
    }

    /// Return the last `n` commit log entries as a formatted string.
    pub fn log(&self, n: usize) -> Result<String, SubcodeError> {
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(|e| SubcodeError::Git(format!("revwalk: {e}")))?;
        revwalk
            .push_head()
            .map_err(|e| SubcodeError::Git(format!("push HEAD: {e}")))?;

        let mut out = String::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= n {
                break;
            }
            let oid = oid.map_err(|e| SubcodeError::Git(format!("walk oid: {e}")))?;
            let commit = self
                .repo
                .find_commit(oid)
                .map_err(|e| SubcodeError::Git(format!("find commit: {e}")))?;
            let short = &oid.to_string()[..7];
            let msg = commit.summary().unwrap_or("<no message>");
            let author = commit.author();
            let name = author.name().unwrap_or("?");
            out.push_str(&format!("{short} {name}: {msg}\n"));
        }
        Ok(out)
    }

    /// Stage every change in the working tree (equivalent to `git add -A`).
    pub fn stage_all(&self) -> Result<(), SubcodeError> {
        let mut index = self
            .repo
            .index()
            .map_err(|e| SubcodeError::Git(format!("index: {e}")))?;
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| SubcodeError::Git(format!("add_all: {e}")))?;
        index
            .write()
            .map_err(|e| SubcodeError::Git(format!("index write: {e}")))?;
        Ok(())
    }

    /// Stage specific file paths.
    pub fn stage_files(&self, paths: &[PathBuf]) -> Result<(), SubcodeError> {
        let mut index = self
            .repo
            .index()
            .map_err(|e| SubcodeError::Git(format!("index: {e}")))?;
        for path in paths {
            index
                .add_path(path)
                .map_err(|e| SubcodeError::Git(format!("add path {}: {e}", path.display())))?;
        }
        index
            .write()
            .map_err(|e| SubcodeError::Git(format!("index write: {e}")))?;
        Ok(())
    }

    /// Create a commit with `msg` using the repo's configured author/email.
    pub fn commit(&self, msg: &str) -> Result<git2::Oid, SubcodeError> {
        let sig = self
            .repo
            .signature()
            .map_err(|e| SubcodeError::Git(format!("signature: {e}")))?;
        self.commit_with_sig(msg, &sig)
    }

    fn commit_with_sig(&self, msg: &str, sig: &Signature<'_>) -> Result<git2::Oid, SubcodeError> {
        let mut index = self
            .repo
            .index()
            .map_err(|e| SubcodeError::Git(format!("index: {e}")))?;
        let tree_oid = index
            .write_tree()
            .map_err(|e| SubcodeError::Git(format!("write tree: {e}")))?;
        let tree = self
            .repo
            .find_tree(tree_oid)
            .map_err(|e| SubcodeError::Git(format!("find tree: {e}")))?;

        let parents: Vec<git2::Commit<'_>> = match self.repo.head() {
            Ok(head_ref) => {
                let commit = head_ref
                    .peel_to_commit()
                    .map_err(|e| SubcodeError::Git(format!("peel: {e}")))?;
                vec![commit]
            }
            Err(_) => vec![], // initial commit
        };
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

        let oid = self
            .repo
            .commit(Some("HEAD"), sig, sig, msg, &tree, &parent_refs)
            .map_err(|e| SubcodeError::Git(format!("commit: {e}")))?;
        Ok(oid)
    }

    /// Create a new local branch pointing to HEAD.
    pub fn create_branch(&self, name: &str) -> Result<(), SubcodeError> {
        let head_commit = self
            .repo
            .head()
            .and_then(|r| r.peel_to_commit())
            .map_err(|e| SubcodeError::Git(format!("HEAD: {e}")))?;
        self.repo
            .branch(name, &head_commit, false)
            .map_err(|e| SubcodeError::Git(format!("create branch '{name}': {e}")))?;
        Ok(())
    }

    /// List all local branch names.
    pub fn list_branches(&self) -> Result<Vec<String>, SubcodeError> {
        let branches = self
            .repo
            .branches(Some(git2::BranchType::Local))
            .map_err(|e| SubcodeError::Git(format!("list branches: {e}")))?;

        let mut names = Vec::new();
        for branch in branches {
            let (branch, _) =
                branch.map_err(|e| SubcodeError::Git(format!("branch iter: {e}")))?;
            if let Some(name) = branch.name().map_err(|e| {
                SubcodeError::Git(format!("branch name utf-8: {e}"))
            })? {
                names.push(name.to_string());
            }
        }
        Ok(names)
    }

    /// Ask the LLM to generate a conventional commit message from `diff`.
    ///
    /// Streams tokens and collects the full reply before returning.
    pub async fn generate_commit_message(
        diff: &str,
        llm: &LlmRouter,
    ) -> Result<String, SubcodeError> {
        let system = "You are a helpful assistant that writes concise git commit messages \
            following the Conventional Commits specification. \
            Output only the commit message — no explanation, no markdown, no quotes.";

        let user = format!(
            "Write a single-line conventional commit message for the following diff:\n\n```diff\n{diff}\n```"
        );

        let mut stream = llm
            .stream_chat(system, &user)
            .await
            .map_err(|e| SubcodeError::Llm(e.to_string()))?;

        let mut message = String::new();
        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| SubcodeError::Llm(e.to_string()))?;
            match event {
                crate::llm::TokenEvent::Token(tok) => message.push_str(&tok),
                crate::llm::TokenEvent::Done => break,
            }
        }

        let message = message.trim().to_string();
        if message.is_empty() {
            return Err(SubcodeError::Llm(
                "LLM returned empty commit message".to_string(),
            ));
        }
        Ok(message)
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Convert a `git2::Diff` to a UTF-8 patch string.
fn diff_to_string(diff: &git2::Diff<'_>) -> Result<String, SubcodeError> {
    let mut out = Vec::<u8>::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        use git2::DiffLineType::*;
        let prefix = match line.origin_value() {
            Addition => b'+',
            Deletion => b'-',
            Context => b' ',
            _ => b' ',
        };
        out.push(prefix);
        out.extend_from_slice(line.content());
        true
    })
    .map_err(|e| SubcodeError::Git(format!("diff print: {e}")))?;

    String::from_utf8(out)
        .map_err(|e| SubcodeError::Git(format!("diff utf-8: {e}")))
}

/// Map a `git2::Status` bitfield to a short two-character label.
fn status_label(st: git2::Status) -> &'static str {
    use git2::Status;
    if st.contains(Status::INDEX_NEW) {
        "A "
    } else if st.contains(Status::INDEX_MODIFIED) {
        "M "
    } else if st.contains(Status::INDEX_DELETED) {
        "D "
    } else if st.contains(Status::INDEX_RENAMED) {
        "R "
    } else if st.contains(Status::WT_NEW) {
        "??"
    } else if st.contains(Status::WT_MODIFIED) {
        " M"
    } else if st.contains(Status::WT_DELETED) {
        " D"
    } else {
        "  "
    }
}
