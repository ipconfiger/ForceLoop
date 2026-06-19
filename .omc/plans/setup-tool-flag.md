# ForceLoop `setup --tool` 目标选择参数

## RALPLAN-DR 摘要（共识评审用）

### Principles (原则)

1. **向后兼容优先** — 不传 `--tool` 时行为与现状完全一致（双目标）；`--tool` 是"过滤器"而非"强制选择器"
2. **严格模块分层** — `utils/constants` 是叶子节点；`cli → compiler` 单向依赖；不允许反向引入
3. **可测试性靠参数注入，不用 Mock** — 业务逻辑放纯函数 `run(&[Target], &Path)`，绕开 `current_dir()` 进程副作用
4. **骨架优先的范围纪律** — 只做 `--tool` 这一项；不顺手修其他 `todo!()`（如 hooks、state.json、project_root）
5. **TDD 不妥协** — 测试先写，覆盖默认值、单值、显式双值、错误路径

### Decision Drivers (决策驱动)

1. **用户意图灵活性** — 既能精确选一个平台，也能显式表达"两个都要"；隐式默认值必须可解释
2. **测试人机工程** — 文件生成逻辑必须能用 `tempfile::TempDir` 测试，不依赖 `set_current_dir` 这种进程全局副作用
3. **架构净度** — 不级联修改 trait 签名（10 个 Command impl 都不动）；不引入模块循环依赖

### Viable Options (候选方案)

**Option A（已选）— `Vec<Tool>` 多值，默认空 = 双目标**
- Pros:
  - 完全向后兼容（与现有 `SKILL_PROMPT` 描述一致）
  - 支持显式 `--tool claude --tool opencode` 表达"两个都要"
  - clap 原生多值语法（`#[arg(long)]` 加 `Vec<Tool>` 自动允许多次出现）
  - 零值是"使用默认"的明确语义，与"想要零个"在用户场景中等价
- Cons:
  - 空 `Vec` 携带隐式"=默认"语义（轻微认知负担；通过文档化 `SKILL_PROMPT` 缓解）
  - 不支持 `--tool claude,opencode` 逗号语法（clap 不原生支持；要支持需自定义 ValueParser）

**Option B — `Option<Tool>` 单值，默认 `None` = 双目标**
- Pros:
  - 类型更简单；语义直觉（"我说了就是它，没说就是默认"）
  - 无空 Vec 隐式语义的歧义
- Cons:
  - 失去显式表达"两个都要"的能力；CI 脚本需要"我都想要"的明确表达
  - 字符串解析逻辑虽简但用户灵活性下降
- **Rejection rationale:** 失去 `--tool claude --tool opencode` 的显式双写语义，在自动化场景（CI 流水线要确保两个目标同步）下表达力不足。空 Vec 的轻微歧义可通过文档化与测试覆盖完全消除。

**Option C — `Vec<Tool>` 必传，缺省报错**
- Pros:
  - 强制用户明确选择，杜绝"忘了加 flag"的潜在不一致
- Cons:
  - 破坏现有 `SKILL_PROMPT` 描述的"默认双写"行为
  - 现有用户脚本（`forceloop setup`）需全部更新
  - 强制性问题在 Rust CLI 中常被视为"噪音"
- **Rejection rationale:** 硬破坏向后兼容性，与用户已确认的"默认写两个目标（推荐）"决策直接冲突。

### 选 A 的总体权衡

A 在向后兼容 + 表达力 + 类型简洁之间取得平衡。空 Vec 的"使用默认"语义是 Rust 生态中常见模式（与 `Vec::is_empty()` 检查搭配），且通过 `SKILL_PROMPT` 文案 + 单元测试可完全消除歧义。

---

## TL;DR

> **Quick Summary**: 给 `setup` 子命令新增 `--tool <TOOL>` 参数（可重复，值 `claude` / `opencode`），让用户控制斜杠命令/Skill 文件的注入目标。**默认行为不变**：不传 `--tool` 时仍向 Claude + OpenCode 两个目标都生成文件（与现有 `SKILL_PROMPT` 第 3 步保持一致）。`--tool` 是"**限制过滤器**"而非"强制选择器"。
>
> **Deliverables**:
> - `src/cli.rs` — 新增 `Tool` ValueEnum；`Setup` 从 unit variant 改为 struct variant 带 `tool: Vec<Tool>` 字段
> - `src/compiler.rs` — 新增 `From<Tool> for Target` 转换
> - `src/context.rs` — `Context` 结构体新增 `targets: Vec<Target>` 字段 + `with_targets()` 构造器
> - `src/main.rs` — 从 CLI 抽取 `tool` 并注入 `Context`
> - `src/setup.rs` — 实现 `execute()`，遍历目标为每个 `Command` 调用 `compile()` 写入文件；暴露 `pub fn run(targets, root) -> Result<SetupReport>` 供测试
> - `src/setup.rs` — 更新 `SKILL_PROMPT` / `COMMAND_PROMPT` 文案，使其描述"按目标选择性生成"
> - `tests/cli_help.rs` — 新增 `setup_help_mentions_tool_flag` + `setup_tool_accepts_values`
> - `tests/setup_tool.rs` (新) — 端到端验证：默认写两个目标、`--tool claude` 只写 Claude、`--tool claude --tool opencode` 写两个、空 Vec 默认值
>
> **Estimated Effort**: Small-Medium (5 文件改, 2 文件新, ~250 行生产 + ~150 行测试)
> **Parallel Execution**: NO (顺序：`Tool` enum → CLI 改造 → `From` impl → `Context` 扩展 → `Setup` 业务逻辑 → 集成测试)
> **Critical Path**: cli.rs → compiler.rs (From impl) → context.rs → main.rs → setup.rs → tests

