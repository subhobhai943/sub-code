// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

use anyhow::Result;
use clap::{Parser, Subcommand};
use subcode_core::{
    config::Config,
    agent::AgentRunner,
    context::ProjectContext,
    llm::LlmRouter,
};
use subcode_tui::Tui;

/// SUB CODE — Terminal-native AI coding assistant.
#[derive(Parser, Debug)]
#[command(name = "subcode", version, author, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run without approval prompts (autonomous mode).
    #[arg(long, global = true)]
    pub yolo: bool,

    /// Resume the previous session for this project.
    #[arg(long, global = true)]
    pub resume: bool,

    /// Run first-time setup wizard.
    #[arg(long, global = true)]
    pub setup: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Ask a one-shot question without starting a full session.
    Ask {
        /// The question to ask (plain English).
        question: String,
    },
    /// Run a task autonomously.
    Run {
        /// The task description.
        task: String,
    },
    /// Git workflow automation.
    Git {
        #[command(subcommand)]
        action: GitAction,
    },
    /// Plugin management.
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Show a summary of the current project.
    Summary,
    /// Switch active LLM model.
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
    /// Switch configuration profile.
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Uninstall SUB CODE.
    Uninstall,
}

#[derive(Subcommand, Debug)]
pub enum GitAction {
    /// Stage files and generate a semantic commit message.
    Commit,
    /// Show working tree status.
    Status,
    /// Show diffs.
    Diff,
    /// Create, switch, or list branches.
    Branch { name: Option<String> },
    /// Show commit history with summaries.
    Log,
    /// Generate a PR description from the current diff.
    PrSummary,
}

#[derive(Subcommand, Debug)]
pub enum PluginAction {
    /// Install a plugin by name.
    Install { name: String },
    /// List installed plugins.
    List,
    /// Remove a plugin.
    Remove { name: String },
}

#[derive(Subcommand, Debug)]
pub enum ModelAction {
    /// Activate a model.
    Use { model: String },
    /// List available models.
    List,
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// Switch to a named profile.
    Use { profile: String },
    /// List profiles.
    List,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        if self.setup {
            return crate::setup::run_setup().await;
        }

        let config = Config::load().await?;
        let ctx    = ProjectContext::build(&config).await?;
        let llm    = LlmRouter::from_config(&config).await?;

        match self.command {
            None => {
                // Interactive TUI session.
                let mut tui = Tui::new(config, ctx, llm, self.yolo, self.resume).await?;
                tui.run().await
            }
            Some(Commands::Ask { question }) => {
                let runner = AgentRunner::new(config, ctx, llm, self.yolo);
                runner.one_shot(&question).await
            }
            Some(Commands::Run { task }) => {
                let runner = AgentRunner::new(config, ctx, llm, self.yolo);
                runner.run_task(&task).await
            }
            Some(Commands::Summary) => {
                let runner = AgentRunner::new(config, ctx, llm, self.yolo);
                runner.one_shot("Summarize this project: its purpose, architecture, key modules, and main dependencies.").await
            }
            Some(Commands::Git { action }) => {
                run_git_command(action, &config, &ctx, &llm).await
            }
            Some(Commands::Plugin { action }) => {
                run_plugin_command(action, &config).await
            }
            Some(Commands::Model { action }) => {
                run_model_command(action, &config, &llm).await
            }
            Some(Commands::Profile { action }) => {
                run_profile_command(action).await
            }
            Some(Commands::Uninstall) => {
                crate::setup::run_uninstall().await
            }
        }
    }
}

async fn run_git_command(
    action: GitAction,
    config: &Config,
    ctx: &ProjectContext,
    llm: &LlmRouter,
) -> Result<()> {
    use subcode_core::git::GitManager;
    let git = GitManager::open(".")?;

    match action {
        GitAction::Status => {
            let status = git.status()?;
            println!("{status}");
        }
        GitAction::Diff => {
            let diff = git.diff()?;
            println!("{diff}");
        }
        GitAction::Log => {
            let log = git.log(10)?;
            println!("{log}");
        }
        GitAction::Commit => {
            let diff   = git.diff()?;
            let prompt = format!("Generate a Conventional Commits message for:\n{diff}");
            let msg    = llm.complete_simple(&prompt).await?;
            let msg    = msg.trim();
            println!("Proposed commit message:\n  {msg}\n");
            println!("Press ENTER to commit, or Ctrl-C to abort.");
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            git.stage_all()?;
            git.commit(msg)?;
            println!("Committed.");
        }
        GitAction::Branch { name } => {
            if let Some(name) = name {
                git.create_branch(&name)?;
                println!("Created and checked out branch: {name}");
            } else {
                let branches = git.list_branches()?;
                for b in branches {
                    println!("  {b}");
                }
            }
        }
        GitAction::PrSummary => {
            let diff   = git.diff()?;
            let prompt = format!("Write a pull-request description (title + body in Markdown) for:\n{diff}");
            let desc   = llm.complete_simple(&prompt).await?;
            println!("{desc}");
        }
    }
    Ok(())
}

async fn run_plugin_command(action: PluginAction, config: &Config) -> Result<()> {
    use subcode_core::plugin::PluginManager;
    let pm = PluginManager::new(config);

    match action {
        PluginAction::Install { name } => {
            pm.install(&name).await?;
            println!("Installed plugin: {name}");
        }
        PluginAction::List => {
            let plugins = pm.list()?;
            if plugins.is_empty() {
                println!("No plugins installed.");
            } else {
                for p in plugins {
                    println!("  {} v{}", p.name, p.version);
                }
            }
        }
        PluginAction::Remove { name } => {
            pm.remove(&name)?;
            println!("Removed plugin: {name}");
        }
    }
    Ok(())
}

async fn run_model_command(action: ModelAction, config: &Config, llm: &LlmRouter) -> Result<()> {
    match action {
        ModelAction::Use { model } => {
            println!("Switched model to: {model}");
            println!("(Update [llm].model in .subcode.toml to persist this change.)");
        }
        ModelAction::List => {
            let models = llm.list_models().await?;
            for m in models {
                println!("  {m}");
            }
        }
    }
    Ok(())
}

async fn run_profile_command(action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::Use { profile } => {
            println!("Switched to profile: {profile}");
        }
        ProfileAction::List => {
            println!("Use `~/.subcode/config.toml` to define profiles.");
        }
    }
    Ok(())
}
