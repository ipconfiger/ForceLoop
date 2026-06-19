use crate::constants::REVIEW_RESULT;
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, verify_checklist, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Review Skill

Cross-verify the implemented code against specs and plans.
Run full test suite and call metis for high-precision review.

## Steps
0. Run the shell command `fl review`.
   This checks that implement is complete before proceeding.
1. Read spec files from `.forceloop/specs/` (start with `index.md`).
2. Read plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Read the wave state from `.forceloop/wave_state.md`.
4. Cross-verify the implemented code against specs and plans:
   - Does the code match the design spec?
   - Are all plan acceptance criteria met?
   - Are there any deviations or omissions?
5. Run the full test suite: `cargo test`.
6. Run lints: `cargo clippy --all-targets`.
7. Call `metis` for high-precision code review.
8. Ensure ALL tests pass and no lint errors remain.
9. Write the review result to `.forceloop/review_result.md` with:
   - Summary of findings
   - Test results (all green?)
   - Metis review verdict
   - A **checklist** at the end with all review items.
     Every item MUST be marked `- [x]` or `- [✅]` (completed).

## Verification
- `.forceloop/review_result.md` exists.
- All checklist items are `- [x]` or `- [✅]`.
- All tests pass (`cargo test` returns 0).
- Clippy passes (`cargo clippy --all-targets` returns 0).
- Metis review completed with no CRITICAL or HIGH issues.
";

const COMMAND_PROMPT: &str = "\
Review the implementation against specs and plans.

Arguments: $ARGUMENTS

## Steps
0. Run `fl review` first.
1. Read specs, plans, and wave state.
2. Cross-verify code against docs — match spec, meet criteria.
3. Run `cargo test` and `cargo clippy --all-targets`.
4. Call `metis` for high-precision code review.
5. Write `.forceloop/review_result.md` with findings + checklist (all `- [x]`).

## Verification
- `cargo test` passes. `cargo clippy` passes.
- `.forceloop/review_result.md` has all checklist items completed.
";

fn review_skill() -> CommandSchema {
    CommandSchema {
        name: "fl-review",
        description: "Regression-validate the implementation",
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
        verify_artifact(&result_path)?;

        // 2. Verify all checklist items are completed.
        verify_checklist(&result_path)
    }
}