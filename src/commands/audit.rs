use crate::constants::AUDIT_FILE;
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, verify_checklist, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Audit Skill

Cross-verify the design specs and development plans for consistency,
feasibility, and completeness. Fix any CRITICAL or HIGH issues found,
then generate the final audit report.

Read from: `.forceloop/specs/` and `.forceloop/plans/`

## Steps
0. Run the shell command `fl audit`.
   This checks that specs and plans are ready before proceeding.
1. Read all spec files from `.forceloop/specs/` (start with `index.md`).
2. Read all plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Cross-verify across ALL of the following dimensions:

   **a. Design conflicts** — Do any spec modules contradict each other?
   **b. Plan-spec misinterpretation** — Does any plan wave misread the spec's
      intent or omit a spec requirement?
   **c. Missing coverage** — Is every spec aspect addressed by at least one
      plan wave? (Bidirectional: track each spec → its plan wave(s).)
   **d. Contradictory requirements** — Are there requirements across modules
      that cannot simultaneously be true?
   **e. Feasibility** — Can each plan wave be completed given the project's
      constraints (tech stack, dependencies, existing code patterns)?
   **f. Dependency coherence** — Are all `Depends on:` declarations correct?
      Is the wave DAG acyclic? Do all dependency references resolve?
   **g. Test adequacy** — Do the test requirements in each wave cover the
      boundaries and edge cases specified in the spec?
   **h. NFR coverage** — Are all quantified NFRs from the spec addressed in
      the plan (either as a test requirement or explicit implementation
      concern)?

4. **Remediation**: For every CRITICAL or HIGH issue found:
   a. Determine which file(s) need fixing (a spec file under
      `.forceloop/specs/` or a plan file under `.forceloop/plans/`).
   b. Edit the file(s) to fix the issue. Add a comment note in the file
      if helpful (e.g. \"Audit fix: resolved NFR quantification\").
   c. Re-read the affected files to confirm the fix resolves the issue.
   d. If a CRITICAL or HIGH issue genuinely cannot be fixed without breaking
      the goal, mark it as \"escalated\" in the report with the reason.
   Only proceed after all CRITICAL and HIGH issues are resolved or
   explicitly escalated.

5. Write the audit report to `.forceloop/audit.md` with:

   - **Summary of findings** (high-level overview, 2-4 sentences)

   - **Issues** — each issue MUST use this format:
     - **Severity**: CRITICAL | HIGH | MEDIUM | LOW
     - **Location**: [file reference, e.g. `architecture.md:12`]
     - **Description**: [what the issue is]
     - **Recommended fix**: [concrete action — \"modify plan wave 2 to add X\",
       not \"improve coverage\"]
     - **Status**: fixed / escalated / no_action

   - **Quality Scores** (score each / 10):
     - Spec completeness: score / 10
     - Plan feasibility: score / 10
     - Spec-plan alignment: score / 10

   - **Recommendation** (one of):
     - **Approved** — no blocking issues; ready for implement
     - **Conditional** — all CRITICAL/HIGH fixed; MEDIUM/LOW items tracked
     - **Blocked** — CRITICAL/HIGH issues remain unresolved; must fix before
       proceeding

   - A **checklist** at the end with ALL audit items completed
     (`- [x]` or `- [✅]`). Every item MUST be marked completed.
     The gate will reject the report if any item is still `- [ ]`.

## Verification
- `.forceloop/audit.md` exists.
- All checklist items are `- [x]` or `- [✅]` — none left as `- [ ]`.
- No CRITICAL or HIGH issues remain unresolved (all fixed or escalated).
- Quality scores are assigned.
- Recommendation verdict is present.
";

const COMMAND_PROMPT: &str = "\
Audit the design specs and development plans for consistency,
feasibility, and completeness. Fix any CRITICAL/HIGH issues found,
then generate `.forceloop/audit.md` with a completed checklist.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl audit`.
   This checks that specs and plans are ready before proceeding.
1. Read all spec files from `.forceloop/specs/` (start with `index.md`).
2. Read all plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Cross-verify across 8 dimensions: design conflicts, plan-spec
   misinterpretation, missing coverage, contradictory requirements,
   feasibility, dependency coherence (DAG), test adequacy, NFR coverage.
4. **Remediation**: Edit spec/plan files to fix CRITICAL/HIGH issues.
   Only escalate if genuinely unfixable.
5. Write `.forceloop/audit.md` with: Summary, Issues (severity/location/
   description/recommended-fix/status), Quality Scores (/10 each),
   Recommendation verdict (Approved/Conditional/Blocked), checklist all `- [x]`.

## Verification
- `.forceloop/audit.md` exists.
- Every checklist item completed; no CRITICAL/HIGH unresolved.
- Quality scores and recommendation verdict present.
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
                "Audit report incomplete. Re-run the audit and ensure the \
                 remediation step fixed all CRITICAL/HIGH issues.".into(),
            )
        })?;

        // 2. Verify all checklist items are completed.
        verify_checklist(&report_path).map_err(|_| {
            ForceLoopError::Execution(
                "Audit report has uncompleted checklist items. Re-run the \
                 audit and complete all items.".into(),
            )
        })
    }
}