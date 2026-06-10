use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

/// Skill workflow: create a new development goal and design spec.
const SKILL_PROMPT: &str = "\
# New Goal Skill

Create a new development goal and design specification.

## Steps
1. Interview the user about the goal (purpose, scope, constraints).
2. Capture goal in `.forceloop/plan.json#goal`.
3. Write a design spec covering:
   - Problem statement
   - Success criteria (measurable)
   - Non-goals
   - High-level approach
   - Open questions
4. Hand off to the `audit` skill for review (do not advance yet).

## Verification
- `.forceloop/plan.json` contains `goal` and `spec` sections.
- `spec.success_criteria` is non-empty.
";

/// Slash command: invoke the new-goal workflow.
const COMMAND_PROMPT: &str = "\
Create a new development goal and design spec.

Interactively captures the goal and produces a design spec,
which is then validated by the `audit` skill.
";

fn new_skill() -> CommandSchema {
    CommandSchema {
        name: "new",
        description: "Create a new development goal and design spec",
        model: None,
        argument_hint: Some("[goal description]"),
        tools: &["Read", "Write"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn new_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..new_skill()
    }
}

pub struct New;

impl Executable for New {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for New {
    fn skill_template(&self) -> CommandSchema {
        new_skill()
    }
    fn command_template(&self) -> CommandSchema {
        new_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/plan.json"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
