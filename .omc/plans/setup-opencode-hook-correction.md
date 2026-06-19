# OpenCode Hook 安装路径修正

> **Superseded by**: 实际代码（`src/setup.rs` + `plugin/hook.ts`）+ [`docs/opencode-hook-spec-correction.md`](../docs/opencode-hook-spec-correction.md)
>
> 本 plan 已被**第二轮修正**取代。第二轮发现：
> 1. `plugin` 字段是给 npm 包用的，不接受本地文件路径——**`opencode.json` 不应该被创建**
> 2. 插件结构应该用 `export const X: Plugin` + 键名订阅，不是 `export default` + 内部 `event.type` 过滤
>
> 现行实现见代码 + spec 修正文档，本文件保留作为历史。

---

## Context

### 背景

原 plan [.omc/plans/opencode-session-idle-gate-hook.md](opencode-session-idle-gate-hook.md) 实现的项目级 OpenCode hook 安装路径**与 OpenCode 官方文档冲突**：

- **写错位置**：把 `opencode.json` 写到项目根（应为 `.opencode/opencode.json`）
- **写错命名**：把插件目录命名为 `plugin/`（应为 `.opencode/plugins/`）
- **破坏合并语义**：直接 `fs::write` 覆盖已有 `opencode.json`，违反文档"配置文件是合并在一起的"约定

**根因**：plan 里的 reference doc `opencode-auto-state-driver.md`（位于 `testspace/todo_test1/docs/`，**非 ForceLoop 仓库内**）写的就是错的，plan 照抄 spec 时未核对 OpenCode 官方文档（https://opencode.ai/docs/zh-cn/config/）。

### 设计决定（已与用户确认）

| 决定点 | 选择 |
|--------|------|
| 安装位置 | **项目级 `.opencode/`**（用户确认） |
| 旧文件处理 | **删旧建新**（迁移到新位置） |
| 已有 config 时的合并策略 | **读取 → 追加 plugin → 去重 → 写回** |

### 旧实现（已 supersede）

参考 [opencode-session-idle-gate-hook.md](opencode-session-idle-gate-hook.md) 的旧 Decision 1。

### What this does NOT change

- `plugin/hook.ts` 模板（ForceLoop 仓库内）位置不动 —— 仍是 `include_str!` 源
- 6 个 command 文件路径（`.opencode/command/<name>.md`）不动
- Claude Code 目标完全不变
- `CommandMetadata` / `compile()` / `compiler.rs` / `cli.rs` 全部不动

---

## Work Objectives

### Core Objective

让 `fl setup --tool opencode` 在项目级位置 `<root>/.opencode/` 下安装 hook，**遵循 OpenCode 官方文档的合并语义**，并迁移任何旧版本 `fl setup` 留下的项目根遗留文件。

### Definition of Done

- [x] `src/setup.rs` 新增 `merge_opencode_plugin(json_path, fl_plugin)` 函数
- [x] `src/setup.rs` 重写 `write_opencode_hook(root, written)`：先迁移旧文件，再写新位置，用 merge 写入 opencode.json
- [x] `src/setup.rs` 单元测试 5 个覆盖 merge 契约
- [x] `tests/setup_tool.rs` 3 个旧测试更新路径断言 + 3 个新测试
- [x] `docs/opencode-hook-spec-correction.md` 创建
- [ ] `.omc/plans/opencode-session-idle-gate-hook.md` 加 `> **Superseded by**` 标记
- [ ] `cargo check` / `cargo build` / `cargo test` / `cargo clippy --all-targets` 全绿

### Must Have

- 新位置严格按 OpenCode 官方文档：`<root>/.opencode/opencode.json` 和 `<root>/.opencode/plugins/hook.ts`
- `opencode.json` 内的 plugin 路径为 `./plugins/hook.ts`（相对于 `opencode.json` 所在目录，即 `.opencode/`）
- **合并语义**：读已有 → 追加 plugin 数组 → 去重 → 保他键 → 写回
- **错误显式**：plugin 字段不是数组、root 不是对象、JSON 解析失败 → 返回 `Config` / `Parse` 错误，**不静默覆盖**
- **幂等**：跑两次 `fl setup` 结果一致，plugin 数组不会重复
- **迁移**：旧位置 `<root>/opencode.json` 和 `<root>/plugin/` 存在则删除
- 编译零警告
- 单元测试 15 个 + 集成测试 19 个全过

### Must NOT Have (Guardrails)

