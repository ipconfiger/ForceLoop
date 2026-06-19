# ForceLoop Command Schema & Multi-Target Compiler

## TL;DR

> **Quick Summary**: 在 ForceLoop 中实现一套**自有 command schema**（`CommandSchema` struct），把 `CommandMetadata::skill_template()` / `command_template()` 的返回类型从 `&'static str` 改为 `CommandSchema`。新增 `compiler` 模块把 schema 编译成 **Claude Code** 和 **sst/opencode** 两种原生 markdown 文件格式（带 YAML frontmatter）。所有 10 个 Command 对象（4 个 subcommand + 6 个 skill/command）按 TDD 顺序改造。
>
> **Deliverables**:
> - 新模块 `src/schema.rs` — `CommandSchema` 数据结构 + 单元测试
> - 新模块 `src/compiler.rs` — `Target` enum + `compile()` 函数 + 双目标模板 + 单元测试
> - `src/traits.rs` — `CommandMetadata` 返回类型从 `&'static str` 改为 `CommandSchema`
> - 10 个 Command 实现文件 — 改为返回 `CommandSchema`（先用 `default()` 占位，骨架先行）
> - 集成测试 `tests/command_compile.rs` — 端到端验证 schema → markdown
>
> **Estimated Effort**: Small-Medium (4-6 files changed, 2 new modules, ~200 lines production + ~150 lines tests)
> **Parallel Execution**: NO (sequential, trait signature change cascades)
> **Critical Path**: schema.rs → compiler.rs → traits.rs → 10 impls → integration test → commit

---

## Context

### Background

[`.omc/plans/command-metadata-skeleton.md`](command-metadata-skeleton.md) 已为所有 10 个 Command 对象添加了 `CommandMetadata` trait 骨架，方法 `skill_template()` / `command_template()` 当前返回空字符串。

[`docs/command-format-comparison.html`](../../docs/command-format-comparison.html) 调研了 Claude Code 和 sst/opencode 两边自定义命令的格式：
- **核心结论**：两边底层都是「YAML frontmatter + markdown body」结构，可以共用
- **最大公约数字段**：`name` / `description` / `model` / `argument-hint` / 工具白名单（语法不同）
- **最低成本方案**：写一份 ForceLoop 自己的 schema + 编译层，转成两边原生格式

### Original Request（用户原话）

> 根据 @docs/command-format-comparison.html 调查的结果。先实现一个自己的 schema，在 CommandMetadata 里返回 skill_template 和 command_template 的时候返回自己的schema，然后在实际输出的时候，用各自的模版格式化成实际的文件内容

### Why Now

- `CommandMetadata` trait 当前不可用（空字符串）
- 不实现 schema 就无法把 ForceLoop 命令导出到外部平台
- 后续 `setup` / `status` 等 subcommand 需要读取 schema 才能生成平台原生文件

### Captured Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Trait return type** | 返回 `CommandSchema` (owned struct) | 类型安全；零运行时解析开销；编译期验证字段名 |
| **Schema field stringness** | 全部用 `&'static str` + `&'static [&'static str]` | 与项目现有 pattern 一致（`&'static str` constants，零分配） |
| **Target 支持** | 仅 `Claude` 和 `OpenCode` 两目标 | 调研明确；其他平台 out of scope |
| **Platform-specific overrides** | 暂不支持（最小可用） | YAGNI；统一字段已覆盖 80% 用例 |
| **Skill vs Command 区分** | 同一 `CommandSchema` struct，body 内容不同 | 调研结论：两边都把 skill/command 视为同一类对象 |
| **测试方法** | TDD (tests first)，unit + integration | 项目规约强制 |
| **是否新增 CLI 子命令** | ❌ 不新增 | 用户未要求；当前只暴露 `compile()` 库函数；后续 `setup` 子命令会调用 |

---

## Work Objectives

### Core Objective

把「ForceLoop 自有 schema」作为命令声明的**单一事实源 (single source of truth)**，并通过编译层无损地输出为 Claude Code 和 sst/opencode 两种平台原生 markdown 文件。

### Concrete Deliverables

