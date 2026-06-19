# Global Constants and Utility Functions

## TL;DR

> **Quick Summary**: 新增两个文件：`src/constants.rs`（全局常量定义）+ `src/utils.rs`（全局工具函数）。常量采用真实声明（这些就是数据），工具函数区分两类：直接包装 stdlib 的简单函数（直接实现），需要业务逻辑的复杂函数（`todo!()` 骨架）。
>
> **Deliverables**:
> - `src/constants.rs` — 10+ 个 `pub const` 声明（目录名、文件名、项目标记、env 变量名）
> - `src/utils.rs` — 6 个函数签名（2 个 stdlib 包装真实实现，4 个 `todo!()` 骨架）
> - `src/lib.rs` — 注册新模块
> - 通过 `cargo check` + `cargo build` + `cargo test` 验证
>
> **Estimated Effort**: Small
> **Parallel Execution**: NO（lib.rs 必须先于两个新文件）
> **Critical Path**: lib.rs → constants.rs + utils.rs → cargo check

---

## Context

### Background
ForceLoop CLI 框架已搭建完毕（[`.sisyphus/plans/cli-framework.md`](../../.sisyphus/plans/cli-framework.md) + [`.omc/plans/command-metadata-skeleton.md`](command-metadata-skeleton.md)）。当前 10 个 Command 对象都有 `CommandMetadata` trait，但所有元数据（state 路径、目录名）都是硬编码的占位值。

### Original Request（用户原话）
> 增加一个全局常量的文件，用于定义一些全局要用的常量信息，比如 state文件的地址，另外定义一个工具函数的文件，用于定义一些全局的工具函数，比如获取当前项目的地址，运行可执行文件的地址等

### What We Need
| 文件 | 用途 | 内容深度 |
|------|------|----------|
| `src/constants.rs` | 全局常量（路径名、文件名、env 变量） | 真实声明（数据） |
| `src/utils.rs` | 全局工具函数（路径解析、I/O 辅助） | 混合：简单函数真实实现，复杂函数 `todo!()` 骨架 |

### Why Now
当前 `Context` 结构体（[src/context.rs](../../src/context.rs:1-15)）为空占位，`gate()` 方法无法真正检查上一步的产物（[src/traits.rs](../../src/traits.rs:21-39)），`setup`/`status`/`archive` 子命令也无法找到 state 文件。在实现这些功能之前，必须先有路径常量定义和路径解析工具函数。

---

## Work Objectives

### Core Objective
建立全局常量与工具函数的基础模块，为后续 `setup`/`gate`/`status`/`archive` 子命令实现提供路径基础。

### Concrete Deliverables
- `src/constants.rs` — 全局常量模块
- `src/utils.rs` — 全局工具函数模块
- `src/lib.rs` — 注册 `pub mod constants;` 和 `pub mod utils;`

### Definition of Done
- [ ] `src/constants.rs` 包含至少 10 个 `pub const` 声明
- [ ] `src/utils.rs` 包含至少 6 个 pub 函数签名
- [ ] `src/lib.rs` 注册新模块
- [ ] `cargo check` 零错误零警告
- [ ] `cargo test` 通过
- [ ] `cargo clippy` 零问题
- [ ] 已提交

### Must Have
- `constants.rs` 中的常量是真实声明（不是 `todo!()`）
- `utils.rs` 中简单函数（current_dir, executable_path）真实实现
- `utils.rs` 中复杂函数（project_root 等）签名完整，函数体 `todo!()`
- 命名规范一致（UPPER_SNAKE_CASE for consts, snake_case for funcs）
- 公开 API（`pub` 修饰符）
- 文档注释简洁（一行说明用途）

### Must NOT Have (Guardrails)
- **零新依赖**: 不添加 `dirs` / `directories` / `once_cell` / `lazy_static`
- **零业务逻辑**: 不在 `project_root()` 中实现具体的 marker 查找逻辑
- **零全局可变状态**: 不使用 `static mut` 或 `lazy_static`
- **零 I/O 测试**: 不为 `project_root` 等未实现函数添加测试
- **零宏**: 不定义宏
- **零 trait 实现**: 不为常量/工具函数实现 trait
- **不修改 `Context`**: 本任务不扩展 `Context` 字段（留给后续任务）

