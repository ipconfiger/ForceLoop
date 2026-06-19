# OpenCode Hook — 规范修正（含路径 + 插件结构）

## TL;DR

`fl setup` 在 OpenCode 目标下写入的**项目级文件路径**和**插件代码结构**都与 OpenCode 官方文档的约定不一致。

本文档记录两轮修正：

1. **路径修正**（第一轮）：文件应放项目级 `<root>/.opencode/`，不是项目根
2. **插件结构修正**（第二轮）：插件应该用 `export const X: Plugin` + 键名订阅 `session.idle`，不是 `export default` + 内部 `event.type` 过滤

## 源参考

- OpenCode 插件文档：<https://opencode.ai/docs/plugins/>
- OpenCode 配置文档：<https://opencode.ai/docs/zh-cn/config/>
- 真实工作示例：`~/.config/opencode/plugins/rtk.ts`
- 修正 plan 链：
  - v1 (superseded)：`.omc/plans/opencode-session-idle-gate-hook.md`
  - v2 (superseded)：`.omc/plans/setup-opencode-hook-correction.md`
  - 现行代码：`src/setup.rs` + `plugin/fl.ts`

---

## 第一轮修正：路径

### 正确路径（按 OpenCode 文档）

| 文件 | 位置 | 说明 |
|------|------|------|
| 插件目录 | `<project_root>/.opencode/plugins/` | 项目级，**OpenCode 启动时自动加载** |
| 插件目录 | `~/.config/opencode/plugins/` | 用户级，跨项目 |
| 配置文件 | `~/.config/opencode/opencode.json` | 用户级配置 |
| 配置文件 | `<project_root>/.opencode/opencode.json` | 项目级配置（**仅当需要项目级 npm 插件时**） |

文档原文：

> "Place JavaScript or TypeScript files in the plugin directory:
> - `.opencode/plugins/` - Project-level plugins
> - `~/.config/opencode/plugins/` - Global plugins
>
> Files in these directories are **automatically loaded at startup**."
>
> "Specify npm packages in your config file:
> ```json
> { "plugin": ["opencode-helicone-session", "opencode-wakatime"] }
> ```"

### 关键点

- **本地 TS/JS 插件靠目录约定自动加载**——**不需要在 `opencode.json` 里登记**
- `opencode.json` 的 `plugin` 字段**只接受 npm 包名**，不接受本地文件路径
- 把 `./plugins/fl.ts` 塞进 `plugin` 数组 = OpenCode 试图 `bun install ./plugins/fl.ts`，**静默失败/忽略**

### ForceLoop 实现

`fl setup --tool opencode` 只写一个文件：

| 写入 | 路径 | 来源 |
|------|------|------|
| 插件 | `<root>/.opencode/plugins/fl.ts` | `include_str!("../plugin/fl.ts")` 嵌入模板 |

**不写 `opencode.json`**——本插件靠目录约定加载，不需要配置项。

### 旧位置迁移

`write_opencode_hook()` 在写新位置前会检测并删除：

- `<root>/opencode.json`（v1 错的位置）
- `<root>/plugin/`（v1 错的目录）
- `<root>/.opencode/opencode.json`（v2 错误创建的配置）

---

## 第二轮修正：插件代码结构

### 错的结构（v1 + v2 用的）

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export default (async (ctx) => {
  const { client, $ } = ctx
  return {
    event: async ({ event }) => {
      if (event.type !== "session.idle") return  // ❌ 字符串比对过滤
      // ...
    },
  }
}) satisfies Plugin
```

**问题**：
- `export default` 而非 `export const X: Plugin`
- `event:` 不是合法 hook 订阅键——是个普通字符串 key
- `if (event.type !== "session.idle")` 用字符串比对过滤——**机制完全错**

### 对的结构（按 [OpenCode 插件文档](https://opencode.ai/docs/plugins/)）

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const FlGateHook: Plugin = async ({ client, $ }) => {
  return {
    "session.idle": async ({ event }) => {
      // 键名本身就是事件订阅，handler 只在 session.idle 触发
      const sessionID = event.properties?.sessionID
      if (!sessionID) return

      const result = await $`fl gate`.timeout(60_000).nothrow()
      if (result.exitCode !== 0) {
        await client.session.prompt({
          path: { id: sessionID },
          body: {
            noReply: false,
            parts: [{ type: "text", text: result.stdout + result.stderr }],
          },
        })
      }
    },
  }
}
```

