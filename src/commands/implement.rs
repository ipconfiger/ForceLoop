use crate::context::Context;
use crate::errors::Result;
use crate::traits::Executable;

pub struct Implement;

impl Executable for Implement {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}
