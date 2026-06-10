use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Status View Skill

Read `.forceloop/state.json` and `.forceloop/plan.json`, print a
human-readable summary:

- Current pipeline phase
- Last completed step
- Next pending step
- Gate status (can advance: yes / no + reason)
- Artifacts produced so far
- Plan goal (one line)
";

const COMMAND_PROMPT: &str = "\
View current ForceLoop pipeline status.

Reads state.json and plan.json, prints a summary of phase,
last/next step, gate status, and artifacts.
";

fn status_skill() -> CommandSchema {
    CommandSchema {
        name: "status",
        description: "View current status",
        model: None,
        argument_hint: None,
        tools: &["Read"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn status_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..status_skill()
    }
}

pub struct Status;

impl Executable for Status {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Status {
    fn name(&self) -> &'static str {
        "status"
    }
    fn description(&self) -> &'static str {
        "View current status"
    }
}

impl CommandMetadata for Status {
    fn skill_template(&self) -> CommandSchema {
        status_skill()
    }
    fn command_template(&self) -> CommandSchema {
        status_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
