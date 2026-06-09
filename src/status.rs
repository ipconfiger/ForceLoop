use crate::context::Context;
use crate::errors::Result;
use crate::traits::{Executable, Subcommand};

pub struct Status;

impl Executable for Status {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Status {
    fn name(&self) -> &'static str {
        "status"
    }
    fn description(&self) -> &'static str {
        "View current status"
    }
}
