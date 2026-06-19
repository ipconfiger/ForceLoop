# Command Metadata Skeleton

## TL;DR

> **Quick Summary**: 为所有 10 个 Command 对象扩展 4 个声明性元数据 trait 方法（Skill 模版、Command 模版、生成产物文件列表、门控方法），仅构建骨架不实现具体逻辑。所有方法用空值/空集/`Ok(())` 作为占位，保持可编译可运行。
>
> **Deliverables**:
> - `src/traits.rs` 新增 `CommandMetadata` trait（4 个方法签名）
> - 4 个顶层子命令 + 6 个技能/自定义命令 共 10 个 struct 各添加 `impl CommandMetadata` 块
> - 占位值：空字符串 / 空切片 / `Ok(())`，明确表示"骨架阶段"
> - 通过 `cargo check` + `cargo build` 编译验证
>
> **Estimated Effort**: Small (单一 trait + 10 个 impl 块)
> **Parallel Execution**: YES — 所有 10 个 impl 块独立可并行
> **Critical Path**: traits.rs → 任一 struct impl → cargo check

---

## Context

### Background
ForceLoop CLI 框架骨架已搭建完毕（[`.sisyphus/plans/cli-framework.md`](../../.sisyphus/plans/cli-framework.md)）。所有 10 个 Command 对象目前仅实现 `Executable`（`execute()` 体为 `todo!()`），4 个顶层子命令额外实现 `Subcommand`（`name()` / `description()`）。

### Original Request（用户原话）
> 给所有的Command对象都要增加：
> 1: Skill 的模版
> 2: Command 的模版
> 3: 生成产物文件的列表
> 4: 门控方法（用于验证能否进入下一步）
> 同样的先构建骨架，不具体实现

### Current State Inventory
| Category | Count | Files | Current Traits |
|----------|-------|-------|----------------|
| 顶层子命令 | 4 | [setup.rs](../../src/setup.rs:1-21), [gate.rs](../../src/gate.rs:1-21), [status.rs](../../src/status.rs:1-21), [archive.rs](../../src/archive.rs:1-21) | `Executable` + `Subcommand` |
| 技能/自定义命令 | 6 | [new_cmd.rs](../../src/commands/new_cmd.rs:1-12), [plan.rs](../../src/commands/plan.rs:1-12), [audit.rs](../../src/commands/audit.rs:1-12), [implement.rs](../../src/commands/implement.rs:1-12), [review.rs](../../src/commands/review.rs:1-12), [try_finish.rs](../../src/commands/try_finish.rs:1-12) | `Executable` only |

### Why Now
当前 Command 对象只有行为（`execute()`）缺少元数据。Skill 模版、产物文件、门控方法是后续 6 步开发流程（new → plan → audit → implement → review → try_finish）的核心协调数据，必须在骨架阶段就纳入 trait 表面。

---

## Work Objectives

### Core Objective
为所有 10 个 Command 对象声明 4 个元数据方法，建立可扩展的 trait 表面，骨架阶段不实现具体业务逻辑。

### Concrete Deliverables
- `src/traits.rs` 新增 `CommandMetadata` trait（4 个方法签名）
- 10 个 struct 各添加 `impl CommandMetadata for X` 块（占位实现）
- 编译验证：`cargo check` 零错误零警告
- 构建验证：`cargo build` 成功

### Definition of Done
- [ ] `CommandMetadata` trait 编译通过，4 个方法签名清晰
- [ ] 10 个 Command struct 全部实现 `CommandMetadata`
- [ ] `cargo check` 零错误零警告
- [ ] `cargo build` 产出二进制
- [ ] 无 panic（gate() 返回 `Ok(())`，不调用 `todo!()`）

### Must Have
- `CommandMetadata` trait 定义在 `src/traits.rs`
- 4 个方法：`skill_template`, `command_template`, `artifacts`, `gate`
- 所有 10 个 struct 各实现一次
- 方法签名稳定（占位实现，逻辑留待后续）

### Must NOT Have (Guardrails)
- **零业务逻辑**: 所有元数据方法体只有空值（`""`/`&[]`/`Ok(())`）
- **零新依赖**: 不添加 `handlebars`/`tera`/`askama` 等模板引擎
- **零文件 I/O**: `gate()` 不读写任何文件
- **零 `todo!()`**: 元数据方法不应 panic（与 `execute()` 区分）
- **零 trait 改动**: 不修改 `Executable` 或 `Subcommand` 的现有签名
- **零测试新增**: 不为占位方法编写测试（避免假阳性）
- **零 CLI 暴露**: 不修改 `cli.rs` / `main.rs`（元数据是内部协调数据）

---

## Design Decisions

### Decision 1: 新 trait vs 扩展 `Executable`
**决策**：新增 `CommandMetadata` trait，不扩展 `Executable`。

**理由**：
- 单一职责：`Executable` 管行为，`CommandMetadata` 管声明性元数据
- 可选性：未来可能有不带元数据的纯执行器（如 helper）
- 测试性：可单独验证元数据 trait 而无需触发 `execute()`
- 解耦：避免在 `Executable` 上加方法时所有实现者被迫更新

