use std::fs;
use std::path::{Path, PathBuf};

use crate::archive::Archive;
use crate::commands::{Audit, Implement, New, Plan, Review, TryFinish};
use crate::compiler::{compile, Target};
use crate::context::Context;
use crate::errors::Result;
use crate::gate::Gate;
use crate::schema::CommandSchema;
use crate::status::Status;
use crate::traits::{CommandMetadata, Executable, Subcommand};

const SKILL_PROMPT: &str = "\
# Project Setup Skill

Initialize the current directory as a ForceLoop project.

## Steps
1. Create `.forceloop/{skills,commands,hooks,archive}/` directory tree.
2. Write initial `.forceloop/state.json` (phase 0, no active command).
3. Generate platform-native command files from `CommandMetadata` for
   each target specified by `--tool` (omit `--tool` to install to
   both Claude Code and OpenCode):
   - `--tool claude`   → `.claude/commands/<name>.md`
   - `--tool opencode` → `.opencode/command/<name>.md`
4. Install git hooks for `gate` control.
5. Print summary of installed components.

## Verification
- `.forceloop/state.json` exists
- 10 command files generated
- Hooks executable
";

const COMMAND_PROMPT: &str = "\
Initialize the current directory as a ForceLoop project.

Creates `.forceloop/` tree, generates platform-native command files for
each target specified by `--tool` (Claude Code and/or OpenCode),
installs hooks, writes initial state.

Use once per project, or to repair corrupted state.
";

fn setup_skill() -> CommandSchema {
    CommandSchema {
        name: "setup",
        description: "Initialize project directory structure, state, subcommands, skills, and hooks",
        model: None,
        argument_hint: None,
        tools: &["Bash", "Read", "Write", "Glob"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn setup_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..setup_skill()
    }
}

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

/// Static table of all 10 Command objects.
///
/// This table intentionally enumerates every Command — adding a new
/// Command without adding an entry here is a build-time oversight that
/// the `all_10_commands_have_populated_schemas` test in
/// `tests/command_compile.rs` will not catch, but the `run()` invariant
/// (10 files per target) will.
const COMMANDS: &[CommandEntry] = &[
    ("setup", || Setup.command_template()),
    ("gate", || Gate.command_template()),
    ("status", || Status.command_template()),
    ("archive", || Archive.command_template()),
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
    }
    Ok(SetupReport { written })
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

impl CommandMetadata for Setup {
    fn skill_template(&self) -> CommandSchema {
        setup_skill()
    }
    fn command_template(&self) -> CommandSchema {
        setup_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[".forceloop/state.json"]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
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
    fn skill_prompt_describes_default_both_targets() {
        assert!(SKILL_PROMPT.contains("Claude Code") || SKILL_PROMPT.contains("claude"));
        assert!(SKILL_PROMPT.contains("OpenCode") || SKILL_PROMPT.contains("opencode"));
        assert!(
            SKILL_PROMPT.contains("--tool"),
            "SKILL_PROMPT should reference --tool flag explicitly"
        );
    }

    #[test]
    fn command_prompt_describes_default_both_targets() {
        assert!(COMMAND_PROMPT.contains("Claude") || COMMAND_PROMPT.contains("claude"));
        assert!(COMMAND_PROMPT.contains("OpenCode") || COMMAND_PROMPT.contains("opencode"));
    }

    #[test]
    fn commands_table_has_ten_entries() {
        // If you add a new Command, add it here too. The 10-file
        // invariant in `run()` tests depends on this.
        assert_eq!(COMMANDS.len(), 10);
    }
}
