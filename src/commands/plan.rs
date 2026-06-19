use std::fs;

use std::path::Path;

use crate::constants::{PLANS_DIR, PLANS_INDEX, WAVE_STATE};
use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Plan Skill

Decompose the design spec into multiple development plan waves.

Spec source: `.forceloop/specs/`

## Steps
0. Run the shell command `fl plan`.
   This creates the `.forceloop/plans/` directory scaffold.
1. Read all spec files from `.forceloop/specs/`.
   Start with `index.md` to discover all modules, then read each module file.
2. Analyze the specs and decompose the work into waves. Each wave should
   be a coherent, independently implementable chunk of work.
3. For each wave, create an independent markdown file under
   `.forceloop/plans/`. Use kebab-case with `.md` extension
   (e.g. `wave-1-core-model.md`, `wave-2-api.md`).
4. At the TOP of each wave file, add a wiki link to the spec file(s)
   it implements, e.g. `Based on: [[architecture.md]]`.
5. Each wave file MUST follow TDD Red-Green structure:
   - **Test Requirements**:
     - Left/right boundary tests
     - Success case tests
     - Error/failure case tests
   - **Coding** steps
   - **Run tests** task (after coding)
   - **Fix and regression** task (on test failure)
   - **Cross-fact verification** task:
     Verify generated code against the spec — no stub implementations,
     no mock code, no placeholder logic.
6. Each wave file MUST end with a **Checklist**:
   - [ ] Tests written with boundary + success + error cases
   - [ ] Implementation complete
   - [ ] All tests pass
   - [ ] Cross-fact verification passed
7. Create the index file `.forceloop/plans/index.md` that:
   - Lists all waves in order
   - Links to each wave using wiki link syntax: `[[wave-file]]`
   - Briefly describes what each wave covers

## Verification
- `.forceloop/plans/index.md` exists.
- Every wiki link `[[...]]` in index.md resolves to a wave file.
- Each wave file has a checklist at the end.
- Each wave file references its spec file(s) at the top.
- No stub or mock implementations remain after each wave.
";

const COMMAND_PROMPT: &str = "\
Create a multi-wave development plan from the design specs.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl plan`.
   This creates the `.forceloop/plans/` directory scaffold.
1. Read all spec files from `.forceloop/specs/`.
   Start with `index.md` to discover all modules, then read each module file.
2. Analyze the specs and decompose the work into waves. Each wave should
   be a coherent, independently implementable chunk of work.
3. For each wave, create an independent markdown file under
   `.forceloop/plans/`. Use kebab-case with `.md` extension
   (e.g. `wave-1-core-model.md`, `wave-2-api.md`).
4. At the TOP of each wave file, add a wiki link to the spec file(s)
   it implements, e.g. `Based on: [[architecture.md]]`.
5. Each wave file MUST follow TDD Red-Green structure:
   - **Test Requirements**:
     - Left/right boundary tests
     - Success case tests
     - Error/failure case tests
   - **Coding** steps
   - **Run tests** task (after coding)
   - **Fix and regression** task (on test failure)
   - **Cross-fact verification** task
6. Each wave file MUST end with a **Checklist**.
7. Create the index file `.forceloop/plans/index.md` that:
   - Lists all waves in order
   - Links to each wave using wiki link syntax: `[[wave-file]]`
   - Briefly describes what each wave covers

## Verification
- `.forceloop/plans/index.md` exists.
- Every wiki link `[[...]]` in index.md resolves to a wave file.
- Each wave file has a checklist at the end.
- Each wave file references its spec file(s) at the top.
";

/// Read all wave files in `plans_dir` (excluding `index.md`), extract
/// their checklist items, and write a combined `wave_state.md` with all
/// items initially unchecked (`- [ ]`).
fn generate_wave_state(plans_dir: &Path, wave_path: &Path) -> Result<()> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# Wave State".into());
    lines.push(String::new());

    let Ok(entries) = fs::read_dir(plans_dir) else {
        return Ok(());
    };
    let mut files: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let path = e.path();
            path.extension().and_then(|e| e.to_str()) == Some("md")
                && path.file_stem().and_then(|s| s.to_str()) != Some("index")
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    for entry in &files {
        let stem = entry.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        lines.push(format!("- [ ] {stem}"));
    }

    let body = lines.join("\n");
    fs::write(wave_path, body)?;
    Ok(())
}

fn plan_skill() -> CommandSchema {
    CommandSchema {
        name: "fl-plan",
        description: "Create development plan (multiple waves)",
        model: None,
        argument_hint: Some("[spec reference]"),
        tools: &["Read", "Write"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn plan_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..plan_skill()
    }
}

pub struct Plan;

impl Executable for Plan {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let plans_dir = forceloop_dir.join(PLANS_DIR);
        fs::create_dir_all(&plans_dir)?;

        Ok(())
    }
}

impl Subcommand for Plan {
    fn name(&self) -> &'static str {
        "plan"
    }
    fn description(&self) -> &'static str {
        "Create development plan (multiple waves)"
    }
}

impl CommandMetadata for Plan {
    fn skill_template(&self) -> CommandSchema {
        plan_skill()
    }
    fn command_template(&self) -> CommandSchema {
        plan_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/plans/index.md"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let index_path = forceloop_dir.join(PLANS_INDEX);

        // 1. Verify artifact (plans/index.md) exists with valid wiki links.
        verify_artifact(&index_path)?;

        // 2. Auto-generate wave_state.md from all wave files.
        //    Each wave file's checklist items become entries in wave_state.md,
        //    all initially unchecked. This ensures implement's gate can
        //    validate checklist completion without relying on LLM prompt
        //    to create this file.
        let plans_dir = forceloop_dir.join(PLANS_DIR);
        let wave_path = forceloop_dir.join(WAVE_STATE);
        generate_wave_state(&plans_dir, &wave_path)?;

        Ok(())
    }
}