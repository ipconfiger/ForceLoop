# Plan: Add Claude Code Stop Hook to `fl setup`

## Requirements Summary

`fl setup --tool claude` currently writes command files (`.claude/commands/fl-*.md`) but does NOT configure a hook. Claude Code needs a `Stop` hook in `.claude/settings.json` to automatically run `fl gate` after each response — matching what OpenCode and omp already have.

Claude Code's `Stop` hook only feeds stderr to the AI on **exit code 2** (blocking error). `fl gate` currently exits with 1 (non-blocking). A wrapper script is needed.

## Acceptance Criteria

- [ ] `fl setup --tool claude` writes `.claude/settings.json` with `Stop` hook configured
- [ ] `fl setup --tool claude` writes `.claude/hooks/fl-gate.sh` wrapper script
- [ ] The wrapper script runs `fl gate`, exits with code 2 on failure
- [ ] Non-zero `fl gate` stderr is automatically fed back to Claude as error message
- [ ] Existing `.claude/settings.json` content (other hooks/settings) is preserved via merge
- [ ] All tests pass, clippy clean

## Design

### Hook script: `.claude/hooks/fl-gate.sh`

```bash
#!/bin/bash
set -euo pipefail
fl gate || exit 2
```

- `fl gate` succeeds → exit 0 → hook silently passes
- `fl gate` fails (exit 1) → wrapper `exit 2` → Claude Code treats as blocking → feeds stderr to Claude → AI auto-fix loop triggers
- Script stored at `.claude/hooks/fl-gate.sh`, embedded at compile time via `include_str!`

### settings.json structure

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/fl-gate.sh",
            "timeout": 60,
            "statusMessage": "Running gate check..."
          }
        ]
      }
    ]
  }
}
```

**Why `timeout: 60`**: 60 seconds matches the 60s default in the OpenCode plugin's Bun shell.

**Why `${CLAUDE_PROJECT_DIR}`**: Resolves to the project root automatically, making the hook portable.

### Merge strategy for existing `.claude/settings.json`

```json
// Before
{ "disableAllHooks": false, "otherSetting": "value" }

// After
{
  "disableAllHooks": false,
  "otherSetting": "value",
  "hooks": {
    "Stop": [ ... ]
  }
}
```

If `hooks.Stop` already exists → no change (idempotent).
If `hooks` exists without `Stop` → add `Stop` entry.
If neither exists → add both.

### Source file: `plugin/claude-hook.sh`

Plain shell script embedded via `include_str!`, same pattern as `plugin/fl.ts` and `plugin/omp-fl-gate.ts`.

## Implementation Steps

### Step 1: Create `plugin/claude-hook.sh`

```bash
#!/bin/bash
set -euo pipefail
fl gate || exit 2
```

### Step 2: Add hook writing in `setup.rs`

**New functions:**
- `claude_hook_sh_content()` — `include_str!("../plugin/claude-hook.sh")`
- `write_claude_hook(root, written)` — writes `.claude/hooks/fl-gate.sh` + merges `.claude/settings.json`

**`run()` update:** After the compile loop for `Target::Claude`:
```rust
if target == Target::Claude {
    write_claude_hook(root, &mut written)?;
}
```

**Merge logic for `.claude/settings.json`:**
```rust
fn merge_claude_hook(settings_path: &Path) -> Result<()> {
    // Read existing settings or start empty
    // Add/merge hooks.Stop entry
    // Write back (pretty-printed)
}
```

### Step 3: Update tests

**Unit tests** in `setup.rs`:
- `claude_hook_sh_uses_fl_gate` — script calls `fl gate`
- `claude_hook_sh_exits_with_2` — script exits with 2 on failure
- `claude_hook_merge_preserves_existing` — merge doesn't overwrite other settings
- `claude_hook_merge_adds_stop` — merge adds Stop entry when absent

**Integration tests** in `tests/setup_tool.rs`:
- Update `run_default_writes_all_targets` count (was 17, now +1 script + merge → 18 total... actually `.claude/settings.json` is a merge, not a new file written to `written` list... hmm)

Wait, looking at the current `SetupReport` — it tracks files written via `written: Vec<PathBuf>`. The wrapper script `.claude/hooks/fl-gate.sh` is a new file that should be in this list. But `.claude/settings.json` is a merge — it's already tracked in `written` if we write it, or we don't need to track it if we just modify in place.

Actually, the `.claude/settings.json` is important — it's the actual hook configuration. Both the script AND the settings.json should be in `written`. But settings.json might already exist (from prior `fl setup` or user config), so we need to handle that.

Let me simplify: both files go into `written`:
- `.claude/hooks/fl-gate.sh` — always new
- `.claude/settings.json` — merged, written back

Update test counts:

| Test | Before | After |
|------|--------|-------|
| `run_default_writes_all_targets` | 17 (5×3 + 2 hooks) | 19 (5×3 + 2 hooks + 2 claude hook files) |
| `claude_only_writes_claude_dir` | 5 | 7 (5 commands + 1 script + 1 settings) |
| `opencode_only_writes_opencode_dir` | 6 | 6 (unchanged) |
| `omp_only_writes_omp_dir` | 6 | 6 (unchanged) |

### Step 4: No other source changes needed

- `plugin/fl.ts` — unchanged
- `plugin/omp-fl-gate.ts` — unchanged
- `compiler.rs` — unchanged
- `cli.rs` — unchanged
- `lib.rs` — unchanged
- `main.rs` — unchanged

## File Change Summary

| File | Change |
|------|--------|
| `plugin/claude-hook.sh` | **New** — shell wrapper script |
| `src/setup.rs` | Add `write_claude_hook()`, `claude_hook_sh_content()`, merge logic, tests |
| `tests/setup_tool.rs` | Update counts, add claude hook integration tests |

## Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `.claude/settings.json` overwrites user settings | Low | Merge approach — only adds/updates `hooks.Stop`, preserves all other keys |
| Script not executable | Medium | `fl setup` sets `chmod +x` on the hook script |
| `fl gate` not on PATH when hook runs | Medium | Use absolute path or verify `fl` is on PATH in the script |
| Hook runs too slow (blocks Claude) | Low | 60s timeout, `fl gate` is typically sub-second |
| Settings.json already has `hooks.Stop` | Low | Idempotent — don't overwrite existing Stop hooks