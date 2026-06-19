# ForceLoop CLI Framework Scaffolding

## TL;DR

> **Quick Summary**: 为 ForceLoop 搭建 Rust CLI 应用框架骨架——定义 trait、创建子命令/技能结构体、配置 Cargo.toml 和模块结构。所有具体业务逻辑使用 `todo!()` 占位，后续逐步实现。
> 
> **Deliverables**:
> - Cargo.toml 配置 (clap, anyhow, thiserror, serde, serde_json)
> - src/ 目录结构 (~20 个 .rs 文件)
> - 2 个核心 trait: `Subcommand`, `Executable`
> - 4 个子命令骨架: setup, gate, status, archive
> - 6 个技能/命令骨架: new, plan, audit, implement, review, try_finish
> - 基础集成测试验证 --help 输出
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 1 (Cargo.toml) → Task 2 (errors.rs) → Task 3 (context.rs) → Task 4 (traits) → Tasks 5-11 → Task 12 (cli.rs) → Task 13 (main.rs) → Task 14 (tests)

---

## Context

### Original Request
根据需求文档 docs/requirment.md 生成一个基础的应用框架，包含所有的子命令和 skills，commands 的结构定义，具体功能实现留空，等后期再一一实现。

### Interview Summary
**Key Discussions**:
- CLI 框架选择: clap v4 derive API (Rust CLI 子命令事实标准)
- Skills 和 Commands 是同一组 6 个结构体 (不是 12 个)
- trait 方法用同步 (sync)，不用 async
- 错误处理: anyhow (应用层) + thiserror (域错误)
- Rust edition 2024
- 基础测试需求确认

**Research Findings**:
- clap derive 是 Rust CLI 子命令首选 (16k stars)
- 参考 gitoxide CLI 和 Fyrox 命令系统架构
- `impl` 是 Rust 关键字，必须重命名为 `Implement`
- `try_finish` 在 CLI 中应为 `try-finish`，用 clap attribute 处理
- 模块依赖方向: `main → cli → {commands/, skills/} → {context, errors}` 无反向依赖

### Metis Review
**Identified Gaps** (all addressed):
- `impl` 关键字冲突 → 重命名为 `Implement`
- Skills vs Commands 关系确认 → 同一组 6 个 struct
- sync vs async → sync
- trait objects vs enum dispatch → 使用 trait objects (需求明确要求 trait 抽象)
- 模块循环依赖风险 → 严格单向依赖图
- `todo!()` 在测试中的行为 → trait 方法用 `todo!()`，测试不调用 execute()

---

## Work Objectives

### Core Objective
搭建 ForceLoop CLI 应用框架骨架，定义完整的类型系统和模块结构，确保编译通过且 --help 正常输出。

### Concrete Deliverables
- `Cargo.toml` — 项目配置和依赖声明
- `rust-toolchain.toml` — 固定 Rust 版本
- `src/main.rs` — 入口点
- `src/cli.rs` — CLI 定义 (clap derive)
- `src/context.rs` — 上下文结构体
- `src/errors.rs` — 错误类型
- `src/commands/{mod,new,plan,audit,implement,review,try_finish}.rs` — 6 个技能/命令
- `src/{setup,gate,status,archive}.rs` — 4 个子命令
- `tests/cli_help.rs` — 基础集成测试

### Definition of Done
- [x] `cargo check` 通过，零错误零警告
- [x] `cargo build` 产出二进制 `forceloop`
- [x] `./target/debug/forceloop --help` 显示 4 个子命令
- [x] `./target/debug/forceloop setup --help` 显示 setup 帮助
- [x] `cargo test` 通过 (基础测试)

### Must Have
- 所有 trait 定义完整且可编译
- 所有子命令和技能结构体存在
- `forceloop --help` 正确显示子命令列表
- 模块依赖方向严格单向

