use crate::context::Context;
use crate::errors::Result;
use crate::traits::Executable;

pub struct Plan;

impl Executable for Plan {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}
