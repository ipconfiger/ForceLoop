# Plan: 修改 `audit.rs` — 前提条件检测 + 交叉事实审核

## Requirements

按 `new_cmd.rs` / `plan.rs` 相同的模式更新 `audit.rs`：

1. 注册 `Audit` 为 CLI 子命令（`fl audit`）
2. `execute()` 检测 `.forceloop/state.json` 中前两步（specs/plans）是否完成
3. 有未完成 → 阻止后续执行，提示用户
4. 全部完成 → 通过 prompt 驱动 LLM 交叉审核 specs 和 plans
5. 交叉审核：检查设计冲突、意图理解错误

## Changes

### 1. `src/commands/audit.rs`

#### Imports
```rust
use std::fs;
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::state::PipelineState;
use crate::traits::{CommandMetadata, Executable, Subcommand};
```

#### `execute()`
```rust
fn execute(&self, _ctx: &Context) -> Result<()> {
    // 1. Read current pipeline state
    let state_path = PipelineState::locate_state_file()?;
    let state = PipelineState::read_or_default(&state_path)?;

    // 2. Check prerequisites: New and Plan gates must have passed
    //    (current phase must be at or beyond Plan)
    use crate::state::PipelinePhase::*;
    match state.current_phase {
        New => {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: specs not ready. Run `/fl-new` first.".into()
            ));
        }
        Plan => {
            return Err(ForceLoopError::Execution(
                "Prerequisites not met: plans not ready. Run `/fl-plan` first.".into()
            ));
        }
        _ => {} // Audit or beyond — proceed
    }

    println!("✓ Prerequisites met: specs and plans are ready.");
    println!("  Use the `/fl-audit` skill to cross-verify specs and plans.");
    Ok(())
}
```

#### SKILL_PROMPT — 交叉事实审核
```rust
# Audit Skill

Cross-verify the design specs and development plans.

Read from: `.forceloop/specs/` and `.forceloop/plans/`

## Steps
0. Run the shell command `fl audit`.
   This checks that specs and plans are ready before proceeding.
1. Read all spec files from `.forceloop/specs/` (start with `index.md`).
2. Read all plan wave files from `.forceloop/plans/` (start with `index.md`).
3. Cross-verify for:
   - Design conflicts between spec modules
   - Plan waves that misinterpret the spec intent
   - Missing coverage in plans (spec aspects not addressed)
   - Contradictory requirements across modules
4. Output a structured review with severity-rated issues:
   - CRITICAL: blocker, must fix before implement
   - HIGH: significant gap
   - MEDIUM: improvement recommended
   - LOW: nitpick
```

#### Subcommand impl
```rust
impl Subcommand for Audit {
    fn name(&self) -> &'static str { "audit" }
    fn description(&self) -> &'static str {
        "Audit design spec and development plan"
    }
}
```

### 2. `src/cli.rs` — 添加 Audit 变体

### 3. `src/main.rs` — 分发

### 4. `tests/cli_help.rs` — 更新

## Verification
```bash
cargo test
cargo clippy
fl --help  # 显示 audit
fl audit   # 检查前提条件
```