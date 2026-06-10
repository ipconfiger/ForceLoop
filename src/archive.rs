use crate::context::Context;
use crate::errors::Result;
use crate::traits::{CommandMetadata, Executable, Subcommand};

pub struct Archive;

impl Executable for Archive {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Archive {
    fn name(&self) -> &'static str {
        "archive"
    }
    fn description(&self) -> &'static str {
        "Archive development plan"
    }
}

impl CommandMetadata for Archive {
    fn skill_template(&self) -> &'static str {
        ""
    }
    fn command_template(&self) -> &'static str {
        ""
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