### Decision 2: 单个 `gate()` 方法 vs `pre_gate()` + `post_gate()`
**决策**：单方法 `gate()`，按用户原话。

**理由**：
- 用户原话"门控方法"为单数
- 用途"验证能否进入下一步"是 post-execution 语义
- 单一入口更易理解和扩展
- 如需 pre-gate，可后续追加为 trait 的默认方法（向后兼容）

### Decision 3: 占位值策略
**决策**：使用语义性空值（`""`/`&[]`/`Ok(())`），不用 `todo!()`，不用描述性占位字符串。

**理由**：
- `todo!()` 会 panic，无法被静态调用（如 `skill_template()` 在文档生成时被调用）
- 描述性字符串（如 `"TODO: skill template for New"`）会污染未来真实数据
- 空值诚实表达"骨架阶段"且不引入歧义
- 后续替换为真实值时不会留下误导性痕迹

### Decision 4: 返回类型选择
- `skill_template()` → `&'static str`（短字符串，无需堆分配）
- `command_template()` → `&'static str`（同上）
- `artifacts()` → `&'static str`（路径列表，骨架阶段先用字符串占位，复杂类型后续再升级）
- `gate(ctx)` → `Result<()>`（与 `execute()` 一致）

---

## Implementation Steps

### Step 1: 扩展 `src/traits.rs`

**File**: [src/traits.rs](../../src/traits.rs:1-13)

**What to do**:
在 `Subcommand` trait 之后追加新 trait：

```rust
/// Trait for declarative metadata shared by all Command objects.
/// Provides skill/command templates, artifact file lists, and gating logic.
pub trait CommandMetadata {
    /// Returns the Skill template that defines this command's workflow steps.
    fn skill_template(&self) -> &'static str;

    /// Returns the Command template that defines how to invoke this command.
    fn command_template(&self) -> &'static str;

    /// Returns the list of artifact files this command produces.
    fn artifacts(&self) -> &[&'static str];

    /// Gate method: verifies whether the next step in the pipeline can proceed.
    /// Skeleton implementation returns Ok(()).
    fn gate(&self, ctx: &Context) -> Result<()>;
}
```

**Must NOT do**:
- 不为 trait 方法提供默认实现（保持 4 个方法都为 required）
- 不引入泛型参数 / 关联类型
- 不改 `Executable` / `Subcommand` 现有签名
- 不为 `gate()` 添加异步

