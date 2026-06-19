use crate::constants::AUDIT_FILE;
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, verify_checklist, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Audit Skill

Cross-verify the design specs and development plans for consistency.
Generate an audit report with a completed checklist.

Read from: `.forceloop/specs/` and `.forceloop/plans/`

## Steps
0. Run the shell command `fl audit`.
   This checks that specs and plans are ready before proceeding.
1. Read all spec files from `.forceloop/specs/` (start with `index.md`).
2. Read all plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Cross-verify for:
   - Design conflicts between spec modules
   - Plan waves that misinterpret the spec intent
   - Missing coverage in plans (spec aspects not addressed)
   - Contradictory requirements across modules
4. Write the audit report to `.forceloop/audit.md` with:
   - Summary of findings
   - Severity-rated issues (CRITICAL / HIGH / MEDIUM / LOW)
   - A **checklist** at the end with all audit items.
     Every item MUST be marked `- [x]` or `- [✅]` (completed).
     The gate will reject the report if any item is still `- [ ]`.

## Verification
- `.forceloop/audit.md` exists.
- All checklist items are `- [x]` or `- [✅]` — none left as `- [ ]`.
- No CRITICAL or HIGH issues remain unresolved.
";

const COMMAND_PROMPT: &str = "\
Audit the design specs and development plans for consistency.
Generate `.forceloop/audit.md` with a completed checklist.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl audit`.
   This checks that specs and plans are ready before proceeding.
1. Read all spec files from `.forceloop/specs/` (start with `index.md`).
2. Read all plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Cross-verify for design conflicts, misinterpretations, missing coverage.
4. Write `.forceloop/audit.md` with findings + checklist (all `- [x]`).

## Verification
- `.forceloop/audit.md` exists.
- Every checklist item is completed (`- [x]` or `- [✅]`).
- No CRITICAL or HIGH issues remain.
";

fn audit_skill() -> CommandSchema {
    CommandSchema {
        name: "fl-audit",
        description: "Audit design spec and development plan",
        model: None,
        argument_hint: Some("[files...]"),
        tools: &["Read", "Grep", "Glob"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn audit_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..audit_skill()
    }
}

pub struct Audit;

impl Executable for Audit {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        // 1. Read current pipeline state
        let state_path = PipelineState::locate_state_file()?;
        let state = PipelineState::read_or_default(&state_path)?;

        // 2. Check prerequisites: both specs (new) and plans must be done.
        if !state.new {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: specs not ready. \
                 Run `/fl-new` first."
                    .into(),
            ));
        }
        if !state.plan {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: plans not ready. \
                 Run `/fl-plan` first."
                    .into(),
            ));
        }

        // 3. Prerequisites met — pass to LLM via prompt.

        Ok(())
    }
}

impl Subcommand for Audit {
    fn name(&self) -> &'static str {
        "audit"
    }
    fn description(&self) -> &'static str {
        "Audit design spec and development plan"
    }
}

impl CommandMetadata for Audit {
    fn skill_template(&self) -> CommandSchema {
        audit_skill()
    }
    fn command_template(&self) -> CommandSchema {
        audit_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/audit.md"]
    }
    fn check_list(&self) -> bool {
        true
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let report_path = forceloop_dir.join(AUDIT_FILE);

        // 1. Verify artifact exists and wiki links are valid.
        verify_artifact(&report_path).map_err(|_| {
            ForceLoopError::Execution(
                "Audit report incomplete. Re-run the audit.".into(),
            )
        })?;

        // 2. Verify all checklist items are completed.
        verify_checklist(&report_path).map_err(|_| {
            ForceLoopError::Execution(
                "Audit report incomplete. Re-run the audit.".into(),
            )
        })
    }
}