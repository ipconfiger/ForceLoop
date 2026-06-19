# WIKI Link Validator

## TL;DR

> **Quick Summary**: 在 [src/utils.rs](../../src/utils.rs) 中新增 WIKI Link 验证器。给定起始 `.md` 文件，递归验证其内部所有 wiki link 指向的文件是否存在，使用 `HashSet<PathBuf>` 防止回环（基于 canonical 路径）。支持多语法（Obsidian `[[Page]]` + 标准 MD `[text](file.md)`），返回详细报告 `WikiLinkReport`。
>
> **Deliverables**:
> - `src/utils.rs` 新增 `WikiLinkReport` struct
> - `src/utils.rs` 新增 `validate_wiki_links()` 公开函数
> - `src/utils.rs` 新增 4 个内部辅助函数（regex 提取、链接解析、路径解析、递归）
> - 单元测试覆盖主要场景（README 暂不要求）
>
> **Estimated Effort**: Medium (200-300 lines, regex + recursion + tests)
> **Parallel Execution**: NO (single module)
> **Critical Path**: utils.rs → cargo test → commit

---

## Context

### Background
ForceLoop CLI 框架已搭建完毕（[`.sisyphus/plans/cli-framework.md`](../../.sisyphus/plans/cli-framework.md) + [`.omc/plans/command-metadata-skeleton.md`](command-metadata-skeleton.md) + [`.omc/plans/constants-and-utils-skeleton.md`](constants-and-utils-skeleton.md)）。`utils.rs` 目前有 6 个工具函数（[src/utils.rs](../../src/utils.rs)），其中 `project_root()` 等 4 个还是 `todo!()` 骨架。

### Original Request（用户原话）
> 需要在工具函数里实现一个WIKI Link验证器， 也就是先验证一个md文件是否存在，存在就读取，再抓取出md文件内的wiki link，然后递归验证link里的md文件，为了防止回环，需要给每个文件地址一个hash，用hash表存储，已经验证过的文件就不要重复验证。

### Captured Design Decisions (from interview)

| Decision | Choice |
|----------|--------|
| **Link syntax** | **Multi-syntax**: Obsidian `[[Page]]` + standard MD `[text](file.md)` |
| **Return type** | **Detailed report struct**: `WikiLinkReport { visited, missing, cycles_prevented }` |
| **Link resolution** | **Both**: try relative to source file first, then project root |

### Why Now
- 当前 `gate()` 方法无法验证上一步产物的 wiki link 完整性
- 后续 `audit` / `review` / `try_finish` 技能需要验证文档链路的完整性
- 在 `project_root()` 仍为骨架的阶段，先实现独立功能（验证器不依赖 `project_root` 真实实现，可用 `Option<PathBuf>` 注入）

---

## Work Objectives

### Core Objective
在 [src/utils.rs](../../src/utils.rs) 中实现 WIKI Link 验证器，支持递归验证 + 循环检测，返回详细报告。

### Concrete Deliverables
- `pub struct WikiLinkReport` — 验证报告
- `pub fn validate_wiki_links(start: &Path, project_root: Option<&Path>) -> Result<WikiLinkReport>` — 主入口
- 4 个内部辅助函数（`fn`-scope, 非 `pub`）
- 单元测试覆盖 6+ 场景

### Definition of Done
- [ ] `WikiLinkReport` struct 定义，含 `visited`、`missing`、`cycles_prevented` 字段
- [ ] `validate_wiki_links()` 公开函数可调用
- [ ] 支持 Obsidian `[[Page]]` 语法（含 alias `|` 和 heading `#` 处理）
- [ ] 支持标准 MD `[text](file.md)` 语法
- [ ] 循环检测：使用 `HashSet<PathBuf>` + canonical 路径
- [ ] 链接解析：先尝试相对于源文件，失败则尝试相对于 project root
- [ ] 单元测试通过（6+ 场景）
- [ ] `cargo check` / `cargo test` / `cargo clippy` 零问题
- [ ] 已提交

### Must Have
- 使用 `regex` crate（已在依赖中？需检查）或纯 Rust 手写简单解析器
- 使用 `std::collections::HashSet<PathBuf>` 存储已访问路径
- 使用 `Path::canonicalize()` 规范化路径（处理 `..` 和 symlinks）
- 错误传播使用 `crate::errors::Result`
- 单元测试在文件底部 `#[cfg(test)] mod tests` 块中
- 公开 API 的 `pub` 修饰

