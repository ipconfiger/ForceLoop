use crate::context::Context;
use crate::errors::Result;
use crate::traits::Executable;

pub struct Audit;

impl Executable for Audit {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}