### Must NOT Have (Guardrails)
- **零业务逻辑**: 所有 `fn` 体只有 `todo!()` 或 `Ok(())`
- **零额外依赖**: 仅 clap, anyhow, thiserror, serde, serde_json
- **无 async**: 同步 trait 签名
- **无 builder/helper 方法**: 不添加 `fn new()`, `fn default()`, `fn builder()`
- **无 CI/CD 配置**: 不创建 .github/workflows/
- **无多余注释**: 不在 `todo!()` 上方添加文档注释
- **无 config 文件解析**: Context 只占位，不实现 load/save

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: NO (greenfield)
- **Automated tests**: YES (基础集成测试)
- **Framework**: Rust built-in test framework (`cargo test`)
- **Type**: Integration test verifying CLI help output

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — foundation):
├── Task 1: Cargo.toml + rust-toolchain.toml [quick]
├── Task 2: errors.rs — 错误类型定义 [quick]
└── Task 3: context.rs — 上下文结构体 [quick]

Wave 2 (After Wave 1 — core abstractions + modules):
├── Task 4: Executable trait + Subcommand trait 定义 [quick]
├── Task 5: commands/mod.rs + 6 个技能/命令骨架 [unspecified-high]
├── Task 6: setup.rs 子命令骨架 [quick]
├── Task 7: gate.rs 子命令骨架 [quick]
├── Task 8: status.rs 子命令骨架 [quick]
└── Task 9: archive.rs 子命令骨架 [quick]

Wave 3 (After Wave 2 — integration + wiring):
├── Task 10: cli.rs — CLI 定义 (clap derive enum) [quick]
├── Task 11: main.rs — 入口点 [quick]
└── Task 12: tests/cli_help.rs — 基础集成测试 [quick]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
→ Present results → Get explicit user okay

Critical Path: T1 → T2 → T4 → T5 → T10 → T11 → T12 → F1-F4
Parallel Speedup: ~50% faster than sequential
Max Concurrent: 5 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1    | -         | 2-11   | 1    |
| 2    | 1         | 4      | 1    |
| 3    | 1         | 4, 5   | 1    |
| 4    | 2, 3      | 5-9    | 2    |
| 5    | 4         | 10     | 2    |
| 6    | 4         | 10     | 2    |
| 7    | 4         | 10     | 2    |
| 8    | 4         | 10     | 2    |
| 9    | 4         | 10     | 2    |
| 10   | 5-9       | 11     | 3    |
| 11   | 10        | 12     | 3    |
| 12   | 11        | F1-F4  | 3    |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 `quick`, T2 `quick`, T3 `quick`
- **Wave 2**: 5 tasks — T4 `quick`, T5 `unspecified-high`, T6 `quick`, T7 `quick`, T8 `quick`, T9 `quick`
- **Wave 3**: 3 tasks — T10 `quick`, T11 `quick`, T12 `quick`
- **FINAL**: 4 tasks — F1 `oracle`, F2 `unspecified-high`, F3 `unspecified-high`, F4 `deep`

---

## TODOs

- [x] 1. Cargo.toml + rust-toolchain.toml — 项目配置

  **What to do**:
  - 创建 `Cargo.toml`:
    ```toml
    [package]
    name = "forceloop"
    version = "0.1.0"
    edition = "2024"
    description = "A CLI tool for structured development workflow"

    [dependencies]
    clap = { version = "4", features = ["derive"] }
    anyhow = "1"
    thiserror = "2"
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    ```
  - 创建 `rust-toolchain.toml` 固定 Rust 版本 (channel = "1.85.0" 或 "stable")
  - 创建 `src/` 目录

  **Must NOT do**:
  - 不添加除指定外的任何依赖
  - 不添加 build.rs

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (foundation for everything)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 2, 3
  - **Blocked By**: None

  **References**:
  - `docs/requirment.md` — 项目需求定义
  - Cargo.toml edition 2024: https://doc.rust-lang.org/edition-guide/rust-2024/

  **Acceptance Criteria**:

  **QA Scenarios:**
  ```
  Scenario: Cargo.toml compiles
    Tool: Bash
    Preconditions: Cargo.toml and src/lib.rs (or src/main.rs) exist
    Steps:
      1. Run `cargo check`
      2. Assert exit code is 0
    Expected Result: cargo check succeeds with no errors
    Failure Indicators: exit code non-zero, dependency resolution failure
    Evidence: .sisyphus/evidence/task-1-cargo-check.txt

  Scenario: Dependencies resolve correctly
    Tool: Bash
    Steps:
      1. Run `cargo tree --depth 0`
      2. Assert output contains "clap", "anyhow", "thiserror", "serde", "serde_json"
    Expected Result: All 5 dependencies listed
    Evidence: .sisyphus/evidence/task-1-deps.txt
  ```

  **Commit**: YES
  - Message: `feat(core): initialize Cargo.toml and rust-toolchain`
  - Files: `Cargo.toml, rust-toolchain.toml`
  - Pre-commit: `cargo check`

