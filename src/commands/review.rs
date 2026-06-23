use crate::constants::REVIEW_RESULT;
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, verify_checklist, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Review Skill

Cross-verify the implemented code against specs and plans across
structured dimensions. Run full test suite and produce a quality verdict.

## Steps
0. Run the shell command `fl review`.
   This checks that implement is complete before proceeding.
   **Also verify**: read `.forceloop/wave_state.md` and confirm ALL waves
   are marked `- [x]`. If not, run `/fl-implement` first.
0.5. **Read the audit report**: Check `.forceloop/audit.md` for known issues.
     Verify that issues marked \"fixed\" are genuinely resolved in the code.
     Flag any audit issue that appears unresolved.
1. Read spec files from `.forceloop/specs/` (start with `index.md`).
2. Read plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Read the wave state from `.forceloop/wave_state.md`.
4. Cross-verify the implemented code against specs and plans.
   Focus on git diff / changed files in the current wave.
   Check across these structured dimensions:

   **a. Spec compliance** — Does the code match each spec module's Purpose,
      Inputs & Outputs, and Constraints?
   **b. Acceptance criteria** — Are ALL plan Given/When/Then acceptance
      criteria satisfied?
   **c. Boundary handling** — Are all edge cases from the spec's \"Boundaries
      & Edge Cases\" section handled in the code?
   **d. NFR verification** — Are the spec's quantified NFRs addressed?
      (e.g. performance, security — run relevant checks if possible.)
   **e. Test adequacy** — Do the actual test cases cover the plan wave's
      Test Requirements (boundary/success/error)?
   **f. Regression** — `cargo test` passes including previous wave tests?
      `cargo clippy --all-targets` — zero warnings?
   **g. Code quality** — No `unwrap()` in production code? No commented-out
      code? No debug prints? Follows existing project patterns?

5. Run the full test suite: `cargo test`.
6. Run lints: `cargo clippy --all-targets`.
7. Ensure ALL tests pass and no lint errors remain.
8. Write the review result to `.forceloop/review_result.md` with:

   - **Summary** of findings (2-4 sentences)

   - **Issues** (if any):
     - **Severity**: CRITICAL | HIGH | MEDIUM | LOW
     - **Location**: [file:line reference]
     - **Description**: [what's wrong]
     - **Recommended fix**: [concrete action]
     - **Status**: open / fixed

   - **Verdict** (one of):
     - **Approved** — all dimensions pass; ready to proceed
     - **Needs Fixes** — issues found; fix and re-review
     - **Rejected** — CRITICAL issues; must re-implement

   - A **checklist** at the end with ALL review items.
     Every item MUST be marked `- [x]` or `- [✅]` (completed).

## Verification
- `.forceloop/review_result.md` exists.
- All checklist items are `- [x]` or `- [✅]`.
- All tests pass (`cargo test` returns 0).
- Clippy passes (`cargo clippy --all-targets` returns 0).
- Verdict and Issues sections are present.

## Pipeline Completion
**STOP**. The pipeline is complete.
The hook will automatically run `fl gate` to advance to \"done\".
";

const COMMAND_PROMPT: &str = "\
Review the implementation against specs and plans.

Arguments: $ARGUMENTS

## Steps
0. Run `fl review` first.
   Confirm all waves done in `.forceloop/wave_state.md`.
0.5. Read `.forceloop/audit.md` — verify audit fixes are genuinely resolved.
1. Read specs, plans, and wave state.
2. Cross-verify code across 7 dimensions: spec compliance, acceptance
   criteria, boundary handling, NFRs, test adequacy, regression, code quality.
   Focus on git diff / changed files.
3. Run `cargo test` and `cargo clippy --all-targets`.
4. Write `.forceloop/review_result.md` with: Summary, Issues (severity/
   location/recommended-fix/status), Verdict (Approved/Needs Fixes/Rejected),
   and checklist (all `- [x]`).

## Verification
- `cargo test` passes. `cargo clippy` passes.
- `.forceloop/review_result.md` has Verdict, Issues list, and all checkboxes completed.

## Pipeline Completion
**STOP**. The pipeline is complete.
The hook will advance the pipeline automatically.
";

fn review_skill() -> CommandSchema {
    CommandSchema {
        name: "fl-review",
        description: "Regression-validate the implementation (7-dimension cross-verify)",
        model: None,
        argument_hint: Some("[files...]"),
        tools: &["Read", "Grep", "Bash"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn review_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..review_skill()
    }
}

pub struct Review;

impl Executable for Review {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        // 1. Read current pipeline state
        let state_path = PipelineState::locate_state_file()?;
        let state = PipelineState::read_or_default(&state_path)?;

        // 2. Check prerequisite: implement must be complete.
        if !state.implement {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: implement not complete. \
                 Run `/fl-implement` first."
                    .into(),
            ));
        }

        // 3. Prerequisite met — pass to LLM via prompt.

        Ok(())
    }
}

impl Subcommand for Review {
    fn name(&self) -> &'static str {
        "review"
    }
    fn description(&self) -> &'static str {
        "Regression-validate the implementation"
    }
}

impl CommandMetadata for Review {
    fn skill_template(&self) -> CommandSchema {
        review_skill()
    }
    fn command_template(&self) -> CommandSchema {
        review_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/review_result.md"]
    }
    fn check_list(&self) -> bool {
        true
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let result_path = forceloop_dir.join(REVIEW_RESULT);

        // 1. Verify artifact exists and wiki links are valid.
        verify_artifact(&result_path).map_err(|_| {
            ForceLoopError::Execution(
                "Code review report not found. Re-run the review and ensure \
                 the Verdict and checklist are populated.".into(),
            )
        })?;

        // 2. Verify all checklist items are completed.
        verify_checklist(&result_path).map_err(|_| {
            ForceLoopError::Execution(
                "Code review report has uncompleted checklist items. \
                 Address all open issues and re-run the review.".into(),
            )
        })
    }
}