# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

Tradeoff: These guidelines bias toward caution over speed. For trivial tasks, use judgment.

1. Think Before Coding
Don't assume. Don't hide confusion. Surface tradeoffs.

Before implementing:

State your assumptions explicitly. If uncertain, ask.
If multiple interpretations exist, present them - don't pick silently.
If a simpler approach exists, say so. Push back when warranted.
If something is unclear, stop. Name what's confusing. Ask.
2. Simplicity First
Minimum code that solves the problem. Nothing speculative.

No features beyond what was asked.
No abstractions for single-use code.
No "flexibility" or "configurability" that wasn't requested.
No error handling for impossible scenarios.
If you write 200 lines and it could be 50, rewrite it.
Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

3. Surgical Changes
Touch only what you must. Clean up only your own mess.

When editing existing code:

Don't "improve" adjacent code, comments, or formatting.
Don't refactor things that aren't broken.
Match existing style, even if you'd do it differently.
If you notice unrelated dead code, mention it - don't delete it.
When your changes create orphans:

Remove imports/variables/functions that YOUR changes made unused.
Don't remove pre-existing dead code unless asked.
The test: Every changed line should trace directly to the user's request.

4. Goal-Driven Execution
Define success criteria. Loop until verified.

Transform tasks into verifiable goals:

"Add validation" → "Write tests for invalid inputs, then make them pass"
"Fix the bug" → "Write a test that reproduces it, then make it pass"
"Refactor X" → "Ensure tests pass before and after"
For multi-step tasks, state a brief plan:

1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

ForceLoop is a Rust CLI tool for structured development workflow (per [docs/requirment.md](docs/requirment.md)). Binary name: `fl`. The codebase was built **skeleton-first**: multiple modules have graduated from `todo!()` placeholders to full implementations. Five modules (`setup`, `gate`, `state`, `compiler`, `schema`, `archive`) are fully implemented; three modules (`status`, and 4 `utils.rs` helpers) remain skeletons.

## Build & Test Commands

```bash
cargo check                    # Quick compile check (1 crate, ~0.1s)
cargo build                    # Compile binary to target/debug/fl
cargo test                     # Run all tests (unit + integration)
cargo clippy --all-targets     # Lint incl. tests
cargo run -- --help            # Show top-level CLI help
cargo run -- setup --help      # Subcommand help
```

**Integration tests** invoke the binary via `cargo run --` rather than the compiled path, so they work without a prior `cargo build`. Three integration test files:
- [tests/cli_help.rs](tests/cli_help.rs) — CLI argument parsing and help output (7 tests)
- [tests/command_compile.rs](tests/command_compile.rs) — Schema compilation for Claude, OpenCode, and OhMyPi; compile_agent tool-to-permission mapping; skill vs command schema comparison (11 tests)
- [tests/setup_tool.rs](tests/setup_tool.rs) — Setup run with various targets, file output, hook generation, migration (21 tests)

**Unit tests** for FS-touching code (see [src/utils.rs](src/utils.rs) wiki-link tests) use `tempfile::TempDir` — a `dev-dependency`. No fixtures should be added to the repo.

To run a single test:
```bash
cargo test utils::tests::test_cycle_detection
cargo test --test cli_help setup_help_works
```

## Module Layout

[src/lib.rs](src/lib.rs) declares 15 public modules, arranged in a strict dependency hierarchy:

```
main → cli → {commands/, setup, gate, status, archive}
                 ↓
   {context, errors, traits, schema, compiler, state}
                 ↓
          {constants, utils}
```

**Leaf modules** (`utils.rs`, `constants.rs`) must not import any other crate module. If a util needs project state, accept it as a function parameter.

## Architecture

### Trait-based Command Dispatch

Three traits in [src/traits.rs](src/traits.rs) form a layered hierarchy:

- **`Executable`** — all 10 Command objects implement this. Single method `execute(&self, ctx: &Context) -> Result<()>`.
- **`Subcommand: Executable`** — only the 4 top-level CLI subcommands (Setup, Gate, Status, Archive) implement this. Adds `name()` and `description()` for clap help.
- **`CommandMetadata`** — the 5 skill/command structs implement this. Adds `skill_template()`, `command_template()`, `artifacts()`, `gate()`, and `check_list()`. Declaratively describes what each command does, what files it produces, and whether the next pipeline step can proceed.

The 9 Command objects:
- **4 top-level subcommands** in `src/{setup,gate,status,archive}.rs`
- **5 skill/command structs** in `src/commands/{new_cmd,plan,audit,implement,review}.rs`