1. **`src/schema.rs`** (新) — `CommandSchema` 数据结构
2. **`src/compiler.rs`** (新) — `Target` enum + `compile_to_claude()` + `compile_to_opencode()`
3. **`src/traits.rs`** (改) — `skill_template()` / `command_template()` 返回 `CommandSchema`
4. **10 个 Command 实现文件** (改) — 改为返回 `CommandSchema`（先 `default()` 占位）
5. **`src/lib.rs`** (改) — 注册 `pub mod schema;` `pub mod compiler;`
6. **`tests/command_compile.rs`** (新) — 集成测试

### Definition of Done

- [ ] `CommandSchema` struct 定义，字段与 [HTML §5 通用子集](../../docs/command-format-comparison.html) 一致
- [ ] `Target { Claude, OpenCode }` enum
- [ ] `compile(schema, target) -> Result<String>` 公开函数
- [ ] `compile_to_claude()` 输出含 `description`, `allowed-tools`, `argument-hint`, `model` 字段
- [ ] `compile_to_opencode()` 输出含 `description`, `agent`, `model` 字段
- [ ] `CommandMetadata` trait 返回类型从 `&'static str` 改为 `CommandSchema`
- [ ] 10 个 Command impl 全部更新（用 `CommandSchema::default()` 占位）
- [ ] 单元测试覆盖：schema 构造、Claude 编译、OpenCode 编译、空 schema 边界
- [ ] 集成测试：构造真实 schema → 调用 compile → 断言输出格式
- [ ] `cargo check` 零错误零警告
- [ ] `cargo test` 全部通过
- [ ] `cargo clippy --all-targets` 零问题
- [ ] 已有测试（12 个 wiki-link 单元测试）无回归
- [ ] 已提交

### Must Have

- `CommandSchema` 全部字段用 `&'static str` (或 `Option<&'static str>`) — 零堆分配
- `compile()` 函数是**纯函数**（输入决定输出，无 IO 无副作用）
- 单元测试在 `#[cfg(test)] mod tests` 块中
- 公开 API `pub` 修饰
- 使用 `crate::errors::Result` 传播错误
- 错误类型用 `ForceLoopError::Parse(String)` / `Execution(String)` 现有 variant
- YAML frontmatter 序列化用手写（不引入 `serde_yaml` 依赖，与项目"零新增运行时依赖"原则一致）

### Must NOT Have (Guardrails)

- **不**引入新运行时依赖（`serde_yaml` / `yaml-rust` / `tera` / `handlebars`）
- **不**实现 `Display` / `Serialize` 给 `CommandSchema`（保持最小；后续需要时再加）
- **不**写文件（`compile` 只返回 string，IO 由调用方负责）
- **不**做 YAML 转义复杂度优化（直接 `format!()` 字符串拼接，描述字段如有引号需要转义在调用方处理）
- **不**新增 CLI 子命令（`forceloop command compile` out of scope）
- **不**实现 platform-specific overrides（YAGNI）
- **不**实现 schema 持久化（不读写 .yaml/.json 文件）

---

## Design Decisions

### Decision 1: Trait 返回 `CommandSchema` 而非 `&'static str`

**决策**: 改 trait 签名
```rust
pub trait CommandMetadata {
    fn skill_template(&self) -> CommandSchema;
    fn command_template(&self) -> CommandSchema;
    fn artifacts(&self) -> &[&'static str];
    fn gate(&self, ctx: &Context) -> Result<()>;
}
```

**理由**:
- **类型安全**：编译期保证字段名拼写正确
- **零解析**：调用方拿到的是结构体，不是需要解析的字符串
- **可演进**：未来加字段不破坏 ABI
- **与项目一致**：trait 里其他方法（如 `Subcommand::name() -> &'static str`）也走类型化方向

**取舍**:
- 需要更新 10 个 impl（一次性，可控）
- `CommandSchema` 包含 7 个字段（不算大），按值返回成本低

### Decision 2: 字段全用 `&'static str`，零分配

