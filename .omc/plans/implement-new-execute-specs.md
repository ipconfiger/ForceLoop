# Plan: 实现 `fl implement` — 波次驱动开发 + wave_state.md 门控

## Requirements

1. 注册 `Implement` 为 CLI 子命令
2. `execute()` 检测 prerequisites（new, plan, audit 全部 true）
3. Prompt 驱动 LLM：
   - 读取 `.forceloop/wave_state.md`（波次 checklist）
   - 如果不存在，从 `.forceloop/plans/index.md` 读取所有 wave，生成 wave_state.md
   - 找到第一个未 check 的 item → 去 `.forceloop/plans/` 找对应计划文件 → 开发
   - 完成后更新 plan 文件的 checklist + wave_state.md
   - 重复直到所有 item 全部 ✅
4. `artifacts() → &[".forceloop/wave_state.md"]`
5. `gate()`: verify_artifact + verify_checklist（同 audit 模式）
6. `check_list() → true`

## Changes

### 1. `src/constants.rs`

```rust
pub const WAVE_STATE: &str = "wave_state.md";
```

### 2. `src/commands/implement.rs`

- `Implements` Subcommand: `name() -> "implement"`
- `execute()`: 读 state → 检查 new && plan && audit → 通过后提示
- `check_list() -> true`
- `artifacts() -> &[".forceloop/wave_state.md"]`
- `gate()`: verify_artifact + verify_checklist
- SKILL_PROMPT: 详细步骤（读 wave_state.md → 取未完成 item → 开发 → 更新 checklist）
- COMMAND_PROMPT: 合并完整步骤

### 3. `src/cli.rs`

```rust
pub enum Commands {
    ...
    /// Develop implementation following wave-based TDD
    Implement,
    ...
}
```

### 4. `src/main.rs`

```rust
Commands::Implement => forceloop::commands::Implement.execute(&ctx)?,
```

### 5. `tests/cli_help.rs`

增加 `"implement"` 断言

## Verification

```bash
cargo test
cargo clippy
fl --help       # 显示 implement
fl implement    # 检查前提条件
fl gate         # 验证 wave_state.md 的 checklist
```