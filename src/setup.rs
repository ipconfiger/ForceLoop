use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::{Audit, Implement, New, Plan, Review, TryFinish};
use crate::compiler::{compile, Target};
use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

/// **Source of truth** for the default `setup` behavior when `--tool`
/// is not specified: install to BOTH Claude Code and OpenCode.
///
/// This constant is the single point of change if the default ever
/// needs to expand (e.g., add `Target::Cursor`) or contract (e.g., drop
/// OpenCode support). The `default_targets_constant_is_both_platforms`
/// test in `tests/setup_tool.rs` pins this — any change requires
/// updating both the test assertion AND the `SKILL_PROMPT` text (which
/// says \"install to both Claude Code and OpenCode\").
pub const DEFAULT_TARGETS: &[Target] = &[Target::Claude, Target::OpenCode];

/// Returns a `Vec` copy of [`DEFAULT_TARGETS`].
///
/// Use this at the boundary between `Context.targets` and `run()` to
/// expand the \"user didn't specify\" case into an explicit target list.
pub fn default_targets() -> Vec<Target> {
    DEFAULT_TARGETS.to_vec()
}

/// Expand `ctx.targets` into the effective target list for execution.
///
/// If the user passed no `--tool` flag (empty Vec), expand to
/// [`DEFAULT_TARGETS`]. Otherwise pass through unchanged.
///
/// Pure function — extracted from `execute()` so it can be tested
/// without invoking `current_dir()`.
pub fn effective_targets(ctx_targets: &[Target]) -> Vec<Target> {
    if ctx_targets.is_empty() {
        default_targets()
    } else {
        ctx_targets.to_vec()
    }
}

pub struct SetupReport {
    pub written: Vec<PathBuf>,
}

/// Static table type: (command_name, command_template factory).
///
/// `CommandSchema` is `Copy`, so the factory is zero-cost. Factored
/// into a type alias to keep the `COMMANDS` literal readable.
type CommandEntry = (&'static str, fn() -> CommandSchema);

/// Static table of the 6 Skill / Custom Command objects that get
/// registered as platform-native slash command / Skill files.
///
/// The 4 top-level subcommands (Setup, Gate, Status, Archive) are
/// intentionally excluded: they are terminal-only CLI subcommands,
/// not runtime-invokable skills. Including any of them would write
/// `<name>.md` to `.claude/commands/` and `.opencode/command/`,
/// surfacing entries in the IDE's command palette that should never
/// be clicked (project init and pipeline orchestration are terminal
/// actions, not skills).
///
/// This table is the single source of truth for which Commands get
/// registered. Adding a new Skill / Custom Command requires:
///   1. Add the struct in `src/commands/`
///   2. Implement `CommandMetadata` for it
///   3. Add a row here
///
/// If you add a new row, `run_writes_all_six_commands_per_target` in
/// `tests/setup_tool.rs` will fail until you update its expected set —
/// this is intentional, the test pins the contract.
const COMMANDS: &[CommandEntry] = &[
    ("new", || New.command_template()),
    ("plan", || Plan.command_template()),
    ("audit", || Audit.command_template()),
    ("implement", || Implement.command_template()),
    ("review", || Review.command_template()),
    ("try_finish", || TryFinish.command_template()),
];

/// Pure business logic for `setup`. Writes `compile(s, target)` to the
/// platform-specific subdirectory of `root` for each (target, command)
/// pair.
///
/// Does NOT auto-default `targets` — callers must pass a fully-resolved
/// list (use [`effective_targets`] before calling). This keeps `run()`
/// honest: it does exactly what its arguments say, no surprises.
pub fn run(targets: &[Target], root: &Path) -> Result<SetupReport> {
    let mut written = Vec::new();
    for &target in targets {
        let dir = target_subdir(root, target);
        fs::create_dir_all(&dir)?;
        for (name, t_fn) in COMMANDS {
            let body = compile(&t_fn(), target)?;
            let path = dir.join(format!("{}.md", name));
            fs::write(&path, body)?;
            written.push(path);
        }
        // OpenCode additionally gets a project-level hook so that
        // `session.idle` automatically invokes `fl gate`. See
        // `.omc/plans/opencode-session-idle-gate-hook.md`.
        if target == Target::OpenCode {
            write_opencode_hook(root, &mut written)?;
        }
    }
    Ok(SetupReport { written })
}

/// Project-level OpenCode config that registers our `plugin/hook.ts`.
///
/// Output: `<root>/opencode.json`
///
/// The file tells OpenCode to load the TypeScript plugin at
/// `./plugin/hook.ts` (relative to the directory containing
/// `opencode.json`, i.e. the project root). Per reference doc:
/// `opencode-auto-state-driver.md`.
fn opencode_json_content() -> &'static str {
    "{\n  \"plugin\": [\"./plugin/hook.ts\"]\n}\n"
}

/// OpenCode TypeScript plugin that calls `fl gate` on `session.idle`.
///
/// Output: `<root>/plugin/hook.ts`
///
/// Behavior (per reference doc):
/// - On `session.idle`, run `fl gate` with a 60-second timeout
/// - Exit 0: silent pass
/// - Exit != 0: inject stdout+stderr into the session as a prompt
///   with `noReply: false` to trigger the AI's auto-reply / fix loop
///
/// The actual TypeScript source lives at `plugin/hook.ts` in the
/// project root, embedded at compile time via `include_str!` so the
/// file is editable in-place and the binary has zero runtime cost.
fn plugin_hook_ts_content() -> &'static str {
    include_str!("../plugin/hook.ts")
}