- [x] 2. errors.rs — 错误类型定义

  **What to do**:
  - 创建 `src/errors.rs`:
    ```rust
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ForceLoopError {
        #[error("Configuration error: {0}")]
        Config(String),

        #[error("I/O error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Parse error: {0}")]
        Parse(String),

        #[error("Execution error: {0}")]
        Execution(String),
    }

    pub type Result<T> = std::result::Result<T, ForceLoopError>;
    ```

  **Must NOT do**:
  - 不添加超过 4 个错误变体
  - 不实现 Display/From 超出 thiserror derive 提供的

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 3)
  - **Parallel Group**: Wave 1 (with Task 3)
  - **Blocks**: Task 4
  - **Blocked By**: Task 1

  **References**:
  - `docs/requirment.md` — 错误处理需求
  - thiserror docs: https://docs.rs/thiserror

  **Acceptance Criteria**:

  **QA Scenarios:**
  ```
  Scenario: Error type compiles
    Tool: Bash
    Preconditions: Cargo.toml exists with thiserror dep
    Steps:
      1. Run `cargo check`
      2. Assert exit code 0
    Expected Result: ForceLoopError compiles, Result<T> alias available
    Evidence: .sisyphus/evidence/task-2-errors-compile.txt
  ```

  **Commit**: NO (groups with Wave 1 commit)

- [x] 3. context.rs — 上下文结构体

  **What to do**:
  - 创建 `src/context.rs`:
    ```rust
    /// Application context passed to subcommands and skills
    pub struct Context {
        // TODO: add fields as business logic is implemented
    }

    impl Context {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Default for Context {
        fn default() -> Self {
            Self::new()
        }
    }
    ```
  - Context 只做占位，不依赖 commands/ 或 skills/

  **Must NOT do**:
  - 不添加实际字段
  - 不实现任何 I/O 方法

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: Tasks 4, 5
  - **Blocked By**: Task 1

  **References**:
  - `docs/requirment.md` — 子命令和 Skill 共用 Context

  **Acceptance Criteria**:

  **QA Scenarios:**
  ```
  Scenario: Context compiles standalone
    Tool: Bash
    Steps:
      1. Run `cargo check`
      2. Assert exit code 0
    Expected Result: Context::new() and Default impl compile
    Evidence: .sisyphus/evidence/task-3-context-compile.txt
  ```

  **Commit**: NO (groups with Wave 1 commit)

- [x] 4. Executable trait + Subcommand trait 定义

  **What to do**:
  - 创建 `src/traits.rs` (或直接在 src/lib.rs 中):
    ```rust
    use crate::context::Context;
    use crate::errors::Result;

    /// Trait shared by Skills and Commands
    pub trait Executable {
        fn execute(&self, ctx: &Context) -> Result<()>;
    }

    /// Trait for top-level subcommands (setup, gate, status, archive)
    pub trait Subcommand: Executable {
        fn name(&self) -> &'static str;
        fn description(&self) -> &'static str;
    }
    ```
  - Subcommand 继承 Executable (所有子命令也是可执行的)

  **Must NOT do**:
  - 不添加泛型参数或关联类型
  - 不添加 async
  - 不添加生命周期标注

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (foundation for Wave 2)
  - **Parallel Group**: Wave 2 start
  - **Blocks**: Tasks 5-9
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `docs/requirment.md` — "Skills和Command有关联，共用一个trait"
  - `docs/requirment.md` — "子命令、Skill、自定义Command 都要抽象出trait"

  **Acceptance Criteria**:

  **QA Scenarios:**
  ```
  Scenario: Traits compile and are usable
    Tool: Bash
    Steps:
      1. Run `cargo check`
      2. Assert exit code 0
    Expected Result: Executable and Subcommand traits compile, Subcommand: Executable bound works
    Evidence: .sisyphus/evidence/task-4-traits-compile.txt
  ```

  **Commit**: NO (groups with Wave 2 commit)

