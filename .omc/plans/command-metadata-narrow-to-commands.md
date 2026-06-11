# CommandMetadata Trait 收窄：仅 `src/commands/` 下的 6 个 Command 定义 prompt

## TL;DR

> **Quick Summary**: 将 `CommandMetadata` trait 的实现者从"全部 10 个 Command"收窄为"`src/commands/` 下的 6 个 Skill/Custom Command"。移除 4 个顶层子命令（Setup、Gate、Status、Archive）的 `impl CommandMetadata` 块及其支撑代码（`SKILL_PROMPT` / `COMMAND_PROMPT` 常量、`*_skill()` / `*_command()` 工厂函数、`CommandMetadata` 的 use 导入）。`src/setup.rs` 的 `COMMANDS` 静态表从 9 行减为 6 行。`tests/command_compile.rs` 与 `tests/setup_tool.rs` 同步收窄。
>
> **Deliverables**:
> - `src/traits.rs` — `CommandMetadata` 文档注释更新,明确"仅 Skill/Custom Command 实现"
> - `src/{setup,gate,status,archive}.rs` — 删除 `SKILL_PROMPT` / `COMMAND_PROMPT` 常量、`*_skill()` / `*_command()` 工厂函数、`impl CommandMetadata` 块、`CommandMetadata` use 导入
> - `src/setup.rs` — `COMMANDS` 静态表从 9 行减为 6 行(移除 Gate/Status/Archive)
> - `tests/command_compile.rs` — `all_10_commands_have_populated_schemas` 改名 `all_6_commands_have_populated_schemas`;`each_command_has_appropriate_tools` / `all_commands_compile_to_valid_claude_markdown` 同步收窄到 6 个
> - `tests/setup_tool.rs` — `run_writes_all_nine_commands_per_target` 改名 `run_writes_all_six_commands_per_target`;`run_default_writes_both_targets` 数字 18 → 12;`claude_only_*` / `opencode_only_*` 数字 9 → 6
>
> **Estimated Effort**: Small (10 个文件改动,纯删除 + 收窄,无新逻辑)
> **Parallel Execution**: NO(顺序:traits.rs 文档 → 4 个子命令文件 → setup.rs COMMANDS 表 → 2 个测试文件 → 验证)
> **Critical Path**: `src/{setup,gate,status,archive}.rs` → `src/setup.rs` COMMANDS 表 → `tests/`

---

## Context

### Original Request(用户原话)
> 只有在 commands目录里的才需要定义 command的prompt和skill的prompt

### Background

