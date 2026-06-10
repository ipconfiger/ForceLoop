use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Gate Control Skill

Verify whether the current pipeline step can advance to the next.

## Behavior
1. Read `.forceloop/state.json` to determine the current step.
2. Invoke `gate()` on the active `Command` object.
3. Emit PASS/FAIL signal to stdout.
4. On PASS, advance `state.json` to the next phase.
5. On FAIL, surface the gate reason and stop.

## Invocation
Called by git hooks (post-commit, pre-push) and the `forceloop gate` CLI subcommand.
";

const COMMAND_PROMPT: &str = "\
Gate control — verify the current pipeline step can advance.

Reads `.forceloop/state.json`, calls `gate()` on the active command,
emits PASS/FAIL. Used by git hooks; rarely invoked manually.
";

fn gate_skill() -> CommandSchema {
    CommandSchema {
        name: "gate",
        description: "Gate control command, typically invoked by hooks",
        model: None,
        argument_hint: None,
        tools: &["Read"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn gate_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..gate_skill()
    }
}

pub struct Gate;

impl Executable for Gate {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
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

impl CommandMetadata for Gate {
    fn skill_template(&self) -> CommandSchema {
        gate_skill()
    }
    fn command_template(&self) -> CommandSchema {
        gate_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
