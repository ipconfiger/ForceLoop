use std::fs;

use crate::constants::{SPECS_DIR, SPECS_INDEX};
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

/// Skill workflow: create a new development goal and design spec.
const SKILL_PROMPT: &str = "\
# New Goal Skill

Create a new development goal and modular design specification.

Goal description: $ARGUMENTS

## Steps
0. First, run `fl new` to create the `.forceloop/specs/` directory
   (this ensures the directory scaffold exists before writing files).
1. Interview the user about the goal (purpose, scope, constraints).
   Use the goal description above as a starting point. If the user
   provides reference documents, read and incorporate them.
2. Analyze the goal and decompose it into logical modules. Each module
   should cover a distinct aspect of the design (e.g. architecture,
   data model, API, UI, security, etc.).
3. For each module, create an independent markdown file under
   `.forceloop/specs/`. The file name should be kebab-case with a
   `.md` extension (e.g. `architecture.md`, `data-model.md`).
4. Create an index file `.forceloop/specs/index.md` that:
   - Lists all modules with a brief description
   - Links to each module using wiki link syntax: `[[module-file]]`
   - Follows any cross-references between modules
5. Verify that all wiki links in `index.md` and module files resolve
   to existing files under `.forceloop/specs/`.

## Verification
- `.forceloop/specs/index.md` exists.
- Every wiki link `[[...]]` in index.md resolves to a module file.
- Each module file has a clear title and content.
";

/// Slash command: invoke the new-goal workflow.
const COMMAND_PROMPT: &str = "\
Create a new development goal and modular design spec.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl new`.
   This creates the `.forceloop/specs/` directory scaffold.
1. Interview the user about the goal (purpose, scope, constraints).
   Use the goal description above as a starting point. If the user
   provides reference documents, read and incorporate them.
2. Analyze the goal and decompose it into logical modules. Each module
   should cover a distinct aspect of the design (e.g. architecture,
   data model, API, UI, security, etc.).
3. For each module, create an independent markdown file under
   `.forceloop/specs/`. The file name should be kebab-case with a
   `.md` extension (e.g. `architecture.md`, `data-model.md`).
4. Create an index file `.forceloop/specs/index.md` that:
   - Lists all modules with a brief description
   - Links to each module using wiki link syntax: `[[module-file]]`
   - Follows any cross-references between modules
5. Verify that all wiki links in `index.md` and module files resolve
   to existing files under `.forceloop/specs/`.

## Verification
- `.forceloop/specs/index.md` exists.
- Every wiki link `[[...]]` in index.md resolves to a module file.
- Each module file has a clear title and content.
";

fn new_skill() -> CommandSchema {
    CommandSchema {
        name: "fl-new",
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
        // 1. Locate .forceloop/ directory
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let specs_dir = forceloop_dir.join(SPECS_DIR);

        // 2. Create .forceloop/specs/ if not exists
        fs::create_dir_all(&specs_dir)?;

        Ok(())
    }
}

impl Subcommand for New {
    fn name(&self) -> &'static str {
        "new"
    }
    fn description(&self) -> &'static str {
        "Create a new development goal and design spec"
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
        &[".forceloop/specs/index.md"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let index_path = forceloop_dir.join(SPECS_INDEX);
        verify_artifact(&index_path).map_err(|_| {
            ForceLoopError::Execution(
                "Spec generation verification failed. Review the files under specs/ \
                 directory and regenerate if needed."
                    .into(),
            )
        })
    }
}