/// Write the 2 OpenCode project-level hook files to `root`:
///   - `<root>/opencode.json`
///   - `<root>/plugin/hook.ts`
///
/// Both paths are pushed into `written` for the `SetupReport`.
///
/// Idempotent: re-running overwrites both files (matches the
/// `fs::write` semantics used for command files).
fn write_opencode_hook(root: &Path, written: &mut Vec<PathBuf>) -> Result<()> {
    let json_path = root.join("opencode.json");
    fs::write(&json_path, opencode_json_content())?;
    written.push(json_path);

    let plugin_dir = root.join("plugin");
    fs::create_dir_all(&plugin_dir)?;
    let ts_path = plugin_dir.join("hook.ts");
    fs::write(&ts_path, plugin_hook_ts_content())?;
    written.push(ts_path);

    Ok(())
}

fn target_subdir(root: &Path, target: Target) -> PathBuf {
    let sub = match target {
        Target::Claude => ".claude/commands",
        Target::OpenCode => ".opencode/command",
    };
    root.join(sub)
}

pub struct Setup;

impl Executable for Setup {
    fn execute(&self, ctx: &Context) -> Result<()> {
        let targets = effective_targets(&ctx.targets);
        let root = crate::utils::current_dir()?;
        let report = run(&targets, &root)?;
        // Future: print summary to stdout (matches SKILL_PROMPT step 5).
        // Currently silent — the file system is the observable side
        // effect and tests assert on it directly.
        let _ = report;
        Ok(())
    }
}

impl Subcommand for Setup {
    fn name(&self) -> &'static str {
        "setup"
    }
    fn description(&self) -> &'static str {
        "Initialize project directory structure, state, subcommands, skills, and hooks"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_targets_is_both_platforms() {
        // Pins DEFAULT_TARGETS. If the default ever changes, this
        // test forces a conscious update alongside SKILL_PROMPT text.
        assert_eq!(default_targets(), vec![Target::Claude, Target::OpenCode]);
    }

    #[test]
    fn effective_targets_expands_empty_to_default() {
        let ctx = Context::new();
        assert_eq!(effective_targets(&ctx.targets), default_targets());
    }

    #[test]
    fn effective_targets_preserves_non_empty() {
        let ctx = Context::with_targets(vec![Target::Claude]);
        assert_eq!(effective_targets(&ctx.targets), vec![Target::Claude]);
    }

    #[test]
    fn commands_table_has_six_entries() {
        // Only the 6 Skill / Custom Command objects (in `src/commands/`)
        // are registered. The 4 top-level subcommands (Setup, Gate,
        // Status, Archive) are terminal CLI subcommands and intentionally
        // excluded from this table — they should never appear in the
        // IDE's command palette. See
        // `.omc/plans/command-metadata-narrow-to-commands.md` for rationale.
        // The 6-file invariant in `run()` tests (see `tests/setup_tool.rs`)
        // depends on this count.
        assert_eq!(COMMANDS.len(), 6);
    }

    // ----------------------------------------------------------------
    // OpenCode hook content contracts — see
    // `.omc/plans/opencode-session-idle-gate-hook.md`.
    // ----------------------------------------------------------------

    #[test]
    fn opencode_json_has_plugin_entry() {
        let s = opencode_json_content();
        // Must be valid JSON.
        let v: serde_json::Value =
            serde_json::from_str(s).expect("opencode.json content must be valid JSON");
        let plugins = v
            .get("plugin")
            .and_then(|p| p.as_array())
            .expect("plugin must be a JSON array");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0], "./plugin/hook.ts");
    }

    #[test]
    fn plugin_hook_ts_uses_fl_gate() {
        // Replaces the doc's `./check.sh` placeholder with the actual
        // `fl gate` invocation. This is the core of the integration.
        let s = plugin_hook_ts_content();
        assert!(s.contains("fl gate"), "must call `fl gate`");
        assert!(
            !s.contains("./check.sh"),
            "doc placeholder must be replaced; found `./check.sh`"
        );
    }

    #[test]
    fn plugin_hook_ts_filters_session_idle() {
        let s = plugin_hook_ts_content();
        assert!(
            s.contains("session.idle"),
            "must filter on session.idle event"
        );
        assert!(s.contains("event.type"), "must inspect event.type");
    }

    #[test]
    fn plugin_hook_ts_prompts_on_nonzero_exit() {
        let s = plugin_hook_ts_content();
        assert!(s.contains("exitCode"), "must inspect result.exitCode");
        assert!(
            s.contains("client.session.prompt"),
            "must call client.session.prompt on non-zero exit"
        );
        assert!(
            s.contains("noReply: false"),
            "noReply: false triggers AI auto-reply / fix loop"
        );
    }

    #[test]
    fn plugin_hook_ts_has_timeout() {
        let s = plugin_hook_ts_content();
        assert!(
            s.contains(".timeout("),
            "must use BunShell $.timeout() to bound gate execution"
        );
        assert!(s.contains("60_000"), "default 60s timeout per reference doc");
    }
}
