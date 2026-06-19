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
1. **Understand the project context**: Before writing anything, search the
   codebase to understand existing patterns, architecture, and constraints.
   The spec MUST build on the existing project structure, not assume a
   greenfield project.
2. **Interview the user about the goal**.
   Ask ONE question at a time, building on each answer. Cover these dimensions
   in order (skip any the user has already provided):

   - **Goal & success**: What user problem does this solve? What's the primary
     success metric?
   - **Scope**: What's the boundary? What is explicitly NOT in scope?
   - **Users & frequency**: Who uses it? How often? Interactive or batch?
   - **Technical constraints**: Must-use / must-not-use tech? Compatibility
     requirements?
   - **Risk tolerance**: What happens if this breaks? Is data loss acceptable?

   Use the goal description above as a starting point. If the user provides
   reference documents, read and incorporate them.
   If the user has already provided enough detail, skip the interview.
3. **Analyze the goal and decompose into logical modules**.
   Apply these decomposition heuristics:
   - Each module covers ONE orthogonal concern (separation of concerns)
   - If two modules have extensive cross-references, consider merging them
   - If a module would exceed ~300 lines of content, consider splitting
   - Aim for 3-8 modules per spec; <3 is likely too coarse, >8 likely too fine
   - Each module must have clearly scoped Inputs & Outputs
4. **For each module, create an independent markdown file** under
   `.forceloop/specs/`. File name must be kebab-case with a `.md` extension
   (e.g. `architecture.md`, `data-model.md`).

   Each file MUST follow this structured template:

   ```
   # Module: [kebab-case-name]

   **Type**: Architecture / Data Model / API / UI / Security / Workflow / ...

   ## Purpose
   - What problem does this module solve? (1-3 sentences)
   - What is explicitly NOT in scope?

   ## Inputs & Outputs
   - **Inputs**: [data, events, user actions]
   - **Outputs**: [data, files, side effects]
   - **External dependencies**: [services, libraries, APIs]

   ## Key Design Decisions
   For each open decision, list:
   - **Question**: [what needs deciding]
   - **Options** (>=2): Option A (pros/cons), Option B (pros/cons)
   - **Current preference**: [A / B / undecided]

   ## Non-Functional Requirements
   - [Quantified requirements only — NO vague terms like \"fast\" or \"secure\"]
   - e.g. \"p99 latency < 200ms\", \"RTO < 5 min\", \"support 100 concurrent users\"

   ## Constraints & Assumptions
   - [explicit constraints, e.g. \"no new database dependency\"]
   - [assumptions that, if wrong, would change the design]

   ## Boundaries & Edge Cases
   - [error scenarios, failure modes, boundary conditions]

   ## Cross-References
   - [[related-module-1]] — [relationship description]
   ```
5. Create the index file `.forceloop/specs/index.md` that:
   - Lists all modules with a brief description
   - Links to each module using wiki link syntax: `[[module-file]]`
   - Follows any cross-references between modules
6. **Self-Check**: Review all created spec files for:
   - **Completeness**: does every module have all required sections filled?
   - **Consistency**: no contradictory statements across modules
   - **Quantification**: every NFR is quantified (or marked \"TBD with [owner]\")
   - **Risk identification**: key risks documented under Boundaries
   If any file fails a check, fix it before proceeding.
7. Verify that all wiki links in `index.md` and module files resolve
   to existing files under `.forceloop/specs/`.

## Verification
- `.forceloop/specs/index.md` exists.
- Every wiki link `[[...]]` in index.md resolves to a module file.
- Each module file has all required sections filled (no empty sections).
- Each NFR is quantified (no vague terms like \"fast\" or \"secure\").
- Module count is between 3 and 8 (or explicitly justified if outside this range).
";

/// Slash command: invoke the new-goal workflow.
const COMMAND_PROMPT: &str = "\
Create a new development goal and modular design spec.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl new`.
   This creates the `.forceloop/specs/` directory scaffold.
1. **Understand the project context**: Search the codebase to understand
   existing patterns, architecture, and constraints. The spec MUST build on
   the existing project structure.
2. **Interview the user about the goal**.
   Ask ONE question at a time. Cover these dimensions in order:
   - Goal & success metric, Scope & exclusions, Users & frequency,
     Technical constraints, Risk tolerance
3. **Analyze and decompose** using heuristics: one concern per module,
   3-8 modules per spec, clear in/out boundaries.
4. **For each module, create a markdown file** under `.forceloop/specs/`
   using the structured template (Purpose, Inputs/Outputs, Design Decisions,
   NFR, Constraints, Boundaries, Cross-References — see skill prompt).
5. Create `.forceloop/specs/index.md` with wiki links.
6. **Self-Check**: completeness, consistency, NFR quantification, risk docs.
7. Verify all wiki links resolve.

## Verification
- `.forceloop/specs/index.md` exists.
- Every wiki link resolves.
- Each module file has all required sections.
- NFRs are quantified; module count 3-8 or justified.
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
