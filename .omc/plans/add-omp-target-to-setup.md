# Plan: Add oh-my-pi (omp) Target to `fl setup`

## Requirements Summary

Add `oh-my-pi` as a third platform target in `fl setup`, alongside Claude Code and OpenCode. Based on `docs/omp-hook-porting.html`:

1. **Command files**: Reuse OpenCode's compiled format → `.omp/commands/fl-*.md`
2. **Hook file**: Write `.omp/hooks/pre/fl-gate.ts` — listens to `session_stop`, runs `fl gate`, returns `{ continue: true, additionalContext }` on failure
3. **CLI flag**: `--tool omp` (alongside existing `--tool claude --tool opencode`)
4. **Default**: Add omp to `DEFAULT_TARGETS` so `fl setup` installs to all three by default

## Acceptance Criteria

- [ ] `fl setup` writes 5 command files to `.omp/commands/fl-*.md` (same format as OpenCode)
- [ ] `fl setup` writes `.omp/hooks/pre/fl-gate.ts` with omp's `session_stop` hook
- [ ] `fl setup --tool omp` installs ONLY to omp (no Claude, no OpenCode)
- [ ] `fl setup --tool claude --tool opencode` installs ONLY to Claude + OpenCode (no omp)
- [ ] `fl setup` (default, no flags) installs to Claude + OpenCode + omp
- [ ] All existing tests updated/added, all pass
- [ ] `cargo clippy --all-targets` — zero warnings

## Design Decisions

### Target enum & compile format

| Aspect | Decision |
|--------|----------|
| `Target::OhMyPi` | New variant in `compiler.rs` |
| Compile format | Reuses `compile_to_opencode()` — omp uses the same YAML frontmatter |
| CLI tool name | `Tool::OhMyPi`, clap value name `omp` |

### Hook file: `plugin/omp-fl-gate.ts`

Following the same `include_str!` pattern as OpenCode's `plugin/fl.ts`:

```
plugin/
├── fl.ts              # OpenCode hook (session.idle)
└── omp-fl-gate.ts     # omp hook (session_stop) — NEW
```

The omp hook listens to `session_stop`, runs `fl gate`, and on non-zero exit returns `{ continue: true, additionalContext: text }` — matching the reference doc's implementation.

### Directory layout (output)

```
.omp/
├── commands/
│   ├── fl-new.md
│   ├── fl-plan.md
│   ├── fl-audit.md
│   ├── fl-implement.md
│   └── fl-review.md
└── hooks/
    └── pre/
        └── fl-gate.ts
```

### DEFAULT_TARGETS change

Add `Target::OhMyPi` to `DEFAULT_TARGETS = [Claude, OpenCode, OhMyPi]`. This means bare `fl setup` installs to all three. Users can opt out with `--tool claude --tool opencode` if they don't use omp.

## Implementation Steps

### Step 1: Add `plugin/omp-fl-gate.ts`

New file at `plugin/omp-fl-gate.ts`:

```typescript
import type { ExtensionAPI } from "@oh-my-pi/pi-coding-agent";
import { $ } from "bun";

export default function flGateHook(pi: ExtensionAPI): void {
  pi.setLabel("ForceLoop Gate");

  pi.on("session_stop", async (_event, _ctx) => {
    let text = "";
    try {
      const result = await $`fl gate`.nothrow().quiet();
      if (result.exitCode === 0) return;
      text = result.text();
    } catch (err) {
      text = String(err);
    }
    if (!text.trim()) return;
    return {
      continue: true,
      additionalContext: text,
    };
  });
}
```

