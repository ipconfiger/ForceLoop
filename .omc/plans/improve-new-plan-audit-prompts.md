# Plan: Improve `new`, `plan`, `audit` Skill Prompts for Higher Quality Specs & Plans

## Requirements Summary

Based on the prior analysis, the three skill prompts in the ForceLoop pipeline (`new` → `plan` → `audit`) have structural gaps that allow LLMs to produce inconsistent-quality specs and plans. The gaps fall into three categories:

1. **`new` — no spec content template**: The prompt creates markdown files but doesn't constrain *what goes in them*, leading to wildly varying spec quality.
2. **`plan` — no dependency/risk ordering**: Wave decomposition lacks guidance on dependency tracking, risk-first ordering, per-wave approach descriptions, and acceptance criteria.
3. **`audit` — too narrow + no fix loop**: Four audit dimensions miss key checks (feasibility, test adequacy, spec-plan coverage) and — critically — the prompt doesn't require the auditor to *fix the issues found*, making the checklist-only gate pass effectively a rubber stamp.

The pipeline state model (`src/state.rs`, boolean flags) and the existing gate/verify infrastructure (`verify_checklist`, `verify_artifact`) are sound and don't need changes.

## Acceptance Criteria

- [ ] `new` prompt produces specs with consistent structure (all module files follow the same section template)
- [ ] `new` prompt includes structured interview guidance covering at least 5 design dimensions
- [ ] `new` prompt includes module decomposition heuristics (cohesion, size bounds, cross-ref rules)
- [ ] `new` prompt includes a spec self-check step covering completeness, consistency, and quantization of NFRs
- [ ] `plan` prompt requires per-wave dependency declaration (`Depends on:`) and validates DAG at index level
- [ ] `plan` prompt includes risk-first wave-ordering guidance
- [ ] `plan` prompt requires an "Implementation Approach" section per wave (approach, files, decisions)
- [ ] `plan` prompt requires per-wave "Acceptance Criteria" (Given/When/Then)
- [ ] `plan` prompt includes wave size heuristics (1-2 rounds, ≤3 unrelated files per wave)
- [ ] `audit` prompt expands to at least 8 verification dimensions (current 4 + feasibility, test adequacy, spec-plan bidirectional coverage, NFR coverage, consistency with existing code)
- [ ] `audit` prompt requires remediating CRITICAL/HIGH issues by editing specs/plans before writing the final report
- [ ] `audit` prompt requires each issue entry to include a concrete fix recommendation
- [ ] `audit` prompt provides a structured report template with recommendation verdict
- [ ] All existing tests pass (`cargo test`)
- [ ] `cargo clippy --all-targets` passes with no new warnings

## Implementation Steps

### Wave 1 — `new`: Add Spec Content Template + Structured Interview + Decomposition Rules

**Files to modify:** `src/commands/new_cmd.rs`

#### Step 1.1: Define a spec template as const string

Insert a template section into `SKILL_PROMPT` and `COMMAND_PROMPT` after the "create a markdown file" instruction. The template defines the required sections for *every* spec module file:

```markdown
### Spec Module Template

Each module file MUST follow this structure:

# Module: [kebab-case-name]

**Type**: Architecture / Data Model / API / UI / Security / Workflow / ...

## Purpose
- What problem does this module solve? (1-3 sentences)
- What is explicitly NOT in scope?

## Inputs & Outputs
- **Inputs**: [data, events, user actions the module receives]
- **Outputs**: [data, files, side effects the module produces]
- **External dependencies**: [services, libraries, APIs this module depends on]

## Key Design Decisions
For each open decision, list:
- **Question**: [what needs deciding]
- **Options** (≥2):
  - Option A: [description], Pros: [bullets], Cons: [bullets]
  - Option B: [description], Pros: [bullets], Cons: [bullets]
- **Current preference**: [A / B / undecided]

## Non-Functional Requirements
- [Specific, quantified requirements. NO vague terms like "fast" or "secure".]
- e.g. "p99 latency < 200ms", "RTO < 5 min", "must support 100 concurrent users"

## Constraints & Assumptions
- [explicit constraints, e.g. "no new database dependency", "must work offline"]
- [assumptions that, if wrong, would change the design]

## Boundaries & Edge Cases
- [error scenarios, failure modes, boundary conditions]
- [what happens when inputs are invalid, missing, or out-of-range?]

## Cross-References
- [[related-module-1]] — [relationship description]
- [[related-module-2]] — [relationship description]
```

**Acceptance:** The template is embedded; any module file generated after this change will have all sections.

#### Step 1.2: Add structured interview guidance

Append to the "Interview the user" step:

