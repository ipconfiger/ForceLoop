use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

pub struct TryFinish;

impl Executable for TryFinish {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for TryFinish {
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
