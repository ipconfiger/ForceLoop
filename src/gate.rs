use crate::context::Context;
use crate::errors::Result;
use crate::traits::{Executable, Subcommand};

pub struct Gate;

impl Executable for Gate {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Gate {
    fn name(&self) -> &'static str {
        "gate"
    }
    fn description(&self) -> &'static str {
        "Gate control command, typically invoked by hooks"
    }
}
