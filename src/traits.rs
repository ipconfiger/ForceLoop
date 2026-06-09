use crate::context::Context;
use crate::errors::Result;

/// Trait shared by Skills and Commands
pub trait Executable {
    fn execute(&self, ctx: &Context) -> Result<()>;
}

/// Trait for top-level subcommands (setup, gate, status, archive)
pub trait Subcommand: Executable {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
