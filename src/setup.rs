use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::commands::{Audit, Implement, New, Plan, Review};
use crate::compiler::{compile, Target};
use crate::constants::{FORCELOOP_DIR, STATE_FILE};
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable, Subcommand};

/// **Source of truth** for the default `setup` behavior when `--tool`
/// is not specified: install to Claude Code, OpenCode, AND oh-my-pi.
///
/// This constant is the single point of change if the default ever
/// needs to expand (e.g., add more targets) or contract (e.g., drop
/// OpenCode support). The `default_targets` test in `tests/setup_tool.rs`
/// pins this — any change requires updating both the test assertion AND
/// the `SKILL_PROMPT` text (which says \"install to both Claude Code
/// and OpenCode\").
pub const DEFAULT_TARGETS: &[Target] = &[Target::Claude, Target::OpenCode, Target::OhMyPi];

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

/// Static table of the 5 Skill / Custom Command objects that get
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
/// If you add a new row, `run_writes_all_five_commands_per_target` in
/// `tests/setup_tool.rs` will fail until you update its expected set —
/// this is intentional, the test pins the contract.
const COMMANDS: &[CommandEntry] = &[
    ("fl-new", || New.command_template()),
    ("fl-plan", || Plan.command_template()),
    ("fl-audit", || Audit.command_template()),
    ("fl-implement", || Implement.command_template()),
    ("fl-review", || Review.command_template()),
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
        // oh-my-pi additionally gets a session_stop hook.
        if target == Target::OhMyPi {
            write_omp_hook(root, &mut written)?;
        }
    }
    Ok(SetupReport { written })
}

/// OpenCode TypeScript plugin that calls `fl gate` on `session.idle`.
///
/// Output: `<root>/.opencode/plugins/fl.ts`
///
/// Behavior (per reference doc):
/// - On `session.idle`, run `fl gate`
/// - Exit 0: silent pass
/// - Exit != 0: inject stdout+stderr into the session as a prompt
///   with `noReply: false` to trigger the AI's auto-reply / fix loop
///
/// The actual TypeScript source lives at `plugin/fl.ts` in the
/// project root, embedded at compile time via `include_str!` so the
/// file is editable in-place and the binary has zero runtime cost.
fn plugin_hook_ts_content() -> &'static str {
    include_str!("../plugin/fl.ts")
}

/// Write the 2 OpenCode project-level hook files to `root`:
///   - `<root>/.opencode/opencode.json` (merged with existing content)
///   - `<root>/.opencode/plugins/fl.ts`
///
/// Both paths are pushed into `written` for the `SetupReport`.
///
/// Migration: legacy files written by older `fl setup` versions live
/// at `<root>/opencode.json` and `<root>/plugin/`. They are deleted
/// before the new files are written, so the project root is cleaned up
/// during the first run on an already-set-up project.
///
/// Idempotent: re-running merges (deduped) into `opencode.json` and
/// overwrites `fl.ts`. Matches the contract described in
/// `docs/opencode-hook-spec-correction.md`.
fn write_opencode_hook(root: &Path, written: &mut Vec<PathBuf>) -> Result<()> {
    // Migration: remove legacy files at the old (incorrect) paths.
    let old_json = root.join("opencode.json");
    if old_json.exists() {
        fs::remove_file(&old_json)?;
    }
    let old_plugin = root.join("plugin");
    if old_plugin.exists() {
        fs::remove_dir_all(&old_plugin)?;
    }
    // v2 error: we used to write a plugin entry into opencode.json,
    // but local plugins are auto-loaded from .opencode/plugins/ —
    // they must NOT be listed in the `plugin` array (npm packages only).
    let old_opencode_json = root.join(".opencode/opencode.json");
    if old_opencode_json.exists() {
        fs::remove_file(&old_opencode_json)?;
    }

    // Write plugin file to .opencode/plugins/ — auto-loaded by OpenCode.
    let plugins_dir = root.join(".opencode/plugins");
    fs::create_dir_all(&plugins_dir)?;
    let ts_path = plugins_dir.join("fl.ts");
    fs::write(&ts_path, plugin_hook_ts_content())?;
    written.push(ts_path);

    Ok(())
}

