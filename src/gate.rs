use crate::commands::{Audit, Implement, New, Plan, Review, TryFinish};
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::state::{PipelinePhase, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

pub struct Gate;

impl Executable for Gate {
    fn execute(&self, ctx: &Context) -> Result<()> {
        // 1. Locate and read current pipeline state
        let state_path = PipelineState::locate_state_file()?;
        let state = PipelineState::read_or_default(&state_path)?;

        if state.current_phase == PipelinePhase::Done {
            println!("Pipeline: all phases complete (current phase: Done).");
            return Ok(());
        }

        // 2. Call gate() on the command corresponding to the current phase.
        //    The gate verifies that the step has been completed satisfactorily.
        match state.current_phase {
            PipelinePhase::New => New.gate(ctx)?,
            PipelinePhase::Plan => Plan.gate(ctx)?,
            PipelinePhase::Audit => Audit.gate(ctx)?,
            PipelinePhase::Implement => Implement.gate(ctx)?,
            PipelinePhase::Review => Review.gate(ctx)?,
            PipelinePhase::TryFinish => TryFinish.gate(ctx)?,
            PipelinePhase::Done => unreachable!(),
        }

        // 3. Gate passed — advance to next phase
        let next_phase = state
            .next_phase()
            .ok_or_else(|| ForceLoopError::Execution("pipeline already complete".into()))?;
        let next_state = PipelineState {
            current_phase: next_phase,
        };
        next_state.write(&state_path)?;

        println!("✓ Gate passed: {} → {}", state.current_phase, next_phase);
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