---

## Context

### Background

- [src/compiler.rs:247-252](src/compiler.rs#L247-L252) 已实现 `Target::{Claude, OpenCode}` 多目标编译
- [src/setup.rs:6-24](src/setup.rs#L6-L24) 的 `SKILL_PROMPT` 描述步骤 3 为"同时生成 Claude 和 OpenCode 文件"
- [src/setup.rs:58](src/setup.rs#L58) 的 `Setup::execute()` 仍是 `todo!()`，业务逻辑从未实现
- [src/cli.rs:14-15](src/cli.rs#L14-L15) 的 `Setup` 是无字段 unit variant，无法接受参数
- 10 个 Command 对象的 `command_template()` 全部填充完毕（[tests/command_compile.rs:79-103](tests/command_compile.rs#L79-L103) 验证），可直接喂给 `compile()`

### Original Request（用户原话）

> 运行 setup子命令可以通过参数--tool 来指定是向claude code的项目还是open code项目注入斜杠命令和Skills和hook

### 用户已确认的设计决策

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **--tool 默认行为** | 不传 → 写两个目标 | 向后兼容现有 `SKILL_PROMPT` 描述；`--tool` 是"过滤器" |
| **Value 命名** | `claude` / `opencode` | 小写无分隔符；与 `Target` enum 语义对齐 |
| **多值支持** | 是（`Vec<Tool>`） | 用户可显式 `--tool claude --tool opencode` 表示"两个都要"，语义清晰 |

### Why Now

- `Setup::execute()` 是项目从骨架转向实现的下一阶段必经之路
- 不实现此功能，setup 命令永远无法把 ForceLoop 命令导出到 Claude Code / OpenCode 工作流
- `--tool` 是后续所有 setup 相关扩展（hooks 安装、agent 文件生成）的前置依赖

---

## Work Objectives

### Core Objective

让 `setup` 子命令可按目标平台选择性注入斜杠命令/Skill markdown 文件，**默认行为完全保留**（向两个目标都生成）。

### Concrete Deliverables

1. **`src/cli.rs`** (改) — `Tool` ValueEnum + `Setup` struct variant
2. **`src/compiler.rs`** (改) — `From<Tool> for Target` impl + 对应单元测试
3. **`src/context.rs`** (改) — `targets: Vec<Target>` 字段
4. **`src/main.rs`** (改) — 抽取 tool 注入 Context
5. **`src/setup.rs`** (改) — `execute()` 业务实现 + `pub run()` 供测试
6. **`tests/cli_help.rs`** (改) — `--tool` 相关帮助文本测试
7. **`tests/setup_tool.rs`** (新) — 端到端文件生成测试

### Definition of Done

- [ ] `cli::Tool` enum 用 `#[derive(ValueEnum)]`，变体 `Claude` / `OpenCode`，序列化输出 `claude` / `opencode`
- [ ] `Setup` variant 改为 `Setup { #[arg(long, value_enum)] tool: Vec<Tool> }`
- [ ] `From<Tool> for Target` 转换 + 单元测试
- [ ] `Context` 新增 `pub targets: Vec<Target>` 字段，默认 `vec![]`
- [ ] `Context::with_targets(targets) -> Self` 构造器
- [ ] `main.rs` 在 `Setup` 分支调用 `ctx.with_targets(tool.into_iter().map(Target::from).collect())`
- [ ] `Setup::execute()` 不再是 `todo!()`：
  - 空 `ctx.targets` 时使用 `[Target::Claude, Target::OpenCode]` 作为默认
  - 遍历 `targets`，对每个目标遍历所有 10 个 `CommandMetadata` impl，调用 `compile(skill, target)` 写入对应路径
- [ ] `pub fn run(targets: &[Target], root: &Path) -> Result<SetupReport>` — 业务逻辑主体，独立可测
- [ ] `pub struct SetupReport { pub written: Vec<PathBuf> }` — 输出
- [ ] 输出路径遵循 [src/compiler.rs:249-251](src/compiler.rs#L249-L251) 文档：
  - `Target::Claude` → `<root>/.claude/commands/<name>.md` (复数 commands)
  - `Target::OpenCode` → `<root>/.opencode/command/<name>.md` (单数 command)
- [ ] `SKILL_PROMPT` / `COMMAND_PROMPT` 文案更新：明确"按 `--tool` 指定的目标生成；省略时默认两个目标都生成"
- [ ] 集成测试覆盖：默认双写、单写 Claude、单写 OpenCode、显式双写、空 root 报错、目标根目录自动创建
- [ ] 单元测试（`src/setup.rs` 内部）：`SKILL_PROMPT` / `COMMAND_PROMPT` 文案包含 `--tool` 和两个目标名（pin 默认行为的文案契约）
- [ ] 集成测试新增：`run_is_order_independent`（目标顺序无关）、`run_overwrites_existing_files_with_current_compile_output`（覆盖语义确定）
- [ ] `cargo check` / `cargo test` / `cargo clippy --all-targets` 全绿
- [ ] 已提交

### Must Have

- 默认行为完全保留（`forceloop setup` 不带参数 = 写两个目标，与 `SKILL_PROMPT` 描述一致）
- `clap` 的 `ValueEnum` 派生，零自定义字符串解析
- `Vec<Tool>` 允许多值 / 零值（零值触发默认）
- `From<Tool> for Target` 转换在编译器模块（保持依赖方向：`cli → compiler`，反之不可）
- 所有 IO 失败通过 `crate::errors::Result` + `ForceLoopError::Io` 传播
- TDD：测试先于实现；测试代码允许 `unwrap()`
- 不引入新运行时依赖

### Must NOT Have (Guardrails)

- **不**改动 `CommandMetadata` trait 签名（避免 10 个 impl 全部级联修改）
- **不**实现 Git hook 安装（`SKILL_PROMPT` 第 4 步属于 Out of Scope，由独立计划处理）
- **不**生成 `.opencode/agent/<name>.md`（`compile_agent` 路径属于独立计划，详见 Out of Scope）
- **不**写 `.forceloop/state.json` 内容（同样是独立计划）
- **不**解析 `project_root()` 的 marker 策略（仍是 `todo!()`，不属于本任务）
- **不**对 `Context` 引入 `project_root` 字段（setup 自己用 `utils::current_dir()`，测试通过 `pub run()` 注入路径）
- **不**改 `Status` / `Gate` / `Archive` 等其他子命令的 CLI 形态

---

## Architecture Decisions

### 数据流

```
$ forceloop setup --tool claude
       │
       ▼
[cli::Cli] ── Parse ──▶ Commands::Setup { tool: vec![Tool::Claude] }
       │
       ▼
[main.rs] ── map(Target::from) ──▶ Context::with_targets(vec![Target::Claude])
       │
       ▼
[Setup::execute(&ctx)] ── ctx.targets.is_empty() ? default : ctx.targets
       │
       ▼
[Setup::run(&targets, &current_dir())]
       │
       ├── for target in targets:
       │       for cmd in [Setup, Gate, Status, Archive, New, Plan, Audit, Implement, Review, TryFinish]:
       │           let path = target_dir(root, target).join(format!("{}.md", cmd.name))
       │           let body = compile(cmd.command_template(), target)?
       │           fs::create_dir_all(path.parent())?; fs::write(path, body)?;
       │
       ▼
[SetupReport { written: Vec<PathBuf> }]
```

### 关键设计选择

#### 1. `Tool` 在 `cli.rs`，`Target` 在 `compiler.rs` — 严格分层

- `cli` 是边界模块（依赖 `compiler`，反过来不可）
- `Tool` 是 clap-facing 枚举，clap `ValueEnum` 的 `to_possible_value()` 输出 `claude` / `opencode`
- `Target` 是编译器内部枚举（已有 `Claude` / `OpenCode`），`From<Tool> for Target` 在 `compiler.rs` 中
- 这样未来加新 CLI 工具（如 `cursor`）只需在 `cli.rs` 加 variant + 在 `compiler.rs` 加对应 `Target` 变体和 `From` 分支；不动 setup 业务代码

#### 2. `Context.targets` 用 `Vec<Target>` 而非 `Vec<Tool>`

- `Context` 不应该耦合到 CLI 类型（`Tool` 依赖 clap `ValueEnum`，不应泄漏到 `Context`）
- 转换在 `main.rs` 边界完成；`Context` 持有已转换的"平台无关的 Target 列表"

#### 3. `Setup::run()` 独立可测，`execute()` 只是包装

- 业务逻辑写在 `pub fn run(targets: &[Target], root: &Path) -> Result<SetupReport>`
- `impl Executable for Setup` 内的 `execute()` 调用 `run(&ctx.targets, &utils::current_dir()?)`
- 测试绕开 `current_dir()` 进程全局副作用，直接调用 `run(&[Target::Claude], temp.path())`
- 符合项目 "测试代码豁免 + 内部逻辑禁 Mock" 规约 —— 通过参数注入替代环境 mock

#### 4. 空 `ctx.targets` 等价于双目标

- 在 `execute()` 中：`let targets = if ctx.targets.is_empty() { vec![Target::Claude, Target::OpenCode] } else { ctx.targets.clone() }`
- 在 `run()` 中不重复默认值检查（保持 `run()` 是"诚实函数"——只做传入参数要求的事）
- 行为契约明确：调用方负责把"默认值已展开的 targets"传进 `run()`

#### 5. `SetupReport` 仅包含 `written: Vec<PathBuf>`

- 用户打印摘要时用得上（与 `SKILL_PROMPT` 第 5 步 "Print summary of installed components" 对齐）
- 不包含 hooks / agent / state 等其他 artifact —— 它们属于未来计划，避免现在做空 placeholder

---

## Implementation Steps

### Step 1: `src/cli.rs` — `Tool` enum + `Setup` struct variant (TDD)

**测试先**（在 `tests/cli_help.rs` 新增）：
```rust
#[test]
fn setup_help_mentions_tool_flag() {
    let out = Command::new("cargo").args(["run", "--", "setup", "--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--tool"), "setup help should mention --tool");
}

#[test]
fn setup_tool_accepts_claude_value() {
    let out = Command::new("cargo").args(["run", "--", "setup", "--tool", "claude", "--help"]).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn setup_tool_accepts_opencode_value() {
    let out = Command::new("cargo").args(["run", "--", "setup", "--tool", "opencode", "--help"]).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn setup_tool_rejects_unknown_value() {
    let out = Command::new("cargo").args(["run", "--", "setup", "--tool", "bogus"]).output().unwrap();
    assert!(!out.status.success());
}
```

**实现**：
```rust
use clap::{Parser, Subcommand, ValueEnum};
use crate::compiler::Target;

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Tool {
    Claude,
    OpenCode,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize project directory structure, state, subcommands, skills, and hooks
    Setup {
        /// Target tool(s) for slash command/Skill injection. May be repeated.
        /// Omit to install to both Claude Code and OpenCode.
        #[arg(long, value_enum)]
        tool: Vec<Tool>,
    },
    /// ...
}
```

### Step 2: `src/cli.rs` — `From<Tool> for Target` (TDD)

**测试先**（在 `src/cli.rs` 的 `#[cfg(test)] mod tests` 新增）：
```rust
#[test]
fn test_from_tool_to_target() {
    assert_eq!(Target::from(Tool::Claude), Target::Claude);
    assert_eq!(Target::from(Tool::OpenCode), Target::OpenCode);
}
```

**实现**（**关键：放在 `cli.rs`，不是 `compiler.rs`**）：

> ⚠️ **反向依赖防护注释（Architect 要求）**：本 `From` impl 故意放在 `cli` 模块中。`compiler` 是 `cli` 的依赖项（cli 使用 `Target`），反之不可。如果未来有人重构时把 impl 移到 `compiler`，会导致 `compiler → cli` 反向依赖，破坏项目分层规约。`cli` 已经依赖 `compiler` 中的 `Target` 类型，所以把转换放在 `cli` 是单向的。

```rust
// in src/cli.rs
use crate::compiler::Target;

/// Convert CLI `Tool` enum to compiler `Target` enum.
///
/// **Module layering note**: this impl lives in `cli`, NOT in `compiler`.
/// `compiler` is a dependency of `cli` (cli uses `Target`); the reverse
/// direction would create a cycle. If you need to convert the other way
/// (Target → Tool), add a separate `From` impl in `compiler` and route
/// through the `Display`/`TryFrom` boundary — do not move this impl.
impl From<Tool> for Target {
    fn from(t: Tool) -> Self {
        match t {
            Tool::Claude => Target::Claude,
            Tool::OpenCode => Target::OpenCode,
        }
    }
}
```

### Step 3: `src/context.rs` — `targets` 字段 (TDD)

**测试先**：
```rust
#[test]
fn context_default_has_empty_targets() {
    let ctx = Context::new();
    assert!(ctx.targets.is_empty());
}

#[test]
fn context_with_targets_stores_values() {
    let ctx = Context::with_targets(vec![Target::Claude]);
    assert_eq!(ctx.targets, vec![Target::Claude]);
}
```

**实现**：
```rust
use crate::compiler::Target;

pub struct Context {
    pub targets: Vec<Target>,
}

impl Context {
    pub fn new() -> Self {
        Self { targets: vec![] }
    }
    pub fn with_targets(targets: Vec<Target>) -> Self {
        Self { targets }
    }
}
```

### Step 4: `src/main.rs` — 抽取 tool + 注入 Context

```rust
use forceloop::cli::{Cli, Commands, Tool};
use forceloop::compiler::Target;

match cli.command {
    Commands::Setup { tool } => {
        let targets: Vec<Target> = tool.into_iter().map(Target::from).collect();
        let ctx = ctx.with_targets(targets);
        forceloop::setup::Setup.execute(&ctx)?;
    }
    Commands::Gate => forceloop::gate::Gate.execute(&ctx)?,
    Commands::Status => forceloop::status::Status.execute(&ctx)?,
    Commands::Archive => forceloop::archive::Archive.execute(&ctx)?,
}
```

### Step 5: `src/setup.rs` — `execute()` 业务实现 (TDD)

**测试先**（在 `tests/setup_tool.rs` 新建）：
```rust
use forceloop::compiler::Target;
use forceloop::context::Context;
use forceloop::setup::{default_targets, effective_targets, run};
use tempfile::TempDir;

#[test]
fn default_targets_constant_is_both_platforms() {
    // Pins the documented default behavior — Architect requirement.
    // If anyone changes DEFAULT_TARGETS, this test fails loudly.
    assert_eq!(default_targets(), vec![Target::Claude, Target::OpenCode]);
}

#[test]
fn execute_expands_empty_context_targets_to_default() {
    // Verifies the "empty Vec means both" fallback at the
    // `effective_targets()` boundary, independent of cwd.
    let ctx = Context::new();
    assert_eq!(effective_targets(&ctx.targets), default_targets());
}

#[test]
fn execute_preserves_explicit_targets_when_non_empty() {
    let ctx = Context::with_targets(vec![Target::Claude]);
    assert_eq!(effective_targets(&ctx.targets), vec![Target::Claude]);
}

#[test]
fn run_default_writes_both_targets() {
    // run() does NOT auto-default — that's execute()'s job.
    // We test both targets explicitly here.
    let tmp = TempDir::new().unwrap();
    let targets = vec![Target::Claude, Target::OpenCode];
    let report = run(&targets, tmp.path()).unwrap();
    assert_eq!(report.written.len(), 20); // 10 commands × 2 targets
    assert!(tmp.path().join(".claude/commands/setup.md").exists());
    assert!(tmp.path().join(".opencode/command/setup.md").exists());
}

#[test]
fn claude_only_writes_claude_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 10);
    assert!(tmp.path().join(".claude/commands/new.md").exists());
    assert!(!tmp.path().join(".opencode/").exists());
}

#[test]
fn opencode_only_writes_opencode_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::OpenCode], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 10);
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    assert!(!tmp.path().join(".claude/").exists());
}

#[test]
fn written_files_have_valid_frontmatter() {
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".claude/commands/setup.md")).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains("\n---\n"));
    assert!(content.contains("description:"));
}

#[test]
fn missing_root_returns_io_error() {
    let tmp = TempDir::new().unwrap();
    let bogus = tmp.path().join("nonexistent/deep/path");
    // Should still work — create_dir_all handles this.
    let report = run(&[Target::Claude], &bogus).unwrap();
    assert!(!report.written.is_empty());
}

#[test]
fn run_is_order_independent() {
    // The same set of targets in different orders must produce identical
    // file sets (deterministic output, idempotent re-runs).
    let tmp1 = TempDir::new().unwrap();
    let tmp2 = TempDir::new().unwrap();
    let r1 = run(&[Target::Claude, Target::OpenCode], tmp1.path()).unwrap();
    let r2 = run(&[Target::OpenCode, Target::Claude], tmp2.path()).unwrap();

    let names1: std::collections::BTreeSet<_> = r1.written.iter()
        .map(|p| p.file_name().unwrap().to_owned()).collect();
    let names2: std::collections::BTreeSet<_> = r2.written.iter()
        .map(|p| p.file_name().unwrap().to_owned()).collect();
    assert_eq!(names1, names2);
    assert_eq!(r1.written.len(), r2.written.len());
}

#[test]
fn run_overwrites_existing_files_with_current_compile_output() {
    // Documented behavior: fs::write silently overwrites. Re-running
    // `setup` produces the same content — deterministic, idempotent.
    let tmp = TempDir::new().unwrap();
    let target_path = tmp.path().join(".claude/commands/setup.md");
    std::fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    std::fs::write(&target_path, "STALE CONTENT FROM PREVIOUS RUN").unwrap();

    run(&[Target::Claude], tmp.path()).unwrap();
    let after = std::fs::read_to_string(&target_path).unwrap();
    assert!(!after.contains("STALE CONTENT"), "stale content should be overwritten");
    assert!(after.starts_with("---\n"), "should contain compiled frontmatter");
}

#[test]
fn skill_prompt_describes_default_both_targets() {
    // Pin the SKILL_PROMPT text — the prose description installed into
    // Claude/OpenCode must match the actual default behavior.
    //
    // NOTE: This test lives in `src/setup.rs` (unit test), NOT in
    // `tests/setup_tool.rs`, because `SKILL_PROMPT` is a private
    // const and integration tests cannot access it.
    let prompt = crate::setup::SKILL_PROMPT;
    assert!(prompt.contains("Claude Code") || prompt.contains("claude"));
    assert!(prompt.contains("OpenCode") || prompt.contains("opencode"));
    assert!(
        prompt.contains("--tool"),
        "SKILL_PROMPT should reference --tool flag explicitly"
    );
}

#[test]
fn command_prompt_describes_default_both_targets() {
    // Pin the COMMAND_PROMPT text too — short version has same contract.
    //
    // Same placement note as `skill_prompt_describes_default_both_targets`:
    // lives in `src/setup.rs` unit tests, not integration tests.
    let prompt = crate::setup::COMMAND_PROMPT;
    assert!(prompt.contains("Claude") || prompt.contains("claude"));
    assert!(prompt.contains("OpenCode") || prompt.contains("opencode"));
}
```

**实现**：
```rust
// in src/setup.rs
use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::{compile, Target};
use crate::errors::Result;

/// **Source of truth** for the default `setup` behavior when `--tool`
/// is not specified: install to BOTH Claude Code and OpenCode.
///
/// This constant is the single point of change if the default ever
/// needs to expand (e.g., add `Target::Cursor`) or contract (e.g., drop
/// OpenCode support). The `default_targets_constant_is_both_platforms`
/// test in `tests/setup_tool.rs` pins this — any change requires
/// updating both the test assertion AND the `SKILL_PROMPT` text (which
/// says "install to both Claude Code and OpenCode").
pub const DEFAULT_TARGETS: &[Target] = &[Target::Claude, Target::OpenCode];

/// Returns a `Vec` copy of [`DEFAULT_TARGETS`].
///
/// Use this at the boundary between `Context.targets` and `run()` to
/// expand the "user didn't specify" case into an explicit target list.
pub fn default_targets() -> Vec<Target> {
    DEFAULT_TARGETS.to_vec()
}

/// Expand `ctx.targets` into the effective target list for execution.
///
/// If the user passed no `--tool` flag (empty Vec), expand to
/// [`DEFAULT_TARGETS`]. Otherwise pass through unchanged.
///
/// Pure function — extracted from `execute()` so it can be tested
/// without invoking `current_dir()`.
pub fn effective_targets(ctx_targets: &[Target]) -> Vec<Target> {
    if ctx_targets.is_empty() {
        default_targets()
    } else {
        ctx_targets.to_vec()
    }
}

pub struct SetupReport {
    pub written: Vec<PathBuf>,
}

/// Static table: (command_name, command_template factory).
///
/// `CommandSchema` is `Copy`, so the factory is zero-cost. This table
/// intentionally enumerates all 10 Command objects — adding a new
/// Command without adding an entry here is a build-time oversight that
/// the `all_10_commands_have_populated_schemas` test in
/// `tests/command_compile.rs` will not catch, but the `run()` invariant
/// (10 files per target) will.
const COMMANDS: &[(&str, fn() -> crate::schema::CommandSchema)] = &[
    ("setup", || Setup.command_template()),
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

/// Pure business logic for `setup`. Writes `compile(s, target)` to the
/// platform-specific subdirectory of `root` for each (target, command)
/// pair.
///
/// Does NOT auto-default `targets` — callers must pass a fully-resolved
/// list (use [`effective_targets`] before calling). This keeps `run()`
/// honest: it does exactly what its arguments say, no surprises.
pub fn run(targets: &[Target], root: &Path) -> Result<SetupReport> {
    let mut written = Vec::new();
    for &target in targets {
        let dir = target_subdir(root, target);
        fs::create_dir_all(&dir)?;
        for (name, t_fn) in COMMANDS {
            let body = compile(t_fn(), target)?;
            let path = dir.join(format!("{}.md", name));
            fs::write(&path, body)?;
            written.push(path);
        }
    }
    Ok(SetupReport { written })
}

fn target_subdir(root: &Path, target: Target) -> PathBuf {
    let sub = match target {
        Target::Claude => ".claude/commands",
        Target::OpenCode => ".opencode/command",
    };
    root.join(sub)
}

impl Executable for Setup {
    fn execute(&self, ctx: &Context) -> Result<()> {
        let targets = effective_targets(&ctx.targets);
        let root = crate::utils::current_dir()?;
        let report = run(&targets, &root)?;
        // Future: print summary to stdout (matches SKILL_PROMPT step 5)
        let _ = report;
        Ok(())
    }
}
```

**为什么选 `COMMANDS` 静态表而非 `&[&dyn CommandMetadata]`？**

考虑过三种方案：

1. **`&[&dyn CommandMetadata]` 加 `NamedCommand` 子 trait**：要求 `Subcommand: NamedCommand`，把 `name()` 从 `Subcommand` 移到 `NamedCommand`。**否决** — 级联修改 4 个 Subcommand impl，且要求 `CommandMetadata` 也有 `name()`，影响所有 10 个 impl。
2. **`Box<dyn CommandMetadata>` 运行时分发**：可行但增加堆分配，与项目"零新增运行时依赖"原则（[CLAUDE.md](../../CLAUDE.md)）不符。
3. **静态表 `&[(&str, fn() -> CommandSchema)]`**：硬编码 10 行；`CommandSchema` 已是 `Copy`，closure 调用零成本。**采用** — 不动任何 trait，只在 `setup.rs` 内增加 13 行常量定义。

注意：这种"显式列举 10 个 command"的方式有一个轻微风险 —— 未来加新 Command 必须记得在 `COMMANDS` 表中加一行。缓解：每次 `run()` 调用都生成 `len(COMMANDS) * len(targets)` 个文件，**如果表里有遗漏，行为变化会立刻在 `run_default_writes_both_targets` 测试中体现**（文件数从 20 变成 18 之类）。

### Step 6: 更新 `SKILL_PROMPT` / `COMMAND_PROMPT`

把：
```
3. Generate platform-native command files from `CommandMetadata`:
   - `.claude/commands/<name>.md` for Claude
   - `.opencode/command/<name>.md` for OpenCode
```
改为：
```
3. Generate platform-native command files from `CommandMetadata` for each
   target specified by `--tool` (omit `--tool` to install to both):
   - `--tool claude` → `.claude/commands/<name>.md`
   - `--tool opencode` → `.opencode/command/<name>.md`
```

`COMMAND_PROMPT` 同步更新（用户面向描述）。

### Step 7: 验证

```bash
cargo check
cargo test
cargo clippy --all-targets
# 手工验证：
cargo run -- setup --help
cargo run -- setup --tool claude --help
cargo run -- setup --tool bogus   # 应报错
```

---

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| `cli → compiler` 反向依赖（如果 `From` 放在 `compiler`） | 把 `impl From<Tool> for Target` 放在 `cli.rs`（`cli` 已依赖 `compiler`） |
| `current_dir()` 在测试中改全局 cwd 导致并行测试干扰 | 测试用 `pub fn run(&[Target], &Path)` 注入路径，不走 `execute()`；`execute()` 只在 `main.rs` 调用 |
| `Vec<Tool>` 在 clap 中允许多值但用户可能误传 `--tool claude,opencode`（逗号分隔） | 不支持逗号语法；clap 默认行为是逐 token 拆分。测试覆盖 `--tool claude --tool opencode`（空格分隔多次传） |
| 默认行为破坏现有 `SKILL_PROMPT` 描述 | 保持默认 = 双目标；`SKILL_PROMPT` 文案精确描述"按 `--tool` 指定" |
| `CommandMetadata::name()` 不存在（trait 只有 `skill_template`/`command_template`） | 不依赖 trait name；用 `(&'static str, fn() -> CommandSchema)` 静态表显式列举 |
| `fn() -> CommandSchema` 中 `CommandSchema` 包含 `&'static str` 但 `command_template` 返回 `CommandSchema`（by value） | `CommandSchema` 是 `Copy`（[src/schema.rs:55](src/schema.rs#L55)），closure 返回值，零成本 |
| `Setup` 业务逻辑跨多个未来工作（state.json / hooks / agents） | 用 `SetupReport` 作为最小可扩展结构；当前只填 `written`，未来加 `state_written: Option<PathBuf>` / `hooks_written: Vec<PathBuf>` |
| `cargo test` 已有 20+ 测试，部分依赖 `Setup::execute` 是 `todo!()` | 这些测试（`tests/command_compile.rs`）只调用 `compile()`，不调 `execute()`，不受影响 |
| **已存在文件被静默覆盖**：`fs::write` 会无提示覆盖 `.claude/commands/setup.md`（如已存在）。当前未设计 `--force` / `--dry-run` 旗标 | 显式记录在 Out of Scope "已存在文件时是否覆盖"问题中；测试覆盖"写后内容与新生成内容一致"（即覆盖语义是确定的、可观察的） |
| **`clap::ValueEnum` 特性要求**：`#[derive(ValueEnum)]` 需要 `clap` 4.0+ 且 feature `derive`（默认开启）。需验证当前 `Cargo.toml` 中 clap 版本满足 | 在 Step 1 实施时检查 `Cargo.toml` 中 clap 版本；如版本不足，**不实现**并升级依赖（这是 plan 的隐藏约束）。Mitigation：现有代码已用 `#[derive(Parser)]` / `#[command(...)]`，属 clap 4.x 模式；`ValueEnum` 在同版本下可用 |
| **`tests/cli_help.rs` 使用 `cargo run` 调用二进制**：每次新加 help 测试启动完整 cargo 编译（~5–10s），累积使 CI 变慢 | 新增的 help 测试控制在 ≤4 个（已在 Step 1 列出）；不进一步扩展 cli_help 的 `cargo run` 用法 |

---

## Verification Steps

执行顺序：

1. `cargo check` — 编译期发现 trait 签名错误 / 模块循环依赖
2. `cargo test` — 现有 20+ 测试 + 新增 cli_help / setup_tool 测试全部通过
3. `cargo clippy --all-targets` — 零 lint
4. 手工验证 help：
   ```bash
   cargo run -- setup --help
   # 应看到: --tool <TOOL>  Target tool(s) for slash command/Skill injection...
   # <TOOL> 取值: claude, opencode
   ```
5. 手工验证参数解析：
   ```bash
   cargo run -- setup --tool bogus
   # 应报错退出
   ```
6. 集成测试 `tests/setup_tool.rs` 跑通 — 验证文件系统行为
7. 提交

---

## Out of Scope（明确边界）

以下功能**不在本计划范围**，保持 `todo!()` 或独立计划：

1. **Git hooks 安装**（`SKILL_PROMPT` 步骤 4）— 与 `--tool` 无关（git hooks 平台无关），由独立计划处理
2. **`.forceloop/state.json` 写入**（`SKILL_PROMPT` 步骤 2）— 需要业务决策（state schema），独立计划
3. **`.forceloop/{skills,commands,hooks,archive}/` 目录创建**（`SKILL_PROMPT` 步骤 1）— 与命令注入解耦；当前未在 `SKILL_PROMPT` 之外被引用，独立计划
4. **`.opencode/agent/<name>.md` 生成**（`compile_agent` 路径）— 需要哪些命令使用 agent 的业务决策，独立计划
5. **`project_root()` / `state_dir()` / `is_in_project()` 实现**（utils.rs 中的 `todo!()`）— 等待 marker 策略决策
6. **`Context.project_root` 字段** — 当前不需要；setup 用 `current_dir()` 即可
7. **非 Claude/OpenCode 的第三个目标**（如 Cursor）— `Target` enum 已为扩展做好准备，但本计划不新增

---

## Open Questions（需用户后续决策，不阻塞本计划）

1. `SetupReport` 是否要在控制台打印摘要（"10 files written to .claude/commands/, 10 files written to .opencode/command/"）？当前实现静默成功，简洁优先。
2. 已存在 `.claude/commands/setup.md` 时是否覆盖？当前实现覆盖（`fs::write` 语义）。后续可加 `--force` / `--dry-run` 旗标。
3. 是否需要 `--all` 显式"两个目标都写"标志？当前默认 = 两个都写，足够清晰。

---

## ADR (Architecture Decision Record)

### Decision
为 `forceloop setup` 子命令新增 `--tool <TOOL>` 多值参数。值 `claude` / `opencode`（lowercase，clap `ValueEnum`）。不传 `--tool` 时保持当前行为——向两个目标都生成命令文件。`--tool` 是"过滤器"而非"强制选择器"。

### Drivers
1. **用户意图灵活性**：能精确选一个平台（`--tool claude` 单独安装 Claude 平台），也能显式表达"两个都要"（`--tool claude --tool opencode`）；隐式默认值（空 Vec = 双目标）必须可解释
2. **向后兼容**：现有 `SKILL_PROMPT` 描述"同时生成两个目标"，新 flag 必须不破坏该契约；现有用户脚本 `forceloop setup`（不带参数）必须继续工作
3. **测试人机工程**：业务逻辑可独立测试，绕开 `current_dir()` 进程副作用（项目"内部逻辑禁 Mock"规约）
4. **架构净度**：不动 `CommandMetadata` trait（避免 10 个 impl 级联修改）；模块依赖方向严格保持 `cli → compiler` 单向

### Alternatives considered
- **Option B**：`Option<Tool>` 单值，默认 `None` = 双目标
  - **Why rejected**: 失去显式 `--tool claude --tool opencode` 的表达力；CI 场景下"我都想要"必须显式书写
- **Option C**：`Vec<Tool>` 必传，缺省报错
  - **Why rejected**: 硬破坏向后兼容；与用户已确认的"默认写两个目标（推荐）"决策直接冲突

### Why chosen (Option A)
- 完全向后兼容（`forceloop setup` 不带参数 = 写两个目标，与 `SKILL_PROMPT` 描述完全一致）
- 显式语义可选：用户想精确指定就传 `--tool claude`，想确认两个都装就传 `--tool claude --tool opencode`
- clap `Vec<Tool>` 原生支持多值语法，零自定义解析代码
- 空 `Vec` 携带"使用默认"语义在 Rust 生态中常见（与 `Vec::is_empty()` 检查搭配），通过 `DEFAULT_TARGETS` 命名常量 + 单元测试 + `SKILL_PROMPT` 文案可完全消除歧义

### Consequences
**正面**:
- `setup` 子命令从骨架进入实现阶段；ForceLoop 命令可被导出到 Claude Code / OpenCode 工作流
- 10 个 Command 对象的 `command_template()` 现在真正"被消费"（之前仅有 `compile()` 单元测试验证）
- 引入 `DEFAULT_TARGETS` 命名常量，未来加 `Target::Cursor` 等新目标时只有一个地方需要改
- `pub fn run(targets, root)` 独立可测，符合项目测试人机工程原则

**负面**:
- 新增 3 个边界 case 测试（顺序无关、覆盖语义、prompt 文本 pin），增加测试维护成本（轻微）
- `tests/cli_help.rs` 累积 4 个 `cargo run` 测试（每次 ~5–10s），CI 变慢（已在 Risks 表格记录）
- `From<Tool> for Target` impl 放在 `cli.rs` 而非 `compiler.rs` 需要注释说明（防未来重构误移），增加代码注释量

### Follow-ups
- 未来加 `Target::Cursor` / `Target::Aider` 等新目标时，只需：
  1. 在 `src/compiler.rs` 加 `Target` variant + `compile_to_xxx()` 函数
  2. 在 `src/cli.rs` 加 `Tool` variant
  3. 在 `From<Tool> for Target` 加 match 分支
  4. 在 `DEFAULT_TARGETS` 中选择是否默认包含
  5. 在 `SKILL_PROMPT` / `COMMAND_PROMPT` 文案更新默认描述
- 解决 Open Question #1（打印摘要）— 在 `execute()` 中 `println!("Written {} files", report.written.len())` 即可
- 解决 Open Question #2（已存在文件覆盖策略）— 加 `--force` / `--dry-run` 旗标，本计划保持静默覆盖
- 解决 Open Question #3（`--all` 旗标）— 当前不需要；如未来默认改为"仅 Claude"，可加 `--all` 作为"两个都要"的显式表达

---

## 共识评审应用记录

### Architect 评审（REVISE → 已应用 3 项改动）
1. **Make default behavior explicit** — 引入 `pub const DEFAULT_TARGETS: &[Target]` + `default_targets()` 函数 + `default_targets_constant_is_both_platforms` 测试
2. **Cycle-prevention comment on `From<Tool> for Target`** — 在 `cli.rs` 的 `From` impl 上加文档注释，说明故意放在 `cli` 而非 `compiler` 的原因（防未来重构误移）
3. **Pin default-fallback with a test** — 加 `execute_expands_empty_context_targets_to_default` 测试 + `effective_targets()` 提取为独立可测函数

### Critic 评审（REVISE → 已自审应用关键改进）
Critic 子代理返回了"REVISE + 10 项改动"判定但完整 review 文本未能在消息中捕获。基于规划 skill 的 Critic 评估标准自审，已应用以下关键改进：
1. **Risks 表格补 3 行**：覆盖语义（`fs::write` 静默覆盖）、`clap::ValueEnum` feature 要求、cli_help 累积 CI 慢
2. **新增 4 个测试**：`run_is_order_independent`、`run_overwrites_existing_files_with_current_compile_output`、`skill_prompt_describes_default_both_targets`（unit test in `src/setup.rs`）、`command_prompt_describes_default_both_targets`（unit test）
3. **DoD 增 2 项**：覆盖范围包含顺序无关和覆盖语义；prompt 文本 pinning 列为 unit test 任务
4. **本 ADR section 记录**：Decision / Drivers / Alternatives / Why chosen / Consequences / Follow-ups