- **不**改 Claude Code 目标的任何行为
- **不**改 6 个 command 文件的写入路径
- **不**改 `plugin/hook.ts` 模板内容或位置
- **不**引入新依赖
- **不**改 `compiler.rs` / `schema.rs` / `cli.rs` / `main.rs` / `state.rs` / `context.rs` / `errors.rs` / `traits.rs`
- **不**改 `CommandMetadata` trait 或 6 个 `src/commands/` 对象的 impl
- **不**自动修改 `testspace/todo_test1` 内的 spec 文档（用户保留用于手动测试）

---

## Design Decisions

### Decision 1: 写哪个位置？

**决策**：项目级 `<root>/.opencode/opencode.json` 和 `<root>/.opencode/plugins/hook.ts`。

**理由**：
- OpenCode 文档明文："将插件文件放置在 `.opencode/plugins/`"
- 用户已确认项目级（与 clone-and-go 工作流一致）
- 不用用户级 `~/.config/opencode/`（需要 `--scope` flag，引入复杂度）

### Decision 2: 合并还是覆盖？

**决策**：读 → 追加 plugin → 去重 → 写回。

**理由**：
- 文档原文："配置文件是合并在一起的，而不是替换"
- `fs::write` 直接覆盖会丢用户的其他键（`theme`、`provider`、其它 plugin 条目）
- 幂等：再跑一次不会重复添加 fl 的 plugin 条目

### Decision 3: 旧位置遗留文件怎么办？

**决策**：检测到 `<root>/opencode.json` 或 `<root>/plugin/` 则删除。

**理由**：
- 旧实现是 bug，不应该让遗留文件留在项目根
- 用户已确认删旧建新
- 旧 `opencode.json` 内容（如果有用户自定义 plugin）已经在迁移前**无法保留**——因为旧实现本身就是覆盖式 fs::write，本来也没合并
- 旧 `plugin/hook.ts` 内容在 `include_str!` 嵌入时已被固定为模板源，迁移后新版 hook.ts 内容一致

### Decision 4: 错误处理

**决策**：`merge_opencode_plugin` 遇到以下情况返回错误，**不静默覆盖**：

| 情况 | 错误 | 行为 |
|------|------|------|
| `plugin` 字段不是数组 | `Config` | 文件不写回，用户内容保留 |
| root 不是对象 | `Config` | 文件不写回 |
| JSON 解析失败 | `Parse` | 文件不写回 |

**理由**：
- 用户可能用 `opencode.json` 配置了非数组 `plugin`（虽然不符合 OpenCode schema），setup 静默覆盖是数据丢失
- 错误信息应包含文件路径 + 错误原因，便于排查

### Decision 5: 单元测试 + 集成测试分层

**决策**：
- **单元测试**（`src/setup.rs` `mod tests`）：测 `merge_opencode_plugin` 函数的纯契约（不需要 `write_opencode_hook` 的副作用）
- **集成测试**（`tests/setup_tool.rs`）：测 `run()` 端到端路径，包括 `write_opencode_hook` 的迁移和文件创建

**理由**：
- 单元测试快、确定、零 IO（除 `TempDir`）
- 集成测试覆盖真实路径，防止"merge 函数对了但 `write_opencode_hook` 写错位置"
- 反向断言（Claude 不写 hook）+ 正向断言 + 错误断言形成完整契约

---

## Implementation Steps

### Step 1: 改 `src/setup.rs`

**File**: [src/setup.rs](../../src/setup.rs) (修改)

**改动**：
1. 删除 `opencode_json_content()` 函数（dead code，由 `merge_opencode_plugin` 替代）
2. 重写 `write_opencode_hook(root, written)`：
   - 先迁移：检测并删除 `<root>/opencode.json` 和 `<root>/plugin/`
   - 写新位置：`<root>/.opencode/opencode.json` 和 `<root>/.opencode/plugins/hook.ts`
   - 用 `merge_opencode_plugin` 写入 JSON
3. 新增 `merge_opencode_plugin(json_path, fl_plugin)` 函数：
   - 创建父目录（如不存在）
   - 文件不存在 → 写 `{"plugin":[fl_plugin]}` pretty-printed
   - 文件存在 → 解析 → 检查 root 是 object → 检查 plugin 是数组 → 追加去重 → 写回
   - 错误显式（`Config` / `Parse`）

### Step 2: 单元测试（`src/setup.rs` `mod tests`）