```rust
pub struct CommandSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub model: Option<&'static str>,
    pub argument_hint: Option<&'static str>,
    pub tools: &'static [&'static str],   // 空 slice = 不限制
    pub agent: Option<&'static str>,
    pub prompt: &'static str,
}
```

**理由**:
- 项目规约：`&'static str` 是 path 字段首选（见 `src/constants.rs`）
- 10 个 Command impl 中所有字符串字面量都是 `'static`
- 零堆分配 → 编译时 `const` 表达式友好
- 与现有 `artifacts() -> &[&'static str]` 风格统一

**取舍**:
- 不支持运行时构造（这超出当前需求）
- 字符串转义（如描述含 `"`）由调用方负责

### Decision 3: 手写 YAML 序列化，不引入依赖

**决策**: 用 `format!()` + 字符串拼接生成 YAML frontmatter

**理由**:
- 项目规约：尽可能 hand-roll（wiki link validator 是先例）
- YAML frontmatter 结构简单（5-7 个字段），模板化输出 < 50 行
- 避免新增 `serde_yaml`（约 80 个 transitive deps）
- 字段名都是 `&'static str` → 不需要 escape

**取舍**:
- 不支持嵌套结构 / 复杂 YAML（当前不需要）
- 描述字段如含 `:` 或 `#` 需要 quote——简单规则：含特殊字符就用双引号包裹

### Decision 4: `Target` enum 而非字符串

```rust
pub enum Target {
    Claude,
    OpenCode,
}
```

**理由**:
- 编译期检查，调用方无法传入 `Target::Gpt` 这种东西
- 易于扩展（未来加 `Target::Cursor` 只需加一个 variant）
- pattern match 强制处理所有目标

### Decision 5: 单元测试 + 集成测试双层

- **单元测试** (`src/compiler.rs` 内部 `#[cfg(test)] mod tests`): 覆盖 `compile_to_claude` / `compile_to_opencode` 的各种字段组合
- **集成测试** (`tests/command_compile.rs`): 端到端「构造 schema → 调 compile → 字符串比较」

**理由**:
- 单元测试：快，定位精确
- 集成测试：验证公共 API 形态稳定

---

## Public API

### Schema (`src/schema.rs`)

```rust
/// ForceLoop-native command/skill schema.
/// Single source of truth — compiled to platform-native formats via `crate::compiler`.
#[derive(Debug, Clone, Copy)]
pub struct CommandSchema {
    /// Command identifier (e.g. "code-review", "implement").
    pub name: &'static str,

    /// Short human-readable description (used for `description` frontmatter).
    pub description: &'static str,

    /// Model identifier (e.g. "opus", "sonnet"). None = use platform default.
    pub model: Option<&'static str>,

    /// Argument hint for slash command (e.g. "[file] [query]"). Claude only.
    pub argument_hint: Option<&'static str>,

    /// Whitelist of tool names this command may use (e.g. `["Read", "Grep", "Bash"]`).
    /// Empty slice = no restriction. Compiled to `allowed-tools` in Claude,
    /// dropped (with warning in future) in OpenCode command body.
    pub tools: &'static [&'static str],

    /// Name of the sub-agent to delegate to (OpenCode only). None = execute inline.
    pub agent: Option<&'static str>,

    /// Body markdown — the actual prompt/workflow definition.
    /// Supports `$ARGUMENTS` placeholder (both platforms).
    pub prompt: &'static str,
}

impl Default for CommandSchema {
    fn default() -> Self {
        Self {
            name: "",
            description: "",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt: "",
        }
    }
}
```

### Compiler (`src/compiler.rs`)

```rust
use crate::errors::Result;
use crate::schema::CommandSchema;

/// Target platform for compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Anthropic Claude Code (`.claude/commands/<name>.md`)
    Claude,
    /// sst/opencode v2 (`.opencode/command/<name>.md`)
    OpenCode,
}

/// Compile a ForceLoop schema into a platform-native markdown file.
///
/// Returns the full file content (YAML frontmatter + body markdown).
/// Pure function — no IO, no side effects.
pub fn compile(schema: &CommandSchema, target: Target) -> Result<String> {
    match target {
        Target::Claude => compile_to_claude(schema),
        Target::OpenCode => compile_to_opencode(schema),
    }
}

fn compile_to_claude(schema: &CommandSchema) -> Result<String>;
fn compile_to_opencode(schema: &CommandSchema) -> Result<String>;
```

