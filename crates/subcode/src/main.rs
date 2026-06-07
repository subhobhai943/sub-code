// SUB CODE — AI Coding Assistant | Author: subhobhai | License: MIT

mod cli;
mod setup;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise structured logging — off by default, RUST_LOG env activates it.
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let cli = cli::Cli::parse();
    cli.run().await
}
