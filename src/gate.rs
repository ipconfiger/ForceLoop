use crate::commands::{Audit, Implement, New, Plan, Review};
use crate::context::Context;
use crate::errors::Result;
use crate::state::PipelineState;
use crate::traits::{CommandMetadata, Executable, Subcommand};

pub struct Gate;

impl Executable for Gate {
    fn execute(&self, ctx: &Context) -> Result<()> {
        // 1. Read current pipeline state
        let state_path = PipelineState::locate_state_file()?;
        let mut state = PipelineState::read_or_default(&state_path)?;

        // 2. Find the first uncompleted gate, run it, set its flag.
        //    This is idempotent — already-passed gates remain true.
        if !state.new {
            New.gate(ctx)?;
            state.new = true;
        } else if !state.plan {
            Plan.gate(ctx)?;
            state.plan = true;
        } else if !state.audit {
            Audit.gate(ctx)?;
            state.audit = true;
        } else if !state.implement {
            Implement.gate(ctx)?;
            state.implement = true;
        } else if !state.review {
            Review.gate(ctx)?;
            state.review = true;
        } else {
            state.done = true;
        }

        // 3. Write updated state
        state.write(&state_path)?;

        Ok(())
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