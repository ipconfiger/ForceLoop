# Plan: 斜杠命令 fl- 前缀 + Prompt 触发 `fl new`

## Requirements

1. **所有斜杠命令加 `fl-` 前缀**，避免与系统命令冲突
   - `new` → `fl-new`
   - `plan` → `fl-plan`
   - `audit` → `fl-audit`
   - `implement` → `fl-implement`
   - `review` → `fl-review`
   - `try_finish` → `fl-try-finish`

2. **COMMAND_PROMPT / SKILL_PROMPT 加入 `fl new` 执行步骤**
   - 当前 SKILL_PROMPT 只描述"目录由 `fl new` 自动创建"，但 LLM 实际上不会去执行它
   - 需在 prompt 中明确告诉 LLM 先执行 `fl new` 创建目录，再生成 spec 文件

---

## 变更清单

### 1. `src/setup.rs` — COMMANDS 表

```
("new", ...)    → ("fl-new", ...)
("plan", ...)   → ("fl-plan", ...)
("audit", ...)  → ("fl-audit", ...)
("implement", ...) → ("fl-implement", ...)
("review", ...) → ("fl-review", ...)
("try_finish", ...) → ("fl-try-finish", ...)
```

### 2. `src/commands/new_cmd.rs` — Prompt 更新

**SKILL_PROMPT**: 增加步骤 0 "Run `fl new` to create `.forceloop/specs/` directory"

**COMMAND_PROMPT**: 改为引用完整工作流，明确指示：
1. 先运行 `fl new`（创建目录）
2. 再进行需求分析、模块拆分
3. 生成 spec 文件

**execute()**: 更新输出信息中的 `/new` 引用为 `/fl-new`

### 3. `src/commands/{plan,audit,implement,review,try_finish}.rs`

Schema name 加 `fl-` 前缀：
- `plan.rs` → `name: "fl-plan"`
- `audit.rs` → `name: "fl-audit"`
- `implement.rs` → `name: "fl-implement"`
- `review.rs` → `name: "fl-review"`
- `try_finish.rs` → `name: "fl-try-finish"`

### 4. `tests/setup_tool.rs`

所有 `new.md` 断言 → `fl-new.md`，命令文件名期望集更新

### 5. `tests/command_compile.rs`

`compile_agent("implement", ...)` → `compile_agent("fl-implement", ...)`  
断言 `name: fl-implement`

---

## 不更改

- `src/state.rs` 中的 `PipelinePhase` 枚举值（`new`, `plan`, ...）— 这是内部状态枚举，不是斜杠命令名

---

## 验证

```bash
cargo test
cargo clippy --all-targets
cargo check
cd ~/Projects/testspace/todo_test1 && fl setup --tool opencode
ls .opencode/command/  # 应看到 fl-new.md 而非 new.md
```