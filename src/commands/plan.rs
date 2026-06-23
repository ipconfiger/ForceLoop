use std::fs;

use std::path::Path;

use crate::constants::{PLANS_DIR, PLANS_INDEX, WAVE_STATE};
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
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

   **Wave Ordering Principles (MUST follow):**
   - **Dependencies first**: If Wave B depends on Wave A, Wave A MUST come
     before Wave B.
   - **Risk first**: Waves with the highest implementation risk or uncertainty
     MUST come first (retire risk early). If uncertain, put it early.
   - **Core infrastructure first**: Foundational types, models, and interfaces
     before concrete features.
   - **Validate DAG**: The dependency graph formed by all `Depends on:` entries
     MUST be acyclic. The `index.md` must reflect the correct execution order.

   **Wave Size Heuristics:**
   - Each wave should be completable in 1-2 coding rounds.
   - If a wave requires modifying 4+ unrelated files, consider splitting.
   - If a wave is trivial (~<20 lines across 1 file), merge with adjacent wave.
   - Each wave should be independently testable.

3. For each wave, create an independent markdown file under
   `.forceloop/plans/`. Use kebab-case with `.md` extension
   (e.g. `wave-1-core-model.md`, `wave-2-api.md`).

4. At the TOP of each wave file:
   - Add a wiki link to the spec file(s) it implements:
     `Based on: [[architecture.md]]`
   - Add a dependency wiki link to prerequisite wave(s):
     `Depends on: [[wave-1-core-model]]`
     (Omit if this wave has no dependencies.)

5. Each wave file MUST follow this structure:

   ### Acceptance Criteria
   For each key scenario, write a Given/When/Then criterion:
   - Given [precondition], When [action], Then [observable result]
   - At least 2 criteria per wave. These are high-level user/API-facing
     checks, NOT internal test cases.

   ### Test Requirements
   - Left/right boundary tests
   - Success case tests
   - Error/failure case tests

   ### Implementation Approach
   - **Core idea**: [1-3 sentences on the implementation strategy]
   - **Files to create**: [list new files with relative paths]
   - **Files to modify**: [list existing files with relative paths]
   - **Key decisions**: [design decisions made during planning]
   - **Not in this wave**: [explicit exclusions deferred to later waves]

   ### Coding
   [steps to implement]

   ### Run Tests
   `cargo check && cargo test && cargo clippy --all-targets`

   ### Fix and Regression
   [on test failure]

   ### Cross-Fact Verification
   Verify generated code against the spec — no stub implementations,
   no mock code, no placeholder logic. Check:
   - Does the code match the spec's Inputs & Outputs?
   - Are all spec boundary cases handled?
   - Any hidden assumptions not in the spec?

6. Each wave file MUST end with a **Checklist**:
   - [ ] Tests written with boundary + success + error cases
   - [ ] Implementation complete
   - [ ] All tests pass
   - [ ] Cross-fact verification passed

7. Create the index file `.forceloop/plans/index.md` that:
   - Lists all waves in order (respecting dependency ordering)
   - Links to each wave using wiki link syntax: `[[wave-file]]`
   - Briefly describes what each wave covers
   - Verifies no dependency cycles exist

## Verification
- `.forceloop/plans/index.md` exists.
- Every wiki link `[[...]]` in index.md resolves to a wave file.
- Each wave file has a checklist at the end.
- Each wave file references its spec file(s) at the top.
- Each wave file (except wave 1) declares `Depends on:` or justifies absence.
- No stub or mock implementations remain after each wave.
- Wave dependency graph is a DAG (no cycles).

## Pipeline Completion
**STOP**. Do NOT attempt to continue with the next phase
(auditing). The hook will automatically run `fl gate`
to advance the pipeline. The next phase is ready when you run `/fl-audit`.
";

const COMMAND_PROMPT: &str = "\
Create a multi-wave development plan from the design specs.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl plan`.
   This creates the `.forceloop/plans/` directory scaffold.
1. Read all spec files from `.forceloop/specs/`.
   Start with `index.md` to discover all modules, then read each module file.
2. Analyze the specs and decompose the work into waves. Follow:
   - **Ordering**: dependencies first, risk first, core infrastructure first.
   - **Size**: 1-2 coding rounds per wave, ≤3 unrelated files.
3. For each wave, create an independent markdown file under
   `.forceloop/plans/`. Use kebab-case with `.md` extension.
4. At the TOP of each wave file: `Based on: [[spec-file]]` and
   `Depends on: [[dependency-wave]]` (omit if no dependencies).
5. Each wave file MUST follow this structure:
   - **Acceptance Criteria** (Given/When/Then, ≥2 per wave)
   - **Test Requirements** (boundary / success / error cases)
   - **Implementation Approach** (core idea, files, decisions, exclusions)
   - **Coding** steps
   - **Run Tests**: `cargo check && cargo test && cargo clippy --all-targets`
   - **Fix and Regression**
   - **Cross-Fact Verification** (code vs spec match check)
6. Each wave file MUST end with a **Checklist**.
7. Create the index file `.forceloop/plans/index.md` (wiki links, DAG check).

## Verification
- `.forceloop/plans/index.md` exists.
- Every wiki link resolves; each wave has spec ref + optional dep ref.
- Wave dependency graph is a DAG.

## Pipeline Completion
**STOP**. Do NOT continue to the next phase.
The hook will advance the pipeline automatically.
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
        verify_artifact(&index_path).map_err(|_| {
            ForceLoopError::Execution(
                "Plan generation incomplete. Cross-review the files under specs/ \
                 and plans/ directories."
                    .into(),
            )
        })?;

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