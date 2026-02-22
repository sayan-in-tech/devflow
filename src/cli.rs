use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "devflow", version, about = "Developer workflow automation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Up,
    Port(PortArgs),
    Watch,
    Env(EnvArgs),
    Logs,
    Deps,
    Snap(SnapArgs),
    Dash,
    Init,
    Plugin(PluginArgs),
}

#[derive(Debug, Args)]
pub struct PortArgs {
    #[arg(long)]
    pub free: bool,
    #[arg(long)]
    pub watch: bool,
    #[arg(short, long)]
    pub port: Option<u16>,
}

#[derive(Debug, Args)]
pub struct EnvArgs {
    #[arg(value_enum)]
    pub mode: EnvMode,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum EnvMode {
    Doctor,
    Fix,
    Diff,
}

#[derive(Debug, Args)]
pub struct SnapArgs {
    #[arg(value_enum)]
    pub mode: SnapMode,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SnapMode {
    Save,
    Restore,
}

#[derive(Debug, Args)]
pub struct PluginArgs {
    pub name: String,
    #[arg(short, long)]
    pub payload: Option<String>,
}
