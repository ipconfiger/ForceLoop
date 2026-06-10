use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

const SKILL_PROMPT: &str = "\
# Try Finish Skill

Verify whether the development goal has been achieved.

## Steps
1. Read `.forceloop/plan.json` (goal + spec).
2. Compare produced artifacts against `spec.success_criteria`.
3. Run final smoke test.
4. Write `.forceloop/result.json`:
   - `goal`: original goal text
   - `achieved`: true / false
   - `evidence`: list of supporting artifacts
   - `gaps`: list of unmet criteria (if any)

## Decision
- All criteria met → `achieved: true`
- Any criterion unmet → `achieved: false`, list `gaps`
";

const COMMAND_PROMPT: &str = "\
Verify whether the development goal has been achieved.

Compares artifacts to the success criteria, writes
`.forceloop/result.json` with verdict and evidence.
";

fn try_finish_skill() -> CommandSchema {
    CommandSchema {
        name: "try-finish",
        description: "Verify whether the development goal has been achieved",
        model: None,
        argument_hint: Some("[goal reference]"),
        tools: &["Read", "Grep", "Bash"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn try_finish_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..try_finish_skill()
    }
}

pub struct TryFinish;

impl Executable for TryFinish {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for TryFinish {
    fn skill_template(&self) -> CommandSchema {
        try_finish_skill()
    }
    fn command_template(&self) -> CommandSchema {
        try_finish_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/result.json"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