---

## Design Decisions

### Decision 1: 文件命名
**决策**: `src/constants.rs` + `src/utils.rs`

**理由**:
- 严格对应用户原话"全局常量的文件"和"工具函数的文件"
- 单数形式（`constants`/`utils`）符合 Rust 命名习惯
- 模块名短而清晰

### Decision 2: 常量类型
**决策**: 全部使用 `&'static str`（字符串字面量），不用 `PathBuf` 常量

**理由**:
- 路径分隔符在不同 OS 上不同（`/` vs `\`），`&'static str` 更灵活
- 调用方可用 `Path::new(STATE_FILE_NAME)` 组合
- 避免 `lazy_static` / `once_cell` 依赖
- 跨平台兼容性更好

### Decision 3: 工具函数实现深度
**决策**: 分两类实现

| 类型 | 函数 | 实现深度 |
|------|------|----------|
| stdlib 包装 | `current_dir`, `executable_path` | **真实实现**（一行包装） |
| 需要业务决策 | `project_root`, `state_dir`, `state_file`, `is_in_project` | **`todo!()` 骨架** |

**理由**:
- stdlib 包装没有设计选择（怎么实现都一样），直接实现减少骨架噪音
- `project_root()` 需要决定"如何识别项目根"（找 `.git`？找 `Cargo.toml`？最多向上找 N 层？）—— 这是后续业务决策，不在本计划范围
- 用户原话"先构建骨架"在上一任务中已确立为通用规约

### Decision 4: 函数签名风格
**决策**: 一律使用 `Result<PathBuf>` 返回（基于 `crate::errors::Result`），不用 panic / unwrap

**理由**:
- 与现有 `execute()` / `gate()` 错误处理风格一致
- 路径解析可能失败（权限、磁盘错误），应向上传播
- 不违反 CLAUDE.md "禁止裸 `unwrap()`"

### Decision 5: 不引入 `dirs` crate
**决策**: 不使用 `dirs`/`directories` crate 查找系统目录

**理由**:
- 当前不涉及用户 home 目录（`~/.forceloop/` 之类）
- 用户原话"当前项目的地址"和"运行可执行文件的地址"都是简单的 stdlib 调用
- 减少依赖，符合"零新依赖"约束
- 未来如需 system dirs，可单独评估

---

## Implementation Steps

### Step 1: 注册新模块到 `src/lib.rs`

**File**: [src/lib.rs](../../src/lib.rs:1-9)

**What to do**:
在 `lib.rs` 中按字母顺序插入两个新模块声明：

```rust
pub mod archive;
pub mod cli;
pub mod commands;
pub mod constants;   // 新增
pub mod context;
pub mod errors;
pub mod gate;
pub mod setup;
pub mod status;
pub mod traits;
pub mod utils;       // 新增
```

**Must NOT do**:
- 不调整其他模块顺序
- 不重命名现有模块
- 不添加 `pub(crate)` 等可见性修饰