### Trait Change (`src/traits.rs`)

```rust
use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;

pub trait Executable {
    fn execute(&self, ctx: &Context) -> Result<()>;
}

pub trait Subcommand: Executable {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

pub trait CommandMetadata {
    /// Returns the ForceLoop-native schema for the Skill variant of this command.
    fn skill_template(&self) -> CommandSchema;

    /// Returns the ForceLoop-native schema for the Command variant of this command.
    fn command_template(&self) -> CommandSchema;

    fn artifacts(&self) -> &[&'static str];
    fn gate(&self, ctx: &Context) -> Result<()>;
}
```

---

## Compiler Output Format

### Claude Code

```yaml
---
description: <quoted description>
allowed-tools: [<tool1>, <tool2>, ...]   # omitted if tools is empty
argument-hint: "<hint>"                   # omitted if None
model: <model>                            # omitted if None
---

<prompt body>
```

**Rules**:
- `description` 始终输出（即使是空字符串）
- `allowed-tools` 列表化（YAML list 风格，调研建议）
- `argument-hint` / `model` 仅在 `Some` 时输出
- `agent` 字段**不输出**（Claude 无此概念；如果 Schema 有 agent 值，编译时忽略，行为等同 inline）
- Body 直接是 `prompt`

### sst/opencode

```yaml
---
description: <quoted description>
agent: <agent-name>                      # omitted if None
model: <model>                           # omitted if None
---

<prompt body>
```

**Rules**:
- `description` 始终输出
- `agent` / `model` 仅在 `Some` 时输出
- `allowed-tools` / `argument-hint` **不输出**（OpenCode command 不支持）
- `tools` 字段 schema 有但 OpenCode command 暂不支持——忽略（未来可透传到所委派 agent 的 `permissions`）
- Body 直接是 `prompt`（OpenCode 内置的 `!` `` ` `` shell escape 和 `@file/path` 引用保留）

---

## Algorithm

```
compile(schema, target):
    match target:
        Claude => build frontmatter with [description, allowed-tools, argument-hint, model]
                  + body = schema.prompt
        OpenCode => build frontmatter with [description, agent, model]
                    + body = schema.prompt
    return "---\n<frontmatter>\n---\n\n<body>"

build_frontmatter_claude(schema):
    parts = ["description: \"<quote>\""]   # always
    if not empty(schema.tools):
        parts.push("allowed-tools: [" + join(", ") + "]")
    if let Some(hint) = schema.argument_hint:
        parts.push("argument-hint: \"<hint>\"")
    if let Some(model) = schema.model:
        parts.push("model: <model>")
    return parts.join("\n")

build_frontmatter_opencode(schema):
    parts = ["description: \"<quote>\""]   # always
    if let Some(agent) = schema.agent:
        parts.push("agent: <agent>")
    if let Some(model) = schema.model:
        parts.push("model: <model>")
    return parts.join("\n")
```

**Quote 规则**:
- 描述为空 → `"\"\""`
- 描述含 `\n` / `"` / `:` → 双引号包裹 + 转义内部 `"` 为 `\\\"`
- 否则 → 双引号包裹即可

简单实现即可，不需要完整 YAML 解析。函数内部局部 helper `quote(s: &str) -> String`。

---

## Implementation Steps

### Step 1: Add `src/schema.rs` (TDD)

**File**: [src/schema.rs](../../src/schema.rs) (NEW)

**Tests first** (`#[cfg(test)] mod tests`):
```rust
#[test]
fn test_default_schema() {
    let s = CommandSchema::default();
    assert_eq!(s.name, "");
    assert_eq!(s.description, "");
    assert!(s.model.is_none());
    assert!(s.tools.is_empty());
    assert!(s.agent.is_none());
    assert_eq!(s.prompt, "");
}

#[test]
fn test_schema_construction() {
    static TOOLS: &[&str] = &["Read", "Bash"];
    let s = CommandSchema {
        name: "code-review",
        description: "Review changed files",
        model: Some("opus"),
        argument_hint: Some("[files...]"),
        tools: TOOLS,
        agent: Some("reviewer"),
        prompt: "You are a reviewer...",
    };
    assert_eq!(s.name, "code-review");
    assert_eq!(s.tools.len(), 2);
}
```