### Must NOT Have (Guardrails)
- **零业务逻辑耦合**: 不与 `Context` / `Command` 交互
- **零修改现有代码**: 不修改 [src/constants.rs](../../src/constants.rs) 或其他文件
- **零 panic / unwrap**: 错误用 `Result` 传播
- **零新增依赖（如果可能）**: 优先手写简单解析器；如必须，新增 `regex` 到 `Cargo.toml`
- **零异步**: 全部同步代码
- **零 IO 测试夹具依赖**: 测试用 `tempfile` crate 或 inline 字符串路径（如必须 `tempfile` 加依赖）
- **不输出到 stdout/stderr**: 不打印验证结果（这是库函数，由调用方决定如何呈现）

---

## Design Decisions

### Decision 1: 链接解析策略（regex vs 手写）
**决策**: **手写简单解析器**，不引入 `regex` crate

**理由**:
- 语法固定（2 种），手写解析器代码 < 50 行
- 避免新增依赖（`regex` 是大依赖）
- 手写解析器更易控制和扩展
- 性能：手写解析对单文件 ms 级足够
- 与项目"零业务逻辑"原则一致

### Decision 2: project_root 注入
**决策**: `validate_wiki_links(start, project_root: Option<&Path>)`

**理由**:
- `project_root()` 函数当前是 `todo!()` 骨架
- 解耦：验证器不依赖未实现的 `project_root()`
- 测试友好：测试可注入临时目录
- 真实使用方（如 `gate()`）可传入 `&project_root()?` 即可
- 未来 `project_root()` 实现后，调用方无变化

### Decision 3: Hash 函数
**决策**: 使用 `Path::canonicalize()` + `HashSet<PathBuf>`，**不显式计算 hash**

**理由**:
- `HashSet<PathBuf>` 内部自动哈希（`PathBuf` 已实现 `Hash` trait）
- 用户原话"给每个文件地址一个hash，用hash表存储"——Rust 的 `HashSet` 就是这样工作的
- `canonicalize()` 处理 symlink 和 `..`，避免 `/a/b/../b.md` 和 `/a/b.md` 被视为不同
- 显式 hash 函数无附加值

### Decision 4: 错误处理
**决策**:
- 函数级错误（如 start 文件不存在、读取失败）→ `Result<WikiLinkReport>` 错误传播
- 单个 link 解析失败 → 记录到 `report.missing`，**不**作为函数级错误
- 验证器应能容忍"部分 link 失败"，不应因一个坏 link 而中止

**理由**:
- 验证器的目的是**发现**问题，不是因问题而崩溃
- 与 `Rust` "错误传播须补上下文" 原则一致
- 调用方可遍历 `missing` vec 处理每个问题

### Decision 5: 测试策略
**决策**:
- 在 `src/utils.rs` 底部添加 `#[cfg(test)] mod tests` 块
- 使用 `tempfile::TempDir` 创建临时目录（需新增 `tempfile` dev-dependency）
- 覆盖 6+ 场景（见 Verification 章节）

**理由**:
- 验证器涉及文件系统，单元测试必须有真实 FS
- `tempfile` 是 Rust 生态标准做法，自动清理
- 测试在源码文件内便于阅读和维护

---

## Public API

### Struct Definition

```rust
/// Report from wiki link validation
#[derive(Debug, Clone, Default)]
pub struct WikiLinkReport {
    /// All files successfully visited (canonical paths, no duplicates)
    pub visited: Vec<PathBuf>,

    /// Broken links: (source_file_canonical_path, link_target_text)
    pub missing: Vec<(PathBuf, String)>,

    /// Number of times a recursion was prevented by the visited set
    pub cycles_prevented: u32,
}
```

### Public Function Signature

