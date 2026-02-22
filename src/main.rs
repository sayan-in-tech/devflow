use anyhow::Result;
use clap::Parser;
use devflow::{cli::Cli, commands};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .init();

    let cli = Cli::parse();
    commands::run(cli).await
}