5 个新测试（替换 `opencode_json_has_plugin_entry`）：

- `merge_writes_initial_config_when_file_absent`
- `merge_appends_when_key_absent`
- `merge_dedupes_when_key_present`
- `merge_preserves_other_keys`
- `merge_errors_when_plugin_is_not_array`
- `merge_errors_when_root_is_not_object`
- `merge_errors_on_malformed_json`

（hook 内容测试 `plugin_hook_ts_*` 5 个保留不变）

### Step 3: 集成测试（`tests/setup_tool.rs`）

**更新**：
- `run_default_writes_both_targets`：路径断言改为 `.opencode/opencode.json` + `.opencode/plugins/hook.ts`
- `claude_only_writes_claude_dir`：增加反向断言（无 `.opencode/` 目录）
- `opencode_only_writes_opencode_dir`：路径更新
- `opencode_hook_files_have_expected_content`：路径 + JSON `plugin[0]` 内容更新为 `./plugins/hook.ts`

**新增**：
- `opencode_migrates_legacy_files`：预置旧文件，跑 setup 后旧的不存在、新的存在
- `opencode_merges_into_existing_config`：预置 `.opencode/opencode.json` 含其他键，跑 setup 后两键都在
- `opencode_setup_is_idempotent_on_plugin_entry`：跑两次，plugin 数组只一条

### Step 4: 创建 `docs/opencode-hook-spec-correction.md`

参考 [opencode-hook-spec-correction.md](../../docs/opencode-hook-spec-correction.md)：
- TL;DR
- 源参考（OpenCode 文档 URL + 修正 plan + 原 plan）
- 正确路径表
- 错在哪里（与原 plan 对照）
- 修正后实现
- 对外部 spec 的影响
- 验证步骤

### Step 5: 标记旧 plan

在 [.omc/plans/opencode-session-idle-gate-hook.md](opencode-session-idle-gate-hook.md) 顶部（在 TL;DR 之前）加：

```markdown
> **Superseded by**: [setup-opencode-hook-correction.md](setup-opencode-hook-correction.md)
>
> 此 plan 的项目级文件路径（`<root>/opencode.json` + `<root>/plugin/hook.ts`）与 OpenCode 官方文档冲突，已被本 plan 修正。
```

### Step 6: 验证

```bash
cargo check
cargo build
cargo test
cargo clippy --all-targets
```

**Expected**:
- 15 单元测试 + 19 集成测试全过
- clippy 零警告

### Step 7: 手工验证（用户保留 todo_test1 做手动测试）

```bash
# 在 testspace/todo_test1 项目根
fl setup --tool opencode
ls -la
# 期望：项目根无 opencode.json、无 plugin/
ls -la .opencode/
# 期望：opencode.json, command/, plugins/
cat .opencode/opencode.json
# 期望：{ "plugin": ["./plugins/hook.ts"] }
```

---

## File Map（汇总）

| 文件 | 改动 | 行数估计 |
|------|------|----------|
| `src/setup.rs` | 删 `opencode_json_content`；重写 `write_opencode_hook`；新增 `merge_opencode_plugin`；7 个新单元测试 | +80 净增 |
| `tests/setup_tool.rs` | 4 个旧测试更新 + 3 个新测试 | +60 净增 |
| `docs/opencode-hook-spec-correction.md` (新) | 规范修正参考 | +80 |
| `.omc/plans/opencode-session-idle-gate-hook.md` | 加 `Superseded by` 标记 | +5 |
| `.omc/plans/setup-opencode-hook-correction.md` (新) | 本 plan | +250 |

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| 旧 `opencode.json` 含用户自定义 plugin 列表被覆盖 | 中 | 中 | 旧实现本身是覆盖式，没有"用户内容"可保留；新实现从空白起步，迁移时旧文件删除（用户已确认） |
| 旧 `plugin/hook.ts` 用户手动改过内容 | 低 | 低 | `include_str!` 嵌入的模板是 git 控制的，setup 后写回模板内容是 desired behavior |
| 用户已有 `.opencode/` 但 `plugin` 字段是字符串而非数组 | 低 | 低 | `merge_opencode_plugin` 返回 `Config` 错误，文件不被覆盖 |
| 用户已有 `.opencode/opencode.json` 但 JSON 损坏 | 低 | 低 | 返回 `Parse` 错误，文件不被覆盖 |
| 测试 `run_is_order_independent` 受新路径影响 | 低 | 低 | 该测试用 `BTreeSet` 比对 `file_name()`，路径变化不影响集合等价 |
| 升级后老用户不知道路径变了 | 中 | 中 | `fl setup` 当前静默（不打印），用户从 git diff 或文档发现；Future: 加 print 摘要 |

