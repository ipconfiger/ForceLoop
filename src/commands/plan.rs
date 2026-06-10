use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

const SKILL_PROMPT: &str = "\
# Plan Skill

Decompose the design spec into a sequence of development phases.

## Steps
1. Read the design spec from `.forceloop/plan.json#spec`.
2. Identify the major implementation phases.
3. For each phase define:
   - `id` (kebab-case)
   - `title`
   - `acceptance_criteria` (verifiable)
   - `dependencies` (other phase ids)
4. Write phases to `.forceloop/plan.json#phases`.
5. Update `state.json` to first pending phase.

## Validation
- All phases have `acceptance_criteria`.
- Dependency graph is acyclic.
- At least one phase exists.
";

const COMMAND_PROMPT: &str = "\
Create a development plan from the design spec.

Decomposes the goal into phases, each with verifiable acceptance
criteria, and writes them to `.forceloop/plan.json`.
";

fn plan_skill() -> CommandSchema {
    CommandSchema {
        name: "plan",
        description: "Create development plan (multiple phases)",
        model: None,
        argument_hint: Some("[spec reference]"),
        tools: &["Read", "Write"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn plan_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..plan_skill()
    }
}

pub struct Plan;

impl Executable for Plan {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for Plan {
    fn skill_template(&self) -> CommandSchema {
        plan_skill()
    }
    fn command_template(&self) -> CommandSchema {
        plan_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/plan.json"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
