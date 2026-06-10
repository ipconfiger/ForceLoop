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

ForceLoop is a Rust CLI tool for structured development workflow (per [docs/requirment.md](docs/requirment.md)). The codebase is being built **skeleton-first**: traits, structs, and module layout are in place; business logic is deliberately `todo!()` placeholders to be filled in later phases.

## Build & Test Commands

```bash
cargo check                    # Quick compile check (1 crate, ~0.1s)
cargo build                    # Compile binary to target/debug/forceloop
cargo test                     # Run all tests (unit + integration)
cargo clippy --all-targets     # Lint incl. tests
cargo run -- --help            # Show top-level CLI help
cargo run -- setup --help      # Subcommand help
```

**Integration tests** ([tests/cli_help.rs](tests/cli_help.rs)) invoke the binary via `cargo run --` rather than the compiled path, so they work without a prior `cargo build`.

**Unit tests** for FS-touching code (see [src/utils.rs](src/utils.rs) wiki-link tests) use `tempfile::TempDir` — a `dev-dependency`. No fixtures should be added to the repo.

To run a single test:
```bash
cargo test utils::tests::test_cycle_detection
cargo test --test cli_help setup_help_works
```

## Architecture

### Trait-based Command Dispatch

Three traits in [src/traits.rs](src/traits.rs) form a layered hierarchy:

- **`Executable`** — all 10 Command objects implement this. Single method `execute(&self, ctx: &Context) -> Result<()>`.
- **`Subcommand: Executable`** — only the 4 top-level CLI subcommands (Setup, Gate, Status, Archive) implement this. Adds `name()` and `description()` for clap help.
- **`CommandMetadata`** — all 10 Commands implement this. Adds `skill_template()`, `command_template()`, `artifacts()`, and `gate()`. Used to declaratively describe what each command does, what files it produces, and whether the next pipeline step can proceed.

The 10 Command objects:
- **4 top-level subcommands** in `src/{setup,gate,status,archive}.rs` (each: `pub struct X;` with empty body)
- **6 skill/command structs** in `src/commands/{new_cmd,plan,audit,implement,review,try_finish}.rs` (same pattern)

### Module Dependency Direction (strict, no reverse imports)

```
main → cli → {commands/, setup, gate, status, archive}
                 ↓
            {context, errors, traits}
                 ↓
            {constants, utils}
```

`utils.rs` is the lowest layer — it must not depend on `Context` or any Command type. If a util needs project state, accept it as a function parameter (see `validate_wiki_links(start, project_root: Option<&Path>)` for the pattern).

### Path Constants

[src/constants.rs](src/constants.rs) holds 11 `&'static str` constants (directory names, file names, env var names). **Convention: use `&'static str`, not `PathBuf` constants.** Callers construct `Path::new(STATE_FILE)` at the use site for cross-platform safety. `pub fn` factory functions (e.g., `state_path() -> PathBuf`) are preferred when a full path is needed.

### Utilities ([src/utils.rs](src/utils.rs))

Two categories coexist:
- **Real stdlib wrappers** (2): `current_dir()`, `executable_path()` — one-liners, no design choice
- **`todo!()` skeletons** (4): `project_root()`, `state_dir()`, `state_file()`, `is_in_project()` — pending marker-strategy decision (`.git` vs `Cargo.toml` vs `.forceloop`)
- **Real features**: `WikiLinkReport` + `validate_wiki_links()` + 4 internal helpers + 12 unit tests

When implementing one of the `todo!()` skeletons, do not call other `todo!()` functions from within it (avoids cascade). The body is filled in based on a marker-strategy decision that's still pending.

### Wiki Link Validator

`validate_wiki_links(start, project_root)` recursively validates markdown file links. Key design choices:
- **Hand-rolled parser**, no `regex` crate (deps kept minimal). Patterns: `[[Page]]` (Obsidian) and `[text](file.md)` (standard MD).
- **Auto-extension resolution**: `[[b]]` matches `b.md`. The resolver tries `target` first, then `target.md`.
- **Alias and heading stripped** before resolution: `[[Page|alias]]` → `Page`; `[[Page#h]]` → `Page`.
- **Cycle prevention**: `HashSet<PathBuf>` of `canonicalize()`d paths. Any skip (cycle or duplicate reference) increments `report.cycles_prevented`.
- **Resolution order**: source-relative → project-root-relative → record in `report.missing`.
- Returns `Result<WikiLinkReport>` where `WikiLinkReport { visited: Vec<PathBuf>, missing: Vec<(PathBuf, String)>, cycles_prevented: u32 }`. `visited` is sorted for deterministic output.

### Error Handling

[src/errors.rs](src/errors.rs) defines `ForceLoopError` with 4 variants: `Config(String)`, `Io(#[from] std::io::Error)`, `Parse(String)`, `Execution(String)`. Use `crate::errors::Result` (alias for `Result<T, ForceLoopError>`) throughout. No `unwrap()` in production code. The validator records broken links in the report rather than returning `Err` — only file-level errors (start file missing, canonicalize failure) propagate.

## Conventions

- **Skeleton-first**: when adding a new feature, define the trait/struct/fn signature first with `todo!()` bodies, then iterate on implementation in follow-up commits.
- **No async**: all traits are sync.
- **No new runtime deps by default**: when tempted to add `regex`/`dirs`/`serde_yaml`/etc., consider hand-rolling it first (the wiki link parser is the precedent).
- **No reverse module imports**: `utils.rs` and `constants.rs` are leaves; importing anything else from them is a red flag.
- **`impl` is a Rust keyword** — the skill file is `implement.rs` and the struct is `Implement`. `new` keyword avoided with `new_cmd.rs`/`New`. `try_finish` stays as-is.
- **Chinese-language requirements doc**: [docs/requirment.md](docs/requirment.md) is the source of truth. Skills and "custom commands" in that doc refer to the same 6 structs (not 12).
- **Skill/Custom Command terminology**: the 6 structs in `src/commands/` serve both roles — they're invoked as "Skills" in the pipeline and as "Custom Commands" by users. The single struct set is intentional (per the Metis review in [.sisyphus/plans/cli-framework.md](.sisyphus/plans/cli-framework.md)).

## Plans & History

Planning artifacts are split across two locations:
- **`.omc/plans/`** — current OMC plan workflow outputs (most recent plans live here)
- **`.sisyphus/plans/`** — original framework-scaffolding plan ([cli-framework.md](.sisyphus/plans/cli-framework.md))
- **`.history/`** — timestamped snapshots of docs/ as they evolved; do not edit

When asked to plan a feature, check `.omc/plans/` first for prior related work before writing a fresh plan.

## Things Explicitly Out of Scope (current phase)

These are intentional `todo!()` placeholders, not bugs:
- `project_root()`, `state_dir()`, `state_file()`, `is_in_project()` — waiting on marker-strategy decision
- All `execute()` method bodies — command logic is phase-2 work
- `gate()` methods — currently all return `Ok(())`
- `setup`, `status`, `archive` business logic — see `todo!()` in each
