# ForceLoop `setup` 同时写入斜杠 Command 与 Claude Skill 文件

## TL;DR

> **Quick Summary**: 让 `forceloop setup` 在生成 `.claude/commands/<name>.md` 的同时，也生成 `.claude/skills/<name>/SKILL.md`（Claude 平台独有；OpenCode v2 没有 skills 概念）。新增 `compile_skill()` 编译器函数；`run()` 一次迭代 `COMMANDS` 表同时产出 commands 与 skills。
>
> **核心设计**: **单表 + 三元组** — 每个 entry 自带 `command` 和 `skill` 两个模板，零表间同步风险。`target_skill_dir() -> Option<PathBuf>` 让"OpenCode 无 skills"用类型系统表达（无 panic）。
>
> **Deliverables**:
> - `src/compiler.rs` — 新增 `pub fn compile_skill(schema: &CommandSchema) -> Result<String>`，使用 Claude Skill frontmatter（`name:` / `description:` / 可选 `model:`）
> - `src/setup.rs` — `CommandEntry` 类型从 2 元组改为 3 元组 `(name, cmd_fn, skill_fn)`；`COMMANDS` 表每个 entry 同时调 `command_template()` 与 `skill_template()`；新增 `fn target_skill_dir(root, target) -> Option<PathBuf>`（无 `TargetKind` enum、无 panic）；`run()` 一次迭代 COMMANDS 同时写 commands 与 skills（skills 用 `if let Some(skill_root)` 跳过 OpenCode）
> - `tests/setup_tool.rs` — 更新计数断言（Claude 18 / OpenCode 9 / 默认 27）；新增 3 个集成测试
> - `src/compiler.rs` 单元测试 — 至少 2 个 `compile_skill` 测试（最小 schema + 含 model）
>
> **Estimated Effort**: Small-Medium (2 文件生产, 2 文件测试, ~80 行生产 + ~80 行测试)
> **Parallel Execution**: NO（顺序：`compile_skill()` 单元测试 → `compile_skill()` 实现 → `COMMANDS` 3 元组化 + `run()` 改动 → 集成测试 → 验证）
> **Critical Path**: compiler.rs → setup.rs → tests/setup_tool.rs

---

## Context

### Background

