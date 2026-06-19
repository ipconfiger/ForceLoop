use anyhow::Result;
use clap::Parser;
use forceloop::cli::{Cli, Commands};
use forceloop::compiler::Target;
use forceloop::context::Context;
use forceloop::traits::Executable;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = Context::new();

    match cli.command {
        Commands::Setup { tool } => {
            // Convert CLI `Tool` values to compiler `Target` values at
            // the dispatch boundary. The Setup business logic operates
            // exclusively on `Target` (not on clap-layer `Tool`).
            let targets: Vec<Target> = tool.into_iter().map(Target::from).collect();
            let ctx = Context::with_targets(targets);
            forceloop::setup::Setup.execute(&ctx)?;
        }
        Commands::New => forceloop::commands::New.execute(&ctx)?,
        Commands::Plan => forceloop::commands::Plan.execute(&ctx)?,
        Commands::Audit => forceloop::commands::Audit.execute(&ctx)?,
        Commands::Implement => forceloop::commands::Implement.execute(&ctx)?,
        Commands::Review => forceloop::commands::Review.execute(&ctx)?,
        Commands::Gate => forceloop::gate::Gate.execute(&ctx)?,
        Commands::Status => forceloop::status::Status.execute(&ctx)?,
        Commands::Archive => forceloop::archive::Archive.execute(&ctx)?,
    }

    Ok(())
}