**关键约定**：
1. `export const <Name>: Plugin = async (ctx) => { ... }`（命名导出 + 显式类型注解）
2. **hook 是带 dotted key 的对象**——key 名 = 事件名
3. handler 签名：`async ({ event }) => { ... }`（event 已经是过滤后的）
4. BunShell 用 `.nothrow()` 避免非零抛异常，手动检查 `result.exitCode`

### 完整的可用 hook 名（OpenCode 文档）

**Session**: `session.idle`, `session.created`, `session.deleted`, `session.updated`, `session.status`, `session.diff`, `session.error`, `session.compacted`

**Tool**: `tool.execute.before`, `tool.execute.after`

**Message**: `message.updated`, `message.removed`, `message.part.updated`, `message.part.removed`

**File**: `file.edited`, `file.watcher.updated`

**Permission**: `permission.asked`, `permission.replied`

**TUI**: `tui.prompt.append`, `tui.command.execute`, `tui.toast.show`

**Experimental**: `experimental.session.compacting`

**LSP / Shell / Server / Todo / Installation / Command**: 详见 [OpenCode 插件文档](https://opencode.ai/docs/plugins/)

---

## 未经验证的假设

`client.session.prompt({ path: { id: sessionID }, body: { noReply: false, parts: [...] } })` 是按 SDK 描述（"OpenCode SDK client for interacting with the AI"）推断的 API 调用。**未经运行时验证**。如果 OpenCode SDK 签名不同，hook 在 `fl gate` 失败时会抛错。需要：

1. 实际在 OpenCode 里触发 session.idle（`fl gate` 失败路径）
2. 观察是否报错
3. 如报错，查 OpenCode SDK 文档（<https://opencode.ai/docs/sdk>）确认正确 API
4. 更新 `plugin/fl.ts` 和对应单元测试

---

## 验证

### 静态检查

```bash
cd ~/Projects/testspace/todo_test1
cat .opencode/plugins/fl.ts | head -5
# 期望第一行: import type { Plugin } from "@opencode-ai/plugin"
# 期望有:   export const FlGateHook: Plugin = async ({ client, $ }) => {
# 期望有:   "session.idle": async ({ event }) => {
# 不期望有: event: async ({ event }) =>
# 不期望有: if (event.type !== "session.idle")
```

### 运行时检查（推荐）

```bash
cd ~/Projects/testspace/todo_test1
rm -f .opencode/opencode.json   # 旧的（不需要）

# 把 hook 临时改成 fl xxx 触发失败路径
sed -i '' 's/fl gate/fl xxx/' .opencode/plugins/fl.ts
grep "fl " .opencode/plugins/fl.ts

# 启动 OpenCode
opencode
# TUI: > hello （走完一回合触发 session.idle）
# 观察: session 是否自动追加内容（fl xxx 失败的输出）

# 测完恢复
sed -i '' 's/fl xxx/fl gate/' .opencode/plugins/fl.ts
```

### 检查 plugin 加载

启动 OpenCode 时 banner 里应出现 plugin 列表。`stderr` 里可能也有：

```bash
opencode 2>&1 | head -30
# 看 banner / 启动日志里有无 "FlGateHook" 或 "fl.ts" 出现
```

---

## 对外部 spec 的影响

如果外部 spec（如 `testspace/todo_test1/docs/opencode-auto-state-driver.md`）要保持与 ForceLoop 一致：

1. 插件代码从 `export default (async ...) satisfies Plugin` 改为 `export const X: Plugin = async (...)`
2. 事件订阅从 `event: { if (event.type !== "X") return; ... }` 改为 `"X": async ({ event }) => { ... }`
3. 路径从 `<root>/opencode.json` + `<root>/plugin/hook.ts` 改为 `<root>/.opencode/plugins/fl.ts`
4. 删除 `opencode.json` 的创建——目录约定就是声明
5. BunShell 改用 `.nothrow()` 修饰符 + 手动检查 `result.exitCode`

