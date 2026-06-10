use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Archive Plan Skill

Move a completed development plan to `.forceloop/archive/`.

## Steps
1. Read `.forceloop/plan.json` (current plan + spec).
2. Generate archive metadata (id = ISO timestamp, summary = goal).
3. Move `plan.json` to `.forceloop/archive/<id>.json`.
4. Reset `state.json` to phase 0 (ready for a new plan).

## Verification
- `.forceloop/archive/<id>.json` exists and is readable.
- `.forceloop/plan.json` is either empty or removed.
";

const COMMAND_PROMPT: &str = "\
Archive the completed development plan.

Moves `plan.json` to `.forceloop/archive/<id>.json` with a timestamped
id. Use after `try_finish` succeeds, to start a fresh plan.
";

fn archive_skill() -> CommandSchema {
    CommandSchema {
        name: "archive",
        description: "Archive development plan",
        model: None,
        argument_hint: Some("[plan-id]"),
        tools: &["Bash", "Read"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn archive_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..archive_skill()
    }
}

pub struct Archive;

impl Executable for Archive {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Archive {
    fn name(&self) -> &'static str {
        "archive"
    }
    fn description(&self) -> &'static str {
        "Archive development plan"
    }
}

impl CommandMetadata for Archive {
    fn skill_template(&self) -> CommandSchema {
        archive_skill()
    }
    fn command_template(&self) -> CommandSchema {
        archive_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/archive/"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