### Pipeline State Machine ([src/state.rs](src/state.rs))

Tracks a 6-phase pipeline with JSON-persisted state in `.forceloop/state.json`:

**New → Plan → Audit → Implement → Review → Done**

Key types:
- `PipelineState` — struct with 6 boolean flags (`new`, `plan`, `audit`, `implement`, `review`, `done`), all `#[serde(default)]`
- Methods: `locate_state_file()`, `locate_forceloop_dir()`, `read_or_default(path)`, `write(path)`, `next_pending() -> Option<&'static str>`

**State file protection**: On Unix, `write()` makes the file **read-only** (`chmod 444`) after every write so LLM tools cannot accidentally corrupt pipeline state. `write()` temporarily makes it writable, writes, then re-locks.

**Legacy migration**: `read_or_default()` automatically migrates from the legacy `{"current_phase":"..."}` format to the new boolean-flag format, rewriting the file on detection.

**Artifact verification**: `verify_artifact(path)` — validates artifact files exist; for `.md` files, also runs wiki-link validation.
**Checklist verification**: `verify_checklist(path)` — scans a markdown file for `- [ ]` (unchecked) items; rejects if any remain.
**Utility**: `count_wave_files(dir)`, `count_completed_items(path)`, `append_error_log(detail)`.

The `locate_state_file()` and `locate_forceloop_dir()` methods walk up from cwd looking for `.forceloop/` directory.

### Schema & Compiler ([src/schema.rs](src/schema.rs), [src/compiler.rs](src/compiler.rs))

Both built tests-first (16 tests before compiler prod code).

**`CommandSchema`** in [src/schema.rs](src/schema.rs): compile-time constant struct with all `&'static str` fields (`name`, `description`, `model`, `argument_hint`, `tools`, `agent`, `prompt`). Zero heap allocation.

**`Target`** enum in [src/compiler.rs](src/compiler.rs): `Claude`, `OpenCode`, or `OhMyPi`.

**`compile(schema, target)`**: generates platform-native YAML-frontmatter markdown command files.
- `Target::Claude` — emits `description`, `allowed-tools`, `argument-hint`, `model`
- `Target::OpenCode` | `Target::OhMyPi` — emit `description`, `agent`, `model`. Drop `allowed-tools` and `argument-hint` (not supported).

**`compile_agent(agent_name, schema)`**: generates OpenCode agent files with tool-to-permission mapping (maps tools to `read`/`edit`/`bash`/`webfetch`/`websearch`/`task` permission keys).

### CLI Layer ([src/cli.rs](src/cli.rs))

`Tool` (ValueEnum with `Claude`/`OpenCode`/`OhMyPi`) is converted to `Target` at dispatch time in `main.rs` via `From<Tool> for Target`. This boundary is explicit — internal modules (compiler, setup) never depend on clap's `Tool` type.

### Context ([src/context.rs](src/context.rs))

The `Context` struct carries cross-cutting data (currently only `targets: Vec<Target>`).
- `Context::new()` — empty targets
- `Context::with_targets(targets)` — for `setup --tool ...`

### Setup ([src/setup.rs](src/setup.rs)) — Fully Implemented

Writes compiled schema files to platform directories and configures hooks. Writes 5 command files per target (fl-new, fl-plan, fl-audit, fl-implement, fl-review). Key functions:
- `run(targets, root)` — core business logic: writes 5 command files per target, platform-specific hooks
- `effective_targets(ctx_targets)` — empty => defaults (`[Claude, OpenCode, OhMyPi]`), else pass-through
- `write_opencode_hook(root, written)` — writes `.opencode/plugins/fl.ts` (compile-time embedded, auto-loaded by OpenCode from directory). Does NOT write `opencode.json` — local plugins are directory-loaded.
- `write_omp_hook(root, written)` — writes `.omp/hooks/pre/fl-gate.ts` (oh-my-pi session_stop hook)
- `write_claude_hook(root, written)` — writes `.claude/hooks/fl-gate.sh` (shell script) + merges Stop hook entry into `.claude/settings.json`
- `merge_claude_settings(settings_path)` — merges a Stop hook entry into `.claude/settings.json` (preserves existing settings)
- `merge_opencode_plugin(json_path, fl_plugin)` — `#[allow(dead_code)]` — writing opencode.json for local plugins is incorrect; retained for reference
- `SetupReport { written: Vec<PathBuf> }` — tracks files written (used for tests and output)

**~30 unit tests**: default targets (includes OhMyPi), effective targets, command table count, plugin content contracts (OpenCode/omp/Claude all validated), merge edge cases, file count validation per target, migration from legacy paths, idempotent re-runs.