**References**: 现有 lib.rs 结构 [src/lib.rs:1-9](../../src/lib.rs#L1-L9)

**Acceptance Criteria**:
- `lib.rs` 包含两个新 `pub mod` 声明
- 字母顺序保持

---

### Step 2: 创建 `src/constants.rs`

**File**: `src/constants.rs`（新文件）

**What to do**:
创建文件，定义以下常量（全部 `pub const NAME: &str = "value";`）：

```rust
/// ForceLoop project directory name (under project root)
pub const FORCELOOP_DIR: &str = ".forceloop";

/// State file name (inside .forceloop/)
pub const STATE_FILE: &str = "state.json";

/// Development plan file name (inside .forceloop/)
pub const PLAN_FILE: &str = "plan.json";

/// Skills subdirectory name (inside .forceloop/)
pub const SKILLS_DIR: &str = "skills";

/// Custom commands subdirectory name (inside .forceloop/)
pub const COMMANDS_DIR: &str = "commands";

/// Hooks subdirectory name (inside .forceloop/)
pub const HOOKS_DIR: &str = "hooks";

/// Archive subdirectory name (inside .forceloop/, for archived plans)
pub const ARCHIVE_DIR: &str = "archive";

/// Git directory name (project marker)
pub const GIT_DIR: &str = ".git";

/// Cargo manifest file name (project marker)
pub const CARGO_MANIFEST: &str = "Cargo.toml";

/// Environment variable name for forcing project root override
pub const ENV_PROJECT_ROOT: &str = "FORCELOOP_PROJECT_ROOT";

/// Environment variable name for debug/verbose output
pub const ENV_DEBUG: &str = "FORCELOOP_DEBUG";
```

**Must NOT do**:
- 不使用 `pub static`（应为 `pub const`）
- 不使用 `String` 类型（应用 `&'static str`）
- 不添加 `cfg(test)` 块
- 不在常量值中嵌入路径分隔符（保持纯文件名/目录名）

**Acceptance Criteria**:
- 文件包含 11 个 `pub const` 声明
- 全部为 `&'static str` 类型
- 每个常量有 1 行文档注释
- `cargo check` 通过

---

### Step 3: 创建 `src/utils.rs`

**File**: `src/utils.rs`（新文件）

**What to do**:
创建文件，定义以下函数（混合实现深度）：

```rust
use std::path::{Path, PathBuf};

use crate::errors::Result;

/// Returns the current working directory.
///
/// Wraps `std::env::current_dir()` for consistency with the rest of the API.
pub fn current_dir() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

/// Returns the path to the currently running executable.
///
/// Wraps `std::env::current_exe()` for consistency.
pub fn executable_path() -> Result<PathBuf> {
    Ok(std::env::current_exe()?)
}

/// Returns the project root directory (where `.forceloop/` lives).
///
/// Skeleton: requires business decision on marker file (`.git` vs `Cargo.toml`).
pub fn project_root() -> Result<PathBuf> {
    todo!()
}

/// Returns the `.forceloop/` directory under the project root.
pub fn state_dir() -> Result<PathBuf> {
    todo!()
}

/// Returns the absolute path to the state file.
pub fn state_file() -> Result<PathBuf> {
    todo!()
}

/// Returns `true` if the current directory is inside a ForceLoop project.
pub fn is_in_project() -> bool {
    todo!()
}
```

**Must NOT do**:
- 不为 `todo!()` 函数提供任何实现
- 不在 `project_root()` 中实际遍历目录
- 不在 `state_dir()` 中调用 `project_root()`（避免循环依赖）
- 不添加单元测试
- 不为函数添加默认参数或泛型
- 不使用 `unsafe`

**Acceptance Criteria**:
- 文件包含 6 个 `pub fn` 声明
- 2 个 stdlib 包装函数（current_dir, executable_path）真实实现
- 4 个 `todo!()` 函数（project_root, state_dir, state_file, is_in_project）
- 每个函数有 1 行文档注释
- `cargo check` 通过

**Note on inter-function calls**: `state_dir()` / `state_file()` / `is_in_project()` 在骨架阶段**不调用** `project_root()`，避免未实现函数级联。后续任务中会先确定 `project_root()` 的 marker 策略，再串联调用链。

---

### Step 4: 编译验证

**What to do**:
```bash
cargo check
cargo build
cargo test
cargo clippy
```

**Expected**:
- `cargo check` exit 0，零错误零警告
- `cargo build` 成功
- `cargo test` 全部通过
- `cargo clippy` 零问题

**Must NOT do**:
- 不为 `todo!()` 函数添加测试（会 panic）
- 不修改 `Context` 或任何现有结构体

**Acceptance Criteria**:
- 所有命令零退出码
- `forceloop --help` 仍正常显示 4 个子命令

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| `project_root()` 的 marker 选择后续改变需重写 | 低 | 中 | 函数体为 `todo!()`，marker 策略独立决策 |
| `state_dir()` / `state_file()` 串联调用导致 `todo!()` 传播 | 中 | 中 | 骨架阶段各自独立 `todo!()`，不串联 |
| `&'static str` 常量在 Windows 路径拼接时需转换 | 低 | 低 | 调用方用 `Path::new(STATE_FILE)` 转换，Rust 自动处理 |
| `pub` 暴露过多内部常量 | 低 | 低 | 所有常量都是路径名/文件名，无敏感信息 |
| 未来需要 `PathBuf` 常量（如已构造的路径） | 中 | 中 | 当前全用 `&'static str`，未来可加 `pub fn xxx_path() -> PathBuf` 工厂函数 |
| 测试 `forceloop --help` 时 panic（如果集成测试触发 todo!()） | 低 | 低 | `--help` 不调用任何 utils 函数 |

---

## Verification Strategy

### Build Verification
```bash
cargo check                    # 零错误零警告
cargo build                    # exit 0
cargo test                     # all tests pass
cargo clippy                   # No issues
./target/debug/forceloop --help  # 仍显示 4 个子命令
```

### Structural Verification
- `grep -c "pub const" src/constants.rs` → 11
- `grep -c "pub fn" src/utils.rs` → 6
- `grep "pub mod" src/lib.rs` → 包含 `constants` 和 `utils`

### Functional Verification (for implemented functions only)
- `current_dir()` 调用后返回的 PathBuf 等于 `std::env::current_dir()`
- `executable_path()` 调用后返回的 PathBuf 等于 `std::env::current_exe()`
- `todo!()` 函数**不调用**（避免 panic）

**注意**: 不为 `todo!()` 函数添加测试（违反"零 I/O 测试"约束）

---

## Commit Strategy

### Single Commit（3 个文件）
```
feat(core): add global constants and utility functions modules

- New: src/constants.rs with 11 pub const declarations
  (FORCELOOP_DIR, STATE_FILE, PLAN_FILE, SKILLS_DIR, COMMANDS_DIR,
   HOOKS_DIR, ARCHIVE_DIR, GIT_DIR, CARGO_MANIFEST, env var names)
- New: src/utils.rs with 6 pub functions
  (current_dir, executable_path as real stdlib wrappers;
   project_root, state_dir, state_file, is_in_project as todo!() skeleton)
- Modified: src/lib.rs to register constants and utils modules

Real implementations only for trivial stdlib wrappers.
Complex path-resolution functions remain todo!() pending marker design.

Files: src/lib.rs, src/constants.rs, src/utils.rs
```

**Pre-commit**: `cargo check` (zero errors/warnings) + `cargo test` (all pass) + `cargo clippy` (no issues)

---

## Final Checklist

- [ ] `src/lib.rs` 注册 `constants` 和 `utils` 模块
- [ ] `src/constants.rs` 包含 ≥10 个 `pub const` 声明
- [ ] `src/utils.rs` 包含 6 个 `pub fn` 声明
- [ ] `current_dir` 和 `executable_path` 真实实现（stdlib 包装）
- [ ] `project_root`, `state_dir`, `state_file`, `is_in_project` 为 `todo!()` 骨架
- [ ] `cargo check` 零错误零警告
- [ ] `cargo test` 通过
- [ ] `cargo clippy` 零问题
- [ ] `forceloop --help` 仍正常
- [ ] 无新依赖
- [ ] 无 `unwrap()` / `panic!()`
- [ ] 所有常量是 `&'static str`
- [ ] 文档注释简洁（≤1 行/项）

---

## Follow-ups (Out of Scope)

以下内容**不在本计划范围**，留待后续任务：

1. **`project_root()` 实现**: 需要决定 marker 文件（`.git` / `Cargo.toml` / `.forceloop`），向上遍历的最大层数，错误处理策略
2. **串联调用链**: `state_dir()` = `project_root()?.join(FORCELOOP_DIR)` 等
3. **PathBuf 工厂函数**: 如 `pub fn state_path() -> PathBuf` 直接返回构造好的路径
4. **常量分组**: 当前所有常量平铺，未来可能按子模块分组（state 常量、env 常量等）
5. **I/O 辅助函数**: `read_state() / write_state() / state_exists()` 等
6. **错误变体**: 在 [src/errors.rs](../../src/errors.rs:1-19) 中增加 `ProjectRootNotFound` 等专用错误
7. **集成测试**: 端到端测试 utils 函数（需要先实现 `project_root`）
8. **Windows 兼容性**: 当前 `&'static str` + `Path::new()` 已处理，无需特殊关注
