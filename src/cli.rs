use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forceloop")]
#[command(about = "A CLI tool for structured development workflow")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize project directory structure, state, subcommands, skills, and hooks
    Setup,
    /// Gate control command, typically invoked by hooks
    Gate,
    /// View current status
    Status,
    /// Archive development plan
    Archive,
}
