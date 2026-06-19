# Plan: 修改 `plan.rs` — 模块化 Wave 计划生成（对标 `new_cmd.rs`）

## Requirements

按照与 `new_cmd.rs` 相同的原则，将 `plan.rs` 从单文件 `plan.json` 模式改为多文件 wave 计划模式：

1. **Prompt 驱动**：LLM 读取 `.forceloop/specs/` 下的设计文稿，生成多个波次的开发计划
2. **多文件输出**：每个 wave 一个独立 `.md` 文件，写入 `.forceloop/plans/`
3. **Wiki link 引用**：每个计划文件顶部引用关联的 spec 文件
4. **Checklist**：每个计划文件结尾有可勾选的执行检测项
5. **TDD Red-Green 模式**：测试包含左右边界、正确/出错用例；编码后有测试运行任务、修复回归任务、交叉验证任务（无桩/Mock）
6. **Index 索引**：`plans/index.md` 引用全部 wave 文件
7. **Gate 验证**：`plans/index.md` 作为 artifact，用 `verify_artifact` 检查 wiki link 完整性

---

## Changes

### 1. `src/constants.rs` — 新增 PLANS 常量

```rust
pub const PLANS_DIR: &str = "plans";
pub const PLANS_INDEX: &str = "plans/index.md";
```

### 2. `src/commands/plan.rs` — 全面更新

#### 2a. Imports

```rust
use std::fs;
use crate::constants::{PLANS_DIR, PLANS_INDEX};
use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::state::{verify_artifact, PipelineState};
use crate::traits::{CommandMetadata, Executable, Subcommand};
```

#### 2b. SKILL_PROMPT

```rust
const SKILL_PROMPT: &str = "\
# Plan Skill

Decompose the design spec into multiple development plan waves.

Spec source: `.forceloop/specs/`

## Steps
0. Run the shell command `fl plan`.
   This creates the `.forceloop/plans/` directory scaffold.
1. Read all spec files from `.forceloop/specs/`.
   Start with `index.md` to discover all modules, then read each module file.
2. Analyze the specs and decompose the work into waves.
   Each wave should be a coherent, independently implementable chunk.
3. For each wave, create an independent markdown file under
   `.forceloop/plans/`. File name: kebab-case with `.md` extension
   (e.g. `wave-1-core-model.md`, `wave-2-api.md`).
4. At the TOP of each wave file, add a wiki link to the spec file(s)
   it implements, e.g. `Based on: [[architecture.md]]`, `See: [[data-model.md]]`.
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
```

#### 2c. COMMAND_PROMPT — 合并 SKILL_PROMPT 的完整内容

和 `new_cmd.rs` 一样，COMMAND_PROMPT 包含完整步骤指令：

```rust
const COMMAND_PROMPT: &str = "\
Create a multi-wave development plan from the design specs.

Arguments: $ARGUMENTS

## Steps
0. Run the shell command `fl plan`.
1. Read all spec files from `.forceloop/specs/`...
... (same as SKILL_PROMPT)
## Verification
...
";
```

#### 2d. execute()

```rust
fn execute(&self, _ctx: &Context) -> Result<()> {
    let forceloop_dir = PipelineState::locate_forceloop_dir()?;
    let plans_dir = forceloop_dir.join(PLANS_DIR);
    fs::create_dir_all(&plans_dir)?;
    println!("✓ .forceloop/plans/ directory created");
    println!("  Use the `/fl-plan` skill in Claude Code or OpenCode");
    println!("  to generate multi-wave development plans.");
    println!("  Or run `fl gate` when plans are ready.");
    Ok(())
}
```

#### 2e. Subcommand

```rust
impl Subcommand for Plan {
    fn name(&self) -> &'static str { "plan" }
    fn description(&self) -> &'static str {
        "Create development plan (multiple waves)"
    }
}
```

#### 2f. artifacts()

```rust
fn artifacts(&self) -> &[&'static str] {
    &[".forceloop/plans/index.md"]
}
```

#### 2g. gate()

```rust
fn gate(&self, _ctx: &Context) -> Result<()> {
    let forceloop_dir = PipelineState::locate_forceloop_dir()?;
    let index_path = forceloop_dir.join(PLANS_INDEX);
    verify_artifact(&index_path)
}
```

### 3. `src/cli.rs` — 添加 Plan 变体

```rust
pub enum Commands {
    Setup { ... },
    Gate,
    New,
    /// Create development plan (multiple waves)
    Plan,
    Status,
    Archive,
}
```

### 4. `src/main.rs` — 分发

```rust
Commands::Plan => forceloop::commands::Plan.execute(&ctx)?,
```

### 5. `tests/cli_help.rs` — 更新

增加 `stdout.contains("plan")`

---

## 不变

- `plan.json` 的引用被**移除**，替换为 `plans/index.md`
- 其他命令（review, try_finish, audit, implement）的 prompt 中仍然引用 `plan.json`，后续再统一更新

## Verification

```bash
cargo test
cargo clippy --all-targets
cargo run -- --help  # 显示 plan
cargo run -- plan    # 创建 .forceloop/plans/
```