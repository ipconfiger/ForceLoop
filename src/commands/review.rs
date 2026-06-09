use crate::context::Context;
use crate::errors::Result;
use crate::traits::Executable;

pub struct Review;

impl Executable for Review {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}