**Production code**: struct + `Default` impl (above).

**Acceptance**:
- `cargo test schema::tests` 全部通过
- 字段类型与设计一致

### Step 2: Add `src/compiler.rs` (TDD)

**File**: [src/compiler.rs](../../src/compiler.rs) (NEW)

**Tests first**:
```rust
#[test]
fn test_compile_claude_minimal() {
    let s = CommandSchema {
        name: "code-review",
        description: "Review changes",
        model: None,
        argument_hint: None,
        tools: &[],
        agent: None,
        prompt: "You are a reviewer.",
    };
    let out = compile(&s, Target::Claude).unwrap();
    assert!(out.starts_with("---\ndescription: \"Review changes\"\n---\n"));
    assert!(out.ends_with("\nYou are a reviewer.\n"));
}

#[test]
fn test_compile_claude_with_tools() {
    static TOOLS: &[&str] = &["Read", "Bash"];
    let s = CommandSchema {
        name: "test",
        description: "d",
        model: None,
        argument_hint: None,
        tools: TOOLS,
        agent: None,
        prompt: "p",
    };
    let out = compile(&s, Target::Claude).unwrap();
    assert!(out.contains("allowed-tools: [Read, Bash]"));
}

#[test]
fn test_compile_claude_with_hint_and_model() {
    let s = CommandSchema {
        name: "x",
        description: "d",
        model: Some("opus"),
        argument_hint: Some("[file]"),
        tools: &[],
        agent: None,
        prompt: "p",
    };
    let out = compile(&s, Target::Claude).unwrap();
    assert!(out.contains("argument-hint: \"[file]\""));
    assert!(out.contains("model: opus"));
}

#[test]
fn test_compile_opencode_with_agent() {
    let s = CommandSchema {
        name: "x",
        description: "d",
        model: None,
        argument_hint: None,
        tools: &[],
        agent: Some("reviewer"),
        prompt: "p",
    };
    let out = compile(&s, Target::OpenCode).unwrap();
    assert!(out.contains("agent: reviewer"));
    assert!(!out.contains("allowed-tools"));   // OpenCode 不支持
    assert!(!out.contains("argument-hint"));  // OpenCode 不支持
}

#[test]
fn test_compile_opencode_drops_tools() {
    // schema.tools 在 OpenCode 输出中被忽略（command 不支持，agent 透传未来实现）
    static TOOLS: &[&str] = &["Read"];
    let s = CommandSchema {
        name: "x",
        description: "d",
        model: None,
        argument_hint: None,
        tools: TOOLS,
        agent: Some("a"),
        prompt: "p",
    };
    let out = compile(&s, Target::OpenCode).unwrap();
    assert!(!out.contains("allowed-tools"));
    assert!(!out.contains("tools:"));
}

#[test]
fn test_compile_preserves_prompt() {
    let prompt = "# Step 1\nDo thing.\n\n## Step 2\nDo other.";
    let s = CommandSchema { name: "x", description: "d", model: None, argument_hint: None, tools: &[], agent: None, prompt };
    let claude = compile(&s, Target::Claude).unwrap();
    let opencode = compile(&s, Target::OpenCode).unwrap();
    assert!(claude.contains(prompt));
    assert!(opencode.contains(prompt));
}

#[test]
fn test_quote_description() {
    // 描述含冒号和双引号时正确转义
    let s = CommandSchema {
        name: "x",
        description: "He said: \"hello\"",
        model: None, argument_hint: None, tools: &[], agent: None, prompt: "p",
    };
    let out = compile(&s, Target::Claude).unwrap();
    // 内部 " 转义为 \"
    assert!(out.contains("description: \"He said: \\\"hello\\\"\""));
}
```

