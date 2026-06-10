use crate::context::Context;
use crate::errors::Result;
use crate::traits::{CommandMetadata, Executable};

pub struct Audit;

impl Executable for Audit {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for Audit {
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
