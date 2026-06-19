# ForceLoop `setup` 子命令自我排除（不注册自身为 Skill / Slash Command）

## TL;DR

> **Quick Summary**: `Setup` 是终端命令，仅用于项目初始化；不应作为可调用的 Skill / Slash Command 出现在 `.claude/commands/setup.md` 与 `.opencode/command/setup.md`。从 `src/setup.rs` 的 `COMMANDS` 静态表中删除 `("setup", ...)` 一行；同步更新 8 个引用 `setup.md` 的测试断言、`commands_table_has_ten_entries` 常量、以及 `run()` 写文件数（10→9）。
>
> **Deliverables**:
> - `src/setup.rs` — `COMMANDS` 表移除 `Setup` 条目；`commands_table_has_ten_entries` 改为 9；`COMMANDS` 表注释从 "10 Command objects" 改为 "9 Command objects"
> - `tests/setup_tool.rs` — 8 处 `setup.md` / `10` 数字 / `20` 数字替换为非 setup 命令（如 `gate.md` / `9` / `18`）；`run_writes_all_ten_commands_per_target` 重命名为 `run_writes_all_nine_commands_per_target`，期望集合移除 `setup.md`；新增 `setup_md_is_not_generated` 反向断言
> - `src/setup.rs` 中其他 `tests` 模块 — `commands_table_has_ten_entries` 改名+改值
> - **`Setup::command_template()` / `Setup::skill_template()` impl 不动** — `CommandMetadata` trait 要求实现，移除会破 trait
>
> **Estimated Effort**: Small (1 文件改生产代码, 1 文件改测试, ~20 行生产 + ~25 行测试)
> **Parallel Execution**: NO（顺序：改 `COMMANDS` 表 → 改单元测试 → 改集成测试 → 跑 `cargo test`）
> **Critical Path**: setup.rs:117 → setup.rs:240-242 (unit test) → tests/setup_tool.rs

---

## Context

### Background