**Production code**: `Target` enum + `compile()` + 2 个 `compile_to_*` + `quote()` helper。

**Acceptance**:
- 7 个单元测试全部通过
- 无 panic / unwrap

### Step 3: Register modules in `src/lib.rs`

**File**: [src/lib.rs](../../src/lib.rs)

**改动**:
```rust
pub mod archive;
pub mod cli;
pub mod commands;
pub mod compiler;     // NEW
pub mod constants;
pub mod context;
pub mod errors;
pub mod gate;
pub mod schema;       // NEW
pub mod setup;
pub mod status;
pub mod traits;
pub mod utils;
```

（按字母顺序插入）

### Step 4: Update `src/traits.rs`

**File**: [src/traits.rs](../../src/traits.rs)

**改动**:
- 加 `use crate::schema::CommandSchema;`
- `skill_template()` / `command_template()` 返回类型改 `CommandSchema`

### Step 5: Update 10 个 Command impl（骨架先 default()）

**Files**:
- [src/commands/audit.rs](../../src/commands/audit.rs)
- [src/commands/implement.rs](../../src/commands/implement.rs)
- [src/commands/new_cmd.rs](../../src/commands/new_cmd.rs)
- [src/commands/plan.rs](../../src/commands/plan.rs)
- [src/commands/review.rs](../../src/commands/review.rs)
- [src/commands/try_finish.rs](../../src/commands/try_finish.rs)
- [src/setup.rs](../../src/setup.rs)
- [src/gate.rs](../../src/gate.rs)
- [src/status.rs](../../src/status.rs)
- [src/archive.rs](../../src/archive.rs)

**每个文件改动**:
```rust
// before:
fn skill_template(&self) -> &'static str { "" }
fn command_template(&self) -> &'static str { "" }

// after:
fn skill_template(&self) -> CommandSchema { CommandSchema::default() }
fn command_template(&self) -> CommandSchema { CommandSchema::default() }
```

加 `use crate::schema::CommandSchema;` 到每个文件。

> **Note**: 这是骨架——所有 Command 暂时返回 `default()`。后续任务会填充真实 schema。

### Step 6: Add integration test `tests/command_compile.rs`

**File**: [tests/command_compile.rs](../../tests/command_compile.rs) (NEW)

```rust
use forceloop::compiler::{compile, Target};
use forceloop::schema::CommandSchema;

#[test]
fn end_to_end_claude() {
    static TOOLS: &[&str] = &["Read", "Grep", "Bash"];
    let s = CommandSchema {
        name: "code-review",
        description: "Review changes with severity",
        model: Some("opus"),
        argument_hint: Some("[files...]"),
        tools: TOOLS,
        agent: None,    // Claude 不委派
        prompt: "You are Code Reviewer.\n\nSeverity: CRITICAL > HIGH > MEDIUM > LOW",
    };
    let out = compile(&s, Target::Claude).unwrap();
    
    // Frontmatter 校验
    assert!(out.contains("description: \"Review changes with severity\""));
    assert!(out.contains("allowed-tools: [Read, Grep, Bash]"));
    assert!(out.contains("argument-hint: \"[files...]\""));
    assert!(out.contains("model: opus"));
    // Body 校验
    assert!(out.contains("Severity: CRITICAL > HIGH > MEDIUM > LOW"));
    // 不应出现 OpenCode 字段
    assert!(!out.contains("agent:"));
}

#[test]
fn end_to_end_opencode() {
    let s = CommandSchema {
        name: "code-review",
        description: "Review changes",
        model: Some("opus"),
        argument_hint: Some("[files...]"),   // OpenCode 不支持
        tools: &[],                          // OpenCode command 暂不支持
        agent: Some("reviewer"),
        prompt: "Delegate to reviewer agent.",
    };
    let out = compile(&s, Target::OpenCode).unwrap();
    
    assert!(out.contains("description: \"Review changes\""));
    assert!(out.contains("agent: reviewer"));
    assert!(out.contains("model: opus"));
    // OpenCode 不支持的字段不应出现
    assert!(!out.contains("allowed-tools"));
    assert!(!out.contains("argument-hint"));
    assert!(!out.contains("tools:"));
}
```