### Gate ([src/gate.rs](src/gate.rs)) — Fully Implemented

Pipeline gate control, typically invoked by hooks (Stop hook on Claude, session.idle on OpenCode, session_stop on OhMyPi):
1. Reads `.forceloop/state.json` via `PipelineState::locate_state_file()` + `read_or_default()`
2. Finds the first uncompleted gate via sequential boolean checks (`!state.new → !state.plan → !state.audit → !state.implement → !state.review`)
3. Calls `gate()` on the corresponding command (New, Plan, Audit, Implement, or Review)
4. If gate passes, sets the boolean flag to `true` and writes updated state
5. If all gates passed, sets `state.done = true`

### Platform Hooks

Three platform-specific hooks, all embedded at compile time via `include_str!`:

| Platform | Hook Script | Event | Location |
|----------|-------------|-------|----------|
| **Claude Code** | [plugin/claude-hook.sh](plugin/claude-hook.sh) | `Stop` (after each response) | `.claude/hooks/fl-gate.sh` |
| **OpenCode** | [plugin/fl.ts](plugin/fl.ts) | `session.idle` | `.opencode/plugins/fl.ts` |
| **OhMyPi** | [plugin/omp-fl-gate.ts](plugin/omp-fl-gate.ts) | `session_stop` | `.omp/hooks/pre/fl-gate.ts` |

All three run `fl gate` and inject failure output back into the AI session to trigger auto-fix loops:
- **Claude Code**: shell script exits with code 2 (blocking error, feeds stderr to AI)
- **OpenCode**: TypeScript plugin via Bun Shell with `.nothrow()`, calls `client.session.prompt({ noReply: false })`
- **OhMyPi**: TypeScript via `ExtensionAPI`, returns `{ continue: true, additionalContext }`

### Path Constants ([src/constants.rs](src/constants.rs))

All `&'static str` constants (directory/file/env var names). **Convention: use `&'static str`, not `PathBuf` constants.** Callers construct `Path::new(CONST)` at use site for cross-platform safety.

16 constants total (all `&'static str`):
- **Directories**: `FORCELOOP_DIR` (`.forceloop`), `SKILLS_DIR`, `COMMANDS_DIR`, `HOOKS_DIR`, `ARCHIVE_DIR`, `SPECS_DIR`, `PLANS_DIR`, `GIT_DIR`
- **Files**: `STATE_FILE`, `RESULT_FILE`, `PLAN_FILE`, `SPECS_INDEX` (`specs/index.md`), `PLANS_INDEX` (`plans/index.md`), `AUDIT_FILE`, `WAVE_STATE`, `REVIEW_RESULT`, `ERROR_LOG`
- **Env vars**: `ENV_PROJECT_ROOT` (`FORCELOOP_PROJECT_ROOT`), `ENV_DEBUG` (`FORCELOOP_DEBUG`)

### Utilities ([src/utils.rs](src/utils.rs))

Three categories:
- **Real stdlib wrappers** (2): `current_dir()`, `executable_path()`
- **`todo!()` skeletons** (4): `project_root()`, `state_dir()`, `state_file()`, `is_in_project()` — pending marker-strategy decision (`.git` vs `Cargo.toml` vs `.forceloop`). When implementing one, do not call other `todo!()` functions from within it.
- **Real features**: `WikiLinkReport` + `validate_wiki_links()` + 5 internal helpers + 14 unit tests (single link, broken link, cycle detection, standard MD links, alias and heading, relative resolution, project root fallback, deduplication, nonexistent start, sorted output, external URLs)

### Wiki Link Validator

`validate_wiki_links(start, project_root)` recursively validates markdown file links. Key design choices:
- **Hand-rolled parser**, no `regex` crate (deps kept minimal). Patterns: `[[Page]]` (Obsidian) and `[text](file.md)` (standard MD).
- **Auto-extension resolution**: `[[b]]` matches `b.md`. The resolver tries `target` first, then `target.md`.
- **Alias and heading stripped** before resolution: `[[Page|alias]]` → `Page`; `[[Page#h]]` → `Page`.
- **Cycle prevention**: `HashSet<PathBuf>` of `canonicalize()`d paths. Any skip (cycle or duplicate reference) increments `report.cycles_prevented`.
- **Resolution order**: source-relative → project-root-relative → record in `report.missing`.
- Returns `Result<WikiLinkReport>` where `WikiLinkReport { visited: Vec<PathBuf>, missing: Vec<(PathBuf, String)>, cycles_prevented: u32 }`. `visited` is sorted for deterministic output.