- [x] 5. commands/ — 6 个技能/命令骨架

  **What to do**:
  - 创建 `src/commands/mod.rs` 声明并导出 6 个子模块
  - 创建 6 个文件: `new_cmd.rs` (New), `plan.rs` (Plan), `audit.rs` (Audit), `implement.rs` (Implement), `review.rs` (Review), `try_finish.rs` (TryFinish)
  - 每个 struct 实现 `Executable` trait, execute() 体为 `todo!()`
  - 注意: `new` 文件名用 `new_cmd.rs` 避免混淆, `impl` 用 `implement.rs` 避免关键字冲突

  **Must NOT do**:
  - 不在 execute() 中添加任何逻辑
  - 不添加 struct 字段
  - 不添加 builder/构造方法

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high` (6 files to create)
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 6-9)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 10
  - **Blocked By**: Task 4

  **References**:
  - `docs/requirment.md` — Skills列表和自定义Command列表 (同一组 6 个)

  **QA Scenarios:**
  ```
  Scenario: All command modules compile
    Tool: Bash
    Steps:
      1. Run `cargo check`
    Expected Result: exit 0, no dead_code warnings on command structs
    Evidence: .sisyphus/evidence/task-5-commands-compile.txt

  Scenario: File structure correct
    Tool: Bash
    Steps:
      1. Run `find src/commands/ -name '*.rs' | sort`
      2. Assert 7 files: mod.rs, new_cmd.rs, plan.rs, audit.rs, implement.rs, review.rs, try_finish.rs
    Expected Result: Exactly 7 .rs files
    Evidence: .sisyphus/evidence/task-5-commands-files.txt
  ```

  **Commit**: NO (groups with Wave 2 commit)

- [x] 6. setup.rs 子命令骨架

  **What to do**:
  - 创建 `src/setup.rs`
  - struct Setup 实现 `Executable` + `Subcommand` trait
  - name() -> "setup", description() -> "Initialize project directory structure, state, subcommands, skills, and hooks"
  - execute() -> todo!()

  **Must NOT do**: 不添加实际初始化逻辑, 不添加 struct 字段

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 2 parallel (with Tasks 5, 7, 8, 9), Blocks: Task 10, Blocked By: Task 4

  **References**: `docs/requirment.md` — "setup 在项目中初始化目录结构，状态数据，子command，Skill，hook"

  **QA Scenarios:**
  ```
  Scenario: Setup compiles
    Tool: Bash
    Steps:
      1. `cargo check`
    Expected Result: exit 0
    Evidence: .sisyphus/evidence/task-6-setup-compile.txt
  ```

  **Commit**: NO (groups with Wave 2 commit)

- [x] 7. gate.rs 子命令骨架

  **What to do**:
  - 创建 `src/gate.rs`
  - struct Gate 实现 Executable + Subcommand
  - name() -> "gate", description() -> "Gate control command, typically invoked by hooks", execute() -> todo!()

  **Must NOT do**: 同 Task 6

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 2 parallel, Blocks: Task 10, Blocked By: Task 4

  **References**: `docs/requirment.md` — "gate 门控指令，一般由钩子调用"

  **QA Scenarios:**
  ```
  Scenario: Gate compiles
    Tool: Bash
    Steps:
      1. `cargo check`
    Expected Result: exit 0
    Evidence: .sisyphus/evidence/task-7-gate-compile.txt
  ```

  **Commit**: NO (groups with Wave 2 commit)

- [x] 8. status.rs 子命令骨架

  **What to do**:
  - 创建 `src/status.rs`
  - struct Status 实现 Executable + Subcommand
  - name() -> "status", description() -> "View current status", execute() -> todo!()

  **Must NOT do**: 同 Task 6

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 2 parallel, Blocks: Task 10, Blocked By: Task 4

  **References**: `docs/requirment.md` — "status 查看当前状态"

  **QA Scenarios:**
  ```
  Scenario: Status compiles
    Tool: Bash
    Steps:
      1. `cargo check`
    Expected Result: exit 0
    Evidence: .sisyphus/evidence/task-8-status-compile.txt
  ```

  **Commit**: NO (groups with Wave 2 commit)

- [x] 9. archive.rs 子命令骨架

  **What to do**:
  - 创建 `src/archive.rs`
  - struct Archive 实现 Executable + Subcommand
  - name() -> "archive", description() -> "Archive development plan", execute() -> todo!()

  **Must NOT do**: 同 Task 6

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 2 parallel, Blocks: Task 10, Blocked By: Task 4

  **References**: `docs/requirment.md` — "archive 归档开发计划"

  **QA Scenarios:**
  ```
  Scenario: Archive compiles
    Tool: Bash
    Steps:
      1. `cargo check`
    Expected Result: exit 0
    Evidence: .sisyphus/evidence/task-9-archive-compile.txt
  ```

  **Commit**: NO (groups with Wave 2 commit)

- [x] 10. cli.rs — CLI 定义 (clap derive enum)

  **What to do**:
  - 创建 `src/cli.rs`:
    ```rust
    use clap::{Parser, Subcommand};
    use crate::setup::Setup;
    use crate::gate::Gate;
    use crate::status::Status;
    use crate::archive::Archive;

    #[derive(Parser)]
    #[command(name = "forceloop")]
    #[command(about = "A CLI tool for structured development workflow")]
    #[command(version)]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand)]
    pub enum Commands {
        /// Initialize project directory structure, state, subcommands, skills, and hooks
        Setup,
        /// Gate control command, typically invoked by hooks
        Gate,
        /// View current status
        Status,
        /// Archive development plan
        Archive,
    }
    ```

  **Must NOT do**:
  - 不在 clap struct 上添加多余字段
  - 不实现 From/Into 转换

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 3 start, Blocks: Task 11, Blocked By: Tasks 5-9

  **References**: clap derive docs: https://docs.rs/clap/latest/clap/

  **QA Scenarios:**
  ```
  Scenario: CLI enum compiles
    Tool: Bash
    Steps:
      1. `cargo check`
    Expected Result: exit 0
    Evidence: .sisyphus/evidence/task-10-cli-compile.txt
  ```

  **Commit**: NO (groups with Wave 3 commit)

- [x] 11. main.rs — 入口点

  **What to do**:
  - 创建 `src/main.rs`:
    ```rust
    use anyhow::Result;
    use clap::Parser;
    use forceloop::cli::Cli;
    use forceloop::context::Context;

    fn main() -> Result<()> {
        let cli = Cli::parse();
        let ctx = Context::new();

        match cli.command {
            Commands::Setup => forceloop::setup::Setup.execute(&ctx)?,
            Commands::Gate => forceloop::gate::Gate.execute(&ctx)?,
            Commands::Status => forceloop::status::Status.execute(&ctx)?,
            Commands::Archive => forceloop::archive::Archive.execute(&ctx)?,
        }

        Ok(())
    }
    ```
  - 注意: 实际调用会触发 todo!() panic，这在框架阶段是预期行为

  **Must NOT do**:
  - 不添加错误恢复逻辑
  - 不添加 logger/tracing

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 3, Blocks: Task 12, Blocked By: Task 10

  **References**: `docs/requirment.md` — 子命令入口

  **QA Scenarios:**
  ```
  Scenario: Binary builds and runs --help
    Tool: Bash
    Steps:
      1. `cargo build`
      2. `./target/debug/forceloop --help`
      3. Assert stdout contains "setup", "gate", "status", "archive"
    Expected Result: Help shows all 4 subcommands
    Evidence: .sisyphus/evidence/task-11-help-output.txt

  Scenario: Subcommand --help works
    Tool: Bash
    Steps:
      1. `./target/debug/forceloop setup --help`
      2. Assert exit code 0 and output contains "Initialize"
    Expected Result: Setup help shows description
    Evidence: .sisyphus/evidence/task-11-setup-help.txt
  ```

  **Commit**: NO (groups with Wave 3 commit)

- [x] 12. tests/cli_help.rs — 基础集成测试

  **What to do**:
  - 创建 `tests/cli_help.rs`:
    ```rust
    use std::process::Command;

    #[test]
    fn help_shows_all_subcommands() {
        let output = Command::new("./target/debug/forceloop")
            .arg("--help")
            .output()
            .expect("Failed to execute forceloop");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("setup"));
        assert!(stdout.contains("gate"));
        assert!(stdout.contains("status"));
        assert!(stdout.contains("archive"));
    }

    #[test]
    fn setup_help_works() {
        let output = Command::new("./target/debug/forceloop")
            .args(["setup", "--help"])
            .output()
            .expect("Failed to execute forceloop setup --help");

        assert!(output.status.success());
    }
    ```
  - 注意: 集成测试需要先 `cargo build` 产出二进制

  **Must NOT do**:
  - 不测试 execute() 行为 (会 panic)
  - 不添加 fixture 文件

  **Recommended Agent Profile**: `quick`, Skills: []

  **Parallelization**: Wave 3 final, Blocks: F1-F4, Blocked By: Task 11

  **References**: Rust integration testing: https://doc.rust-lang.org/book/ch11-03-test-organization.html

  **QA Scenarios:**
  ```
  Scenario: Tests compile and pass
    Tool: Bash
    Steps:
      1. `cargo build`
      2. `cargo test`
    Expected Result: All tests pass
    Evidence: .sisyphus/evidence/task-12-tests-pass.txt
  ```

  **Commit**: YES
  - Message: `feat(cli): wire CLI definition, main entry, and basic tests`
  - Files: `src/cli.rs, src/main.rs, tests/cli_help.rs`
  - Pre-commit: `cargo test`

---

## Final Verification Wave (after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo check` + `cargo clippy` + `cargo test`. Review all files for: dead code, unused imports, missing mod declarations, circular dependencies. Verify trait signatures compile correctly. Check module visibility (pub vs pub(crate)).
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state (`cargo clean && cargo build`). Run `./target/debug/forceloop --help` — verify all 4 subcommands listed. Run `./target/debug/forceloop setup --help` — verify help output. Run each subcommand help. Run `cargo test`. Save all output to evidence.
  Output: `Scenarios [N/N pass] | Help Output [verified] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual files. Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT Have" compliance: no business logic, no extra deps, no async, no builder methods. Flag unaccounted files.
  Output: `Tasks [N/N compliant] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **Wave 1**: `feat(core): add Cargo.toml, errors, and context scaffolding` — Cargo.toml, rust-toolchain.toml, src/errors.rs, src/context.rs
  - Pre-commit: `cargo check`
- **Wave 2**: `feat(core): add traits, subcommands, and skill/command structs` — src/traits.rs, src/commands/*.rs, src/{setup,gate,status,archive}.rs
  - Pre-commit: `cargo check`
- **Wave 3**: `feat(cli): wire CLI definition, main entry, and basic tests` — src/cli.rs, src/main.rs, tests/cli_help.rs
  - Pre-commit: `cargo test`

---

## Success Criteria

### Verification Commands
```bash
cargo check                    # Expected: exit 0, zero errors/warnings
cargo build                    # Expected: exit 0, binary created
cargo test                     # Expected: all tests pass
./target/debug/forceloop --help          # Expected: shows setup, gate, status, archive
./target/debug/forceloop setup --help    # Expected: shows setup usage
./target/debug/forceloop gate --help     # Expected: shows gate usage
./target/debug/forceloop status --help   # Expected: shows status usage
./target/debug/forceloop archive --help  # Expected: shows archive usage
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] Binary runs and shows help
- [ ] No circular module dependencies
- [ ] All trait definitions compile
