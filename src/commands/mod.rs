pub mod dash;
pub mod deps;
pub mod env;
pub mod init;
pub mod logs;
pub mod plugin;
pub mod port;
pub mod snap;
pub mod up;
pub mod watch;

use crate::cli::{Cli, Command, EnvMode, SnapMode};
use anyhow::Result;

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Up => up::run().await,
        Command::Port(args) => port::run(args).await,
        Command::Watch => watch::run().await,
        Command::Env(args) => match args.mode {
            EnvMode::Doctor => env::doctor().await,
            EnvMode::Fix => env::fix().await,
            EnvMode::Diff => env::diff().await,
        },
        Command::Logs => logs::run().await,
        Command::Deps => deps::run().await,
        Command::Snap(args) => match args.mode {
            SnapMode::Save => snap::save().await,
            SnapMode::Restore => snap::restore().await,
        },
        Command::Dash => dash::run().await,
        Command::Init => init::run().await,
        Command::Plugin(args) => plugin::run(args).await,
    }
}