```markdown
### Interview Guide

Ask ONE question at a time, in this sequence unless the user has already provided the info:

1. **Goal**: What user problem does this solve? What's the primary success metric?
2. **Scope**: What boundary? What is explicitly NOT doing?
3. **Users & frequency**: Who uses it? How often? Is it interactive or batch?
4. **Technical constraints**: Any must-use / must-not-use tech? Compatibility requirements?
5. **Risk tolerance**: What happens if this breaks? Data loss acceptable? Downtime acceptable?

For each answer, probe for specifics if the answer is vague (e.g. "fast" → "what's the latency target?").
```

**Acceptance:** Prompt includes 5 question categories with probe guidance.

#### Step 1.3: Add module decomposition heuristics

Insert before the decomposition step:

```markdown
### Decomposition Rules

- Each module covers ONE orthogonal concern (separation of concerns).
- If two modules have extensive cross-references, consider merging them.
- If a module would exceed ~300 lines of content, consider splitting.
- Each module must have a clearly scoped Inputs & Outputs section — if in/out boundaries are blurry, the module isn't well-defined.
- Aim for 3-8 modules per spec. Fewer than 3 → likely too coarse. More than 8 → likely too granular.
```

**Acceptance:** Decomposition rules are embedded and self-contained.

#### Step 1.4: Add spec self-check after file creation

Insert a new step after the file-creation step:

```
5.5. **Self-Check** — review all created spec files for:
   - Completeness: does every module have all required sections filled?
   - Consistency: no contradictory statements across modules
   - Quantification: every NFR is quantified (or explicitly marked "TBD with [owner]")
   - Risk identification: every module documents its key risks
   If any file fails a check, fix it before proceeding.
```

**Acceptance:** Self-check step is in the prompt with clear criteria.

---

### Wave 2 — `plan`: Add Dependencies, Risk Ordering, Approach Sections, Acceptance Criteria

**Files to modify:** `src/commands/plan.rs`

#### Step 2.1: Add `Depends on:` field to wave file template

Modify the "At the TOP of each wave file" instruction to include a `Depends on:` field:

```markdown
At the TOP of each wave file:

# Wave N: [Short Name]

Based on: [[architecture.md]], [[data-model.md]]
Depends on: [[wave-1-core-model]]
```

**Acceptance:** Wave file template has `Depends on:` field; validation requirement added.

#### Step 2.2: Add risk-first and dependency-aware wave ordering rule

Insert after the "analyze specs and decompose into waves" instruction:

```markdown
### Wave Ordering Principles

1. **Dependencies first**: If Wave B depends on Wave A, Wave A MUST come before Wave B.
2. **Risk first**: Waves with the highest implementation risk or uncertainty MUST come first (so risk is retired early).
3. **Core infrastructure first**: Foundational types, models, and interfaces before concrete features.
4. **Validated by index**: The dependency graph formed by all `Depends on:` entries MUST be a DAG (no cycles). The `index.md` should reflect the correct execution order.
```

**Acceptance:** Ordering rules are explicit; prompt mentions DAG validation.

#### Step 2.3: Add "Implementation Approach" section to each wave

Modify the wave file structure to include an Approach section:

```markdown
### Implementation Approach

- **Core idea**: [1-3 sentence description of the implementation strategy]
- **Files to create**: [list of new files with relative paths]
- **Files to modify**: [list of existing files with relative paths]
- **Key design decisions within this wave**: [any decisions made during planning]
- **What is NOT done in this wave**: [explicit exclusions, deferred to later waves]
```

Insert this between the "Test Requirements" section and the "Coding" steps.

**Acceptance:** Approach section appears in every generated wave file.

#### Step 2.4: Add "Acceptance Criteria" per wave

Insert a section before "Test Requirements":

```markdown
### Acceptance Criteria

For each key scenario, write a Given/When/Then criterion:

- Given [precondition], When [action], Then [observable result]
- Given [precondition], When [action], Then [observable result]

At least 2 criteria per wave. These are high-level user/API-facing checks, NOT internal test cases (those go in Test Requirements).
```

**Acceptance:** Acceptance Criteria section exists with Given/When/Then format.

#### Step 2.5: Add wave size heuristics

Append to the decomposition instruction:

```markdown
### Wave Size Rules

- Each wave should be completable in 1-2 coding rounds.
- If a wave would require modifying 4+ unrelated files, consider splitting.
- If a wave is trivial (< ~20 lines of code across 1 file), consider merging with an adjacent wave.
- Each wave should be independently testable (you can run tests for just this wave without the others being complete).
```

**Acceptance:** Size heuristics are in the prompt.

---

### Wave 3 — `audit`: Expand Dimensions, Add Fix Loop, Structured Report + Recommendations

**Files to modify:** `src/commands/audit.rs`

#### Step 3.1: Expand cross-verify dimensions

Replace the 4-dimension list with an expanded list:

```markdown
3. Cross-verify across ALL of the following dimensions:

   **a. Design conflicts** — Do any spec modules contradict each other?
   **b. Plan-spec misinterpretation** — Does any plan wave misread the spec's intent?
   **c. Missing coverage** — Is every spec aspect addressed by at least one plan wave?
   **d. Contradictory requirements** — Are there requirements across modules that cannot both be true?
   **e. Feasibility** — Can each plan wave be completed given the project's constraints (tech stack, dependencies, existing code patterns)?
   **f. Dependency coherence** — Are all `Depends on:` declarations correct? Is the wave DAG acyclic?
   **g. Test adequacy** — Do the test requirements in each wave cover the boundaries and edge cases from the spec?
   **h. NFR coverage** — Are all quantified NFRs from the spec addressed in the plan (either as a test requirement or an explicit implementation concern)?
```

**Acceptance:** 8 dimensions (a-h), replacing the previous 4.

#### Step 3.2: Add fix loop before final report

Insert a new step between cross-verify and report writing:

```
4. **Remediation**: For every CRITICAL or HIGH issue found:
   a. Determine which file(s) need fixing (a spec file under `.forceloop/specs/` or a plan file under `.forceloop/plans/`).
   b. Edit the file(s) to fix the issue. Document the fix in the file (e.g. "Audit fix: [description]").
   c. Re-read the affected files to confirm the fix resolves the issue.
   d. If a CRITICAL or HIGH issue cannot be fixed without breaking the goal, escalate in the report.
   Only proceed after all CRITICAL and HIGH issues are resolved or explicitly escalated.
```

**Acceptance:** Remediation step exists; prompt requires actual file edits before report writing.

#### Step 3.3: Add fix recommendation to each issue entry

Modify the severity-d issue format:

```
4. Write the audit report to `.forceloop/audit.md` with:

   - **Summary of findings** (high-level overview)

   - **Issues** — each issue MUST follow this format:
     - **Severity**: CRITICAL | HIGH | MEDIUM | LOW
     - **Location**: [file:line or file reference]
     - **Description**: [what the issue is]
     - **Recommended fix**: [concrete action to resolve — "modify plan wave 2 to add X" not "improve coverage"]
     - **Status**: [fixed / escalated / no_action]

   - **Quality Scores** (per spec module and per plan wave):
     - Completeness: score / 10
     - Feasibility: score / 10
     - Alignment: score / 10

   - **Recommendation** (one of):
     - **Approved** — no blocking issues; ready for implement
     - **Conditional** — all CRITICAL/HIGH fixed; MEDIUM items tracked for follow-up
     - **Blocked** — CRITICAL/HIGH issues remain; must fix before proceeding

   - A **checklist** at the end with ALL audit items completed (`- [x]` or `- [✅]`).
     Every item MUST be marked completed. The gate will reject if any `- [ ]` remains.
```

**Acceptance:** Issue format includes recommended-fix and status fields; report includes Quality Scores and Recommendation verdict.

#### Step 3.4: Update gate message to reflect new expectation

Minor update to `gate()` error message in `audit.rs` to reference the new remediation step.

**Acceptance:** Error message mentions "remediation" or "fix the reported issues" rather than just "re-run".

---

### Wave 4 — Integration & Test

**Files to modify:** None new; `cargo test`, `cargo clippy --all-targets`

#### Step 4.1: Run full test suite

```bash
cargo test
cargo clippy --all-targets
```

Verify no regressions. The skill prompts are compile-time embedded `const &str` values — as long as Rust syntax is valid, no functional tests should break. Update integration tests if they assert specific content in the prompts (check `tests/`).

**Acceptance:** All tests pass; clippy clean.

#### Step 4.2: Verify changed constants

If any constants like `AUDIT_FILE` or `SPECS_INDEX` were referenced by name in the new prompt text, verify they remain consistent with their definitions in `src/constants.rs`.

**Acceptance:** Constants in prompt text match `src/constants.rs` definitions.

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Prompt too long makes compiled binary larger | Low | Low | Prompts are ~2-3KB each; even doubling is ~6KB — negligible |
| Prompt verbosity reduces LLM adherence | Medium | Medium | Template sections are structured and bounded; test after Wave 1 to verify LLM follows the template |
| Audit fix loop makes the skill take longer | Medium | Medium | The tradeoff is intentional — better to spend 2x time on audit than ship bad plan to implement |
| Existing integration tests assert prompt content | Low | Medium | Search for `SKILL_PROMPT` or `COMMAND_PROMPT` references in `tests/` before making changes |

## Verification Steps

1. After each wave, run `cargo test && cargo clippy --all-targets` — no regressions.
2. After Wave 1: manually check that a generated spec file includes all required sections (acceptance criteria 1-4).
3. After Wave 2: manually check that a generated plan has `Depends on:`, `Implementation Approach`, and `Acceptance Criteria` sections.
4. After Wave 3: manually trigger the audit LLM chain and verify the report has expanded dimensions, remediation section, and structured template.