### Step 7: Verification

```bash
cargo check
cargo test
cargo clippy --all-targets
```

**Expected**:
- 零编译错误零警告
- 全部测试通过 (12 wiki-link 单元测试 + 7 schema 单元测试 + 7 compiler 单元测试 + 2 集成测试 = 28 tests)
- clippy 零问题

### Step 8: Commit

```
feat(core): add CommandSchema and multi-target compiler

Add a ForceLoop-native command schema (CommandSchema) as the single source
of truth for command/skill declarations. The CommandMetadata trait now
returns CommandSchema from skill_template() and command_template().

Add a compiler module that converts a CommandSchema into platform-native
markdown files (YAML frontmatter + body):
- Target::Claude → .claude/commands/<name>.md (description, allowed-tools,
  argument-hint, model)
- Target::OpenCode → .opencode/command/<name>.md (description, agent, model)

Hand-rolled YAML serialization (no serde_yaml dep, per project convention).

All 10 Command objects updated to return CommandSchema::default() (skeleton);
real schemas to be filled in follow-up tasks.

Tests: 7 unit (schema) + 7 unit (compiler) + 2 integration = 16 new tests.
No regression in existing 12 wiki-link tests.

Files:
- src/schema.rs (new, ~50 lines)
- src/compiler.rs (new, ~120 lines + ~150 lines tests)
- src/lib.rs (register 2 new modules)
- src/traits.rs (return type change)
- src/{setup,gate,status,archive}.rs (4 files, signature update)
- src/commands/{audit,implement,new_cmd,plan,review,try_finish}.rs (6 files)
- tests/command_compile.rs (new, ~60 lines)
```

---

## File-by-File Change Summary

| File | Action | Lines (prod) | Lines (test) | Notes |
|------|--------|--------------|--------------|-------|
| `src/schema.rs` | NEW | ~40 | ~30 | struct + Default + 2 tests |
| `src/compiler.rs` | NEW | ~80 | ~120 | Target + compile + 2 helpers + 7 tests |
| `src/lib.rs` | edit | +2 | 0 | register modules |
| `src/traits.rs` | edit | +2 / -2 | 0 | return type change |
| `src/setup.rs` | edit | +2 / -2 | 0 | impl update |
| `src/gate.rs` | edit | +2 / -2 | 0 | impl update |
| `src/status.rs` | edit | +2 / -2 | 0 | impl update |
| `src/archive.rs` | edit | +2 / -2 | 0 | impl update |
| `src/commands/audit.rs` | edit | +2 / -2 | 0 | impl update |
| `src/commands/implement.rs` | edit | +2 / -2 | 0 | impl update |
| `src/commands/new_cmd.rs` | edit | +2 / -2 | 0 | impl update |
| `src/commands/plan.rs` | edit | +2 / -2 | 0 | impl update |
| `src/commands/review.rs` | edit | +2 / -2 | 0 | impl update |
| `src/commands/try_finish.rs` | edit | +2 / -2 | 0 | impl update |
| `tests/command_compile.rs` | NEW | 0 | ~60 | 2 integration tests |
| **Total** | 13 files (2 new) | **~140** | **~210** | |

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Trait 签名变化破坏外部调用 | 中 | 低 | 10 个 impl 一次 commit 改完；项目无外部调用方（Phase 2 才用） |
| `CommandSchema` 字段顺序影响 `Debug` 输出 | 低 | 中 | 测试断言用 `contains` 而非精确字符串匹配 |
| YAML quote 转义不完整 | 中 | 中 | 实现 `quote()` helper 处理 `"` `:` `\n`；单元测试覆盖 |
| OpenCode `tools` 字段丢失 | 中 | 高 | 已知 trade-off；注释说明未来需透传到 agent；不阻塞当前任务 |
| Future schema 字段添加破坏 ABI | 低 | 中 | 7 字段都 optional 或有 default；后续 add new field 不破坏现有用法 |
| Skill vs Command 区分不清 | 中 | 中 | 同一 struct，body 内容不同；后续 task 用 `Skill` 包装 / 标记 |