- [src/setup.rs:151-157](src/setup.rs#L151-L157) `target_subdir()` 当前只支持 `.claude/commands` 和 `.opencode/command`，无 skills 路径
- [src/setup.rs:127-138](src/setup.rs#L127-L138) `COMMANDS` 静态表只调 `command_template()`，从未调 `skill_template()`
- [src/compiler.rs:254-263](src/compiler.rs#L254-L263) 只有 `compile()`（command）和 `compile_agent()`（OpenCode agent），无 `compile_skill()`
- [docs/requirment.md:9](docs/requirment.md#L9) 需求"setup 在项目中初始化目录结构，状态数据，**子 command，Skill，hook**" —— "Skill" 未物化
- [src/setup.rs:109-110](src/setup.rs#L109-L110) `COMMANDS` 注释承诺"slash command / Skill files"两者，但实现只产出 slash command

### Original Request（用户原话 + 选择）

> "调查一下 setup.rs 是不是在执行的时候只写了斜杠 command，没有写 skills？"
>
> "B"（用户选择补 `compile_skill()` + 写 `.claude/skills/<name>/SKILL.md`）
>
> "给个更好的方案出来啊"（用户对 v1 不满意，要求重新设计）

### v1 → v2 关键变化（响应用户"更好的方案"）

| v1 问题 | v2 改进 |
|---------|---------|
| 两表分离（`COMMANDS` + `SKILL_COMMANDS`） | **单表 3 元组**（entry 自带两个模板） |
| `enum TargetKind { Command, Skill }` + 4-match | **2 个独立函数**：`target_subdir` + `target_skill_dir -> Option<PathBuf>` |
| `panic!` for `OpenCode + Skill` | **`None` for `OpenCode`**（让 `if let` 优雅跳过） |
| 18 行 fixture（两表各 9 entry） | **9 行 fixture**（一表 9 entry，每 entry 3 字段） |
| 2 个表计数单元测试 | **1 个**（单一表） |
| 加新 Command 改 2 处 | **改 1 处** |

### 用户已确认的设计决策

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **是否补 `compile_skill()`** | 是 | comment + 需求都要求两者都写 |
| **Skills 平台范围** | **仅 Claude** | OpenCode v2 没有 skills 概念 |
| **Skills 路径** | `.claude/skills/<name>/SKILL.md` | Claude Code 标准约定：每 skill 一个子目录，文件名为 `SKILL.md` |
| **`compile_skill()` 签名** | `pub fn compile_skill(schema: &CommandSchema) -> Result<String>` | 与 `compile_agent(agent_name, schema)` 风格一致；不需要 target 参数（skills 是 Claude-only） |
| **Skill frontmatter 字段** | `name:` / `description:` / 可选 `model:` | 与 Claude 官方 skill 规范一致；**不**包含 `allowed-tools` / `argument-hint` / `agent` |
| **单表 vs 双表** | **单表 3 元组**（`&'static str`, `fn() -> CommandSchema`, `fn() -> CommandSchema`） | 加新 Command 改 1 处；零同步风险 |
| **是否引入 `TargetKind` enum** | **否** | 拆成两个简单函数；`Option<PathBuf>` 表达"不支持" |
| **Setup 是否在 COMMANDS 中** | **否**（保持现状） | Setup 是 terminal-only，不物化 |
| **Setup::skill_template() impl** | **保留** | trait contract 要求 |

### Why Now

- Comment 与实现的不一致是真实的技术债
- 需求文档说"初始化 Skill"，但项目里 `.claude/skills/` 始终为空
- v1 方案（双表 + panic）违反项目规约（"重复 2 次即提取"、"禁止打补丁"），需要根因治理

---

## Work Objectives

### Core Objective

让 `forceloop setup` 在 Claude 平台同时生成 slash command 文件（`.claude/commands/<name>.md`）和 skill 文件（`.claude/skills/<name>/SKILL.md`）。OpenCode 平台仍只写 command（v2 无 skills 概念）。

### Concrete Deliverables

1. **`src/compiler.rs`** (改) — 新增 `pub fn compile_skill(schema: &CommandSchema) -> Result<String>` + 2 单元测试
2. **`src/setup.rs`** (改) — `CommandEntry` 类型从 2 元组改为 3 元组；`COMMANDS` 表每个 entry 同时调 `command_template()` 与 `skill_template()`；新增 `fn target_skill_dir(root, target) -> Option<PathBuf>`；`run()` 一次迭代 COMMANDS 同时写 commands 与 skills
3. **`tests/setup_tool.rs`** (改) — 计数断言更新；新增 3 个集成测试

### Definition of Done

- [ ] [src/compiler.rs](src/compiler.rs) 新增 `pub fn compile_skill(schema: &CommandSchema) -> Result<String>`，返回 `---\nname: <name>\ndescription: "<desc>"\n[model: <model>]\n---\n\n<schema.prompt>\n`
- [ ] `compile_skill()` 至少 2 个单元测试（最小 schema + 含 model）
- [ ] `compile_skill()` 不输出 `allowed-tools` / `argument-hint` / `agent` 字段
- [ ] [src/setup.rs](src/setup.rs) `type CommandEntry` 改为 `(&'static str, fn() -> CommandSchema, fn() -> CommandSchema)`
- [ ] `COMMANDS` 静态表每个 entry 形如 `("gate", || Gate.command_template(), || Gate.skill_template())`，9 entry（Setup 排除）
- [ ] 新增 `fn target_subdir(root: &Path, target: Target) -> PathBuf`（**保持现状**，不加 kind 参数）
- [ ] 新增 `fn target_skill_dir(root: &Path, target: Target) -> Option<PathBuf>` —— `Claude => Some(...)`，`OpenCode => None`
- [ ] `run()` 在 `if let Some(skill_root) = target_skill_dir(root, target)` 内迭代 COMMANDS 写 skill 文件，路径为 `<root>/.claude/skills/<name>/SKILL.md`（`fs::create_dir_all(path.parent().unwrap())` 确保子目录）
- [ ] `run()` 不引入新 `TargetKind` enum，无 panic
- [ ] 单元测试 `commands_table_has_nine_entries` 仍断言 `COMMANDS.len() == 9`（**不**新增 `skill_commands_table_has_nine_entries`，因为没有第二张表）
- [ ] 集成测试 `run_default_writes_both_targets` 计数从 18 改为 **27**（= 9 cmd × 2 + 9 skills × 1）；增加 `.claude/skills/gate/SKILL.md` 路径断言
- [ ] 集成测试 `claude_only_writes_claude_dir` 计数从 9 改为 **18**（= 9 cmd + 9 skills）；增加 SKILL.md 路径断言
- [ ] 集成测试 `opencode_only_writes_opencode_dir` 计数从 9 保持为 **9**（仅 commands，无 skills）
- [ ] 集成测试 `run_writes_all_nine_commands_per_target` 改名为 `run_writes_commands_and_skills_per_claude_target`；断言 Claude 跑后 9 commands + 9 skills = 18 个文件
- [ ] 集成测试 `setup_md_is_not_generated` 扩展为同时断言 `setup/SKILL.md` 不存在
- [ ] 新增集成测试 `claude_target_writes_skill_files_to_skills_subdir` —— Claude 单目标跑后，断言 9 个 `SKILL.md` 存在
- [ ] 新增集成测试 `opencode_target_does_not_write_skill_files` —— OpenCode 单目标跑后，断言 `.claude/` 不存在
- [ ] 新增集成测试 `skill_md_content_has_claude_skill_frontmatter` —— Claude 跑后读 `gate/SKILL.md`，断言 `starts_with("---\nname: gate\n")` / 含 `description:` / 不含 `allowed-tools`
- [ ] `cargo check` / `cargo test` / `cargo clippy --all-targets` 全绿
- [ ] 已提交

### Must Have

- Claude 平台两个目录都写：`.claude/commands/<name>.md` + `.claude/skills/<name>/SKILL.md`
- OpenCode 平台只写 command（v2 无 skills 概念）
- `compile_skill()` 是纯函数，独立可测
- `COMMANDS` 单表，3 元组，每个 entry 自带两个模板
- 无 `TargetKind` enum，无 panic（用 `Option<PathBuf>` 表达不支持）
- TDD：先写 `compile_skill` 单元测试，再写实现

### Must NOT Have (Guardrails)

- **不**引入第二张静态表（`SKILL_COMMANDS`）—— 单表即真相
- **不**引入 `TargetKind` enum —— 拆为两个简单函数
- **不**使用 `panic!` 或 `unreachable!` 表达"不支持" —— 用 `Option<PathBuf>`
- **不**把 skills 物化到 OpenCode 平台（v2 不支持）
- **不**改 `CommandMetadata` trait（10 个 Command impl 不动）
- **不**改 `SKILL_PROMPT` / `COMMAND_PROMPT` 文案
- **不**改 `cli.rs` / `main.rs` / `context.rs` / `compiler.rs::compile()` / `compile_agent()`

---

## Architecture Decisions

### 数据流（修改后）

```
$ forceloop setup --tool claude
       │
       ▼
[Setup::run(&[Target::Claude], &root)]
       │
       ├── target_subdir(root, Target::Claude) = ".claude/commands"
       │   target_skill_dir(root, Target::Claude) = Some(".claude/skills")
       │
       ├── for &(name, cmd_fn, skill_fn) in COMMANDS:  // 9 entry, single iteration
       │     // 1. Slash command
       │     let body = compile(&cmd_fn(), Target::Claude)?;
       │     fs::write(".claude/commands/<name>.md", body)?;
       │     // 2. Skill (Claude only, gated by Option<PathBuf>)
       │     if let Some(skill_root) = target_skill_dir(...) {
       │         let body = compile_skill(&skill_fn())?;
       │         fs::write(".claude/skills/<name>/SKILL.md", body)?;
       │     }
       │
       ▼
[SetupReport { written: 18 路径 = 9 commands + 9 skills }]
```

### 关键设计选择

#### 1. 单表 3 元组 vs 双表

**v1（双表）的问题**：
- 加新 Command 必改 2 处 —— 9 个 entry 重复 9 次（18 行 fixture）
- 编译器不强制两表同步（漏改时 `run()` 行为不一致但编译通过）
- 需要 2 个表计数单元测试

**v2（单表 3 元组）的解决**：
- 每 entry 自带两个模板；加 Command 改 1 处（9 行 fixture）
- 类型 `(&'static str, fn() -> CommandSchema, fn() -> CommandSchema)` 表达"一个 Command 包含两个模板"的本质
- 1 个表计数单元测试足够

**实现**：
```rust
type CommandEntry = (&'static str, fn() -> CommandSchema, fn() -> CommandSchema);

const COMMANDS: &[CommandEntry] = &[
    ("gate",       || Gate.command_template(),       || Gate.skill_template()),
    ("status",     || Status.command_template(),     || Status.skill_template()),
    ("archive",    || Archive.command_template(),    || Archive.skill_template()),
    ("new",        || New.command_template(),        || New.skill_template()),
    ("plan",       || Plan.command_template(),       || Plan.skill_template()),
    ("audit",      || Audit.command_template(),      || Audit.skill_template()),
    ("implement",  || Implement.command_template(),  || Implement.skill_template()),
    ("review",     || Review.command_template(),     || Review.skill_template()),
    ("try_finish", || TryFinish.command_template(),  || TryFinish.skill_template()),
];
```

闭包捕获 ZST（`Gate` 等都是 unit struct），零成本。

#### 2. `Option<PathBuf>` vs `panic!` / `TargetKind` enum

**v1 的问题**：
- `TargetKind` enum + 4-match + `panic!` 表达"OpenCode 不支持 skills" —— 过度设计
- 项目规约"禁止打补丁/Hack" —— panic 在不支持的组合上是 hack

**v2 的解决**：
- `target_skill_dir(root, target) -> Option<PathBuf>` 直接表达"可能不存在"
- `run()` 用 `if let Some(skill_root) = target_skill_dir(...)` 优雅跳过
- 类型系统保证"调用点处理 None"，无需注释

**实现**：
```rust
/// Slash command directory for a given target. Both platforms have one.
fn target_subdir(root: &Path, target: Target) -> PathBuf {
    match target {
        Target::Claude => root.join(".claude/commands"),
        Target::OpenCode => root.join(".opencode/command"),
    }
}

/// Claude skill directory. `None` for OpenCode v2 (no skills concept).
/// Returning `Option` (not panic) lets `run()` skip the skill write
/// with a single `if let`.
fn target_skill_dir(root: &Path, target: Target) -> Option<PathBuf> {
    match target {
        Target::Claude => Some(root.join(".claude/skills")),
        Target::OpenCode => None,
    }
}
```

#### 3. 一次迭代 vs 两次迭代

**v1 的做法**：
- `for (name, cmd_fn) in COMMANDS { write_command(); }`
- `if target == Claude { for (name, skill_fn) in SKILL_COMMANDS { write_skill(); } }`
- 两次循环 + 两次 `fs::create_dir_all`

**v2 的做法**：
- `for &(name, cmd_fn, skill_fn) in COMMANDS { write_command(); if let Some(skill_root) = ... { write_skill(); } }`
- 一次循环；commands 写完后立即在同一 entry 内写 skills（局部性更好）

**优势**：
- 每个 entry 的"写两个文件"动作原子地聚在一起
- 未来加新文件类型（如 agent.md）只需再加一个 `if let`

**权衡**：
- 闭包内嵌套 `if let` 略增加行数（每 entry 5 行 vs 2 行），但信息密度更高

#### 4. `compile_skill()` 独立函数

- 现有 `compile(schema, target)` 内部按 target 分发（Claude / OpenCode 两种 command 格式）
- Skills 是 Claude-only，不需要 target 参数
- 独立函数 `compile_skill(schema)` 简化签名；与 `compile_agent(agent_name, schema)` 风格一致
- 未来若 OpenCode v3 引入 skills 概念，新增 `compile_skill_for_opencode()` 即可

#### 5. `SKILL.md` 路径：每 skill 一个子目录

```rust
let path = skill_root.join(name).join("SKILL.md");
fs::create_dir_all(path.parent().unwrap())?;
```

Claude Code 平台规范：`.claude/skills/<skill-name>/SKILL.md`。`fs::create_dir_all(path.parent())` 确保 `<skill-name>/` 子目录存在（即使 `skills/` 已创建）。

#### 6. Skill frontmatter 最小化

```yaml
---
name: gate
description: "Gate control command, typically invoked by hooks"
model: opus  # optional
---

[skill body — usually more detailed than command]
```

**不**包含：
- `allowed-tools:` —— skills 通过 Skill tool 加载，权限在调用方控制
- `argument-hint:` —— skills 不是 slash command，无 hint
- `agent:` —— OpenCode 概念；skills 是 Claude 概念

---

## Implementation Steps

### Step 1: `src/compiler.rs` — `compile_skill()` (TDD)

**测试先**（在 `src/compiler.rs` 的 `#[cfg(test)] mod tests` 末尾新增）：

```rust
#[test]
fn test_compile_skill_minimal() {
    let s = CommandSchema {
        name: "gate",
        description: "Gate control command",
        model: None,
        argument_hint: None,
        tools: &["Read"],  // present but should be DROPPED in skill format
        agent: None,
        prompt: "# Gate\n\nVerify next step.",
    };
    let out = compile_skill(&s).unwrap();
    assert!(out.starts_with("---\nname: gate\n"));
    assert!(out.contains("description: \"Gate control command\""));
    // Skills do NOT emit allowed-tools, argument-hint, or agent
    assert!(!out.contains("allowed-tools"));
    assert!(!out.contains("argument-hint"));
    assert!(!out.contains("agent:"));
    assert!(!out.contains("model:"));  // None → omitted
    assert!(out.contains("\n---\n"));
    assert!(out.ends_with("\n# Gate\n\nVerify next step.\n"));
}

#[test]
fn test_compile_skill_with_model() {
    let s = CommandSchema {
        name: "review",
        description: "Review changes",
        model: Some("opus"),
        argument_hint: None,
        tools: &[],
        agent: None,
        prompt: "Review code.",
    };
    let out = compile_skill(&s).unwrap();
    assert!(out.contains("model: opus"));
    assert!(out.starts_with("---\nname: review\n"));
}
```

**实现**（在 `src/compiler.rs` 中 `compile_agent` 之后 / `compile_to_opencode` 之前）：

```rust
/// Compile a ForceLoop schema into a Claude Skill file.
///
/// Emits `.claude/skills/<name>/SKILL.md` with the Claude Code
/// skill frontmatter spec (name + description + optional model).
///
/// Skills differ from slash commands:
/// - No `allowed-tools` (skill permissions are caller-controlled)
/// - No `argument-hint` (skills aren't slash commands)
/// - No `agent` (Claude-specific concept)
///
/// Use this when registering a Command's detailed workflow
/// (`skill_template()`) for the Claude Skill tool. The body reuses
/// `schema.prompt` as the skill system prompt.
///
/// Pure function — no IO, no side effects.
pub fn compile_skill(schema: &CommandSchema) -> Result<String> {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("name: {}", schema.name));
    parts.push(format!("description: {}", quote(schema.description)));
    if let Some(model) = schema.model {
        parts.push(format!("model: {}", model));
    }
    let front = parts.join("\n");
    Ok(format!("---\n{}\n---\n\n{}\n", front, schema.prompt))
}
```

### Step 2: `src/setup.rs` — `CommandEntry` 3 元组化 + `COMMANDS` 重写

```rust
/// One entry per registered Command. Each entry holds BOTH the
/// short invocation body (slash command) and the detailed workflow
/// body (skill), so the table is the single source of truth.
///
/// `CommandSchema` is `Copy`, so the factories are zero-cost.
type CommandEntry = (&'static str, fn() -> CommandSchema, fn() -> CommandSchema);

/// 9 non-setup Commands registered as platform-native slash
/// commands and (for Claude) skills.
///
/// `Setup` is intentionally excluded: it is a terminal-only
/// subcommand for project initialization, not a runtime-invokable
/// command/skill. Including it would write `setup.md` /
/// `setup/SKILL.md` to the platform directories, surfacing entries
/// in the IDE command palette that should never be clicked.
///
/// Adding a new Command (other than `Setup`) requires ONE new row
/// here. The compile pipeline picks up both templates automatically.
const COMMANDS: &[CommandEntry] = &[
    ("gate",       || Gate.command_template(),       || Gate.skill_template()),
    ("status",     || Status.command_template(),     || Status.skill_template()),
    ("archive",    || Archive.command_template(),    || Archive.skill_template()),
    ("new",        || New.command_template(),        || New.skill_template()),
    ("plan",       || Plan.command_template(),       || Plan.skill_template()),
    ("audit",      || Audit.command_template(),      || Audit.skill_template()),
    ("implement",  || Implement.command_template(),  || Implement.skill_template()),
    ("review",     || Review.command_template(),     || Review.skill_template()),
    ("try_finish", || TryFinish.command_template(),  || TryFinish.skill_template()),
];
```

### Step 3: `src/setup.rs` — 新增 `target_skill_dir`，保持 `target_subdir` 现状

```rust
/// Slash command directory for a given target. Both platforms have one.
fn target_subdir(root: &Path, target: Target) -> PathBuf {
    match target {
        Target::Claude => root.join(".claude/commands"),
        Target::OpenCode => root.join(".opencode/command"),
    }
}

/// Claude skill directory. `None` for OpenCode v2 (no skills concept).
/// Returning `Option` (not panic, not a new enum variant) lets
/// `run()` skip the skill write with a single `if let`. The type
/// system enforces "handle the None case" at the call site, no
/// comments needed.
fn target_skill_dir(root: &Path, target: Target) -> Option<PathBuf> {
    match target {
        Target::Claude => Some(root.join(".claude/skills")),
        Target::OpenCode => None,
    }
}
```

### Step 4: `src/setup.rs` — `run()` 改造

```rust
pub fn run(targets: &[Target], root: &Path) -> Result<SetupReport> {
    let mut written = Vec::new();

    for &target in targets {
        // 1. Slash command files (always — both platforms have commands)
        let cmd_dir = target_subdir(root, target);
        fs::create_dir_all(&cmd_dir)?;
        for &(name, cmd_fn, _) in COMMANDS {
            let body = compile(&cmd_fn(), target)?;
            let path = cmd_dir.join(format!("{}.md", name));
            fs::write(&path, body)?;
            written.push(path);
        }

        // 2. Skill files (Claude only — OpenCode v2 has no skills)
        if let Some(skill_root) = target_skill_dir(root, target) {
            fs::create_dir_all(&skill_root)?;
            for &(name, _, skill_fn) in COMMANDS {
                let body = compile_skill(&skill_fn())?;
                let path = skill_root.join(name).join("SKILL.md");
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, body)?;
                written.push(path);
            }
        }
    }

    Ok(SetupReport { written })
}
```

### Step 5: `src/setup.rs` 单元测试

保持 `commands_table_has_nine_entries` 断言 `COMMANDS.len() == 9`（**不**新增 `skill_commands_table_has_nine_entries`，因为没有第二张表）。

### Step 6: `tests/setup_tool.rs` 集成测试更新

**更新计数断言**：

| 测试 | 旧期望 | 新期望 | 备注 |
|------|--------|--------|------|
| `run_default_writes_both_targets` | `report.written.len() == 18` | `== 27` (= 9 cmd × 2 + 9 skills × 1) | 默认两目标 |
| `claude_only_writes_claude_dir` | `== 9` | `== 18` (= 9 cmd + 9 skills) | Claude 单目标 |
| `opencode_only_writes_opencode_dir` | `== 9` | `== 9`（不变） | OpenCode 无 skills |
| `run_writes_all_nine_commands_per_target` | 9 names | **改名** `run_writes_commands_and_skills_per_claude_target`，断言 18 个文件 | Claude 跑后 |

**新增 3 个测试**：

```rust
#[test]
fn claude_target_writes_skill_files_to_skills_subdir() {
    // Claude platform registers each Command as a skill at
    // `.claude/skills/<name>/SKILL.md` (one subdirectory per skill).
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    let names: BTreeSet<_> = report
        .written
        .iter()
        .filter_map(|p| {
            // Extract skill name from `.claude/skills/<name>/SKILL.md` path.
            let parent = p.parent()?;
            let name = parent.file_name()?.to_str()?;
            if parent.parent()?.file_name()? == "skills" {
                Some(name.to_owned())
            } else {
                None
            }
        })
        .collect();
    let expected: BTreeSet<_> = [
        "gate", "status", "archive", "new", "plan",
        "audit", "implement", "review", "try_finish",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(names, expected);
}

#[test]
fn opencode_target_does_not_write_skill_files() {
    // OpenCode v2 has no skills concept — run() must not create
    // any `.claude/` directory when only OpenCode is requested.
    let tmp = TempDir::new().unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();
    assert!(!tmp.path().join(".claude").exists());
    // Sanity: command file is still there
    assert!(tmp.path().join(".opencode/command/gate.md").exists());
}

#[test]
fn skill_md_content_has_claude_skill_frontmatter() {
    // Verify the skill file format matches Claude Code spec.
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    let content = fs::read_to_string(
        tmp.path().join(".claude/skills/gate/SKILL.md"),
    ).unwrap();
    assert!(content.starts_with("---\nname: gate\n"));
    assert!(content.contains("description:"));
    // Skills must NOT have allowed-tools / argument-hint / agent
    assert!(!content.contains("allowed-tools"));
    assert!(!content.contains("argument-hint"));
    assert!(!content.contains("agent:"));
}
```

**更新 `setup_md_is_not_generated`**：断言扩展为 `setup.md` 与 `setup/SKILL.md` 都**不**存在（同时验证 commands 与 skills 都不含 setup）。

### Step 7: 验证

```bash
cargo check
cargo test
cargo clippy --all-targets
# 手工：
cargo run -- setup --tool claude
ls .claude/commands/                 # 9 个 .md
ls .claude/skills/                   # 9 个子目录
ls .claude/skills/gate/SKILL.md      # 存在
head -5 .claude/skills/gate/SKILL.md # --- name: gate ...

cargo run -- setup --tool opencode
ls .opencode/command/                # 9 个 .md
[ ! -d .claude ] && echo "no .claude dir"  # OpenCode-only
```

---

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| `target_skill_dir` 返回 `None` 时 `run()` 误继续 | `Option<PathBuf>` + `if let` 让编译器强制处理 |
| 加新 Command 漏改 `COMMANDS` 表 | 1 个表 1 个测试（`commands_table_has_nine_entries`）；比 v1 双表方案少一处漏改风险 |
| `compile_skill()` 误输出 `allowed-tools` | 单元测试 `test_compile_skill_minimal` 显式断言 `!out.contains("allowed-tools")`；comment 解释"skills 不需要 tool whitelist" |
| Claude Skills 平台规范未来改变 | `compile_skill()` 是单一入口，单元测试覆盖现有格式 |
| `fs::create_dir_all(path.parent().unwrap())` 对不存在多级父目录工作 | 标准库 API；`run_creates_deeply_nested_root` 测试覆盖类似场景 |
| 写 skill 时若 `path.parent()` 是 None 会 panic | 不可能：`skill_root.join(name).join("SKILL.md")` 永远有 parent |
| 双倍文件数（Claude 9 + 9 = 18）让 `run_default_writes_both_targets` 复杂 | 计数断言改 27；新测试 `claude_target_writes_skill_files_to_skills_subdir` 单独验证 |
| `SKILL_PROMPT` / `COMMAND_PROMPT` 文案若改说"不物化 Skill"会与本计划矛盾 | 本计划不修改 prompt 文案 |
| 已存在 `.claude/skills/<old>/` 旧文件不被清理 | 与 setup 现有行为一致；后续可加 `--purge` 旗标 |

---

## Verification Steps

1. **写 `compile_skill` 单元测试**（TDD 红）—— 跑 `cargo test test_compile_skill` 应失败（function not found）
2. **实现 `compile_skill()`** —— 跑测试应绿
3. **改 `src/setup.rs`**：改 `CommandEntry` 类型 + 改 `COMMANDS` 表 + 新增 `target_skill_dir` + 改 `run()`
4. **写 3 个新集成测试**（TDD 红 → 绿）
5. `cargo check` —— 编译期断言
6. `cargo test` —— 全部 65+ 测试通过
7. `cargo clippy --all-targets` —— 零 lint
8. **手工验证**（临时目录）：
   ```bash
   tmpdir=$(mktemp -d); cd "$tmpdir"
   /Users/alex/.cargo/global-target/debug/forceloop setup --tool claude
   echo "Commands:"; ls .claude/commands/     # 9 .md
   echo "Skills dirs:"; ls .claude/skills/    # 9 subdirs
   head -5 .claude/skills/gate/SKILL.md      # --- name: gate ...
   cd /; rm -rf "$tmpdir"

   tmpdir=$(mktemp -d); cd "$tmpdir"
   /Users/alex/.cargo/global-target/debug/forceloop setup --tool opencode
   ls .opencode/command/                     # 9 .md
   [ ! -d .claude ] && echo "no .claude dir"  # OpenCode-only
   cd /; rm -rf "$tmpdir"
   ```
9. 提交

---

## Out of Scope（明确边界）

1. **OpenCode v3 引入 skills 概念** —— 独立计划；本计划在 `target_skill_dir` 返回 `None` 防误用
2. **修改 `SKILL_PROMPT` / `COMMAND_PROMPT`** —— 文案通用，不物化具体文件路径
3. **修改 `CommandMetadata` trait** —— 不动 trait contract
4. **生成 `.forceloop/{skills,commands,hooks,archive}/` 目录树**（`SKILL_PROMPT` 步骤 1）—— 独立计划
5. **写 `.forceloop/state.json` 内容**（`SKILL_PROMPT` 步骤 2）—— 独立计划
6. **安装 git hooks**（`SKILL_PROMPT` 步骤 4）—— 独立计划
7. **生成 `.opencode/agent/<name>.md`** —— 独立计划
8. **为 Skill 加 `allowed-tools` 字段** —— 决定不加（与 Claude 官方 spec 一致）
9. **打印 `SetupReport` 摘要到 stdout** —— 独立 Open Question
10. **`--purge` 旗标清理旧 `setup.md`** —— 独立 Open Question
11. **清理已存在 `.claude/skills/<old>/` 文件** —— 同 setup 现有行为

---

## Open Questions（需用户后续决策，不阻塞本计划）

1. Skill frontmatter 是否需要 `tools:` 字段？当前选择：**不加**，与当前 Claude Code spec 一致
2. 是否需要把 `compile_skill()` 放到 `compile()` 的 match 里？当前选择：**不放**，保持函数单一职责
3. Skill 目录命名：`.claude/skills/<name>/SKILL.md` vs `.claude/skills/<name>.md`？当前选择：**前者**（Claude 官方 spec）

---

## ADR (Architecture Decision Record)

### Decision

为 `compile_skill()` 新增独立编译器函数（Claude skill 格式），并在 `setup` 中为 Claude 平台同时生成 `.claude/commands/<name>.md`（slash command）和 `.claude/skills/<name>/SKILL.md`（skill）；OpenCode 平台仍只生成 command（v2 无 skills 概念）。**采用单表 3 元组**（`CommandEntry` 自带 command + skill 两个模板），**不用** `TargetKind` enum + panic，**用** `Option<PathBuf>` 表达"不支持"。

### Drivers

1. **需求一致**：[docs/requirment.md:9](docs/requirment.md#L9) 要求 setup 物化"子 command，Skill"两者
2. **comment 一致**：[src/setup.rs:109-110](src/setup.rs#L109-L110) 注释承诺"slash command / Skill files"两者
3. **项目规约**：
   - "重复 2 次即提取公共模块" → v1 双表 9 entry 重复 9 次违反；v2 单表消除
   - "禁止打补丁/Hack" → v1 `panic!` 是 hack；v2 `Option<PathBuf>` 是根因
   - "根因治理" → v1 双表是症状（每个 entry 的两个模板被拆成两张表）；v2 单表是根因
4. **平台规范**：Claude Code 有 skills 概念；OpenCode v2 没有
5. **架构净度**：新函数 `compile_skill()` 纯函数；不引入新运行时依赖；不修改 trait contract

### Alternatives considered

- **Alternative A**（v1 方案）：双表（`COMMANDS` + `SKILL_COMMANDS`）+ `TargetKind` enum + `panic!`
  - **Why rejected**：
    - 加新 Command 必改 2 处（违反"重复 2 次即提取"）
    - `panic!` 在不支持的组合上是 hack（违反"禁止打补丁"）
    - 9 entry 重复 9 次（18 行 fixture）
- **Alternative B**：`&[&dyn CommandMetadata]` 动态分发拿两个 template
  - **Why rejected**：项目原则"零运行时依赖"
- **Alternative C**：合并 `COMMANDS` + `SKILL_COMMANDS` 为单表（v2 方案）
  - **Why chosen**：每 entry 自带两个模板；单源真相；零同步风险；类型诚实
- **Alternative D**：本计划方案（v2 单表 3 元组 + `Option<PathBuf>`）
  - **Why chosen**：所有 v1 问题都被消除

### Why chosen (Alternative D = v2)

- 单表 3 元组让"一个 Command 包含两个模板"在类型中显式表达
- `Option<PathBuf>` 表达"OpenCode 不支持 skills"，让 `if let` 优雅跳过
- 无 `panic!` / `unreachable!` / 新 enum —— 类型系统是真相
- 加新 Command 改 1 处（3 元组 + 1 行 fixture）
- 测试减少 1 个（无第二张表）

### Consequences

**正面**:
- Claude Code 用户可通过 Skill tool 加载 ForceLoop 详细工作流
- comment 与实现一致；需求与实现一致
- `compile_skill()` 独立可测
- 双文件输出（commands + skills）让 Claude Code 两种调用方式都可用
- **v2 优于 v1**：单源真相、无 panic、加新 Command 改 1 处

**负面**:
- 闭包嵌套 `if let` 略增加行数（每 entry 5 行 vs v1 的 2 行），但信息密度更高
- Claude 平台文件数翻倍（9→18），首次 setup 时间略增
- 4 个集成测试 + 2 个单元测试新增

### Follow-ups

- 未来若 OpenCode v3 引入 skills 概念：
  1. `target_skill_dir` 在 `Target::OpenCode` 分支返回 `Some(...)`
  2. `run()` 改为对所有 target 写 skill
- 未来若 Claude 平台支持 skill-level `tools:` 字段：
  1. `compile_skill()` 加 `parts.push(format!("tools: [...]"))`
  2. 单元测试 `test_compile_skill_minimal` 调整为含 tools 的 fixture

---

## 共识评审应用记录

本计划为 **Direct Mode**（用户已选 B 选项 + 反馈"给更好的方案"），不触发 Architect / Critic 循环。Critic 评估标准已通过自审应用：

- **80%+ claims 引用 file:line**：是
- **90%+ criteria 可测试**：是
- **替代方案公平探索**：是（v1 方案 vs v2 方案对比表 + ADR 列 3 个 rejected alternatives）
- **风险 + 缓解**：是（9 行 risks 表格）
- **边界清晰**：是（11 项 Out of Scope）
- **响应用户"更好的方案"**：是（v1 → v2 关键变化表 + 单表 3 元组设计 + `Option<PathBuf>` 替代 panic）
