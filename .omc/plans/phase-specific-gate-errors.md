# Plan: Phase-Specific Gate Error Messages

## Problem

`verify_artifact()` and `verify_checklist()` in `state.rs` return generic error messages. All 5 gates produce the same vague text:
- `"Some design files have not been generated yet. Run the current skill first."`
- `"Some checklist items are not yet completed. Finish them and try again."`

When the hook injects these into the AI session, the AI can't tell which phase failed or what specific action to take.

## Proposed Messages

| Phase | Current (vague) | New (actionable) |
|-------|-----------------|------------------|
| **New** | "…not generated yet" | "Spec generation verification failed. Review the files under specs/ and regenerate if needed." |
| **Plan** | "…not generated yet" or "…broken links" | "Plan generation incomplete. Cross-review the files under specs/ and plans/ directories." |
| **Audit** | "…not completed" | "Audit report incomplete. Re-run the audit." |
| **Implement** | "…not completed" or "Gate blocked: N wave(s)" | "Implementation verification failed. Re-run the current wave's development tasks." |
| **Review** | "…not completed" | "Code review report verification failed. Re-run the review." |

## Approach

**Don't modify `verify_artifact` or `verify_checklist`.** Instead, wrap their errors in each `gate()` method with `.map_err()` to produce phase-specific messages. This keeps the shared utilities generic and only changes the surface-level error text.

For **New** and **Plan** (where `verify_artifact` checks both existence and wiki links), wrap generically. The AI gets enough context from the phase name.

For **Audit**, **Implement**, and **Review** (where `verify_checklist` follows `verify_artifact`), wrap both checks.

## Changes

| File | Line | Change |
|------|------|--------|
| `src/commands/new_cmd.rs` | L127-131 | Wrap `verify_artifact` error with phase-specific message |
| `src/commands/plan.rs` | L180-197 | Wrap `verify_artifact` error with phase-specific message |
| `src/commands/audit.rs` | L130-139 | Wrap `verify_artifact` and `verify_checklist` errors |
| `src/commands/implement.rs` | L138-164 | Wrap `verify_artifact`, `verify_checklist`, and wave-count errors |
| `src/commands/review.rs` | L125-133 | Wrap `verify_artifact` and `verify_checklist` errors |
| `tests/command_compile.rs` | L75 | Update test name `all_5_commands_have_populated_schemas` (unchanged) |

## Verification

1. `cargo test` — all pass
2. `cargo clippy --all-targets` — zero warnings
3. Manual: verify each gate produces the new message text