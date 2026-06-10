use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

const SKILL_PROMPT: &str = "\
# Implement Skill

Develop the current phase following TDD (test first, code second).

## Steps
1. Read `.forceloop/plan.json` to identify the current phase.
2. For each acceptance criterion:
   - Write a failing test
   - Run it, confirm it fails for the right reason
   - Write minimal code to pass
   - Refactor
3. Run `cargo check && cargo test && cargo clippy --all-targets`.
4. Update phase status in `plan.json`.
5. Call `gate` skill; on pass, advance to next phase.

## Constraints
- TDD mandatory: tests before code
- No commented-out tests, no `unwrap()` in production
- All checks must pass before commit
";

const COMMAND_PROMPT: &str = "\
Implement the current development phase.

Follows TDD: writes tests first, code second, then refactors.
Advances the pipeline on gate pass.
";

fn implement_skill() -> CommandSchema {
    CommandSchema {
        name: "implement",
        description: "Develop the current phase (TDD)",
        model: None,
        argument_hint: Some("[phase-id]"),
        tools: &["Read", "Write", "Edit", "Bash", "Grep", "Glob"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn implement_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..implement_skill()
    }
}

pub struct Implement;

impl Executable for Implement {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for Implement {
    fn skill_template(&self) -> CommandSchema {
        implement_skill()
    }
    fn command_template(&self) -> CommandSchema {
        implement_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