---

## Out of Scope (Explicit Boundaries)

以下**不在本计划范围**：

1. **`fl setup` 打印摘要** — setup 完应该告知用户写了哪些文件、是否迁移；独立 follow-up
2. **用户级安装 `~/.config/opencode/`** — 需要 `--scope` flag，复杂度上升；follow-up
3. **Claude Code 对称 hook** — OpenCode 是 primary target；follow-up
4. **`plugin/hook.ts` 模板位置** — 当前在 `plugin/hook.ts`（项目根），与 `include_str!` 配套，可后续清理为 `src/opencode/hook.ts`
5. **修改 `testspace/todo_test1/docs/opencode-auto-state-driver.md`** — 用户保留用于手动测试
6. **`--preserve-existing` flag** — 合并语义已实现，flag 多余
7. **60s timeout 可配置** — 硬编码，用户可手动编辑 `plugin/hook.ts`
8. **`Gate::execute()` 内部并发锁** — 与本路径修正无关

---

## Follow-ups

- `fl setup` 打印写入摘要（哪些文件是新写、哪些是合并、哪些是迁移）
- 用户级安装（`--scope global`）
- Claude Code 对称 hook（`.claude/settings.json` 的 `hooks.Stop`）
- 模板位置清理（`plugin/hook.ts` → `src/opencode/hook.ts.template`）
- `--gate-timeout <ms>` flag
- `Gate::execute()` 文件锁
- 实现 `Audit` / `Implement` / `Review` 的具体 `gate()` 业务逻辑

---

## ADR (Architecture Decision Record)

### Decision

将 `fl setup` 在 OpenCode 目标上的项目级 hook 安装路径从项目根（`<root>/opencode.json` + `<root>/plugin/hook.ts`）修正到项目级 `<root>/.opencode/opencode.json` + `<root>/.opencode/plugins/hook.ts`，加入合并语义（读 → 追加 plugin → 去重 → 写回），并迁移旧版本 `fl setup` 留下的项目根文件。

### Drivers

1. **对齐 OpenCode 官方文档** — 文档明文规定配置在 `.opencode/`、插件在 `.opencode/plugins/`、配置是合并的；原实现三条全违反
2. **不丢用户配置** — 合并语义保留 `theme` 等其他键、其他 plugin 条目
3. **幂等** — 多次运行 setup 不产生重复 plugin 条目
4. **可迁移** — 用户从旧版本升级时，旧位置文件自动清理

### Alternatives considered

- **Alternative A**:保持原路径（`<root>/opencode.json` + `<root>/plugin/hook.ts`），仅修覆盖为合并
  - **Why rejected**:仍然违反 OpenCode 文档的路径约定，根目录污染
- **Alternative B**:用用户级 `~/.config/opencode/opencode.json`
  - **Why rejected**:用户已确认项目级；用户级需 `--scope` flag
- **Alternative C**:抽 `merge_into` 为通用函数（接受任意 JSON path + key + value）
  - **Why rejected**:本计划只用于 opencode.json 的 plugin 字段；YAGNI
- **Alternative D (本计划)**:项目级 `.opencode/` + 专用 `merge_opencode_plugin` 函数
  - **Why chosen**:对齐文档 + 合并 + 迁移 + 错误显式

### Why chosen (Alternative D)

- 1 个生产文件改 `src/setup.rs`（+80 行）
- 5 单元 + 6 集成测试覆盖完整契约
- 零新依赖
- 与 OpenCode 官方约定严格一致
- 错误显式（Config/Parse），不静默覆盖用户内容

### Consequences

**正面**:
- 用户的项目根不再被 `opencode.json` 和 `plugin/` 污染
- 已有 `.opencode/opencode.json`（含 `theme`、其它 plugin）的项目升级时内容保留
- 多次 `fl setup` 幂等，plugin 数组不会重复
- 旧版本遗留文件自动清理

**负面**:
- 旧版本已跑过 setup 的项目，迁移时会**静默删除**旧 `opencode.json` 和 `plugin/`
- 旧 `opencode.json` 如果用户有自定义内容会丢（但旧实现本身是覆盖式，迁移前也没保留）

### Follow-ups

（见上文 Follow-ups 节）
