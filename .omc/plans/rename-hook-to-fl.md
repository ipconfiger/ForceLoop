# Plan: Rename `plugin/hook.ts` → `plugin/fl.ts`

## Requirements Summary

把 OpenCode hook 插件文件从泛化的 `hook.ts` 改名为具体的 `fl.ts`，包括模板源、输出路径和所有引用。

## Changes

| # | File | Change | Why |
|---|------|--------|-----|
| 1 | `plugin/hook.ts` | Rename to `plugin/fl.ts` (git mv) | 源文件改名 |
| 2 | `src/setup.rs` | `include_str!("../plugin/fl.ts")`, 输出路径 `./plugins/fl.ts`, join("fl.ts"), doc 注释更新 | 编译嵌入、输出路径、注释 |
| 3 | `tests/setup_tool.rs` | 所有 `hook.ts` 路径断言 → `fl.ts`, 字符串比较 `./plugins/hook.ts` → `./plugins/fl.ts` | 测试文件 |
| 4 | `CLAUDE.md` | 所有 `plugin/hook.ts` → `plugin/fl.ts` | 项目说明文档 |
| 5 | `docs/opencode-hook-spec-correction.md` | 所有 `hook.ts` → `fl.ts` | 参考文档 |

## Not Changed

- `.omc/plans/` — 历史 plan 文件，不改
- `.omc/project-memory.json` — OMC 内部状态，不改

## Acceptance Criteria

- `cargo test` passes
- `cargo clippy --all-targets` passes
- `grep -rn "hook\.ts" plugin/ src/ tests/ CLAUDE.md docs/` returns only historical doc hits (`.omc/`)

## Verification

```bash
cargo test
cargo clippy --all-targets
grep -rn "hook\.ts" plugin/ src/ tests/ CLAUDE.md docs/ --include="*.rs" --include="*.ts" --include="*.md"
```