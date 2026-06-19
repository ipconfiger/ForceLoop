# Plan: 注册 `New` 为 CLI 子命令

## Requirements

1. 注册 `New` 到 CLI 子命令系统，实现 `fl new` 调用 `execute()` 创建目录
2. 厘清 Schema name (`fl-new`) 与 CLI 子命令名 (`new`) 的关系

## Architecture 澄清

当前系统有**两套完全独立的命名系统**：

| 系统 | 来源 | 示例 | 产生 |
|------|------|------|------|
| CLI 子命令名 | `Subcommand::name()` | `fl new`, `fl setup` | main.rs dispatch |
| IDE 斜杠命令名 | `COMMANDS` 表第一个元素 | `/fl-new`, `/fl-plan` | setup.rs 写文件名 |

`CommandSchema::name` (= `"fl-new"`) 只影响 IDE 斜杠命令的文件名，**不影响 CLI 子命令**。不存在 `fl fl-new` 的问题。

## Changes

### 1. `src/commands/new_cmd.rs` — 实现 `Subcommand`

```rust
impl Subcommand for New {
    fn name(&self) -> &'static str { "new" }
    fn description(&self) -> &'static str {
        "Create a new development goal and design spec"
    }
}
```

### 2. `src/cli.rs` — 添加 `New` 变体

```rust
pub enum Commands {
    Setup { ... },
    Gate,
    /// Create a new development goal and design spec
    New,
    Status,
    Archive,
}
```

### 3. `src/main.rs` — 分发

```rust
Commands::New => forceloop::commands::New.execute(&ctx)?,
```

### 4. `tests/cli_help.rs` — 更新测试

`help_shows_all_subcommands`：增加 `stdout.contains("new")`

## 不变

- `CommandSchema::name` 保持 `"fl-new"` — 这是 IDE 斜杠命令名，与 CLI 无关
- `COMMANDS` 表保持 `("fl-new", ...)` — 这是 IDE 文件名
- 其他 5 个 commands 暂不注册（等各自 execute() 实现时再注册）
- `src/new_cmd.rs` 的 `Subcommand` trait import

## Verification

```bash
cargo run -- --help  # 应显示 new
cargo run -- new     # 应创建 .forceloop/specs/ 并打印提示
cargo test
```