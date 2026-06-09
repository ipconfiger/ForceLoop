use crate::context::Context;
use crate::errors::Result;
use crate::traits::{Executable, Subcommand};

pub struct Setup;

impl Executable for Setup {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Setup {
    fn name(&self) -> &'static str {
        "setup"
    }
    fn description(&self) -> &'static str {
        "Initialize project directory structure, state, subcommands, skills, and hooks"
    }
}
