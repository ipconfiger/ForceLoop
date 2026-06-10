use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Project Setup Skill

Initialize the current directory as a ForceLoop project.

## Steps
1. Create `.forceloop/{skills,commands,hooks,archive}/` directory tree.
2. Write initial `.forceloop/state.json` (phase 0, no active command).
3. Generate platform-native command files from `CommandMetadata`:
   - `.claude/commands/<name>.md` for Claude
   - `.opencode/command/<name>.md` for OpenCode
4. Install git hooks for `gate` control.
5. Print summary of installed components.

## Verification
- `.forceloop/state.json` exists
- 10 command files generated
- Hooks executable
";

const COMMAND_PROMPT: &str = "\
Initialize the current directory as a ForceLoop project.

Creates `.forceloop/` tree, generates platform-native command files
(Claude + OpenCode), installs hooks, writes initial state.

Use once per project, or to repair corrupted state.
";

fn setup_skill() -> CommandSchema {
    CommandSchema {
        name: "setup",
        description: "Initialize project directory structure, state, subcommands, skills, and hooks",
        model: None,
        argument_hint: None,
        tools: &["Bash", "Read", "Write", "Glob"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn setup_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..setup_skill()
    }
}

pub struct Setup;

impl Executable for Setup {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl Subcommand for Setup {
    fn name(&self) -> &'static str {
        "setup"
    }
    fn description(&self) -> &'static str {
        "Initialize project directory structure, state, subcommands, skills, and hooks"
    }
}

impl CommandMetadata for Setup {
    fn skill_template(&self) -> CommandSchema {
        setup_skill()
    }
    fn command_template(&self) -> CommandSchema {
        setup_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/state.json"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
