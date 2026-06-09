use anyhow::Result;
use clap::Parser;
use forceloop::cli::{Cli, Commands};
use forceloop::context::Context;
use forceloop::traits::Executable;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = Context::new();

    match cli.command {
        Commands::Setup => forceloop::setup::Setup.execute(&ctx)?,
        Commands::Gate => forceloop::gate::Gate.execute(&ctx)?,
        Commands::Status => forceloop::status::Status.execute(&ctx)?,
        Commands::Archive => forceloop::archive::Archive.execute(&ctx)?,
    }

    Ok(())
}