**References**:
- 现有 trait 定义 [src/traits.rs:1-13](../../src/traits.rs#L1-L13)
- `Context` 定义 [src/context.rs:1-15](../../src/context.rs#L1-L15)
- `Result` 定义 [src/errors.rs:18](../../src/errors.rs#L18)

**Acceptance Criteria**:
- `cargo check` 零错误
- 4 个方法签名符合上述定义
- 文档注释简洁

**QA Scenarios**:
```
Scenario: traits.rs compiles
  Tool: Bash
  Steps:
    1. Run `cargo check`
  Expected: exit 0
  Evidence: .sisyphus/evidence/metadata-step1-traits.txt
```

---

### Step 2-5: 顶层子命令 4 个 impl 块（可并行）

**Files**:
- [src/setup.rs](../../src/setup.rs:1-21)
- [src/gate.rs](../../src/gate.rs:1-21)
- [src/status.rs](../../src/status.rs:1-21)
- [src/archive.rs](../../src/archive.rs:1-21)

**What to do**：
每个文件在 `impl Subcommand` 之后追加：

```rust
use crate::traits::CommandMetadata;

impl CommandMetadata for Setup {  // 或 Gate / Status / Archive
    fn skill_template(&self) -> &'static str { "" }
    fn command_template(&self) -> &'static str { "" }
    fn artifacts(&self) -> &[&'static str] { &[] }
    fn gate(&self, _ctx: &Context) -> Result<()> { Ok(()) }
}
```

**Must NOT do**:
- 不调用 `super::*` 或重导出其他 trait
- 不添加 struct 字段
- 不修改 `execute()` 体

**References**:
- Step 1 中定义的 `CommandMetadata` trait
- 每个 struct 已有 imports（`Context`, `Result`, `Executable`, `Subcommand`）

**Acceptance Criteria**:
- 4 个文件均通过 `cargo check`
- 每个文件多出 1 个 `impl CommandMetadata` 块

---

### Step 6-11: 技能/自定义命令 6 个 impl 块（可并行）

**Files**:
- [src/commands/new_cmd.rs](../../src/commands/new_cmd.rs:1-12)
- [src/commands/plan.rs](../../src/commands/plan.rs:1-12)
- [src/commands/audit.rs](../../src/commands/audit.rs:1-12)
- [src/commands/implement.rs](../../src/commands/implement.rs:1-12)
- [src/commands/review.rs](../../src/commands/review.rs:1-12)
- [src/commands/try_finish.rs](../../src/commands/try_finish.rs:1-12)

**What to do**：
每个文件在 `impl Executable` 之后追加：

```rust
use crate::traits::CommandMetadata;

impl CommandMetadata for New {  // 或 Plan / Audit / Implement / Review / TryFinish
    fn skill_template(&self) -> &'static str { "" }
    fn command_template(&self) -> &'static str { "" }
    fn artifacts(&self) -> &[&'static str] { &[] }
    fn gate(&self, _ctx: &Context) -> Result<()> { Ok(()) }
}
```

**Must NOT do**: 同 Step 2-5

**Acceptance Criteria**:
- 6 个文件均通过 `cargo check`
- 每个文件多出 1 个 `impl CommandMetadata` 块

---

### Step 12: 全量编译验证

**What to do**:
```bash
cargo check
cargo build
```

**Expected**:
- `cargo check` exit 0，零错误零警告
- `cargo build` 成功产出 `target/debug/forceloop`
- 二进制仍可运行 `--help`

**Acceptance Criteria**:
- 所有 10 个 Command struct 实现 `CommandMetadata`
- 编译零警告（特别注意未使用方法的警告，需用 `_ctx` 前缀抑制）

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| `gate(ctx)` 参数未使用触发 `unused_variables` 警告 | 中（破坏零警告目标） | 高 | 使用 `_ctx` 前缀 |
| `artifacts()` 返回 `&[]` 需要 `'static` 生命周期匹配 | 低 | 低 | `&'static str` 切片字面量天然满足 |
| 未来替换为真实模板时需要重新设计 trait | 中 | 中 | 当前 trait 保持简单可扩展；如需复杂类型可后续加关联类型 |
| 6 个 struct impl 块复制粘贴引入不一致 | 低 | 中 | 文档化模板占位值；code review 检查 |
| `CommandMetadata` 与 `Subcommand` 概念重叠 | 低 | 低 | 明确分工：`Subcommand` 是 CLI 元数据（name/description），`CommandMetadata` 是执行元数据（模板/产物/门控） |

---

## Verification Strategy

### Build Verification
```bash
cargo check                    # 零错误零警告
cargo build                    # exit 0
./target/debug/forceloop --help  # 仍显示 4 个子命令
```

### Structural Verification（编译期）
- 所有 10 个 struct 必须实现 `CommandMetadata`，否则 `cargo check` 失败
- 通过以下方式自动验证（在 main.rs 中加入或新建 test）：
  ```rust
  fn _assert_command_metadata<T: CommandMetadata>(_: &T) {}
  // 在 main 或测试中调用 _assert_command_metadata(&Setup); 等
  ```
  **注意**：本计划**不要求**新增此断言（属于"零测试新增"约束）；如需可后续追加

### Manual Inspection
- 每个 impl 块用 4 个空值方法体
- 无 `todo!()` 出现在元数据方法中

---

## Commit Strategy

### Single Commit（所有 11 个文件）
```
feat(core): add CommandMetadata trait skeleton to all Command objects

- Add CommandMetadata trait with 4 methods: skill_template, command_template, artifacts, gate
- Implement trait for 4 top-level subcommands (Setup, Gate, Status, Archive)
- Implement trait for 6 skill/custom commands (New, Plan, Audit, Implement, Review, TryFinish)
- All implementations are skeleton placeholders (empty values, Ok(()))
- No business logic; real values to be filled in subsequent tasks

Files: src/traits.rs, src/setup.rs, src/gate.rs, src/status.rs, src/archive.rs,
       src/commands/{new_cmd,plan,audit,implement,review,try_finish}.rs
```

**Pre-commit**: `cargo check` (zero errors/warnings) + `cargo build` (success)

---

## Final Checklist

- [ ] `CommandMetadata` trait 编译通过
- [ ] 4 个方法签名符合定义
- [ ] 10 个 Command struct 全部实现 `CommandMetadata`
- [ ] `cargo check` 零错误零警告
- [ ] `cargo build` 成功
- [ ] `./target/debug/forceloop --help` 仍正常
- [ ] 无 `todo!()` 出现在元数据方法中
- [ ] 所有占位值为空值（`""`/`&[]`/`Ok(())`）
- [ ] 未修改 `Executable` / `Subcommand` 现有签名
- [ ] 未添加新依赖

---

## Follow-ups (Out of Scope)

以下内容**不在本计划范围**，留待后续任务：

1. **真实模板内容**: 替换 `""` 为实际的 Skill/Command 模板字符串或文件路径
2. **模板引擎集成**: 引入 `handlebars`/`tera` 处理 `{{var}}` 占位符
3. **pre_gate()**: 如需在 `execute()` 之前也做门控，可作为 trait 默认方法追加
4. **artifacts 路径解析**: 当前 `&[&'static str]` 是简化版，未来可能升级为 `Vec<PathBuf>` + glob 匹配
5. **gate 联动**: 真正的门控实现需要读取上一个命令的产物文件、验证后写入状态
6. **CLI 暴露元数据**: 当前元数据仅内部使用；未来 `status` 子命令可读取并展示
7. **State 持久化**: 门控结果/产物路径需要持久化到状态文件（`Context` 字段扩展）
