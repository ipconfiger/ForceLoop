# Plan: 每轮 implement 只加载一个未完成的 wave

## 问题

当前 prompt 让 LLM 一次性加载所有 wave 的 checklist 到 todo list，导致工作项过多。

## 修复

每次 `/fl-implement` 只处理一个未完成的 wave：
1. 读 wave_state.md
2. 找到第一个 `- [ ]` 的 item
3. 只加载这一个 wave 的工作项到 todo
4. 完成后更新 checklist
5. 告诉用户重新运行 `fl implement` 继续下一波

## Changes

### `src/commands/implement.rs` — SKILL_PROMPT + COMMAND_PROMPT

去掉"重复直到全部完成"的逻辑，改为每轮只处理一个 wave：

```rust
## Steps
0. Run `fl implement` first.
1. Read `.forceloop/wave_state.md`.
   - If missing: generate from plans/index.md (aggregate all wave checklists).
2. Find the FIRST unchecked item (`- [ ]`) from top to bottom.
3. Identify which wave file in `.forceloop/plans/` this item belongs to.
4. Load ONLY that wave file. Add its work items to the todo list.
   Do NOT load other wave files — focus on one wave at a time.
5. Execute TDD per the wave plan.
6. Update checklists in BOTH the wave file and wave_state.md.
7. Run `fl gate`. If it fails, run `/fl-implement` again for the next wave.
```

核心变化：步骤 4 强调"只加载这一个 wave"，步骤 7 改为外循环驱动。

## Verification

```
cargo test
cargo clippy
```