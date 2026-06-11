# OpenCode `session.idle` Hook：自动调用 `fl gate`

## TL;DR

> **Quick Summary**: 扩展 `fl setup` 的 OpenCode 注入路径,在生成 6 个 command 文件之外,额外写出 2 个项目级文件,让 OpenCode 在 `session.idle` 事件里自动调用 `fl gate`,实现"AI 回复结束 → gate 检查 → 失败时把 stdout+stderr 注回 prompt 让 AI 自动修复"的闭环。生成物:项目根的 `opencode.json` 和 `plugin/hook.ts`。
>
> **Deliverables**:
> - `src/setup.rs` — 3 个私有项:`opencode_json_content()` / `plugin_hook_ts_content()` 常量,以及 `write_opencode_hook(root, written)` 函数;`run()` 在 `Target::OpenCode` 分支末尾追加该函数调用
> - `src/setup.rs` — 5 个单元测试覆盖 hook 内容
> - `tests/setup_tool.rs` — 4 个集成测试数字更新 + 3 个新增 hook 断言
> - `.omc/plans/opencode-session-idle-gate-hook.md` — 本 plan
>
> **Estimated Effort**: Small(1 个生产文件 + 2 个测试文件,纯新增,无 trait 改动)
> **Parallel Execution**: NO(顺序:常量 → write 函数 → run() 接入 → 单元测试 → 集成测试 → 验证)
> **Critical Path**: `src/setup.rs` 常量/函数 → `run()` 接入 → `tests/setup_tool.rs` 数字更新 → `cargo test`

---

## Context

### Original Request(用户原话)
> 根据文件 `/Users/alex/Projects/testspace/todo_test1/docs/opencode-auto-state-driver.md` 的描述,在 setup 子命令里实现给 opencode 注册项目级别的钩子,在 session.idle 事件里调用 fl gate

### Reference Doc 要点([opencode-auto-state-driver.md](/Users/alex/Projects/testspace/todo_test1/docs/opencode-auto-state-driver.md))
- **配置文件** `opencode.json`:`{ "plugin": ["./plugin/hook.ts"] }`
- **插件代码** `plugin/hook.ts`:监听 `event.type === "session.idle"`,调用外部脚本
- **非零退出码**:把 stdout+stderr 注入 `client.session.prompt`,`noReply: false` 触发 AI 自动回复
- **零退出码**:静默通过
- **调用方式**:BunShell 的 `$` 命令,带 `.timeout(60_000)`
- **本任务的关键替换**:把 doc 示例的 `./check.sh` 替换为 `fl gate`