/// Merge the `fl` plugin entry into `<root>/.opencode/opencode.json`.
///
/// Contract (per OpenCode docs: "配置文件是合并在一起的，而不是替换"):
/// - File absent → write `{"plugin":[<fl_plugin>]}` (pretty-printed).
/// - File present, valid JSON object, `plugin` is array, `fl_plugin`
///   absent → append `fl_plugin`, preserve all other keys.
/// - File present, valid JSON object, `plugin` is array, `fl_plugin`
///   already present → write back unchanged (idempotent re-run).
/// - File present, valid JSON object, but `plugin` is not an array
///   → `Config` error (do not silently overwrite user content).
/// - File present, valid JSON, but root is not an object
///   → `Config` error.
/// - File present, malformed JSON → `Parse` error.
///
/// Pretty-printed on write (trailing newline) to match OpenCode's
/// standard formatting. The relative path `fl_plugin` is interpreted
/// relative to the directory containing `opencode.json` (i.e.
/// `<root>/.opencode/`), so callers should pass `./plugins/fl.ts`
/// and not an absolute or project-root-relative path.
#[allow(dead_code)]
fn merge_opencode_plugin(json_path: &Path, fl_plugin: &str) -> Result<()> {
    if let Some(parent) = json_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let mut root: Value = if json_path.exists() {
        let s = fs::read_to_string(json_path)?;
        serde_json::from_str(&s)
            .map_err(|e| ForceLoopError::Parse(format!("opencode.json: {e}")))?
    } else {
        json!({})
    };

    let obj = root.as_object_mut().ok_or_else(|| {
        ForceLoopError::Config("opencode.json root is not an object".into())
    })?;

    let entry = obj
        .entry("plugin".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let arr = entry.as_array_mut().ok_or_else(|| {
        ForceLoopError::Config("opencode.json 'plugin' is not an array".into())
    })?;

    let fl_value = Value::String(fl_plugin.to_string());
    if !arr.contains(&fl_value) {
        arr.push(fl_value);
    }

    let pretty = serde_json::to_string_pretty(&root)
        .map_err(|e| ForceLoopError::Parse(format!("opencode.json: {e}")))?;
    fs::write(json_path, format!("{pretty}\n"))?;
    Ok(())
}

/// oh-my-pi TypeScript hook that calls `fl gate` on `session_stop`.
///
/// Output: `<root>/.omp/hooks/pre/fl-gate.ts`
///
/// Behavior (per `docs/omp-hook-porting.html`):
/// - On `session_stop`, run `fl gate` via Bun Shell
/// - Exit 0: silent pass (return undefined)
/// - Exit != 0: return `{ continue: true, additionalContext }` to inject
///   gate output as a new user message, driving the agent's auto-fix loop
///
/// The actual TypeScript source lives at `plugin/omp-fl-gate.ts` in the
/// project root, embedded at compile time via `include_str!`.
fn omp_hook_ts_content() -> &'static str {
    include_str!("../plugin/omp-fl-gate.ts")
}

/// Write the omp project-level hook file to `root`:
///   `<root>/.omp/hooks/pre/fl-gate.ts`
///
/// Pushed into `written` for the `SetupReport`.
fn write_omp_hook(root: &Path, written: &mut Vec<PathBuf>) -> Result<()> {
    let hooks_dir = root.join(".omp/hooks/pre");
    fs::create_dir_all(&hooks_dir)?;
    let ts_path = hooks_dir.join("fl-gate.ts");
    fs::write(&ts_path, omp_hook_ts_content())?;
    written.push(ts_path);
    Ok(())
}

fn target_subdir(root: &Path, target: Target) -> PathBuf {
    let sub = match target {
        Target::Claude => ".claude/commands",
        Target::OpenCode => ".opencode/command",
        Target::OhMyPi => ".omp/commands",
    };
    root.join(sub)
}

pub struct Setup;