```rust
/// Recursively validates all wiki links in markdown files starting from `start`.
///
/// Supports:
/// - Obsidian-style: `[[Page]]`, `[[Page.md]]`, `[[path/Page]]`, `[[Page|alias]]`, `[[Page#heading]]`
/// - Standard markdown: `[text](file.md)` or `[text](path/file.md)`
///
/// Link resolution order:
/// 1. Relative to source file's directory
/// 2. Relative to `project_root` (if provided)
/// 3. If neither exists, added to `report.missing`
///
/// Cycle prevention: uses a HashSet<PathBuf> of canonical paths to avoid
/// re-validating already-visited files.
pub fn validate_wiki_links(
    start: &Path,
    project_root: Option<&Path>,
) -> Result<WikiLinkReport>;
```

### Internal Helpers (not pub)

```rust
/// Extract all wiki/markdown link targets from file content
fn extract_link_targets(content: &str) -> Vec<String>;

/// Parse a single wiki link target, stripping alias/heading
/// e.g. "Page|alias" -> "Page", "Page#heading" -> "Page"
fn parse_wiki_target(raw: &str) -> &str;

/// Resolve a link target to an absolute path, trying source-relative first
/// then project-root-relative. Returns None if neither exists.
fn resolve_link(
    source: &Path,
    target: &str,
    project_root: Option<&Path>,
) -> Option<PathBuf>;

/// Internal recursive validator
fn validate_recursive(
    current: &Path,
    project_root: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
    report: &mut WikiLinkReport,
) -> Result<()>;
```

---

## Algorithm

### High-level Flow

```
validate_wiki_links(start, project_root):
    if !start.exists():
        return Err(Execution("start file not found"))
    if !start.is_file():
        return Err(Execution("start is not a file"))
    
    let mut visited = HashSet::new()
    let mut report = WikiLinkReport::default()
    
    validate_recursive(start.canonicalize()?, project_root, &mut visited, &mut report)
    
    report.visited = visited.into_iter().collect()
    report.visited.sort()  // deterministic output
    Ok(report)

validate_recursive(current, project_root, visited, report):
    let canonical = current.canonicalize()?
    if !visited.insert(canonical.clone()):
        report.cycles_prevented += 1
        return Ok(())
    
    let content = fs::read_to_string(current)?
    let targets = extract_link_targets(&content)
    
    for target in targets:
        match resolve_link(current, &target, project_root):
            Some(resolved) => {
                if !resolved.exists():
                    report.missing.push((canonical.clone(), target))
                } else {
                    validate_recursive(&resolved, project_root, visited, report)?
                }
            }
            None => {
                report.missing.push((canonical.clone(), target))
            }
```

### Link Extraction Patterns

**Obsidian wiki links** `[[...]]`:
- Pattern: scan for `[[` ... `]]` pairs
- Strip alias: `Page|alias` → `Page`
- Strip heading: `Page#heading` → `Page`
- Strip `.md` extension: not stripped (keep as-is for resolution)

**Standard markdown links** `[text](url)`:
- Pattern: scan for `](` ... `)` pairs
- Only extract targets ending in `.md` (or with `.md#heading`)
- Skip external URLs (containing `://`)
- Skip anchors (starting with `#`)

### Path Resolution

```
resolve_link(source, target, project_root):
    // Strip any heading anchor first
    let target = target.split('#').next().unwrap_or(target)
    if target.is_empty():
        return None
    
    // 1. Try relative to source file's directory
    let source_dir = source.parent().unwrap_or(Path::new("."))
    let candidate = source_dir.join(target)
    if candidate.exists():
        return Some(candidate)
    
    // 2. Try relative to project root (if provided)
    if let Some(root) = project_root:
        let candidate = root.join(target)
        if candidate.exists():
            return Some(candidate)
    
    None
```

---

## Implementation Steps

### Step 1: Add `tempfile` dev-dependency to `Cargo.toml`

**File**: [Cargo.toml](../../Cargo.toml:1-13)

**What to do**:
```toml
[dev-dependencies]
tempfile = "3"
```

**Acceptance Criteria**:
- `Cargo.toml` 包含 `[dev-dependencies]` 段
- `tempfile = "3"` 声明

### Step 2: Update imports in `src/utils.rs`

**File**: [src/utils.rs](../../src/utils.rs:1-2)

**What to do**:
在文件顶部添加：
```rust
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

use crate::errors::{ForceLoopError, Result};
```

**Note**: 现有 `use std::path::PathBuf;` 需扩展为 `use std::path::{Path, PathBuf};`

### Step 3: Add `WikiLinkReport` struct and `validate_wiki_links` function

**File**: [src/utils.rs](../../src/utils.rs:38+)

**What to do**:
在现有 `is_in_project()` 函数后追加：
- `WikiLinkReport` struct 定义
- `validate_wiki_links()` 公开函数
- 4 个内部辅助函数（`extract_link_targets`, `parse_wiki_target`, `resolve_link`, `validate_recursive`）

完整代码（~150 lines）。

### Step 4: Add unit tests

**File**: [src/utils.rs](../../src/utils.rs)

**What to do**:
在文件末尾添加：
```rust
#[cfg(test)]
mod tests { ... }
```

覆盖 6+ 场景（见 Verification 章节）。

### Step 5: Verification

```bash
cargo check
cargo test
cargo clippy
```

**Expected**:
- `cargo check` exit 0, 零错误零警告
- `cargo test` 全部通过
- `cargo clippy` 零问题

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| 解析 `[[ ]]` 时遇到嵌套或转义 | 中 | 中 | 简单线性扫描，不支持嵌套；测试覆盖正常情况 |
| `canonicalize()` 在文件不存在时失败 | 高 | 中 | 在调用 `canonicalize` 前先检查 `exists()` |
| `tempfile` dev-dep 引入拉低编译速度 | 低 | 中 | tempfile 体积小，仅测试时编译 |
| 大量文件递归导致栈溢出 | 中 | 低 | 当前不限制深度；如需要可加 `max_depth` 参数（未来任务） |
| symlink 循环 (`a -> b -> a` via symlinks) | 低 | 低 | canonicalize 处理 symlinks，HashSet 去重 |
| 大文件读取 OOM | 中 | 低 | 不限制；用户负责传入合理文件 |
| 相对路径在 Windows 上行为不同 | 低 | 低 | `Path` API 抽象跨平台 |
| `.md` 扩展名大小写 (`.MD` / `.Md`) | 低 | 中 | 暂不处理；如需要可在 resolve_link 加 `to_lowercase()` |

---

## Verification Strategy

### Build Verification
```bash
cargo check                    # 零错误零警告
cargo test                     # 全部通过
cargo clippy                   # No issues
```

### Unit Test Scenarios (6+ required)

1. **Single file with no links** — `validate_wiki_links(a.md)` where `a.md` has no links → 1 visited, 0 missing, 0 cycles
2. **File with one valid link** — `a.md → b.md` both exist → 2 visited, 0 missing
3. **File with one broken link** — `a.md → [[missing.md]]` doesn't exist → 1 visited, 1 missing
4. **Cycle detection** — `a.md → b.md → a.md` → 2 visited, 0 missing, 1 cycle prevented
5. **Standard markdown link** — `a.md` contains `[text](b.md)` → should be detected
6. **Alias and heading** — `a.md → [[Page|alias]]` and `[[Page#heading]]` → both resolve to `Page.md`
7. **Relative resolution** — `a/b.md → [[../c.md]]` resolves to `c.md`
8. **Project root fallback** — link doesn't exist relative to source, exists at project root → found
9. **Multiple references to same file** — `a.md → [[b]]` and `a.md → [[b]]` → b.md visited once
10. **Non-existent start file** — `validate_wiki_links(nonexistent.md)` → returns Err

### Manual Verification
```bash
# Create test files
mkdir -p /tmp/wiki_test
echo "[[Page1]]" > /tmp/wiki_test/index.md
echo "Hello" > /tmp/wiki_test/Page1.md

# Run via cargo test or create integration test
```

---

## Commit Strategy

### Single Commit (2 files: utils.rs + Cargo.toml)

```
feat(utils): add wiki link validator with cycle detection

- Add WikiLinkReport struct (visited, missing, cycles_prevented)
- Add validate_wiki_links() public function
- Add 4 internal helpers: extract_link_targets, parse_wiki_target,
  resolve_link, validate_recursive
- Support multi-syntax: Obsidian [[Page]] + standard [text](file.md)
- Cycle prevention via HashSet<PathBuf> with canonical paths
- Link resolution: source-relative first, then project-root-relative
- 10 unit test scenarios in #[cfg(test)] mod
- Add tempfile as dev-dependency

Files: src/utils.rs, Cargo.toml
```

**Pre-commit**: `cargo check` + `cargo test` + `cargo clippy` (all zero issues)

---

## Final Checklist

- [ ] `Cargo.toml` 添加 `tempfile` dev-dependency
- [ ] `WikiLinkReport` struct 定义完整
- [ ] `validate_wiki_links()` 公开函数签名稳定
- [ ] 4 个内部辅助函数存在
- [ ] 支持 Obsidian `[[Page]]` 语法
- [ ] 支持标准 MD `[text](file.md)` 语法
- [ ] Alias (`|`) 和 heading (`#`) 处理
- [ ] 循环检测使用 `HashSet<PathBuf>` + canonical 路径
- [ ] 链接解析：先 source-relative，再 project-root
- [ ] 6+ 单元测试场景
- [ ] `cargo check` 零错误零警告
- [ ] `cargo test` 全部通过
- [ ] `cargo clippy` 零问题
- [ ] 无 `unwrap()` / `panic!()`
- [ ] 无业务逻辑耦合
- [ ] 无新增运行时依赖（仅 dev-dep `tempfile`）

---

## Follow-ups (Out of Scope)

以下内容**不在本计划范围**，留待后续任务：

1. **深度限制**: 防止恶意/错误的超深递归（`max_depth` 参数）
2. **并行验证**: 大型 vault 场景下用 rayon 并行
3. **外部 URL 处理**: 跳过 vs 验证可访问性
4. **链接修复建议**: 检测 typo / 建议最近匹配文件
5. **`.md` 大小写不敏感**: Windows 兼容
6. **Glob 模式**: 支持 `[[category/*]]` 通配
7. **反向链接**: 当前只前向验证；反向（backlinks）查询
8. **实时模式**: 监听文件变更，增量验证
9. **配置化**: 不同项目用不同 wiki link 语法
10. **CLI 集成**: `forceloop validate-links` 子命令包装此函数
