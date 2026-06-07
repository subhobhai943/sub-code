// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

//! Shell command execution with configurable allowlist / denylist policy.
//!
//! Every command is checked against the configured allowlist and denylist
//! before execution.  If the command is denied, a [`SubcodeError::ShellDenied`]
//! error is returned immediately — the process is never spawned.

use std::path::Path;

use tokio::process::Command;

use crate::config::ShellConfig;
use crate::error::SubcodeError;

/// The result of running a shell command.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output captured from the child process.
    pub stdout: String,
    /// Standard error captured from the child process.
    pub stderr: String,
    /// Exit code; `None` if the process was killed by a signal.
    pub exit_code: Option<i32>,
}

/// Sandboxed shell executor that enforces an allowlist / denylist policy.
///
/// # Policy rules
///
/// 1. If the **denylist** is non-empty and the command's base name appears in
///    it, execution is denied.
/// 2. If the **allowlist** is non-empty and the command's base name does
///    *not* appear in it, execution is denied.
/// 3. If both lists are empty, every command is allowed (open policy).
pub struct Shell {
    /// Commands that are always allowed (empty = allow all not denied).
    allowlist: Vec<String>,
    /// Commands that are never allowed (checked first).
    denylist: Vec<String>,
}

impl Shell {
    /// Create a new [`Shell`] from the provided configuration.
    pub fn new(config: &ShellConfig) -> Self {
        Self {
            allowlist: config.allowlist.clone(),
            denylist: config.denylist.clone(),
        }
    }

    /// Return `true` if `cmd` is permitted by the current policy.
    ///
    /// The check is performed against the **base name** of the command
    /// (the part after the last path separator, if any).
    pub fn is_allowed(&self, cmd: &str) -> bool {
        let base = extract_base_command(cmd);

        // Denylist takes priority.
        if self.denylist.iter().any(|d| d == base) {
            return false;
        }

        // If an allowlist exists the command must be in it.
        if !self.allowlist.is_empty() {
            return self.allowlist.iter().any(|a| a == base);
        }

        // Both lists empty → open policy.
        true
    }

    /// Execute `cmd` with `args` in `cwd`, returning captured output.
    ///
    /// # Errors
    ///
    /// * [`SubcodeError::ShellDenied`] — the command is blocked by policy.
    /// * [`SubcodeError::Shell`] — the child process could not be spawned or
    ///   its output could not be collected.
    pub async fn execute(
        &self,
        cmd: &str,
        args: &[&str],
        cwd: &Path,
    ) -> Result<CommandOutput, SubcodeError> {
        if !self.is_allowed(cmd) {
            return Err(SubcodeError::ShellDenied(format!(
                "command '{}' is not allowed by shell policy",
                cmd
            )));
        }

        let output = Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                SubcodeError::Shell(format!("failed to spawn '{}': {e}", cmd))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let exit_code = output.status.code();

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
        })
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Extract the base command name from a potentially-qualified path.
///
/// For example, `/usr/bin/git` → `git`, `C:\Windows\cmd.exe` → `cmd.exe`,
/// and plain `cargo` → `cargo`.
fn extract_base_command(cmd: &str) -> &str {
    // Try both Unix and Windows separators.
    let after_sep = cmd.rsplit(|c| c == '/' || c == '\\').next();
    after_sep.unwrap_or(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_policy_allows_everything() {
        let shell = Shell::new(&ShellConfig {
            allowlist: vec![],
            denylist: vec![],
        });
        assert!(shell.is_allowed("cargo"));
        assert!(shell.is_allowed("rm"));
    }

    #[test]
    fn denylist_blocks_command() {
        let shell = Shell::new(&ShellConfig {
            allowlist: vec![],
            denylist: vec!["rm".into(), "shutdown".into()],
        });
        assert!(!shell.is_allowed("rm"));
        assert!(!shell.is_allowed("/bin/rm"));
        assert!(shell.is_allowed("cargo"));
    }

    #[test]
    fn allowlist_restricts_to_listed() {
        let shell = Shell::new(&ShellConfig {
            allowlist: vec!["cargo".into(), "git".into()],
            denylist: vec![],
        });
        assert!(shell.is_allowed("cargo"));
        assert!(shell.is_allowed("git"));
        assert!(!shell.is_allowed("rm"));
    }

    #[test]
    fn denylist_overrides_allowlist() {
        let shell = Shell::new(&ShellConfig {
            allowlist: vec!["cargo".into(), "rm".into()],
            denylist: vec!["rm".into()],
        });
        assert!(shell.is_allowed("cargo"));
        assert!(!shell.is_allowed("rm"));
    }

    #[test]
    fn base_command_extraction() {
        assert_eq!(extract_base_command("/usr/bin/git"), "git");
        assert_eq!(extract_base_command("C:\\Windows\\cmd.exe"), "cmd.exe");
        assert_eq!(extract_base_command("cargo"), "cargo");
    }
}