### Error Handling

[src/errors.rs](src/errors.rs) defines `ForceLoopError` with 4 variants: `Config(String)`, `Io(#[from] std::io::Error)`, `Parse(String)`, `Execution(String)`. Use `crate::errors::Result` (alias for `Result<T, ForceLoopError>`) throughout. No `unwrap()` in production code. The validator records broken links in the report rather than returning `Err`.

`anyhow::Error` is used at the application boundary (`main.rs`), while `thiserror`/`ForceLoopError` is used in library modules.

## The 5 Skill/Command Objects

All 5 follow the same pattern: `pub struct X;` with a real `execute()` body (checks pipeline prerequisites, creates directory scaffolds), full `CommandMetadata` implementation with skill/command templates, tools list, artifacts, and real `gate()` (verifies artifacts and checklists).

| Struct | File | Tools | Artifacts | Gate |
|--------|------|-------|-----------|------|
| `New` | `commands/new_cmd.rs` | Read, Write | `.forceloop/specs/index.md` | Verifies specs/index.md exists + wiki links |
| `Plan` | `commands/plan.rs` | Read, Write | `.forceloop/plans/index.md` | Verifies plans/index.md exists + wiki links; auto-generates wave_state.md |
| `Audit` | `commands/audit.rs` | Read, Grep, Glob | `.forceloop/audit.md` | Verifies audit.md exists + wiki links + all checklist items `[x]` |
| `Implement` | `commands/implement.rs` | Read, Write, Edit, Bash, Grep, Glob | `.forceloop/wave_state.md` | Verifies wave_state.md exists + all checklist items `[x]` + all waves accounted for |
| `Review` | `commands/review.rs` | Read, Grep, Bash | `.forceloop/review_result.md` | Verifies review_result.md exists + all checklist items `[x]` |

### Remaining Skeletons

These `todo!()` placeholders remain:
| Location | Function | Status |
|----------|----------|--------|
| `src/utils.rs` | `project_root()` | Pending marker-strategy decision |
| `src/utils.rs` | `state_dir()` | Pending marker-strategy decision |
| `src/utils.rs` | `state_file()` | Pending marker-strategy decision |
| `src/utils.rs` | `is_in_project()` | Pending marker-strategy decision |
| `src/status.rs` | `Status::execute()` | Not yet implemented |

## Conventions

- **Skeleton-first**: when adding a new feature, define the trait/struct/fn signature first with `todo!()` bodies, then iterate on implementation in follow-up commits.
- **Tests-first (TDD)**: new modules should define tests before production code (see `compiler.rs` and `schema.rs` for the pattern).
- **No async**: all traits are sync.
- **No new runtime deps by default**: when tempted to add `regex`/`dirs`/`serde_yaml`/etc., consider hand-rolling it first (the wiki link parser is the precedent).
- **No reverse module imports**: `utils.rs` and `constants.rs` are leaves; importing anything else from them is a red flag.
- **`impl` is a Rust keyword** — the skill file is `implement.rs` and the struct is `Implement`. `new` keyword avoided with `new_cmd.rs`/`New`.
- **Chinese-language requirements doc**: [docs/requirment.md](docs/requirment.md) is the source of truth. Skills and "custom commands" in that doc refer to the same 5 structs (not 10). Note: `try_finish` was removed; `archive` is a subcommand, not a skill.
- **Skill/Custom Command terminology**: the 5 structs in `src/commands/` serve both roles — they're invoked as "Skills" in the pipeline and as "Custom Commands" by users. The single struct set is intentional.
- **Compile-time embedding**: TypeScript plugin source in `plugin/fl.ts` is embedded via `include_str!("../plugin/fl.ts")`. The source file remains editable; rebuild picks up changes.

## Plans & History

Planning artifacts are split across two locations:
- **`.omc/plans/`** — current OMC plan workflow outputs (most recent plans live here)
- **`.sisyphus/plans/`** — original framework-scaffolding plan ([cli-framework.md](.sisyphus/plans/cli-framework.md))
- **`.history/`** — timestamped snapshots of docs/ as they evolved; do not edit

When asked to plan a feature, check `.omc/plans/` first for prior related work before writing a fresh plan.

## Things Explicitly Out of Scope (current phase)

These are intentional `todo!()` placeholders, not bugs:
- `project_root()`, `state_dir()`, `state_file()`, `is_in_project()` — waiting on marker-strategy decision
- `Status::execute()` — not yet implemented

Everything else is fully implemented: `setup`, `gate`, `archive`, all 5 command `execute()` bodies, and all 5 command `gate()` methods.