---

## Verification Strategy

### Build Verification
```bash
cargo check                    # 零错误零警告
cargo test                     # 全部通过
cargo clippy --all-targets     # 零问题
```

### Test Scenarios (16 new + 12 existing = 28 total)

**Unit tests in `src/schema.rs`** (2):
1. `test_default_schema` — Default 行为
2. `test_schema_construction` — 显式构造

**Unit tests in `src/compiler.rs`** (7):
3. `test_compile_claude_minimal` — 只 description + prompt
4. `test_compile_claude_with_tools` — `allowed-tools` 列表化
5. `test_compile_claude_with_hint_and_model` — 可选字段
6. `test_compile_opencode_with_agent` — agent 委派
7. `test_compile_opencode_drops_tools` — tools 字段被忽略
8. `test_compile_preserves_prompt` — body 内容透传
9. `test_quote_description` — 特殊字符 quote

**Integration tests in `tests/command_compile.rs`** (2):
10. `end_to_end_claude` — 真实 schema → Claude markdown
11. `end_to_end_opencode` — 真实 schema → OpenCode markdown

**Regression** (12 existing):
- 12 个 wiki-link 单元测试（无回归）

### Manual Verification
```bash
# 简单 smoke：编译一个示例 schema 看看输出
cat > /tmp/test_schema.rs <<'EOF'
use forceloop::compiler::{compile, Target};
use forceloop::schema::CommandSchema;

fn main() {
    let s = CommandSchema {
        name: "code-review",
        description: "Review changed files",
        model: Some("opus"),
        argument_hint: Some("[files...]"),
        tools: &["Read", "Bash"],
        agent: None,
        prompt: "You are a reviewer.",
    };
    println!("{}", compile(&s, Target::Claude).unwrap());
    println!("---\n{}", compile(&s, Target::OpenCode).unwrap());
}
EOF
# 集成到现有 main.rs 临时跑一下，或写个 example/
```

---

## Out of Scope (Follow-ups)

以下**不在本计划范围**，留待后续任务：

1. **填充真实 schema**：所有 10 个 Command 当前返回 `default()`，需要后续 task 填写真实 name/description/prompt
2. **Platform-specific overrides**：未来需要 `Schema { ..., platforms: { claude: {...}, opencode: {...} } }`
3. **Schema 序列化**（`Serialize` trait）：未来需要从 .yaml 文件读 schema 时加
4. **CLI 子命令** `forceloop command compile <name> --target <platform>`：未来 `setup` 子命令会包装
5. **MCP 工具引用**（`mcp__server__tool`）：当前 schema 没有专门字段，未来加 `tools_mcp: &[&'static str]`
6. **OpenCode `tools` 透传**：当前被忽略，未来需要时透传到 `agent.permissions`
7. **`shell escape` (`!` `` ` ``) 处理**：当前是字符串透传，OpenCode 自带解析
8. **动态 schema 加载**：当前是编译期 const，运行时从文件加载是另一个任务

---

## Final Checklist

- [ ] `src/schema.rs` 创建并通过 2 个单元测试
- [ ] `src/compiler.rs` 创建并通过 7 个单元测试
- [ ] `src/lib.rs` 注册新模块
- [ ] `src/traits.rs` 返回类型从 `&'static str` 改为 `CommandSchema`
- [ ] 10 个 Command impl 全部更新（用 `CommandSchema::default()`）
- [ ] `tests/command_compile.rs` 创建并通过 2 个集成测试
- [ ] `cargo check` 零错误零警告
- [ ] `cargo test` 全部 28 个测试通过
- [ ] `cargo clippy --all-targets` 零问题
- [ ] 无 `unwrap()` / `panic!()` 在生产代码
- [ ] 无新增运行时依赖
- [ ] 无业务逻辑耦合
- [ ] 遵循项目 `&'static str` path 约定
- [ ] 遵循 TDD 规约（tests first）
- [ ] 已 git commit
- [ ] `CLAUDE.md` 无需更新（本任务不引入新约定）
