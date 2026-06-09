use crate::context::Context;
use crate::errors::Result;
use crate::traits::Executable;

pub struct TryFinish;

impl Executable for TryFinish {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}
