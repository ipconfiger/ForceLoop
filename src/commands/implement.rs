use crate::constants::WAVE_STATE;
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::{
    count_completed_items, count_wave_files, verify_artifact, verify_checklist, PipelineState,
};
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Implement Skill

Develop the current wave following TDD (test first, code second).

Tracks progress via `.forceloop/wave_state.md`.

## Steps
0. Run the shell command `fl implement`.
   This checks prerequisites before proceeding.
1. Read `.forceloop/wave_state.md`.
   - If the file does NOT exist, generate it:
     Read `.forceloop/plans/index.md` to find all wave files.
     For each wave file, extract its checklist items and aggregate them
     into `.forceloop/wave_state.md` with all items initially `- [ ]`.
2. Find the FIRST unchecked item (`- [ ]`) in wave_state.md
   from top to bottom. If none found, all waves are done.
3. Identify which wave file in `.forceloop/plans/` this item belongs to.
4. Load ONLY that one wave file. Add its work items to the todo list.
   Do NOT load other wave files — focus on one wave at a time.
5. Execute the development task per the wave plan:
   - For each acceptance criterion: Red (test) → Green (code) → Refactor.
   - Run `cargo check && cargo test && cargo clippy --all-targets`.
6. Mark the item as completed in BOTH:
   - The wave file's own checklist in `.forceloop/plans/`
   - `.forceloop/wave_state.md` (change `- [ ]` to `- [x]`)
7. The OpenCode hook calls `fl gate` automatically on idle.
   If it fails, run `/fl-implement` again to load the next wave.

## Constraints
- TDD mandatory: tests before code.
- No commented-out tests, no `unwrap()` in production code.
- All checks must pass before marking an item complete.
- Do not mark items complete if tests fail.
";

const COMMAND_PROMPT: &str = "\
Implement development waves following TDD.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl implement`.
1. Read `.forceloop/wave_state.md` (generate from plans if missing).
2. Pick the FIRST unchecked item, find its wave file in `.forceloop/plans/`.
3. Load ONLY that one wave. Do not load other waves.
4. Execute TDD per the wave plan: Red → Green → Refactor.
5. Check all builds pass.
6. Update checklists in both the wave file and wave_state.md.
7. The hook calls `fl gate` automatically. If it fails, run `/fl-implement` again.

## Constraints
- TDD mandatory. No `unwrap()` in production code.
- All checks must pass before marking complete.
";

fn implement_skill() -> CommandSchema {
    CommandSchema {
        name: "fl-implement",
        description: "Develop the current phase (TDD)",
        model: None,
        argument_hint: Some("[phase-id]"),
        tools: &["Read", "Write", "Edit", "Bash", "Grep", "Glob"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn implement_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..implement_skill()
    }
}

pub struct Implement;

impl Executable for Implement {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        // 1. Read current pipeline state
        let state_path = PipelineState::locate_state_file()?;
        let state = PipelineState::read_or_default(&state_path)?;

        // 2. Check prerequisites: new, plan, and audit must be done.
        if !state.new {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: specs not ready. Run `/fl-new` first.".into(),
            ));
        }
        if !state.plan {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: plans not ready. Run `/fl-plan` first.".into(),
            ));
        }
        if !state.audit {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: audit not done. Run `/fl-audit` first.".into(),
            ));
        }

        // 3. Prerequisites met — pass to LLM via prompt.

        Ok(())
    }
}

impl Subcommand for Implement {
    fn name(&self) -> &'static str {
        "implement"
    }
    fn description(&self) -> &'static str {
        "Develop the current phase (TDD)"
    }
}

impl CommandMetadata for Implement {
    fn skill_template(&self) -> CommandSchema {
        implement_skill()
    }
    fn command_template(&self) -> CommandSchema {
        implement_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/wave_state.md"]
    }
    fn check_list(&self) -> bool {
        true
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let wave_path = forceloop_dir.join(WAVE_STATE);

        // 1. Verify artifact exists and wiki links are valid.
        verify_artifact(&wave_path).map_err(|_| {
            ForceLoopError::Execution(
                "Implementation verification failed. Review wave_state.md and \
                 re-run the current wave's development tasks."
                    .into(),
            )
        })?;

        // 2. Verify all checklist items in wave_state.md are completed.
        verify_checklist(&wave_path).map_err(|_| {
            ForceLoopError::Execution(
                "Implementation verification failed. Re-run the current wave's \
                 development tasks."
                    .into(),
            )
        })?;

        // 3. Cross-verify against actual plan files: count the number of
        //    wave FILES in .forceloop/plans/ and compare with completed
        //    waves in wave_state.md. wave_state.md tracks one entry per
        //    wave file, so both sides must count at the wave level.
        let plans_dir = forceloop_dir.join("plans");
        if plans_dir.is_dir() {
            let total_wave_files = count_wave_files(&plans_dir);
            let completed_wave_items = count_completed_items(&wave_path);
            if completed_wave_items < total_wave_files {
                let remaining = total_wave_files - completed_wave_items;
                return Err(ForceLoopError::Execution(format!(
                    "Implementation verification failed: {remaining} wave(s) \
                     not yet completed. Re-run the current wave's development \
                     tasks."
                )));
            }
        }

        Ok(())
    }
}