### Current State Inventory
| 路径 | 现状 |
|------|------|
| [src/setup.rs:91-104](../../src/setup.rs#L91-L104) | `run()` 写命令文件,只覆盖 `.claude/commands/<name>.md` 和 `.opencode/command/<name>.md` |
| [src/setup.rs:106-112](../../src/setup.rs#L106-L112) | `target_subdir()` 返回平台子目录,**不**返回 hook 文件路径 |
| [src/setup.rs:75-82](../../src/setup.rs#L75-L82) | `COMMANDS` 静态表(6 个),用于命令文件枚举 |
| [src/setup.rs:45](../../src/setup.rs#L45) | `SetupReport { written: Vec<PathBuf> }` 接受任意 `PathBuf`,可直接容纳 hook 路径 |
| [src/setup.rs:107-109](../../src/setup.rs#L107-L109) | `target_subdir()` 注释:OpenCode 用 `.opencode/command/`,**未**涉及 `opencode.json` 或 `plugin/` |
| [src/constants.rs](../../src/constants.rs) | `HOOKS_DIR = "hooks"` 存在但未被 OpenCode 注入路径使用(用于 `.forceloop/hooks/`) |
| [tests/setup_tool.rs](../../tests/setup_tool.rs) | 集成测试断言 `report.written.len()` 在 default=12 / claude=6 / opencode=6 |

### Why Now
- `setup` 已经能为 OpenCode 注入 command 文件,但**没有**事件钩子 —— 用户必须手动调用 `fl gate` 才能让 pipeline 推进
- 有了 `session.idle` 钩子,AI 完成工作后自动 gate → 通过则静默 / 失败则回注 prompt 形成自动修复循环
- 这与项目 CLAUDE.md 中"结构化开发工作流"的定位强一致(参考 [docs/requirment.md](../../docs/requirment.md))

### What this does NOT change
- `CommandMetadata` trait 零修改
- 6 个 `src/commands/` 对象的 impl 零修改
- 4 个顶层子命令的 `Subcommand` / `Executable` impl 零修改
- `cli.rs` / `main.rs` 零修改
- `compiler.rs` / `schema.rs` 零修改(`compile()` 不涉及 hook 注入)
- `state.rs` / `context.rs` / `errors.rs` 零修改
- `install.sh` 零修改

---

## Work Objectives

### Core Objective
让 `fl setup` 在 OpenCode 目标上,除生成 6 个 command 文件外,额外写出 `opencode.json` 和 `plugin/hook.ts`,实现"AI 回复结束 → 调用 `fl gate` → 失败时回注 prompt"的自动化 gate 循环。

### Definition of Done

- [ ] `src/setup.rs` 新增 `opencode_json_content() -> &'static str` 常量函数,返回 `{ "plugin": ["./plugin/hook.ts"] }`
- [ ] `src/setup.rs` 新增 `plugin_hook_ts_content() -> &'static str` 常量函数,返回完整 TypeScript 插件代码(`import type { Plugin }`、`session.idle` 过滤、`fl gate` 调用、60s timeout、零退出静默 / 非零注入 prompt)
- [ ] `src/setup.rs` 新增 `write_opencode_hook(root: &Path, written: &mut Vec<PathBuf>) -> Result<()>` 函数,创建 `.opencode/plugin/` 不需要(实际写到 `<root>/plugin/hook.ts`,见 [opencode-auto-state-driver.md:6](/Users/alex/Projects/testspace/todo_test1/docs/opencode-auto-state-driver.md)),写 2 个文件并 push 到 `written`
- [ ] `src/setup.rs` 的 `run()` 在 `for &target in targets` 内,命令文件循环之后,追加 `if target == Target::OpenCode { write_opencode_hook(root, &mut written)?; }`
- [ ] `src/setup.rs` 单元测试 5 个(`opencode_json_has_plugin_entry` / `plugin_hook_ts_uses_fl_gate` / `plugin_hook_ts_filters_session_idle` / `plugin_hook_ts_prompts_on_nonzero_exit` / `plugin_hook_ts_has_timeout`)
- [ ] `tests/setup_tool.rs` `run_default_writes_both_targets` 数字 12 → 14,并新增 2 个 hook 文件存在断言
- [ ] `tests/setup_tool.rs` `claude_only_writes_claude_dir` 数字 6 不变,新增反向断言 `opencode.json` / `plugin/hook.ts` **不存在**
- [ ] `tests/setup_tool.rs` `opencode_only_writes_opencode_dir` 数字 6 → 8,新增 2 个 hook 文件存在断言
- [ ] `tests/setup_tool.rs` 新增 `opencode_hook_files_have_expected_content`(断言 `fl gate` 字符串、`session.idle` 字符串、`noReply: false` 字符串、JSON 解析合法)
- [ ] `cargo check` / `cargo build` / `cargo test` / `cargo clippy --all-targets` 全绿
- [ ] 提交

### Must Have

- 文件路径**完全按 reference doc 命名**:`opencode.json` 在项目根,`plugin/hook.ts` 在 `<root>/plugin/`
- TypeScript 代码**完全按 reference doc 结构**,仅把 `./check.sh` 替换为 `fl gate`
- 仅在 `Target::OpenCode` 分支写入 hook 文件,Claude 目标**绝不**写入
- `SetupReport.written` **包含**新写入的 2 个 hook 路径
- `fl setup` 重复运行仍幂等(覆盖写入,与现有 command 文件行为一致)
- 编译零警告

### Must NOT Have (Guardrails)

- **不**修改 `compiler.rs` / `schema.rs` / `cli.rs` / `main.rs` / `state.rs` / `context.rs` / `errors.rs` / `traits.rs`
- **不**改 `CommandMetadata` trait 或 6 个 `src/commands/` 对象的 impl
- **不**改 4 个顶层子命令的 `Subcommand` / `Executable` impl
- **不**引入新依赖(`serde_json` 已在 Cargo.toml,hook 内容是 `&'static str` 不用 serde)
- **不**自动合并已存在的 `opencode.json`(本计划直接覆盖;若用户已有自定义 plugin,迁移负担是一次性手动合并;见 Follow-ups)
- **不**注册 git hooks(`SKILL_PROMPT` 步骤 4 的 git hook 与本任务无关,且项目原则"骨架优先")
- **不**写 `.forceloop/hooks/`(那是 `HOOKS_DIR` 的用途,与 OpenCode 事件钩子不同)
- **不**改 `COMMANDS` 静态表(hook 文件不是 command 文件,走单独写入路径)

---

## Design Decisions

### Decision 1: hook 文件写到哪?

**决策**:写到项目根,文件名严格按 doc:`<root>/opencode.json` 和 `<root>/plugin/hook.ts`。

**理由**:
- doc 明确写 `"./plugin/hook.ts"`,相对路径解析从 `opencode.json` 所在目录出发,即项目根
- 不在 `.opencode/` 下,因为 doc 示例的目录树是项目根 + 平行 `plugin/` 目录
- OpenCode 自己负责发现并加载 `opencode.json`,我们只负责写出
- 不用 `.forceloop/hooks/`(`HOOKS_DIR`),那是为 ForceLoop 内部状态用的,与 IDE 集成无关

### Decision 2: 静态 `&'static str` vs 模板引擎

**决策**:用静态 `&'static str` 字面量返回完整文件内容,无模板引擎。

**理由**:
- 文件内容是固定的(只有 1 个变量:`./check.sh` → `fl gate`),无用户自定义
- 静态字符串零运行时开销,零分配,零依赖
- 测试断言直接 `assert!(content.contains("fl gate"))`,简单可读
- 项目原则"零新增运行时依赖",handlebars/tera 都不引入

### Decision 3: hook 文件放在 `setup.rs` 还是新模块 `src/hooks.rs`?

**决策**:放在 `src/setup.rs` 内,作为私有常量函数和私有 `write_*` 函数。

**理由**:
- 只有 2 个文件要写,共约 60 行 TypeScript + 1 行 JSON
- 新模块需要 import + 暴露 public 函数,反而增加间接层
- `setup.rs` 现有的 `COMMANDS` 静态表 + `target_subdir()` 已是相似模式
- 未来若 Claude Code 钩子也走 setup 注入,可在同一文件内对称扩展(`write_claude_hook()`);新模块的拆分需求是 3+ 个不同平台钩子时

### Decision 4: `run()` 内联 `if target == OpenCode` vs 抽 HOOKS 表

**决策**:在 `run()` 内联 `if target == Target::OpenCode { write_opencode_hook(root, &mut written)?; }`。

**理由**:
- 当前只有 1 个平台的 hook 需要写,内联最简
- 若未来加 Claude hook,再抽 `HOOKS: &[HookEntry]` 表;不预先抽象(YAGNI)
- 与现有 `COMMANDS` 表分离,因 hook 不是 command,行为不同(无 `compile()`,直接 `fs::write` 静态内容)

### Decision 5: 覆盖已存在的 `opencode.json`

**决策**:**直接覆盖**(idempotent 写入)。

**理由**:
- 与现有 `command` 文件行为一致(`fs::write` 静默覆盖,见 [tests/setup_tool.rs:110-125](../../tests/setup_tool.rs#L110-L125) 的 `run_overwrites_existing_files_with_current_compile_output` 锁定此行为)
- 用户已存在的自定义 plugin 列表会被覆盖 —— 这是用户已经接受的 setup 语义
- 提供 `--preserve-existing` 旗标属于独立计划(见 Follow-ups)

### Decision 6: TypeScript 内容的参数

**决策**:`timeout(60_000)` 毫秒,硬编码 60 秒。

**理由**:
- 与 reference doc 示例完全一致
- 60 秒够 `fl gate` 跑完 plan gate + state write;若未来需要更长,引入 `--gate-timeout` 旗标(独立计划)
- 硬编码值在 setup 输出后用户可手动编辑 `plugin/hook.ts`,没有 lock-in

### Decision 7: 测试断言策略

**决策**:
- **单元测试**(在 `src/setup.rs` `mod tests`):断言常量函数返回的字符串包含关键短语(`fl gate` / `session.idle` / `noReply: false` / `.timeout(`)
- **集成测试**(在 `tests/setup_tool.rs`):断言 `run()` 后文件存在,`report.written` 包含 hook 路径,以及反向断言(Claude 模式无 hook)

**理由**:
- 单元测试快、确定、零 IO
- 集成测试覆盖 `run()` 真实路径,防止"常量改了但 run() 忘接"或"路径写错"
- 反向断言锁定"Claude 不写 hook"这个不变量

---

## Implementation Steps

### Step 1: 在 `src/setup.rs` 新增 2 个内容常量函数

**File**: [src/setup.rs](../../src/setup.rs) (新增,放在 `run()` 函数前)

**What to do**:

```rust
/// Project-level OpenCode config that registers our `plugin/hook.ts`.
///
/// Output: `<root>/opencode.json`
///
/// The file tells OpenCode to load the TypeScript plugin at
/// `./plugin/hook.ts` (relative to the directory containing
/// `opencode.json`, i.e. the project root). Per reference doc:
/// `opencode-auto-state-driver.md`.
fn opencode_json_content() -> &'static str {
    "{\n  \"plugin\": [\"./plugin/hook.ts\"]\n}\n"
}

/// OpenCode TypeScript plugin that calls `fl gate` on `session.idle`.
///
/// Output: `<root>/plugin/hook.ts`
///
/// Behavior (per reference doc):
/// - On `session.idle`, run `fl gate` with a 60-second timeout
/// - Exit 0: silent pass
/// - Exit != 0: inject stdout+stderr into the session as a prompt
///   with `noReply: false` to trigger the AI's auto-reply / fix loop
fn plugin_hook_ts_content() -> &'static str {
    include_str!("../plugin/hook.ts") // 静态嵌入,见下
}
```

实际内容用 `include_str!` 宏嵌入项目根的 `plugin/hook.ts` 文件,以避免在 Rust 字符串字面量里写 ~50 行 TypeScript 转义(可读性差且难以维护)。

**但本计划同时支持两种写法,优先用方案 A:写一个真实的 `plugin/hook.ts` 文件 + `include_str!`**。

**What to do (方案 A,采用)**:

1. 在项目根创建 `plugin/hook.ts`,内容如下(完全按 reference doc 改写):
```typescript
import type { Plugin } from "@opencode-ai/plugin";

export default (async (ctx) => {
  const { client, $ } = ctx;

  return {
    event: async ({ event }) => {
      if (event.type !== "session.idle") return;

      const sessionID = event.properties.sessionID;

      // Call fl gate; on failure, inject output back to AI as prompt.
      const result = await $`fl gate`.timeout(60_000);

      if (result.exitCode !== 0) {
        // Non-zero exit → re-inject stdout+stderr as prompt so the
        // AI can see the gate reason and auto-fix.
        client.session.prompt({
          path: { id: sessionID },
          body: {
            noReply: false,
            parts: [{
              type: "text",
              text: result.stdout + result.stderr,
            }],
          },
        });
      }
      // Exit 0: silent pass, no AI intervention.
    },
  };
}) satisfies Plugin;
```

2. `src/setup.rs` 用 `include_str!("../plugin/hook.ts")` 嵌入内容:
```rust
fn plugin_hook_ts_content() -> &'static str {
    include_str!("../plugin/hook.ts")
}
```

3. `opencode_json_content()` 用静态字面量(只 1 行 JSON)。

**Acceptance Criteria**:
- `plugin/hook.ts` 文件存在于项目根
- `include_str!` 编译时验证文件存在
- 2 个常量函数可被测试调用

**Why 方案 A over 方案 B**:
- 真实文件 vs 嵌入字符串字面量:真实文件可被用户直接查看/编辑,IDE 有 TypeScript 语法高亮
- `include_str!` 是零开销编译期嵌入,行为与字符串字面量相同
- 测试时 `assert!(content.contains("fl gate"))` 直接验证嵌入内容
- 项目根的 `plugin/hook.ts` 不是"生成的产物"而是"模板源"——它被 `fl setup` 复制到用户项目的同名位置

### Step 2: 在 `src/setup.rs` 新增 `write_opencode_hook()` 函数

**File**: [src/setup.rs](../../src/setup.rs) (新增,放在 `target_subdir()` 之后)

**What to do**:

```rust
/// Write the 2 OpenCode project-level hook files to `root`:
///   - `<root>/opencode.json`
///   - `<root>/plugin/hook.ts`
///
/// Both paths are pushed into `written` for the `SetupReport`.
///
/// Idempotent: re-running overwrites both files (matches the
/// `fs::write` semantics used for command files).
fn write_opencode_hook(root: &Path, written: &mut Vec<PathBuf>) -> Result<()> {
    let json_path = root.join("opencode.json");
    fs::write(&json_path, opencode_json_content())?;
    written.push(json_path);

    let plugin_dir = root.join("plugin");
    fs::create_dir_all(&plugin_dir)?;
    let ts_path = plugin_dir.join("hook.ts");
    fs::write(&ts_path, plugin_hook_ts_content())?;
    written.push(ts_path);

    Ok(())
}
```

**Acceptance Criteria**:
- 2 个文件按 doc 路径创建
- `plugin/` 目录通过 `create_dir_all` 自动创建
- 2 个路径 push 到 `written`

### Step 3: 在 `run()` 内接入 `write_opencode_hook()`

**File**: [src/setup.rs](../../src/setup.rs) (修改 `run()` 函数体)

**What to do**:
修改 [src/setup.rs:91-104](../../src/setup.rs#L91-L104) 的 `run()`:

```rust
pub fn run(targets: &[Target], root: &Path) -> Result<SetupReport> {
    let mut written = Vec::new();
    for &target in targets {
        let dir = target_subdir(root, target);
        fs::create_dir_all(&dir)?;
        for (name, t_fn) in COMMANDS {
            let body = compile(&t_fn(), target)?;
            let path = dir.join(format!("{}.md", name));
            fs::write(&path, body)?;
            written.push(path);
        }
        // OpenCode additionally gets a project-level hook so that
        // `session.idle` automatically invokes `fl gate`. See
        // `.omc/plans/opencode-session-idle-gate-hook.md`.
        if target == Target::OpenCode {
            write_opencode_hook(root, &mut written)?;
        }
    }
    Ok(SetupReport { written })
}
```

**Acceptance Criteria**:
- 仅 `Target::OpenCode` 触发 hook 写入
- Claude 目标完全不变
- 顺序:命令文件先写,hook 后写(便于调试时按顺序看 `SetupReport.written`)

### Step 4: 在 `src/setup.rs` 新增 5 个单元测试

**File**: [src/setup.rs](../../src/setup.rs) (扩展 `#[cfg(test)] mod tests`)

**What to do**:

```rust
#[test]
fn opencode_json_has_plugin_entry() {
    let s = opencode_json_content();
    // Must be valid JSON (cheap check via serde_json).
    let v: serde_json::Value = serde_json::from_str(s)
        .expect("opencode.json content must be valid JSON");
    let plugins = v.get("plugin")
        .and_then(|p| p.as_array())
        .expect("plugin must be a JSON array");
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0], "./plugin/hook.ts");
}

#[test]
fn plugin_hook_ts_uses_fl_gate() {
    // Replaces the doc's `./check.sh` placeholder with the actual
    // `fl gate` invocation. This is the core of the integration.
    let s = plugin_hook_ts_content();
    assert!(s.contains("fl gate"), "must call `fl gate`");
    assert!(
        !s.contains("./check.sh"),
        "doc placeholder must be replaced; found `./check.sh`"
    );
}

#[test]
fn plugin_hook_ts_filters_session_idle() {
    let s = plugin_hook_ts_content();
    assert!(s.contains("session.idle"),
        "must filter on session.idle event");
    assert!(s.contains("event.type"),
        "must inspect event.type");
}

#[test]
fn plugin_hook_ts_prompts_on_nonzero_exit() {
    let s = plugin_hook_ts_content();
    assert!(s.contains("exitCode"),
        "must inspect result.exitCode");
    assert!(s.contains("client.session.prompt"),
        "must call client.session.prompt on non-zero exit");
    assert!(s.contains("noReply: false"),
        "noReply: false triggers AI auto-reply / fix loop");
}

#[test]
fn plugin_hook_ts_has_timeout() {
    let s = plugin_hook_ts_content();
    assert!(s.contains(".timeout("),
        "must use BunShell $.timeout() to bound gate execution");
    assert!(s.contains("60_000"),
        "default 60s timeout per reference doc");
}
```

**Acceptance Criteria**:
- 5 个测试全部通过
- 覆盖所有关键契约:JSON 合法 / 调用 `fl gate` / 事件过滤 / 失败回注 / timeout

### Step 5: 更新 `tests/setup_tool.rs` 集成测试

**File**: [tests/setup_tool.rs](../../tests/setup_tool.rs)

**What to do**:

#### 5a. `run_default_writes_both_targets`(改)
```rust
#[test]
fn run_default_writes_both_targets() {
    let tmp = TempDir::new().unwrap();
    let report = run(&default_targets(), tmp.path()).unwrap();
    // 6 commands × 2 targets = 12 command files
    // + 2 OpenCode hook files (opencode.json + plugin/hook.ts)
    // = 14 total
    assert_eq!(report.written.len(), 14);
    assert!(tmp.path().join(".claude/commands/new.md").exists());
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    // OpenCode hook files
    assert!(tmp.path().join("opencode.json").exists());
    assert!(tmp.path().join("plugin/hook.ts").exists());
}
```

#### 5b. `claude_only_writes_claude_dir`(改)
```rust
#[test]
fn claude_only_writes_claude_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 6);  // unchanged
    assert!(tmp.path().join(".claude/commands/new.md").exists());
    assert!(!tmp.path().join(".opencode/").exists());
    // Claude-only must NOT register the OpenCode hook
    assert!(!tmp.path().join("opencode.json").exists(),
        "Claude-only setup must not write opencode.json");
    assert!(!tmp.path().join("plugin/").exists(),
        "Claude-only setup must not create plugin/ directory");
}
```

#### 5c. `opencode_only_writes_opencode_dir`(改)
```rust
#[test]
fn opencode_only_writes_opencode_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::OpenCode], tmp.path()).unwrap();
    // 6 commands + 2 hook files = 8
    assert_eq!(report.written.len(), 8);
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    assert!(!tmp.path().join(".claude/").exists());
    // OpenCode-only DOES register the hook
    assert!(tmp.path().join("opencode.json").exists());
    assert!(tmp.path().join("plugin/hook.ts").exists());
}
```

#### 5d. 新增 `opencode_hook_files_have_expected_content`
```rust
#[test]
fn opencode_hook_files_have_expected_content() {
    let tmp = TempDir::new().unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();

    // opencode.json is valid JSON pointing to our plugin
    let json = fs::read_to_string(tmp.path().join("opencode.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["plugin"][0], "./plugin/hook.ts");

    // plugin/hook.ts references the actual gate command and the right event
    let ts = fs::read_to_string(tmp.path().join("plugin/hook.ts")).unwrap();
    assert!(ts.contains("fl gate"));
    assert!(ts.contains("session.idle"));
    assert!(ts.contains("noReply: false"));
    assert!(ts.contains(".timeout(60_000)"));
}
```

**Acceptance Criteria**:
- 4 个测试全部通过
- 数字断言与新的写入数量一致

### Step 6: 全量验证

**What to do**:
```bash
cargo check
cargo build
cargo test
cargo clippy --all-targets
```

**Expected**:
- 编译零错误零警告
- 全部测试通过(预期 72 + 5 单元 + 3 集成新断言 = ~80 个测试)

### Step 7: 手工验证(临时目录)

```bash
cd /tmp && mkdir test_opencode_hook && cd test_opencode_hook
fl setup --tool opencode
ls -la
# 期望:
#   opencode.json
#   .opencode/command/{audit,implement,new,plan,review,try_finish}.md
#   plugin/hook.ts
cat opencode.json
# 期望:
#   { "plugin": ["./plugin/hook.ts"] }
head -20 plugin/hook.ts
# 期望: import type { Plugin } from "@opencode-ai/plugin";
fl setup --tool claude
# 然后:
ls -la
# 期望: 无 opencode.json(因为是 --tool claude)
```

---

## File Map(汇总)

| 文件 | 改动 | 行数估计 |
|------|------|----------|
| `plugin/hook.ts` (新) | 项目根 TypeScript 模板,~45 行 | +45 |
| `src/setup.rs` | 2 个内容常量 + 1 个 `write_*` 函数 + `run()` 接入 + 5 个单元测试 | +60 净增 |
| `tests/setup_tool.rs` | 3 个数字/断言更新 + 1 个新测试 | +25 净增 |
| `.omc/plans/opencode-session-idle-gate-hook.md` (新) | 本 plan | +250 |

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| 覆盖已存在的 `opencode.json` 破坏用户自定义 plugin 列表 | 中 | 中 | 与现有 command 文件一致的覆盖语义;在 plan 的 Follow-ups 中记录 `--preserve-existing` 旗标(独立计划) |
| `plugin/hook.ts` 用 `include_str!` 嵌入,如用户改了项目根的 `plugin/hook.ts`,setup 会用其最新版 | 低 | 高 | 这是 desired behavior:用户改模板,下次 setup 用新模板;`include_str!` 编译期嵌入,运行时无 IO |
| 60s timeout 太短,`fl gate` 跑大型 phase 验证超时 | 中 | 中 | `fl gate` 当前实现是 O(1) state 读写 + 几个 artifact 校验,< 1s 完成;未来 phase 复杂时用户可手动编辑 `plugin/hook.ts` 调大 |
| `fl gate` 不在 PATH(未 `cargo install`),`$` 直接失败并 `exitCode != 0`,触发 prompt 注入 | 中 | 高 | 这是 desired behavior:未安装则每次 idle 都被告知;plan Follow-ups 中记录"`install.sh` 应在 setup 之后执行"的提示 |
| TypeScript 文件被 OpenCode 加载但 `fl gate` 命令本身未实现当前 phase 的 gate | 中 | 中 | `Gate::execute()` 当前是 phase 1 完整实现(读 state, 调 phase.gate(), 写 next state),已可用;6 个 commands 的 `gate()` 多数是 `Ok(())` 占位,需后续填实 |
| `plugin/hook.ts` 的 `import type { Plugin }` 要求 `@opencode-ai/plugin` 在 OpenCode 安装时存在 | 中 | 高 | 这是 OpenCode 自身的依赖,不由 setup 负责;用户若装 OpenCode 通常已自带 |
| 多 session 并发时 `fl gate` 串行执行可能 race | 低 | 低 | `Gate::execute()` 读 `state.json` 后写,非原子;但 `session.idle` 是顺序事件,实际并发不常见;Follow-ups 中加锁 |
| Claude Code 没有等价 hook,功能不对称 | 中 | 中 | 本计划只做 OpenCode;Claude Code 的等效机制(hooks API)独立计划(见 Follow-ups) |
| 集成测试 `tests/setup_tool.rs` 的 `run_writes_all_six_commands_per_target` 测试期望集 6 个文件,不影响(它只断言 `.md` 文件名) | 低 | 低 | 该测试用 `BTreeSet` 收集所有 `file_name()`,hook 文件名是 `opencode.json` 和 `hook.ts` 不以 `.md` 结尾,不会污染期望集;验证后再确认 |

---

## Verification Steps

1. `cargo check` — 零错误零警告
2. `cargo build` — exit 0
3. `cargo test` — 全部 80+ 测试通过(含 5 个新单元 + 4 个集成更新 + 1 个新集成)
4. `cargo clippy --all-targets` — 零 lint
5. **手工验证**:
   ```bash
   cd /tmp && mkdir test_hook && cd test_hook
   fl setup --tool opencode
   ls -la
   cat opencode.json
   cat plugin/hook.ts | head -20
   # 临时验证: 在另一个目录跑 claude-only
   cd /tmp && mkdir test_hook2 && cd test_hook2
   fl setup --tool claude
   ls -la
   # 期望: 无 opencode.json, 无 plugin/ 目录
   ```
6. 提交

---

## Out of Scope (Explicit Boundaries)

以下**不在本计划范围**:

1. **保留已存在的 `opencode.json`** —— 直接覆盖;若用户有自定义 plugin,迁移负担一次性手动合并;Follow-ups 中记录 `--preserve-existing` 旗标
2. **Claude Code 等价 hook** —— Claude Code 也有 hooks API(`.claude/settings.json` + `hooks` 字段),但与 OpenCode plugin 机制不同;独立计划
3. **`fl gate` 实现当前 phase 的具体 gate 验证** —— `Audit` / `Implement` / `Review` 的 `gate()` 当前是 `Ok(())` 占位;本计划只注册调用,不实现 gate 业务逻辑(那是 phase 2 业务)
4. **多 session 并发锁** —— `fl gate` 内部读 `state.json` 写 next,非原子;Follow-ups 中考虑文件锁
5. **超时可配置** —— 60s 硬编码;用户可手动编辑 `plugin/hook.ts`;Follow-ups 中加 `--gate-timeout` 旗标
6. **git hooks 集成** —— `.forceloop/hooks/` 与 OpenCode 事件钩子不同;`SKILL_PROMPT` 步骤 4 的 git hook 是独立计划
7. **测试 `run_writes_all_six_commands_per_target` 的修改** —— 该测试断言期望集是 6 个 `.md` 文件名,hook 文件不在期望集,无影响

---

## Follow-ups

- 加 `--preserve-existing` 旗标给 `fl setup`,遇到已存在 `opencode.json` 时合并 plugin 列表
- 加 Claude Code 等价 hook(`.claude/settings.json` 的 `hooks` 字段,在 `Stop` 事件调用 `fl gate`)
- 加 `--gate-timeout <ms>` 旗标,参数化 `plugin/hook.ts` 的 timeout
- `Gate::execute()` 加文件锁,防止多 session 并发写 `state.json`
- 实现 `Audit` / `Implement` / `Review` 的具体 `gate()` 业务逻辑(phase 2 任务,与本计划解耦)
- `install.sh` 完成后给 `fl setup` 加 print 提示:`fl not in PATH? run ./install.sh`

---

## ADR (Architecture Decision Record)

### Decision
扩展 `fl setup` 的 OpenCode 注入路径,在 `Target::OpenCode` 分支末尾额外写出 2 个项目级文件:项目根的 `opencode.json`(内容:`{ "plugin": ["./plugin/hook.ts"] }`)和 `<root>/plugin/hook.ts`(内容:TypeScript 插件代码,监听 `session.idle`,调用 `fl gate`,失败时回注 prompt)。hook 文件的内容分别由 `opencode_json_content()`(静态 `&'static str`)和 `plugin_hook_ts_content()`(`include_str!("../plugin/hook.ts")`)返回。

### Drivers

1. **闭环自动化**:让 AI 每次回复结束自动 gate,通过则静默,失败则自动修复 —— 把 `fl gate` 从"用户记得手动跑"提升为"OpenCode 帮你跑"
2. **零运行时依赖**:用 `include_str!` 嵌入 TypeScript 模板,无 handlebars/serde_json serialize,无新 crate
3. **路径严格按 doc**:doc 是用户的真理来源;`opencode.json` 在项目根、`plugin/hook.ts` 在 `<root>/plugin/`,不引入项目结构的歧义
4. **测试覆盖完整**:5 个单元测试覆盖内容契约,4 个集成测试覆盖 `run()` 路径 + 反向断言(Claude 不写 hook)
5. **与现有 setup 行为对称**:`SetupReport.written` 容纳 hook 路径,idempotent 覆盖写入与 command 文件一致

### Alternatives considered

- **Alternative A**:把 hook 文件生成抽到新模块 `src/hooks.rs`
  - **Why rejected**:仅 2 个文件、~60 行 TypeScript,新模块是 over-engineering;未来加 Claude hook 时再抽(YAGNI)
- **Alternative B**:用 `serde_json::json!` 宏生成 `opencode.json` 内容,用 `handlebars` 生成 `hook.ts`
  - **Why rejected**:handlebars 是新运行时依赖,违反项目"零新增依赖"原则;`include_str!` 是零开销编译期嵌入
- **Alternative C**:把 hook 注册为 `Target::OpenCode` 下的"特殊 command"放进 `COMMANDS` 表
  - **Why rejected**:`COMMANDS` 走 `compile()` 路径(返回 `CommandSchema` → YAML frontmatter),hook 文件不走 `compile()`,类型不兼容;混入会破坏 `COMMANDS` 表的语义单一性
- **Alternative D**:hook 文件由 setup 命令 + 一个新子命令 `fl hooks install` 分别管理
  - **Why rejected**:用户已要求"在 setup 子命令里实现",拆分违反用户意图;若未来需要解耦可独立计划
- **Alternative E (本计划)**:在 `setup.rs` 内新增 2 个内容常量 + 1 个 `write_*` 函数,`run()` 在 OpenCode 分支末尾内联调用
  - **Why chosen**:最小变更,内容用 `include_str!` 嵌入项目根的真实 `.ts` 文件;测试覆盖完整;与现有 setup 模式一致

### Why chosen (Alternative E)

- 1 个新文件(`plugin/hook.ts`)+ 1 个生产文件改 `src/setup.rs` + 1 个测试文件改 `tests/setup_tool.rs`
- `include_str!` 编译期验证模板存在
- 5 单元 + 4 集成测试覆盖内容契约 + 路径契约 + 反向不变量
- 零新依赖
- 与现有 `target_subdir()` / `COMMANDS` 表的模式一致
- 未来扩展(Claude hook)时在同一文件内加 `write_claude_hook()` 对称函数

### Consequences

**正面**:
- 用户跑 `fl setup` 之后,OpenCode 自动接管 `session.idle` 事件,gate 验证失败时回注 prompt 形成自动修复循环
- `SetupReport.written` 现在包含 14 条路径(默认 2 目标)或 8 条(单 OpenCode),用户可看到所有生成物
- 5 个单元测试 + 4 个集成测试把契约钉死,任何破坏 `fl gate` 调用或 `session.idle` 过滤的提交会被测试捕获
- 模板(`plugin/hook.ts`)在项目根,用户可直接编辑调 timeout / 改行为,无需重建二进制

**负面**:
- 旧用户已存在的 `opencode.json` 会被覆盖(若有自定义 plugin 列表需手动合并)
- Claude Code 暂时没有等价 hook,功能不对称(但 OpenCode 是这个项目的 primary target)
- 60s timeout 硬编码,大型 phase 需手动改 `plugin/hook.ts`
- `fl gate` 未安装时,每次 idle 都被提示并回注 prompt(noise);Follow-ups 中 `install.sh` 后给 setup 加 print 提示

### Follow-ups

- `--preserve-existing` 旗标(setup 遇 `opencode.json` 时合并 plugin 列表)
- Claude Code 等价 hook(`.claude/settings.json` 的 `hooks.Stop`)
- `--gate-timeout <ms>` 旗标
- `Gate::execute()` 文件锁
- 实现 `Audit` / `Implement` / `Review` 的具体 `gate()` 业务逻辑
- `install.sh` 完成后 setup 输出 `fl not in PATH? run ./install.sh` 提示
