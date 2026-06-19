# Plan: Remove TryFinish (redundant with Review)

## Requirements Summary

`TryFinish` is the 6th pipeline step (after Review) that verifies whether the development goal is achieved. Its logic overlaps completely with `Review` — Review already cross-verifies code against specs, runs test suite, and produces a verified `review_result.md`. TryFinish only checks that `result.json` exists, which adds no value beyond what Review's `review_result.md` checklist already captures.

Remove `TryFinish` from the pipeline entirely, shortening the sequence from:

```
New → Plan → Audit → Implement → Review → TryFinish → Done
```

to:

```
New → Plan → Audit → Implement → Review → Done
```

## Acceptance Criteria

- [ ] Pipeline advances from Review directly to Done (no TryFinish step)
- [ ] All existing tests pass after removal
- [ ] No dead imports or orphaned references remain
- [ ] `PipelineState` has no `try_finish` field (legacy migration preserved for backward compat)
- [ ] Setup no longer writes `fl-try-finish.md`
- [ ] OpenCode plugin / gate hook unaffected (it only calls `fl gate`, not TryFinish directly)

## Touchpoints (classified)

### NEEDS_REMOVAL (delete the file or struct)

| File | What |
|------|------|
| `src/commands/try_finish.rs` | Delete the entire file |
| `src/commands/mod.rs` | Remove `pub mod try_finish;` and `pub use try_finish::TryFinish;` |

### NEEDS_UPDATE (remove TryFinish from the logic)

| File | Lines | Change |
|------|-------|--------|
| `src/state.rs` | L13 (comment) | `→ review → try_finish → done` → `→ review → done` |
| `src/state.rs` | L28 | Remove `pub try_finish: bool` field |
| `src/state.rs` | L97 | Remove `state.try_finish = true;` from `"done"` legacy branch |
| `src/state.rs` | L100-107 | Remove entire `"try-finish" => { ... }` legacy migration branch |
| `src/state.rs` | L166-167 | Remove `else if !self.try_finish { Some("try_finish") }` from `next_pending()` |
| `src/gate.rs` | L1 | Remove `TryFinish` from imports |
| `src/gate.rs` | L32-35 | Remove `else if !state.try_finish { TryFinish.gate(ctx)?; state.try_finish = true; }` |
| `src/setup.rs` | L6 | Remove `TryFinish` from imports |
| `src/setup.rs` | L84 | Remove `("fl-try-finish", \|\| TryFinish.command_template()),` from `COMMANDS` table |
| `src/traits.rs` | L19 | Update comment: `6` → `5` |

### TEST_UPDATES

| File | Lines | Change |
|------|-------|--------|
| `tests/command_compile.rs` | L1 | Remove `TryFinish` from imports |
| `tests/command_compile.rs` | L92-93 | Remove `TryFinish` assertions |
| `tests/command_compile.rs` | L139 | Remove `TryFinish.skill_template()` from iteration |
| `tests/setup_tool.rs` | L170 | Remove `"fl-try-finish.md"` from expected set |

### DOCUMENTATION_UPDATES

| File | Lines | Change |
|------|-------|--------|
| `src/state.rs` | L342, L364, L366 | Remove `try_finish` assertions from unit tests |
| All `.md` docs listing 6 commands | | Update to 5, remove TryFinish references |

### PIPELINE CHANGE (gate.rs logic)

After Review gate passes, current flow:
```rust
} else if !state.review {
    Review.gate(ctx)?;
    state.review = true;
} else if !state.try_finish {
    TryFinish.gate(ctx)?;       // ← REMOVE
    state.try_finish = true;    // ← REMOVE
} else {
    state.done = true;
}
```

New flow:
```rust
} else if !state.review {
    Review.gate(ctx)?;
    state.review = true;
} else {
    state.done = true;
}
```

### LEGACY MIGRATION PRESERVATION

The `"try-finish"` legacy migration branch in `PipelineState::read_or_default()` must be REMOVED (it maps to a field that no longer exists). The `"done"` legacy branch must also stop setting `state.try_finish = true`.

The `PipelineState` struct loses its `try_finish` field entirely. This is safe because:
- `serde(default)` on all existing fields handles missing keys during deserialization
- Removing a field means new code reads old state files ignoring `try_finish`
- Old code reading new state files won't see the field but `serde(default)` handles it

**No backward compat issue** — the struct uses `#[serde(default)]` on every field, so JSON files with or without the `try_finish` key are both valid.

## Implementation Steps (ordered)

1. **Edit `src/state.rs`** — remove `try_finish` field, legacy branches, `next_pending` entry, and test assertions
2. **Edit `src/gate.rs`** — remove TryFinish from imports and gate logic
3. **Edit `src/setup.rs`** — remove TryFinish from imports and COMMANDS table
4. **Edit `src/traits.rs`** — update comment (6→5)
5. **Delete `src/commands/try_finish.rs`** — remove the file
6. **Edit `src/commands/mod.rs`** — remove `pub mod` and `pub use`
7. **Edit `tests/command_compile.rs`** — remove TryFinish references
8. **Edit `tests/setup_tool.rs`** — remove `fl-try-finish.md` from expected set
9. **Run `cargo test && cargo clippy --all-targets`** — verify clean
10. **Update docs** — `CLAUDE.md`, `AGENTS.md` (update 6→5 counters and pipeline diagram)

## Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Old state.json files (existing projects) break | Low | `#[serde(default)]` handles missing/extra fields; old `try_finish: true` is silently ignored |
| Legacy migration path breaks | Low | The `"try-finish"` branch is removed; any project still using the legacy `current_phase` format that was at "try-finish" will migrate to `review: true` (all prior phases true, implement: true, review: true, done: false) — close enough |
| Tests pin the 6-command contract | High | `run_writes_all_six_commands_per_target` test will fail; intentional, update expected set to 5 |
| OpenCode plugin references | None | Plugin only calls `fl gate`, never references TryFinish directly |

## Verification Steps

1. `cargo test` — all 91 tests pass (after updating expected counts)
2. `cargo clippy --all-targets` — zero warnings
3. `cargo check` — zero errors, no dead imports
4. Grep for `TryFinish`, `try_finish`, `try-finish`, `fl-try-finish` — no hits outside documented docs directory
5. Manual test: create test project, run `fl setup`, verify `fl-try-finish.md` is NOT written