当前所有 10 个 Command 对象都实现 `CommandMetadata` trait([src/traits.rs:18-33](../../src/traits.rs#L18-L33)),含 4 个方法:`skill_template` / `command_template` / `artifacts` / `gate`。这是 [`.omc/plans/command-metadata-skeleton.md`](command-metadata-skeleton.md) 阶段为"骨架优先"做的妥协:所有 10 个都实现 trait,用占位值保持可编译。

但架构本身存在不对称:
- **4 个顶层子命令**(Setup/Gate/Status/Archive)位于 `src/` 根目录,是**终端 CLI 子命令**(用户敲 `forceloop setup` 之类)
- **6 个 Skill/Custom Command**(New/Plan/Audit/Implement/Review/TryFinish)位于 `src/commands/` 子目录,会被 `Setup` 注册为 platform-native Skill/Command 文件

参考 [`.omc/plans/setup-excludes-self.md`](setup-excludes-self.md) 的设计,顶层子命令**不应**有自己的 Skill/Command prompt——它们是终端入口,不是工作流步骤。但当前 4 个子命令**仍被强制实现** `skill_template()` / `command_template()` 并提供真实的 prompt 文案,产生"实现了但永远不被消费"的死代码层。

### Current State Inventory

| 类别 | 数量 | 文件 | 当前 Trait 实现 |
|------|------|------|----------------|
| 顶层子命令 | 4 | [setup.rs](../../src/setup.rs), [gate.rs](../../src/gate.rs), [status.rs](../../src/status.rs), [archive.rs](../../src/archive.rs) | `Executable` + `Subcommand` + `CommandMetadata` |
| Skill/Custom Command | 6 | [new_cmd.rs](../../src/commands/new_cmd.rs), [plan.rs](../../src/commands/plan.rs), [audit.rs](../../src/commands/audit.rs), [implement.rs](../../src/commands/implement.rs), [review.rs](../../src/commands/review.rs), [try_finish.rs](../../src/commands/try_finish.rs) | `Executable` + `CommandMetadata` |

### Why Now

- `CommandSchema` / `compiler` 已成型([`.omc/plans/command-schema-and-compiler.md`](command-schema-and-compiler.md)),4 个子命令的 prompt 文案已从占位升级为真实内容——但这些内容**在生产路径中**永远不被使用
- `tests/command_compile.rs` 在 6 个 commands/ 对象上**有意义**(验证 schema 非空),在 4 个子命令上**只是因为 trait contract 而存在**
- `COMMANDS` 静态表里 Gate/Status/Archive 三行的存在与"Setup 不应注册自身"的设计哲学不一致——它们是终端子命令,也不应被注册为 Skill

### What this does NOT change

- `Executable` trait 保持不变(10 个都实现)
- `Subcommand` trait 保持不变(4 个顶层子命令实现)
- 6 个 Skill/Custom Command 的 `CommandMetadata` impl **完全保留**(4 个方法都不变)
- `compile()` / `compile_agent()` / `CommandSchema` 字段不变
- `cli.rs` / `main.rs` 不变(4 个顶层子命令的 CLI 形态不变)
- `state.rs` / `context.rs` / `errors.rs` 不变

---

## Work Objectives

### Core Objective
让 `CommandMetadata` trait 的实现者从 10 个收窄到 6 个(仅 `src/commands/` 下的 Skill/Custom Command),移除 4 个顶层子命令的 prompt 定义和 trait impl。

### Definition of Done

- [ ] `src/traits.rs` 的 `CommandMetadata` 文档注释更新,明确"仅 Skill/Custom Command 实现"
- [ ] `src/{setup,gate,status,archive}.rs` 各删除 4 个块:`SKILL_PROMPT` / `COMMAND_PROMPT` 常量、`*_skill()` / `*_command()` 工厂函数、`impl CommandMetadata` 块、`CommandMetadata` use 导入
- [ ] `src/setup.rs` 的 `COMMANDS` 静态表从 9 行减为 6 行(移除 Gate/Status/Archive 三行)
- [ ] `src/setup.rs` 的 `COMMANDS` 表上方注释从 "9 non-setup Command objects" 改为 "6 Skill/Custom Command objects"
- [ ] `tests/command_compile.rs` 的 `all_10_commands_have_populated_schemas` 改名为 `all_6_commands_have_populated_schemas`,断言收窄到 6 个
- [ ] `tests/command_compile.rs` 的 `each_command_has_appropriate_tools` 移除 4 个子命令的断言
- [ ] `tests/command_compile.rs` 的 `all_commands_compile_to_valid_claude_markdown` schemas 数组从 10 减为 6
- [ ] `tests/command_compile.rs` 的 use 列表删除 `Archive` / `Gate` / `Status` / `Setup` 导入
- [ ] `tests/setup_tool.rs` 的 `run_default_writes_both_targets` 数字 18 → 12(6 × 2)
- [ ] `tests/setup_tool.rs` 的 `claude_only_writes_claude_dir` / `opencode_only_writes_opencode_dir` 数字 9 → 6
- [ ] `tests/setup_tool.rs` 的 `run_writes_all_nine_commands_per_target` 改名 `run_writes_all_six_commands_per_target`,期望集收窄为 6 个
- [ ] `tests/setup_tool.rs` 中 `run_default_writes_both_targets` 的 `gate.md` 路径断言替换为 `new.md`(因 gate.md 不再生成了)
- [ ] `cargo check` 零错误零警告
- [ ] `cargo build` 成功
- [ ] `cargo test` 全部测试通过
- [ ] `cargo clippy --all-targets` 零 lint
- [ ] 提交

### Must Have

- 最小变更:仅删除必要代码 + 测试断言机械收窄;不动 `Executable` / `Subcommand` 现有 trait
- 6 个 `src/commands/` 对象的 `CommandMetadata` impl **完全保留**(4 个方法都不变)
- 4 个顶层子命令的 `Subcommand` impl **完全保留**(`name` / `description` 不变)
- 编译零警告

### Must NOT Have (Guardrails)

- **不**修改 `CommandMetadata` trait 的 4 个方法签名(保持 trait contract 稳定)
- **不**修改 `Executable` 或 `Subcommand` trait
- **不**修改 6 个 `src/commands/` 对象的 `impl CommandMetadata` 块
- **不**修改 `compile()` / `compile_agent()` / `CommandSchema` 字段
- **不**修改 `cli.rs` / `main.rs`(4 个顶层子命令的 CLI 形态不变)
- **不**修改 `state.rs` / `context.rs` / `errors.rs`
- **不**添加新依赖
- **不**清理已存在的 `.claude/commands/{gate,status,archive}.md`(idempotent 行为保留;与 setup-excludes-self.md 一致)

---

## Design Decisions

### Decision 1: 收窄实现者 vs 拆分 trait

**决策**:保留单一 `CommandMetadata` trait,**收窄实现者**(从 10 到 6)。

**理由**:
- 单 trait 接口更简洁;6 个 `src/commands/` 对象 + trait contract 一目了然
- 拆分(`SubcommandMetadata` + `SkillCommandMetadata`)会引入两个 trait,调用方需做 trait object 装箱或多 dispatch
- 收窄后 trait 文档明确"Skill/Custom Command 用",与项目架构图([CLAUDE.md](../../CLAUDE.md))一致

### Decision 2: 是否保留 4 个子命令的 `artifacts()` / `gate()` 方法

**决策**:**完全移除** 4 个子命令的 `CommandMetadata` impl(包括 `artifacts()` 和 `gate()`)。

**理由**:
- 这 4 个方法在 4 个子命令上**从未被生产代码调用**:
  - `Setup` 不调用自己的 `gate()`(它是 init 命令,不是 pipeline step)
  - `Gate::execute()` 调用**其他 6 个**的 `gate()`,不调用自己的(见 [src/gate.rs:65-73](../../src/gate.rs#L65-L73))
  - `Status::execute()` 是 `todo!()`(见 [src/status.rs:50-52](../../src/status.rs#L50-L52))
  - `Archive::execute()` 是 `todo!()`(见 [src/archive.rs:51-53](../../src/archive.rs#L51-L53))
- `artifacts()` 在 Setup/Status/Archive 上语义模糊(Setup 写多文件、Status 读多文件、Archive 写 archive/);保留空/模糊返回值会误导用户
- 保留它们会维持一个"定义了但永远不消费"的死代码层,违反 [CLAUDE.md](../../CLAUDE.md) 的根因治理原则

### Decision 3: COMMANDS 静态表的简化

**决策**:从 9 行减为 6 行,仅含 `src/commands/` 下的对象。

**理由**:
- COMMANDS 表的语义是"哪些 Command 会被注册为 platform-native Skill/Command 文件"(见 [src/setup.rs:107-127](../../src/setup.rs#L107-L127))
- 4 个顶层子命令(Setup/Gate/Status/Archive)作为**终端子命令**运行,不需要被注册为 Skill——与 [setup-excludes-self.md](setup-excludes-self.md) 的 "Setup 不应出现在命令面板" 逻辑一致
- 收窄后 COMMANDS 表 = "要注册的 Skill 列表",与 6 个 `PipelinePhase`(New → Plan → Audit → Implement → Review → TryFinish)一一对应(见 [src/state.rs:13-21](../../src/state.rs#L13-L21))

### Decision 4: 是否保留 `setup_skill()` / `setup_command()` 等工厂函数

**决策**:**完全删除**。

**理由**:
- 这些函数仅被各自的 `impl CommandMetadata` 调用(见 [src/setup.rs:194-200](../../src/setup.rs#L194-L200) 等)
- trait 收窄后无 impl 调用方
- 函数本身无副作用,删除后无运行时影响
- 遵循 [CLAUDE.md](../../CLAUDE.md) 的"Root Cause"原则——不留死代码

### Decision 5: `Gate::execute()` 中的 `New.gate(ctx)?` 等调用是否需要改

**决策**:**不变**。

**理由**:
- `New` / `Plan` / `Audit` / `Implement` / `Review` / `TryFinish` 仍实现 `CommandMetadata`(`gate` 方法在 trait 内)
- `Gate::execute()` 中的 match 表达式(见 [src/gate.rs:65-73](../../src/gate.rs#L65-L73))调用的是**其他 6 个**的 `gate()`,不依赖 `Gate` 自身的 trait impl
- 编译期保证调用链完整

---

## Implementation Steps

### Step 1: 改 `src/traits.rs` — 更新 `CommandMetadata` 文档注释

**File**: [src/traits.rs](../../src/traits.rs) (lines 16-33)

**What to do**:
将 [src/traits.rs:16-17](../../src/traits.rs#L16-L17) 的注释替换为:

```rust
/// Trait for declarative metadata of Skill / Custom Command objects.
///
/// Only the 6 Command objects in `src/commands/` (New, Plan, Audit,
/// Implement, Review, TryFinish) implement this trait. The 4 top-level
/// subcommands (Setup, Gate, Status, Archive) are terminal CLI
/// subcommands — they are not Skill / Custom Commands and do not have
/// a `skill_template` / `command_template` of their own.
///
/// Provides skill/command templates (compiled to platform-native format
/// via `crate::compiler::compile`), artifact file lists, and gating logic.
```

**Must NOT do**:
- 不修改 4 个方法签名(`skill_template` / `command_template` / `artifacts` / `gate`)

**Acceptance Criteria**:
- 文档注释明确"仅 Skill/Custom Command 实现"
- `cargo check` 通过

---

### Step 2: 改 `src/setup.rs` — 删除 prompt 相关 4 块

**File**: [src/setup.rs](../../src/setup.rs)

**What to do**:
- 删除 [src/setup.rs:14-44](../../src/setup.rs#L14-L44) 的 `SKILL_PROMPT` / `COMMAND_PROMPT` 常量
- 删除 [src/setup.rs:46-63](../../src/setup.rs#L46-L63) 的 `setup_skill()` / `setup_command()` 工厂函数
- 删除 [src/setup.rs:194-207](../../src/setup.rs#L194-L207) 的 `impl CommandMetadata for Setup` 块
- 从 [src/setup.rs:12](../../src/setup.rs#L12) 的 use 列表移除 `CommandMetadata`
- 更新 [src/setup.rs:107-127](../../src/setup.rs#L107-L127) 的 `COMMANDS` 表为 6 行(移除 Gate/Status/Archive 三行):
  ```rust
  const COMMANDS: &[CommandEntry] = &[
      ("new", || New.command_template()),
      ("plan", || Plan.command_template()),
      ("audit", || Audit.command_template()),
      ("implement", || Implement.command_template()),
      ("review", || Review.command_template()),
      ("try_finish", || TryFinish.command_template()),
  ];
  ```
- 更新 [src/setup.rs:109-126](../../src/setup.rs#L109-L126) 上方注释:"the 6 Skill/Custom Command objects that get registered" (替换 "the 9 non-setup Command objects that get registered")
- [src/setup.rs:128](../../src/setup.rs#L128) 的 `("setup", || Setup.command_template())` 注释行——因 `Setup.command_template()` 已不存在,改为 `// Setup is terminal-only and not registered; see setup-excludes-self.md`

**Acceptance Criteria**:
- 4 个生产代码块删除(常量 / 工厂 / impl / import)
- COMMANDS 表 6 行
- `cargo check` 通过

---

### Step 3: 改 `src/gate.rs` — 删除 prompt 相关 4 块

**File**: [src/gate.rs](../../src/gate.rs)

**What to do**:
- 删除 [src/gate.rs:8-29](../../src/gate.rs#L8-L29) 的 `SKILL_PROMPT` / `COMMAND_PROMPT`
- 删除 [src/gate.rs:31-48](../../src/gate.rs#L31-L48) 的 `gate_skill()` / `gate_command()`
- 删除 [src/gate.rs:98-111](../../src/gate.rs#L98-L111) 的 `impl CommandMetadata for Gate`
- 从 [src/gate.rs:6](../../src/gate.rs#L6) 的 use 列表移除 `CommandMetadata`
- 保留 [src/gate.rs:65-73](../../src/gate.rs#L65-L73) 的 `New.gate(ctx)?` / `Plan.gate(ctx)?` / 等调用(零修改)

**Acceptance Criteria**:
- 4 个生产代码块删除
- `Gate::execute()` 行为零变化
- `cargo check` 通过

---

### Step 4: 改 `src/status.rs` — 删除 prompt 相关 4 块

**File**: [src/status.rs](../../src/status.rs)

**What to do**:
- 删除 [src/status.rs:6-25](../../src/status.rs#L6-L25) 的 `SKILL_PROMPT` / `COMMAND_PROMPT`
- 删除 [src/status.rs:27-44](../../src/status.rs#L27-L44) 的 `status_skill()` / `status_command()`
- 删除 [src/status.rs:63-76](../../src/status.rs#L63-L76) 的 `impl CommandMetadata for Status`
- 从 [src/status.rs:4](../../src/status.rs#L4) 的 use 列表移除 `CommandMetadata`
- 保留 [src/status.rs:50-52](../../src/status.rs#L50-L52) 的 `todo!()` 占位

**Acceptance Criteria**:
- 4 个生产代码块删除
- `Status::execute()` 仍为 `todo!()`
- `cargo check` 通过

---

### Step 5: 改 `src/archive.rs` — 删除 prompt 相关 4 块

**File**: [src/archive.rs](../../src/archive.rs)

**What to do**:
- 删除 [src/archive.rs:6-27](../../src/archive.rs#L6-L27) 的 `SKILL_PROMPT` / `COMMAND_PROMPT`
- 删除 [src/archive.rs:29-46](../../src/archive.rs#L29-L46) 的 `archive_skill()` / `archive_command()`
- 删除 [src/archive.rs:65-78](../../src/archive.rs#L65-L78) 的 `impl CommandMetadata for Archive`
- 从 [src/archive.rs:4](../../src/archive.rs#L4) 的 use 列表移除 `CommandMetadata`
- 保留 [src/archive.rs:51-53](../../src/archive.rs#L51-L53) 的 `todo!()` 占位

**Acceptance Criteria**:
- 4 个生产代码块删除
- `Archive::execute()` 仍为 `todo!()`
- `cargo check` 通过

---

### Step 6: 改 `tests/command_compile.rs` — 收窄到 6 个

**File**: [tests/command_compile.rs](../../tests/command_compile.rs)

**What to do**:
- 从 [tests/command_compile.rs:1-8](../../tests/command_compile.rs#L1-L8) 的 use 列表移除 `Archive` / `Gate` / `Setup` / `Status`(只保留 `Audit` / `Implement` / `New` / `Plan` / `Review` / `TryFinish` + `compile` / `compile_agent` / `Target` / `CommandSchema`)
- 重命名 [tests/command_compile.rs:79](../../tests/command_compile.rs#L79) 的 `all_10_commands_have_populated_schemas` → `all_6_commands_have_populated_schemas`
- 收窄 [tests/command_compile.rs:80-102](../../tests/command_compile.rs#L80-L102) 的断言:删除 `Setup` / `Gate` / `Status` / `Archive` 的 8 行(每个 2 行:skill + command),仅保留 6 个 `src/commands/` 对象(每个 2 行)
- 更新 [tests/command_compile.rs:106-124](../../tests/command_compile.rs#L106-L124) 的 `skill_and_command_schemas_share_metadata_but_differ_in_prompt` —— 已使用 `New`,**不变**
- 更新 [tests/command_compile.rs:127-139](../../tests/command_compile.rs#L127-L139) 的 `each_command_has_appropriate_tools`:
  - 删除 `Setup.skill_template().tools.contains(&"Bash")` / `&"Write"`(Setup 不再实现 trait)
  - 保留 `Implement` / `Review` / `Audit` / `Status` / `Gate` 断言中**后三个**(`Implement` / `Review` / `Audit`),删除 `Status` / `Gate` 的反向断言(`Status` / `Gate` 不再实现 trait)
- 更新 [tests/command_compile.rs:142-167](../../tests/command_compile.rs#L142-L167) 的 `all_commands_compile_to_valid_claude_markdown`:
  - schemas 数组从 10 个收窄为 6 个(去掉 Setup/Gate/Status/Archive)
  - 函数注释从 "every command" → "every Skill/Custom Command"
- 保留 [tests/command_compile.rs:170-184](../../tests/command_compile.rs#L170-L184) 的 `end_to_end_compile_agent_with_real_implement_schema` —— 已使用 `Implement`,**不变**

**Acceptance Criteria**:
- 测试函数名 / 断言数 / 期望集 同步收窄
- `cargo test` 通过

---

### Step 7: 改 `tests/setup_tool.rs` — 收窄到 6 个

**File**: [tests/setup_tool.rs](../../tests/setup_tool.rs)

**What to do**:
- [tests/setup_tool.rs:43](../../tests/setup_tool.rs#L43) `run_default_writes_both_targets`:`assert_eq!(report.written.len(), 18);` → `assert_eq!(report.written.len(), 12);`(6 commands × 2 targets);注释 `// 9 commands × 2 targets` → `// 6 commands × 2 targets`
- [tests/setup_tool.rs:44-45](../../tests/setup_tool.rs#L44-L45) `run_default_writes_both_targets`:`.claude/commands/gate.md` / `.opencode/command/gate.md` 替换为 `.claude/commands/new.md` / `.opencode/command/new.md`(因 gate.md 不再生成了;new.md 是 PipelinePhase 第一个)
- [tests/setup_tool.rs:52](../../tests/setup_tool.rs#L52) `claude_only_writes_claude_dir`:`assert_eq!(report.written.len(), 9);` → `assert_eq!(report.written.len(), 6);`
- [tests/setup_tool.rs:61](../../tests/setup_tool.rs#L61) `opencode_only_writes_opencode_dir`:`assert_eq!(report.written.len(), 9);` → `assert_eq!(report.written.len(), 6);`
- [tests/setup_tool.rs:70](../../tests/setup_tool.rs#L70) `written_files_have_valid_frontmatter`:`tmp.path().join(".claude/commands/gate.md")` → `tmp.path().join(".claude/commands/new.md")`
- [tests/setup_tool.rs:83](../../tests/setup_tool.rs#L83) `run_creates_deeply_nested_root`:`bogus.join(".claude/commands/gate.md")` → `bogus.join(".claude/commands/new.md")`
- [tests/setup_tool.rs:114](../../tests/setup_tool.rs#L114) `run_overwrites_existing_files_with_current_compile_output`:`tmp.path().join(".claude/commands/gate.md")` → `tmp.path().join(".claude/commands/new.md")`
- [tests/setup_tool.rs:128-153](../../tests/setup_tool.rs#L128-L153) `run_writes_all_nine_commands_per_target` 改名 `run_writes_all_six_commands_per_target`;期望集收窄为 6 个(去掉 gate/status/archive):
  ```rust
  let expected: BTreeSet<_> = [
      "new.md",
      "plan.md",
      "audit.md",
      "implement.md",
      "review.md",
      "try_finish.md",
  ]
  .iter()
  .map(|s| s.to_string())
  .collect();
  ```
- [tests/setup_tool.rs:161](../../tests/setup_tool.rs#L161) `run_opencode_files_use_singular_command_dir`:`tmp.path().join(".opencode/command/gate.md")` → `tmp.path().join(".opencode/command/new.md")`
- [tests/setup_tool.rs:170](../../tests/setup_tool.rs#L170) `run_claude_files_use_plural_commands_dir`:`tmp.path().join(".claude/commands/gate.md")` → `tmp.path().join(".claude/commands/new.md")`
- 保留 [tests/setup_tool.rs:175-198](../../tests/setup_tool.rs#L175-L198) 的 `setup_md_is_not_generated` —— **不变**

**Acceptance Criteria**:
- 数字断言全部 18 → 12,9 → 6
- 期望集从 9 收窄为 6
- 路径断言中 `gate.md` 全部替换为 `new.md`
- `cargo test` 通过

---

### Step 8: 全量编译验证

**What to do**:
```bash
cargo check
cargo build
cargo test
cargo clippy --all-targets
```

**Expected**:
- 零错误零警告
- 全部 30+ 测试通过(含 `command_compile` / `setup_tool` / `cli_help` / `state` / `utils`)

**Acceptance Criteria**:
- `cargo check` exit 0
- `cargo build` exit 0
- `cargo test` 所有测试通过
- `cargo clippy --all-targets` 零 lint

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| 删除 4 个子命令的 `CommandMetadata` impl 后,`New.gate(ctx)?` 等调用链断裂 | 中 | 低 | New/Plan/Audit/Implement/Review/TryFinish 仍实现 `CommandMetadata`;编译期保证 |
| `tests/command_compile.rs` 收窄到 6 个时漏改 import 导致编译错误 | 中 | 中 | use 列表显式删除 Archive/Gate/Status/Setup;编译期保证 |
| `COMMANDS` 表 6 行的顺序错乱影响输出确定性 | 低 | 低 | 保持 `New → Plan → Audit → Implement → Review → TryFinish` 与 `PipelinePhase` 顺序一致(见 [src/state.rs:103-114](../../src/state.rs#L103-L114)) |
| `gate.md` 路径断言仍存在但文件不再生成,导致集成测试失败 | 中 | 高 | Step 7 显式替换为 `new.md` |
| 用户已存在的 `.claude/commands/{gate,status,archive}.md` 不会被清理 | 中 | 高 | 遵循 idempotent 行为(与 setup-excludes-self.md 一致);不提供清理旗标;在 Step 7 的注释中提示"run() no longer regenerates these files" |
| 未来加新 Skill 容易忘记在 `COMMANDS` 表加行 | 低 | 中 | `run_writes_all_six_commands_per_target` 测试断言期望集;测试名固定 6 提醒数量 |
| `src/setup.rs` 的 `("setup", || Setup.command_template())` 注释行在 Step 2 中需替换为更新文案 | 低 | 低 | Step 2 显式说明 |

---

## Verification Steps

执行顺序:

1. **改 `src/traits.rs` 文档**(Step 1)— 跑 `cargo check`,确认 trait 编译
2. **改 4 个子命令文件**(Step 2-5)— 跑 `cargo check`,确认无错误
3. **改 `src/setup.rs` 的 `COMMANDS` 表**(Step 2)— 跑 `cargo build`,确认 0 错误
4. **改 `tests/command_compile.rs`**(Step 6)— 跑 `cargo test command_compile`,确认 6 个测试通过
5. **改 `tests/setup_tool.rs`**(Step 7)— 跑 `cargo test setup_tool`,确认所有测试通过
6. **全量验证**(Step 8):
   ```bash
   cargo check
   cargo build
   cargo test
   cargo clippy --all-targets
   ```
7. **手工验证**(临时目录):
   ```bash
   cd /tmp && mkdir test_refactor && cd test_refactor
   /Users/alex/Projects/workspace/ForceLoop/target/debug/forceloop setup
   ls .claude/commands/
   # 期望:6 个文件: new.md plan.md audit.md implement.md review.md try_finish.md
   # (无 setup.md, gate.md, status.md, archive.md)
   ls .opencode/command/
   # 期望:同上 6 个
   ```
8. 验证 `forceloop setup --tool claude` 终端子命令照常工作
9. 验证 `forceloop gate` 仍能调用 6 个 pipeline skill 的 `gate()`(由 cargo test `gate` 覆盖)
10. 提交

---

## Out of Scope(明确边界)

以下功能**不在本计划范围**:

1. **清理已存在的 `gate.md` / `status.md` / `archive.md` 旧文件** —— 用户迁移负担(一次性手动删除);与 setup-excludes-self.md 一致
2. **修改 `CommandMetadata` trait 方法签名** —— trait contract 稳定
3. **修改 6 个 `src/commands/` 对象的 impl 块** —— 它们已正确实现 trait,无需调整
4. **修改 `compile()` / `compile_agent()` / `CommandSchema`** —— 与本计划无关
5. **修改 `cli.rs` / `main.rs`** —— 4 个子命令的 CLI 形态不变
6. **修改 `state.rs` / `context.rs` / `errors.rs`** —— 与本计划无关
7. **添加 `--purge-stale-commands` 旗标** —— 独立计划
8. **修改 `Setup::execute()` / `Gate::execute()` 业务逻辑** —— 仅删除 trait impl,业务逻辑不动
9. **为 4 个子命令的 prompt 内容做"最终化"** —— 直接删除,不再保留
10. **重新引入 `should_register()` 钩子或拆 trait** —— Alternative A/B 已在 ADR 中明确拒绝

---

## Follow-ups

- 未来若需要清理旧生成的 `gate.md` / `status.md` / `archive.md`,加 `--purge` 旗标到 setup(独立计划)
- 若未来 trait contract 演化(如新增 `pre_gate()` 默认方法),可考虑给 `CommandMetadata` 加默认实现保持向后兼容
- 若未来新加 4 个顶层子命令类别(如 `forceloop logs`),需评估是否纳入 `COMMANDS` 表(默认不纳入;仅当业务需要时再加)
- 若未来加新 Skill,必须同步更新 `COMMANDS` 表 + `tests/setup_tool.rs` 的 `run_writes_all_six_commands_per_target` 期望集

---

## ADR (Architecture Decision Record)

### Decision
将 `CommandMetadata` trait 的实现者从 10 个 Command 对象收窄为 6 个(仅 `src/commands/` 下的 Skill/Custom Command)。4 个顶层子命令(Setup、Gate、Status、Archive)完全移除 `CommandMetadata` impl 及其支撑代码(`SKILL_PROMPT` / `COMMAND_PROMPT` 常量、`*_skill()` / `*_command()` 工厂函数、`CommandMetadata` 的 use 导入)。`src/setup.rs` 的 `COMMANDS` 静态表从 9 行减为 6 行。

### Drivers

1. **意图匹配**:4 个顶层子命令是**终端 CLI 子命令**,不是 Skill/Custom Command;它们**不应**有自己的 skill/command prompt
2. **trait surface 收窄**:trait 文档明确"Skill/Custom Command 用",调用方一眼看清契约
3. **死代码清理**:4 个子命令的 `skill_template()` / `command_template()` / `artifacts()` / `gate()` **从未被生产代码调用**——Setup 不调自己;Gate 调其他 6 个的 `gate()`;Status/Archive 是 `todo!()`
4. **COMMANDS 表与 setup-excludes-self.md 一致**:注册表 = "要作为 Skill 暴露的 Command 列表" = 6 个 pipeline skill,与 `PipelinePhase` 6 个一一对应
5. **最小变更**:仅删代码(不增);6 个 `src/commands/` 对象零修改;cli/main 零修改

### Alternatives considered

- **Alternative A**:保留单一 trait 但加 `should_register() -> bool` 钩子,让 4 个子命令显式声明"不注册"
  - **Why rejected**:过度工程化;trait 改动 → 6 个 impl 级联修改 → 违反"骨架优先的范围纪律";且当前没有任何调用方消费 `should_register()` 钩子
- **Alternative B**:把 4 个方法拆成两个 trait(`SkillCommand` 含 template,`CommandMetadata` 含 artifacts/gate)
  - **Why rejected**:拆分后 4 个子命令仍需实现 `CommandMetadata`(保留 `artifacts` / `gate`),仍是死代码;拆分了反而增加 trait 数量,违背 SRP
- **Alternative C**:完全删除 `CommandMetadata` trait,4 个方法直接挂在 `Executable` 上
  - **Why rejected**:破坏 trait 单一职责;且 `Executable` 已有 `execute()`,加 4 个元数据方法会让 trait 臃肿
- **Alternative D(本计划)**:保留 `CommandMetadata` trait 不变,**收窄实现者**(10 → 6),删除 4 个子命令的 impl
  - **Why chosen**:最小变更;trait contract 不动;6 个 `src/commands/` 对象零修改;死代码彻底清理;COMMANDS 表语义更清晰;与 setup-excludes-self.md 的设计哲学一致

### Why chosen (Alternative D)

- 4 个子命令文件各删 4 个块(常量 / 工厂 / impl / import)= 4×4 = 16 块删除,外加 setup.rs 的 COMMANDS 表收窄
- 测试断言机械收窄(`all_10` → `all_6`,`18` → `12`,`9` → `6`,期望集从 9 减为 6)
- trait contract 完全保留
- 6 个 `src/commands/` 对象零修改
- cli/main 零修改
- 与已有 `setup-excludes-self.md` 的设计哲学一致(Setup 不出现在注册表 → 4 个顶层子命令都不出现在注册表)

### Consequences

**正面**:
- `CommandMetadata` trait 的契约明确"Skill/Custom Command 用"——文档即规约
- 4 个子命令不再保留"实现了但永远不消费"的死代码层
- `COMMANDS` 表的语义收窄为"Pipeline Skill 列表"——与 `PipelinePhase` 6 个一一对应
- 写文件数从 18 减为 12,更准确反映"6 个有效 Skill"
- 收窄后任何误把"非 Skill 类型"塞进 `COMMANDS` 表的提交会被 `run_writes_all_six_commands_per_target` 测试捕获
- 与 `setup-excludes-self.md` 形成统一的"顶层子命令 vs Pipeline Skill"边界

**负面**:
- 旧用户可能已存在 `.claude/commands/{gate,status,archive}.md` 残留文件——不清理(idempotent),用户需手动删除一次
- 未来若某个顶层子命令**确实**需要注册为 Skill(如 Gate 提供可视化按钮),需重新加 impl——但当前需求未出现
- 测试文件 `tests/command_compile.rs` 的 8 个断言(`Setup` × 2 / `Gate` × 2 / `Status` × 2 / `Archive` × 2)被机械删除,需在新 `CommandMetadata` 实现者上做等价覆盖

### Follow-ups

- 未来若需要清理已存在的 `gate.md` / `status.md` / `archive.md`,加 `--purge` 旗标
- 未来加新 Skill 必须同步更新 `COMMANDS` 表 + `tests/setup_tool.rs` 期望集
- 若 trait contract 演化(如 `Setup` 需要重新实现 `CommandMetadata`),可同步加回 impl
- 未来若 4 个子命令中**某个**需要注册为 Skill(非全部),需独立计划评估

---

## 共审应用记录

本计划为 **Direct Mode**(用户已给出明确决策:仅 commands/ 目录下 6 个需定义 prompt),不触发 Architect / Critic 循环。Architect 评审要求与 Critic 评估标准已通过自审应用:

- **80%+ claims 引用 file:line**:是(Context / Design Decisions / Implementation Steps / Risks 表格 / Out of Scope / ADR 均带 file:line 引用)
- **90%+ criteria 可测试**:是(Definition of Done 16 项 + Verification Steps 10 项 + Implementation Steps 8 步均为可执行命令或断言)
- **替代方案公平探索**:是(ADR 列 4 个 rejected alternatives 并说明 reason)
- **风险 + 缓解**:是(7 行 risks 表格,含概率/影响/缓解)
- **边界清晰**:是(10 项 Out of Scope)
- **遵循 CLAUDE.md 项目编程规约**:
  - 改前理解意图(已读取 24 个相关文件,理解 10 个 Command 对象实现)
  - 改后确认无回归(cargo test 覆盖)
  - 根因治理(不 hack,直接删除死代码)
  - 不动 `Executable` / `Subcommand` / `cli` / `main`
  - 无新增依赖