### Step 2: Add `OhMyPi` to Target enum (`src/compiler.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Claude,
    OpenCode,
    OhMyPi,
}
```

`compile()` maps `OhMyPi` → `compile_to_opencode` (same format).

### Step 3: Add `Tool::OhMyPi` to clap enum (`src/cli.rs`)

```rust
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Tool {
    Claude,
    #[value(name = "opencode")]
    OpenCode,
    #[value(name = "omp")]
    OhMyPi,
}
```

Add `From<Tool> for Target`:
```rust
Tool::OhMyPi => Target::OhMyPi,
```

### Step 4: Update `setup.rs`

**`target_subdir()`** — add `OhMyPi` target:
```rust
Target::OhMyPi => ".omp/commands",
```

**`DEFAULT_TARGETS`** — add `Target::OhMyPi`:
```rust
pub const DEFAULT_TARGETS: &[Target] = &[Target::Claude, Target::OpenCode, Target::OhMyPi];
```

**`run()`** — after the compile loop, write omp hook:
```rust
if target == Target::OhMyPi {
    write_omp_hook(root, &mut written)?;
}
```

**New `write_omp_hook()`** — similar to `write_opencode_hook`:
- Write `.omp/hooks/pre/fl-gate.ts` (embedded via `include_str!("../plugin/omp-fl-gate.ts")`)

### Step 5: Add `omp_hook_ts_content()` function

Embed the new plugin file:
```rust
fn omp_hook_ts_content() -> &'static str {
    include_str!("../plugin/omp-fl-gate.ts")
}
```

### Step 6: Update `setup.rs` tests

- `default_targets_is_both_platforms` → `default_targets_has_three_platforms` (or rename to test all three)
- Add `plugin_hook_ts_*` style tests for `omp_hook_ts_content()`:
  - Uses `fl gate`
  - Uses `session_stop`
  - Uses `continue: true`
  - Uses `additionalContext`
  - Uses `.nothrow().quiet()`
  - Uses `@oh-my-pi/pi-coding-agent`

### Step 7: Update integration tests (`tests/setup_tool.rs`)

Update expected file counts:
| Test | Before | After |
|------|--------|-------|
| `run_default_writes_both_targets` | 11 (5×2 + 1 hook) | 17 (5×3 + 2 hooks) |
| `run_writes_all_five_commands_per_target` | 5 | 5 (unchanged, per-target) |
| `claude_only_writes_claude_dir` | 5 | 5 (unchanged) |
| `opencode_only_writes_opencode_dir` | 6 (5 + 1 hook) | 6 (unchanged) |
| New: `omp_only_writes_omp_dir` | — | 6 (5 + 1 hook) |
| `default_targets_is_both_platforms` | Claude+OpenCode | Claude+OpenCode+OhMyPi |

### Step 8: Update `tests/command_compile.rs`

No change needed — compile tests already use `Target::OpenCode` and the new `OhMyPi` target reuses the same format. Optionally add a test for `compile_to_omp` equivalence.

## File Change Summary

| File | Change |
|------|--------|
| `plugin/omp-fl-gate.ts` | **New** — omp hook TypeScript |
| `src/compiler.rs` | Add `Target::OhMyPi` variant |
| `src/cli.rs` | Add `Tool::OhMyPi` + `From` impl |
| `src/setup.rs` | Add omp subdir, hook writing, DEFAULT_TARGETS update |
| `tests/setup_tool.rs` | Update counts, add omp tests |

## Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| omp not installed in user's environment | Medium | Hook file is harmless if omp isn't running — other tools ignore `.omp/` directories |
| Users who only use Claude/OpenCode see extra files | Low | `--tool claude --tool opencode` to exclude omp; documented in `--help` |
| Plugin format changes between doc and omp release | Low | File is editable at `plugin/omp-fl-gate.ts` — rebuild picks up changes |
| Test count assertions fail | High | Intentional — update expected counts in test file |
| OpenCode plugin tests reference Target count | Low | No OpenCode test checks Target enum count |

## Verification Steps

1. `cargo test` — all tests pass
2. `cargo clippy --all-targets` — zero warnings
3. Manual: `fl setup --tool omp` in test project, verify:
   - `.omp/commands/fl-*.md` (5 files) exist
   - `.omp/hooks/pre/fl-gate.ts` exists with correct content
   - No `.claude/` or `.opencode/` directories created
4. Manual: `fl setup` (default), verify Claude + OpenCode + omp all written
5. Read back `.omp/hooks/pre/fl-gate.ts` — verify `session_stop`, `continue: true`, `fl gate` present