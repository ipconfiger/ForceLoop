use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

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

impl CommandMetadata for Gate {
    fn skill_template(&self) -> CommandSchema {
        CommandSchema::default()
    }
    fn command_template(&self) -> CommandSchema {
        CommandSchema::default()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