impl Executable for Setup {
    fn execute(&self, ctx: &Context) -> Result<()> {
        let targets = effective_targets(&ctx.targets);
        let root = crate::utils::current_dir()?;

        // 1. Initialize .forceloop/ directory and state.json.
        let forceloop_dir = root.join(FORCELOOP_DIR);
        fs::create_dir_all(&forceloop_dir)?;
        let state_path = forceloop_dir.join(STATE_FILE);
        if !state_path.exists() {
            let state = crate::state::PipelineState::default();
            state.write(&state_path)?;
        }

        // 2. Write platform command files.
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn default_targets_is_all_platforms() {
        // Pins DEFAULT_TARGETS. If the default ever changes, this
        // test forces a conscious update alongside SKILL_PROMPT text.
        assert_eq!(
            default_targets(),
            vec![Target::Claude, Target::OpenCode, Target::OhMyPi]
        );
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
    fn commands_table_has_five_entries() {
        // Only the 5 Skill / Custom Command objects (in `src/commands/`)
        // are registered. The 4 top-level subcommands (Setup, Gate,
        // Status, Archive) are terminal CLI subcommands and intentionally
        // excluded from this table — they should never appear in the
        // IDE's command palette. See
        // `.omc/plans/command-metadata-narrow-to-commands.md` for rationale.
        // The 5-file invariant in `run()` tests (see `tests/setup_tool.rs`)
        // depends on this count.
        assert_eq!(COMMANDS.len(), 5);
    }

    // ----------------------------------------------------------------
    // OpenCode hook content contracts — see
    // `.omc/plans/setup-opencode-hook-correction.md`.
    // ----------------------------------------------------------------

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
        // Per `@opencode-ai/plugin` type definitions (v1.17.8):
        // The `Hooks` interface has a generic `event` handler that
        // receives all events. There is no dedicated `"session.idle"`
        // hook key. The handler must filter by `event.type`.
        let s = plugin_hook_ts_content();
        assert!(
            s.contains("event:"),
            "must use the generic `event:` handler"
        );
        assert!(
            s.contains("session.idle"),
            "handler must filter on `session.idle`"
        );
        assert!(
            s.contains("event.properties") && s.contains("sessionID"),
            "handler must read sessionID from event.properties"
        );
    }

    #[test]
    fn plugin_hook_ts_uses_named_export_with_plugin_type() {
        // Regression guard: the old (broken) plugin used
        // `export default (async (ctx) => {...}) satisfies Plugin`.
        // The correct shape (per docs) is
        // `export const FlGateHook: Plugin = async ({...}) => {...}`.
        let s = plugin_hook_ts_content();
        assert!(
            s.contains("export const"),
            "must use named `export const` (not `export default`)"
        );
        assert!(
            s.contains(": Plugin"),
            "must annotate with explicit `Plugin` type"
        );
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
    fn plugin_hook_ts_no_timeout() {
        // OpenCode v1.17.8's BunShellPromise does not support .timeout().
        // The plugin must call .nothrow() without a timeout modifier.
        let s = plugin_hook_ts_content();
        assert!(
            !s.contains(".timeout("),
            "BunShellPromise has no .timeout() in v1.17.8"
        );
        assert!(
            s.contains(".nothrow()"),
            "must use .nothrow() to catch non-zero exit"
        );
    }

    // ----------------------------------------------------------------
    // oh-my-pi hook content contracts — see
    // `docs/omp-hook-porting.html`.
    // ----------------------------------------------------------------

    #[test]
    fn omp_hook_ts_uses_fl_gate() {
        let s = omp_hook_ts_content();
        assert!(s.contains("fl gate"), "must call `fl gate`");
    }

    #[test]
    fn omp_hook_ts_uses_session_stop() {
        let s = omp_hook_ts_content();
        assert!(
            s.contains("session_stop"),
            "hook must listen to session_stop event"
        );
    }

    #[test]
    fn omp_hook_ts_returns_continue_on_failure() {
        let s = omp_hook_ts_content();
        assert!(s.contains("continue: true"), "must set continue: true");
        assert!(
            s.contains("additionalContext"),
            "must provide additionalContext"
        );
    }

    #[test]
    fn omp_hook_ts_uses_nothrow() {
        let s = omp_hook_ts_content();
        assert!(s.contains(".nothrow()"), "must use .nothrow()");
        assert!(
            !s.contains(".quiet()"),
            "must NOT use .quiet() — it suppresses stderr capture"
        );
    }

    #[test]
    fn omp_hook_ts_uses_pi_coding_agent() {
        let s = omp_hook_ts_content();
        assert!(
            s.contains("@oh-my-pi/pi-coding-agent"),
            "must import from @oh-my-pi/pi-coding-agent"
        );
    }

    // ----------------------------------------------------------------
    // merge_opencode_plugin() contracts — see
    // `docs/opencode-hook-spec-correction.md`.
    // ----------------------------------------------------------------

    fn read_json(p: &Path) -> Value {
        let s = fs::read_to_string(p).unwrap();
        serde_json::from_str(&s).unwrap()
    }

    #[test]
    fn merge_writes_initial_config_when_file_absent() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join(".opencode/opencode.json");
        merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap();
        let v = read_json(&json_path);
        let plugins = v
            .get("plugin")
            .and_then(|p| p.as_array())
            .expect("plugin must be array");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0], "./plugins/fl.ts");
    }

    #[test]
    fn merge_appends_when_key_absent() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("opencode.json");
        fs::write(&json_path, "{}\n").unwrap();
        merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap();
        let v = read_json(&json_path);
        assert_eq!(v["plugin"][0], "./plugins/fl.ts");
        assert_eq!(v["plugin"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn merge_dedupes_when_key_present() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("opencode.json");
        fs::write(
            &json_path,
            "{\n  \"plugin\": [\"./plugins/fl.ts\"]\n}\n",
        )
        .unwrap();
        merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap();
        let v = read_json(&json_path);
        let arr = v["plugin"].as_array().unwrap();
        assert_eq!(arr.len(), 1, "re-merge must not duplicate the entry");
        assert_eq!(arr[0], "./plugins/fl.ts");
    }

    #[test]
    fn merge_preserves_other_keys() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("opencode.json");
        fs::write(
            &json_path,
            "{\n  \"plugin\": [\"./other.ts\"],\n  \"theme\": \"dark\"\n}\n",
        )
        .unwrap();
        merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap();
        let v = read_json(&json_path);
        // Other key preserved.
        assert_eq!(v["theme"], "dark");
        // Both plugin entries present, in original + appended order.
        let arr = v["plugin"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], "./other.ts");
        assert_eq!(arr[1], "./plugins/fl.ts");
    }

    #[test]
    fn merge_errors_when_plugin_is_not_array() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("opencode.json");
        fs::write(&json_path, "{ \"plugin\": \"not-an-array\" }\n").unwrap();
        let err = merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap_err();
        assert!(
            matches!(err, ForceLoopError::Config(_)),
            "expected Config error, got: {err:?}"
        );
        // File must NOT be overwritten on error.
        let raw = fs::read_to_string(&json_path).unwrap();
        assert!(
            raw.contains("not-an-array"),
            "user content must be preserved on error; got: {raw}"
        );
    }

    #[test]
    fn merge_errors_when_root_is_not_object() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("opencode.json");
        fs::write(&json_path, "[1, 2, 3]\n").unwrap();
        let err = merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap_err();
        assert!(
            matches!(err, ForceLoopError::Config(_)),
            "expected Config error, got: {err:?}"
        );
    }

    #[test]
    fn merge_errors_on_malformed_json() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("opencode.json");
        fs::write(&json_path, "{ this is not json").unwrap();
        let err = merge_opencode_plugin(&json_path, "./plugins/fl.ts").unwrap_err();
        assert!(
            matches!(err, ForceLoopError::Parse(_)),
            "expected Parse error, got: {err:?}"
        );
    }
}