- [src/setup.rs:116-127](src/setup.rs#L116-L127) 的 `COMMANDS` 表枚举所有 10 个 `Command` 对象，遍历后调用 `compile()` 写文件
- [src/setup.rs:136-149](src/setup.rs#L136-L149) 的 `pub fn run()` 是 `setup` 的纯业务逻辑，被 [`Setup::execute()`](src/setup.rs#L161-L172) 调用
- [tests/command_compile.rs:79-103](tests/command_compile.rs#L79-L103) `all_10_commands_have_populated_schemas` 验证 10 个 `Command` 都有非空 schema（**这里 "10" 指的是 10 个 `Command` impl，不是 COMMANDS 表长度** —— 测试在改后仍然有效）
- [tests/setup_tool.rs](tests/setup_tool.rs) 中有 8 个测试断言硬编码 `setup.md` 路径或 `20`/`10` 写文件数

### Original Request（用户原话）

> setup子命令不需要注册skill和斜杠命令，因为这个命令只在终端里执行，用来初始化项目

### 用户已确认的设计决策

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **`Setup` 是否实现 `CommandMetadata`** | **保留** | trait 要求 `skill_template()` / `command_template()`，移除会破 trait impl |
| **`COMMANDS` 表是否包含 `Setup`** | **移除** | `Setup` 不应出现在 `.claude/commands/setup.md` 与 `.opencode/command/setup.md` |
| **`Setup::command_template()` 是否仍被使用** | **仅在测试中使用**（`tests/command_compile.rs`） | 不消费于生产路径；trait 要求保留 |
| **`setup_skill()` / `setup_command()` 内部函数** | **保留** | 被 `Setup::command_template()` / `skill_template()` 引用 |
| **`SKILL_PROMPT` / `COMMAND_PROMPT` 文案** | **不修改** | 文案说"Generate platform-native command files from `CommandMetadata`"，未列具体名字，无须改 |

### Why Now

- 当前 `forceloop setup` 执行后会在 `.claude/commands/setup.md` 写入一个 "setup" 斜杠命令，但用户永远不会在 Claude Code / OpenCode 中调用它（项目初始化只能跑一次）
- 这是"无意识注册"产生的 dead surface：Skill 面板里出现一个永远不该被点击的条目
- 修复后 `run()` 写文件数从 20 变 18，更准确地反映"9 个有效 Command"的数量
- 修复不影响 4 个顶层 subcommand 的 CLI 形态（`forceloop setup` 终端命令照常工作）

---

## Work Objectives

### Core Objective

让 `Setup` 不在 `.claude/commands/` 与 `.opencode/command/` 中产生 `setup.md` 文件，但仍作为可执行的终端子命令（`forceloop setup`）正常工作。

### Concrete Deliverables

1. **`src/setup.rs`** (改) — `COMMANDS` 表移除 `("setup", ...)`；`commands_table_has_ten_entries` 改名+改值
2. **`tests/setup_tool.rs`** (改) — 8 处 `setup.md` / `20` / `10` 数字替换
3. **`tests/setup_tool.rs`** (增) — 新增 `setup_md_is_not_generated` 反向断言
4. **不动**：`CommandMetadata` trait / 10 个 `Command` impl / `SKILL_PROMPT` / `COMMAND_PROMPT` / `cli.rs` / `main.rs` / `compiler.rs`

### Definition of Done

- [ ] `src/setup.rs:116-127` 的 `COMMANDS` 表不再包含 `Setup` 条目（9 条目）
- [ ] `src/setup.rs` 中 `COMMANDS` 表上方的注释从 "10 Command objects" 改为 "9 Command objects"
- [ ] `src/setup.rs:240-242` 的单元测试改名 `commands_table_has_nine_entries`（或保留 `commands_table_has_ten_entries` 名字但改期望为 9 —— 选 **改名** 更诚实）
- [ ] `tests/setup_tool.rs:43` `run_default_writes_both_targets` 中 `20` → `18`；移除两处 `setup.md` 路径断言，改为 `gate.md`（或任意其他 9 个命令之一）
- [ ] `tests/setup_tool.rs:52` `claude_only_writes_claude_dir` 中 `10` → `9`；`new.md` 断言保留
- [ ] `tests/setup_tool.rs:61` `opencode_only_writes_opencode_dir` 中 `10` → `9`；`new.md` 断言保留
- [ ] `tests/setup_tool.rs:70` `written_files_have_valid_frontmatter` 中 `setup.md` → `gate.md`
- [ ] `tests/setup_tool.rs:83` `run_creates_deeply_nested_root` 中 `setup.md` → `gate.md`
- [ ] `tests/setup_tool.rs:114` `run_overwrites_existing_files_with_current_compile_output` 中 `setup.md` → `gate.md`
- [ ] `tests/setup_tool.rs:128-153` `run_writes_all_ten_commands_per_target` 改名为 `run_writes_all_nine_commands_per_target`；期望集合移除 `"setup.md"`
- [ ] `tests/setup_tool.rs:161` `run_opencode_files_use_singular_command_dir` 中 `setup.md` → `gate.md`
- [ ] `tests/setup_tool.rs:170` `run_claude_files_use_plural_commands_dir` 中 `setup.md` → `gate.md`
- [ ] `tests/setup_tool.rs` 新增 `setup_md_is_not_generated` 测试：双目标写入后，断言 `.claude/commands/setup.md` **不存在** 且 `.opencode/command/setup.md` **不存在**（这是本计划的核心回归测试）
- [ ] `cargo check` / `cargo test` / `cargo clippy --all-targets` 全绿
- [ ] 已提交

### Must Have

- 最小变更：仅修改必要代码与测试；不动 `CommandMetadata` trait、不动 10 个 Command impl 的 `command_template()` / `skill_template()`（trait 要求保留）
- 测试代码允许 `unwrap()`
- 业务逻辑禁 Mock：使用 `tempfile::TempDir` 注入路径
- TDD：先写新断言（`setup_md_is_not_generated`），再改生产代码；同步更新旧测试

### Must NOT Have (Guardrails)

- **不**删除 `Setup::command_template()` / `Setup::skill_template()` impl（trait 要求）
- **不**删除 `setup_skill()` / `setup_command()` 内部函数（被 `Setup` impl 引用）
- **不**删除 `SKILL_PROMPT` / `COMMAND_PROMPT` 常量（被 `setup_skill()` / `setup_command()` 引用）
- **不**修改 `SKILL_PROMPT` / `COMMAND_PROMPT` 文案（未提及 `setup` 自身）
- **不**改 `cli.rs` / `main.rs`（`Setup` 终端子命令照常工作）
- **不**改 `compiler.rs` / `context.rs` / `Context::with_targets` 等无关模块
- **不**清理已存在的 `.claude/commands/setup.md`（用户可能从旧版 setup 残留此文件 —— 不在本计划范围；可作为未来 `--force` flag 的副作用）

---

## Architecture Decisions

### 数据流（修改后）

```
$ forceloop setup --tool claude
       │
       ▼
[cli::Cli] ── Parse ──▶ Commands::Setup { tool: vec![Tool::Claude] }
       │
       ▼
[main.rs:18] ── map(Target::from) ──▶ Context::with_targets(vec![Target::Claude])
       │
       ▼
[Setup::execute(&ctx)] ── ctx.targets.is_empty() ? default : ctx.targets
       │
       ▼
[Setup::run(&targets, &current_dir())]
       │
       ├── for target in targets:
       │       for cmd in [Gate, Status, Archive, New, Plan, Audit, Implement, Review, TryFinish]:  // ← Setup 移除
       │           let path = target_dir(root, target).join(format!("{}.md", cmd.name))
       │           let body = compile(cmd.command_template(), target)?
       │           fs::create_dir_all(path.parent())?; fs::write(path, body)?;
       │
       ▼
[SetupReport { written: Vec<PathBuf> }]   // 18 路径（9 命令 × 2 目标）而非 20
```

### 关键设计选择

#### 1. `CommandMetadata` trait 保持不变

- trait 要求所有 10 个 Command 实现 `command_template()` / `skill_template()`，**不能因为 `Setup` 不再被注册而删除其 impl**
- `Setup::command_template()` / `Setup::skill_template()` 仍在 `tests/command_compile.rs:79-103,127-139,142-167` 中被验证有非空 schema
- 移除 impl 会破坏 trait contract；保留 impl 仅在测试路径被使用，无运行时副作用

#### 2. 移除位置：仅在 `COMMANDS` 静态表中

- `COMMANDS` 表是 setup 业务的"输入列表"；移除 `Setup` 行 = 让 `run()` 不再生成 `setup.md`
- 这是单一职责的修复：traversal 数据与 trait 契约完全解耦

#### 3. 不修改 `SKILL_PROMPT` / `COMMAND_PROMPT`

- 文案描述"按 `CommandMetadata` 生成 platform-native command files" —— 未列具体命令名
- 即使修改（如加上"9 个非 setup 命令"）也是无信息量且脆弱 —— 未来加新命令时要改文案
- 保持文案通用，命令数量由 `COMMANDS` 表 + 单元测试联合验证

#### 4. 不清理旧 `setup.md`

- 旧版 setup 已写入的 `.claude/commands/setup.md` / `.opencode/command/setup.md` 不会被本计划删除
- 这是 idempotent 行为：再跑 `forceloop setup` 不会动这些文件（因为 COMMANDS 表里没有 Setup 了）
- 用户的迁移负担：手动删除一次；本计划不提供 `--purge` 旗标

---

## Implementation Steps

### Step 1: 改 `src/setup.rs:116-127` — 移除 Setup 条目 + 改注释 + 改单元测试

**测试先**（在 `src/setup.rs` 的 `#[cfg(test)] mod tests` 修改）：

```rust
// 旧测试（行 238-242）：
#[test]
fn commands_table_has_ten_entries() {
    assert_eq!(COMMANDS.len(), 10);
}

// 新测试：
#[test]
fn commands_table_has_nine_entries() {
    // Setup is intentionally excluded from the COMMANDS table —
    // it is a terminal-only subcommand, not a registered skill/slash
    // command. See `.omc/plans/setup-excludes-self.md` for rationale.
    assert_eq!(COMMANDS.len(), 9);
}
```

**生产代码改动**（`src/setup.rs:107-127`）：

```rust
/// Static table type: (command_name, command_template factory).
///
/// `CommandSchema` is `Copy`, so the factory is zero-cost. Factored
/// into a type alias to keep the `COMMANDS` literal readable.
type CommandEntry = (&'static str, fn() -> CommandSchema);

/// Static table of the 9 non-setup Command objects that get registered
/// as platform-native slash command / Skill files.
///
/// `Setup` is intentionally excluded: it is a terminal-only subcommand
/// for project initialization, not a runtime-invokable skill. Including
/// it would write `setup.md` to `.claude/commands/` and
/// `.opencode/command/`, surfacing an entry in the IDE's command
/// palette that should never be clicked (project init is a one-shot
/// terminal action).
///
/// This table is the single source of truth for which Commands get
/// registered. Adding a new Command (other than Setup) requires:
///   1. Implement `CommandMetadata` for it
///   2. Add a row here
/// If you add a new row, `run_writes_all_nine_commands_per_target` in
/// `tests/setup_tool.rs` will fail until you update its expected set —
/// this is intentional, the test pins the contract.
const COMMANDS: &[CommandEntry] = &[
    // ("setup", || Setup.command_template()),  // intentionally omitted
    ("gate", || Gate.command_template()),
    ("status", || Status.command_template()),
    ("archive", || Archive.command_template()),
    ("new", || New.command_template()),
    ("plan", || Plan.command_template()),
    ("audit", || Audit.command_template()),
    ("implement", || Implement.command_template()),
    ("review", || Review.command_template()),
    ("try_finish", || TryFinish.command_template()),
];
```

### Step 2: 改 `tests/setup_tool.rs` — 8 处 `setup.md` / 数字替换

**测试先**：在 `tests/setup_tool.rs` 末尾新增核心反向断言：

```rust
#[test]
fn setup_md_is_not_generated() {
    // Regression test for the requirement that `Setup` is a terminal-only
    // subcommand and must not appear in the registered commands / skills
    // of either platform. See `.omc/plans/setup-excludes-self.md`.
    let tmp = TempDir::new().unwrap();
    let report = run(&default_targets(), tmp.path()).unwrap();
    assert!(
        !report.written.iter().any(|p| p.file_name().unwrap() == "setup.md"),
        "run() must not produce a `setup.md` file; got: {:?}",
        report.written
    );
    assert!(
        !tmp.path().join(".claude/commands/setup.md").exists(),
        ".claude/commands/setup.md must not exist"
    );
    assert!(
        !tmp.path().join(".opencode/command/setup.md").exists(),
        ".opencode/command/setup.md must not exist"
    );
}
```

**8 处替换**：

| 位置 | 旧值 | 新值 |
|------|------|------|
| [tests/setup_tool.rs:43](tests/setup_tool.rs#L43) | `assert_eq!(report.written.len(), 20); // 10 commands × 2 targets` | `assert_eq!(report.written.len(), 18); // 9 commands × 2 targets` |
| [tests/setup_tool.rs:44-45](tests/setup_tool.rs#L44-L45) | `.claude/commands/setup.md` / `.opencode/command/setup.md` 两行断言 | 改为 `.claude/commands/gate.md` / `.opencode/command/gate.md` |
| [tests/setup_tool.rs:52](tests/setup_tool.rs#L52) | `assert_eq!(report.written.len(), 10);` | `assert_eq!(report.written.len(), 9);` |
| [tests/setup_tool.rs:61](tests/setup_tool.rs#L61) | `assert_eq!(report.written.len(), 10);` | `assert_eq!(report.written.len(), 9);` |
| [tests/setup_tool.rs:70](tests/setup_tool.rs#L70) | `tmp.path().join(".claude/commands/setup.md")` | `tmp.path().join(".claude/commands/gate.md")` |
| [tests/setup_tool.rs:83](tests/setup_tool.rs#L83) | `bogus.join(".claude/commands/setup.md")` | `bogus.join(".claude/commands/gate.md")` |
| [tests/setup_tool.rs:114](tests/setup_tool.rs#L114) | `tmp.path().join(".claude/commands/setup.md")` | `tmp.path().join(".claude/commands/gate.md")` |
| [tests/setup_tool.rs:128-153](tests/setup_tool.rs#L128-L153) | `run_writes_all_ten_commands_per_target` 测试 + 期望集含 `setup.md` | 改名 `run_writes_all_nine_commands_per_target`；期望集移除 `setup.md` |
| [tests/setup_tool.rs:161](tests/setup_tool.rs#L161) | `tmp.path().join(".opencode/command/setup.md")` | `tmp.path().join(".opencode/command/gate.md")` |
| [tests/setup_tool.rs:170](tests/setup_tool.rs#L170) | `tmp.path().join(".claude/commands/setup.md")` | `tmp.path().join(".claude/commands/gate.md")` |

**为什么选 `gate.md` 作为替代示例**：
- `gate` 是第一个非 setup 的顶层子命令，命名稳定
- 与 `setup` 都是 4 个 Subcommand 之一（与 New/Plan/Audit/Implement/Review/TryFinish 区分），替换的语义对比更清晰

### Step 3: 验证

```bash
cargo check
cargo test
cargo clippy --all-targets
# 手工验证：
cargo run -- setup
ls .claude/commands/      # 应看到 9 个文件，无 setup.md
ls .opencode/command/     # 应看到 9 个文件，无 setup.md
```

---

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| `tests/command_compile.rs:79-103` `all_10_commands_have_populated_schemas` 仍引用 `Setup.skill_template()` / `Setup.command_template()` —— 这是检查 10 个 Command impl 都有非空 schema，与 COMMANDS 表无关 | 测试不动；impl 保留（trait 要求） |
| `tests/command_compile.rs:127-139,142-167` 同样使用 `Setup.skill_template()` —— 同样不消费于生产路径，仅作 trait impl 验证 | 测试不动 |
| 用户已存在的 `.claude/commands/setup.md` / `.opencode/command/setup.md` 不会被清理（idempotent setup） | 本计划不提供清理机制；在 Follow-ups 中记录 |
| 未来加新 Command 容易忘记在 `COMMANDS` 表加行 | `run_writes_all_nine_commands_per_target` 测试断言期望集合；若表加行但测试未更新，测试会失败 |
| 未来加新 Command 容易忘记在 `tests/setup_tool.rs` 的期望集加名字 | 同上；测试名"all nine"提醒固定数量 |
| 误删 `Setup::command_template()` impl（因"反正没人用"） | 注释明确说明 "trait requires it" + 单元测试仍消费之 |
| `SKILL_PROMPT` 描述中 "Generate platform-native command files from `CommandMetadata`" 仍写"from `CommandMetadata`"，可能让人误以为 Setup 自身也生成 | 文案保持通用；本计划不修改 |
| 旧 `SKILL_PROMPT` 步骤 1 提到 "Create `.forceloop/{skills,commands,hooks,archive}/` directory tree" —— 这是 `Setup` 在项目里创建的状态目录树，与"注册 Skill 文件"无关 | 不在本计划范围；本计划不修改 `SKILL_PROMPT` |

---

## Verification Steps

执行顺序：

1. **写新测试**：在 `tests/setup_tool.rs` 加 `setup_md_is_not_generated` —— **先看到它失败**（红）
2. **改 `src/setup.rs:117` 移除 `Setup` 条目** —— 跑 `cargo test setup_md_is_not_generated` 应通过（绿）
3. **改 8 处旧测试断言**（替换 `setup.md` → `gate.md`，`20` → `18`，`10` → `9`，重命名 `run_writes_all_ten_commands_per_target` → `run_writes_all_nine_commands_per_target`）
4. `cargo check` — 编译期断言
5. `cargo test` — 全部 30+ 测试通过（含 `command_compile` / `setup_tool` / `cli_help`）
6. `cargo clippy --all-targets` — 零 lint
7. **手工验证**（在临时目录跑）：
   ```bash
   cd /tmp && mkdir test_setup_exclusion && cd test_setup_exclusion
   /Users/alex/Projects/workspace/ForceLoop/target/debug/forceloop setup
   ls .claude/commands/
   # 期望：archive.md audit.md gate.md implement.md new.md plan.md review.md status.md try_finish.md
   # （9 个文件，无 setup.md）
   ls .opencode/command/
   # 期望：同上 9 个
   ```
8. 验证 `forceloop setup --tool claude` 终端子命令照常工作
9. 提交

---

## Out of Scope（明确边界）

以下功能**不在本计划范围**：

1. **清理已存在的 `setup.md` 旧文件** —— 用户迁移负担（手动一次）；后续可加 `--purge` 旗标（独立计划）
2. **修改 `SKILL_PROMPT` / `COMMAND_PROMPT`** —— 文案通用，命令数量由 `COMMANDS` 表 + 测试验证
3. **修改 `CommandMetadata` trait** —— 不动 trait contract
4. **移除 `Setup::command_template()` / `Setup::skill_template()` impl** —— trait 要求保留
5. **改写 `run()` 为接受 `&[&dyn CommandMetadata]` 动态分发** —— 项目原则"零新增运行时依赖"；当前静态表是 trade-off 选择（参见 [.omc/plans/setup-tool-flag.md:624-632](.omc/plans/setup-tool-flag.md#L624-L632) 的"为什么选静态表"）
6. **生成 `.forceloop/{skills,commands,hooks,archive}/` 目录树**（`SKILL_PROMPT` 步骤 1）—— 独立计划
7. **写 `.forceloop/state.json` 内容**（`SKILL_PROMPT` 步骤 2）—— 独立计划
8. **安装 git hooks**（`SKILL_PROMPT` 步骤 4）—— 独立计划
9. **生成 `.opencode/agent/<name>.md`** —— 独立计划
10. **打印 `SetupReport` 摘要到 stdout** —— Open Question，独立计划

---

## Open Questions（需用户后续决策，不阻塞本计划）

1. 是否需要为旧用户提供 `--purge-stale-commands` 旗标清理之前生成的 `setup.md`？（当前选择：**不需要**；最小变更优先）
2. 是否需要在 `Setup::command_template()` 内部加注释说明 "this is no longer used at runtime; kept for trait compliance and testing"？（当前选择：**不加**；impl 本身已通过 `#[cfg(test)]` 之外的位置被引用，注释会变成噪音）
3. 未来加新 Command 时，是否要把期望集合提取为 `pub const EXPECTED_COMMANDS: &[&str]` 常量（与 `COMMANDS` 表共享）？（当前选择：**不提取**；保持 inline，未来加新命令时手动同步两处 —— 与现有 9 个 command 的 inline 期望风格一致）

---

## ADR (Architecture Decision Record)

### Decision

从 `src/setup.rs` 的 `COMMANDS` 静态表中移除 `("setup", ...)` 条目，使 `Setup::run()` 不再为 `Setup` 自身生成 platform-native 命令文件（`.claude/commands/setup.md` 与 `.opencode/command/setup.md`）。`Setup` 仍作为终端子命令 `forceloop setup` 正常工作，并仍实现 `CommandMetadata` trait（trait contract 要求）。

### Drivers

1. **意图匹配**：`Setup` 的职责是"一次性项目初始化"（终端命令），不是"开发者工作流中的可调用命令"（Skill / Slash Command）
2. **UI 卫生**：避免在 Claude Code / OpenCode 的命令面板中出现"setup"条目（用户永远不会点击它）
3. **trait contract 优先**：`CommandMetadata` trait 要求 `Setup` 实现 `command_template()` / `skill_template()` —— impl 必须保留，但可在 `COMMANDS` 表中不消费
4. **最小变更**：仅一处生产代码改动（删除 1 行）+ 测试断言更新（~10 处）；不动 trait、不动 10 个 Command impl 的方法体

### Alternatives considered

- **Alternative A**：完全删除 `Setup::command_template()` / `Setup::skill_template()` impl
  - **Why rejected**：`CommandMetadata` trait 强制要求；删除会破 trait contract，10 个 Command 类型都会出现编译错误
- **Alternative B**：从 trait 层面给"非注册 Command"开洞（如 `CommandMetadata::should_register() -> bool`）
  - **Why rejected**：过度工程化；引入 trait 改动 → 10 个 impl 级联修改 → 违反项目"骨架优先的范围纪律"
- **Alternative C**：把 `COMMANDS` 表换成 `&[&dyn CommandMetadata]` 动态分发
  - **Why rejected**：与项目"零新增运行时依赖"原则不符；当前静态表方案在 [.omc/plans/setup-tool-flag.md:624-632](.omc/plans/setup-tool-flag.md#L624-L632) 已详细权衡
- **Alternative D**：本计划方案 —— 仅从 `COMMANDS` 表移除 `Setup` 条目
  - **Why chosen**：最小变更；不改 trait；不改 10 个 Command impl；意图清晰（"Setup 不出现在注册表里"）；新增 1 个反向测试 `setup_md_is_not_generated` 把契约钉死

### Why chosen (Alternative D)

- 1 行生产代码改动（删除 `("setup", ...)` 行）
- ~10 处测试断言更新（机械替换 + 1 个反向断言新增）
- trait contract 完全保留
- 未来加新 Command 的 pattern 不变（仍是在 `COMMANDS` 表加一行 + 同步测试期望集）
- 反向测试 `setup_md_is_not_generated` 是新需求的可执行规约 —— 任何意外把 Setup 重新加进 `COMMANDS` 表的提交都会触发该测试失败

### Consequences

**正面**:
- `Setup` 不再污染 Claude Code / OpenCode 的命令面板
- `run()` 写文件数从 20 减到 18，更准确反映"9 个有效 Command"
- 反向测试 `setup_md_is_not_generated` 把用户决策编码为可执行规约，防止回归
- 迁移成本：现有用户需手动删除一次 `.claude/commands/setup.md` / `.opencode/command/setup.md`（一次性）

**负面**:
- 旧 `SKILL_PROMPT` 描述"Generate platform-native command files from `CommandMetadata`" 仍通用 —— 不变（避免脆弱文案）
- `tests/command_compile.rs` 仍引用 `Setup.skill_template()` —— 不变（验证 trait impl，非消费路径）
- `Setup::command_template()` 实现被保留但仅在测试中使用 —— 轻微的"代码存在但不被生产消费"，注释说明

### Follow-ups

- 未来若需要清理已存在的 `setup.md`，添加 `--purge` 旗标到 `forceloop setup`
- 若未来加新 Command，必须同步更新 `tests/setup_tool.rs` 的 `run_writes_all_nine_commands_per_target` 期望集合 —— 测试名固定 9 提醒数量
- 若 trait contract 演化（如未来 `Setup` 不再需要 `CommandMetadata` impl），可同步删除 `Setup` 的 impl —— 当前不做

---

## 共识评审应用记录

本计划为 **Direct Mode**（用户已给出明确决策），不触发 Architect / Critic 循环。Architect 评审要求与 Critic 评估标准已通过自审应用：

- **80%+ claims 引用 file:line**：是（Context / Implementation Steps / Risks 表格 / Out of Scope / ADR 均带 file:line）
- **90%+ criteria 可测试**：是（Definition of Done 12 项 + Verification Steps 9 项均为可执行命令或断言）
- **替代方案公平探索**：是（ADR 列 3 个 rejected alternatives 并说明 reason）
- **风险 + 缓解**：是（8 行 risks 表格）
- **边界清晰**：是（10 项 Out of Scope）
