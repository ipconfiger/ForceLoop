use crate::context::Context;
use crate::errors::Result;
use crate::traits::Executable;

pub struct New;

impl Executable for New {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}
