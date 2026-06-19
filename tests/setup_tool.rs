use std::collections::BTreeSet;
use std::fs;

use forceloop::compiler::Target;
use forceloop::context::Context;
use forceloop::setup::{default_targets, effective_targets, run, DEFAULT_TARGETS};
use tempfile::TempDir;

#[test]
fn default_targets_constant_is_all_platforms() {
    // Pins DEFAULT_TARGETS — if anyone changes the default, this fails.
    assert_eq!(
        default_targets(),
        vec![Target::Claude, Target::OpenCode, Target::OhMyPi]
    );
    assert_eq!(DEFAULT_TARGETS.len(), 3);
}

#[test]
fn execute_expands_empty_context_targets_to_default() {
    // `forceloop setup` (no --tool) — empty Vec must expand to BOTH.
    let ctx = Context::new();
    assert_eq!(effective_targets(&ctx.targets), default_targets());
}

#[test]
fn execute_preserves_explicit_targets_when_non_empty() {
    let ctx = Context::with_targets(vec![Target::Claude]);
    assert_eq!(effective_targets(&ctx.targets), vec![Target::Claude]);
}

#[test]
fn execute_with_two_targets_preserves_order() {
    let ctx = Context::with_targets(vec![Target::OpenCode, Target::Claude]);
    // Order is preserved — matters for file iteration order in run().
    assert_eq!(
        effective_targets(&ctx.targets),
        vec![Target::OpenCode, Target::Claude]
    );
}

#[test]
fn run_default_writes_all_targets() {
    let tmp = TempDir::new().unwrap();
    let report = run(&default_targets(), tmp.path()).unwrap();
    // 5 commands × 3 targets = 15 command files
    // + 1 OpenCode hook + 1 omp hook = 17 total
    assert_eq!(report.written.len(), 17);
    assert!(tmp.path().join(".claude/commands/fl-new.md").exists());
    assert!(tmp.path().join(".opencode/command/fl-new.md").exists());
    assert!(tmp.path().join(".omp/commands/fl-new.md").exists());
    // OpenCode hook files (plugin auto-loaded from .opencode/plugins/).
    assert!(!tmp.path().join(".opencode/opencode.json").exists());
    assert!(tmp.path().join(".opencode/plugins/fl.ts").exists());
    // omp hook file.
    assert!(tmp.path().join(".omp/hooks/pre/fl-gate.ts").exists());
}

#[test]
fn claude_only_writes_claude_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 5);
    assert!(tmp.path().join(".claude/commands/fl-new.md").exists());
    // Claude-only target must NOT touch any OpenCode directory.
    assert!(
        !tmp.path().join(".opencode/").exists(),
        "Claude-only setup must not create .opencode/"
    );
    // Claude-only must NOT write the legacy (root-level) hook paths.
    assert!(
        !tmp.path().join("opencode.json").exists(),
        "Claude-only setup must not write legacy opencode.json"
    );
    assert!(
        !tmp.path().join("plugin/").exists(),
        "Claude-only setup must not create legacy plugin/ directory"
    );
}

#[test]
fn opencode_only_writes_opencode_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::OpenCode], tmp.path()).unwrap();
    // 5 commands + 1 hook file = 6
    assert_eq!(report.written.len(), 6);
    assert!(tmp.path().join(".opencode/command/fl-new.md").exists());
    assert!(!tmp.path().join(".claude/").exists());
    // OpenCode-only registers the plugin file (auto-loaded from dir).
    assert!(!tmp.path().join(".opencode/opencode.json").exists());
    assert!(tmp.path().join(".opencode/plugins/fl.ts").exists());
}

#[test]
fn omp_only_writes_omp_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::OhMyPi], tmp.path()).unwrap();
    // 5 commands + 1 hook file = 6
    assert_eq!(report.written.len(), 6);
    assert!(tmp.path().join(".omp/commands/fl-new.md").exists());
    // omp-only must NOT touch Claude or OpenCode directories.
    assert!(!tmp.path().join(".claude/").exists());
    assert!(!tmp.path().join(".opencode/").exists());
    // omp hook file.
    assert!(tmp.path().join(".omp/hooks/pre/fl-gate.ts").exists());
}

#[test]
fn written_files_have_valid_frontmatter() {
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    let content = fs::read_to_string(tmp.path().join(".claude/commands/fl-new.md")).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains("\n---\n"));
    assert!(content.contains("description:"));
}

#[test]
fn run_creates_deeply_nested_root() {
    let tmp = TempDir::new().unwrap();
    let bogus = tmp.path().join("nonexistent/deep/path");
    // create_dir_all should handle this — should not error.
    let report = run(&[Target::Claude], &bogus).unwrap();
    assert!(!report.written.is_empty());
    assert!(bogus.join(".claude/commands/fl-new.md").exists());
}

#[test]
fn run_is_order_independent() {
    // The same set of targets in different orders must produce
    // identical file sets (deterministic output, idempotent re-runs).
    let tmp1 = TempDir::new().unwrap();
    let tmp2 = TempDir::new().unwrap();
    let r1 = run(&[Target::Claude, Target::OpenCode], tmp1.path()).unwrap();
    let r2 = run(&[Target::OpenCode, Target::Claude], tmp2.path()).unwrap();

    let names1: BTreeSet<_> = r1
        .written
        .iter()
        .map(|p| p.file_name().unwrap().to_owned())
        .collect();
    let names2: BTreeSet<_> = r2
        .written
        .iter()
        .map(|p| p.file_name().unwrap().to_owned())
        .collect();
    assert_eq!(names1, names2);
    assert_eq!(r1.written.len(), r2.written.len());
}

#[test]
fn run_overwrites_existing_files_with_current_compile_output() {
    // Documented behavior: fs::write silently overwrites. Re-running
    // `setup` produces the same content — deterministic, idempotent.
    let tmp = TempDir::new().unwrap();
    let target_path = tmp.path().join(".claude/commands/fl-new.md");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    fs::write(&target_path, "STALE CONTENT FROM PREVIOUS RUN").unwrap();

    run(&[Target::Claude], tmp.path()).unwrap();
    let after = fs::read_to_string(&target_path).unwrap();
    assert!(
        !after.contains("STALE CONTENT"),
        "stale content should be overwritten"
    );
    assert!(after.starts_with("---\n"));
}

#[test]
fn run_writes_all_five_commands_per_target() {
    // Sanity: every Skill / Custom Command in the static table is written.
    // The 4 top-level subcommands (Setup, Gate, Status, Archive) are
    // intentionally absent — they are terminal CLI subcommands, not
    // registered skills. See
    // `.omc/plans/command-metadata-narrow-to-commands.md` for rationale.
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    let names: BTreeSet<_> = report
        .written
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
    let expected: BTreeSet<_> = [
        "fl-new.md",
        "fl-plan.md",
        "fl-audit.md",
        "fl-implement.md",
        "fl-review.md",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(names, expected);
}

#[test]
fn run_opencode_files_use_singular_command_dir() {
    // OpenCode's convention is `.opencode/command/` (singular),
    // not `.opencode/commands/` (plural). Verify path conventions.
    let tmp = TempDir::new().unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();
    assert!(tmp.path().join(".opencode/command/fl-new.md").exists());
    assert!(!tmp.path().join(".opencode/commands/").exists());
}

#[test]
fn run_claude_files_use_plural_commands_dir() {
    // Claude Code's convention is `.claude/commands/` (plural).
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    assert!(tmp.path().join(".claude/commands/fl-new.md").exists());
    assert!(!tmp.path().join(".claude/command/").exists());
}

#[test]
fn setup_md_is_not_generated() {
    // Regression test: `Setup` is a terminal-only subcommand, not a
    // registered skill / slash command. `run()` must not produce
    // `setup.md` on any target. See `.omc/plans/setup-excludes-self.md`.
    let tmp = TempDir::new().unwrap();
    let report = run(&default_targets(), tmp.path()).unwrap();
    assert!(
        !report
            .written
            .iter()
            .any(|p| p.file_name().unwrap() == "setup.md"),
        "run() must not produce a `setup.md` file; got: {:?}",
        report.written
    );
    assert!(
        !tmp.path().join(".claude/commands/setup.md").exists(),
        ".claude/commands/setup.md must not exist"
    );
    assert!(
        !tmp.path().join(".opencode/command/setup.md").exists(),
        ".opencode/command/setup.md must not exist"
    );
    assert!(
        !tmp.path().join(".omp/commands/setup.md").exists(),
        ".omp/commands/setup.md must not exist"
    );
}

#[test]
fn opencode_hook_files_have_expected_content() {
    // Validates the content contracts of the plugin file.
    // Per OpenCode docs, local plugins are auto-loaded from
    // .opencode/plugins/ — no opencode.json registration needed.
    // See `docs/opencode-hook-spec-correction.md` for rationale.
    let tmp = TempDir::new().unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();

    // No opencode.json is written (local plugin, auto-loaded).
    assert!(
        !tmp.path().join(".opencode/opencode.json").exists(),
        "must NOT write opencode.json for directory-loaded plugins"
    );

    // plugins/fl.ts contains the key contracts.
    let ts = fs::read_to_string(tmp.path().join(".opencode/plugins/fl.ts")).unwrap();
    assert!(ts.contains("fl gate"), "hook must call `fl gate`");
    assert!(
        ts.contains("session.idle"),
        "hook must filter on session.idle"
    );
    assert!(
        ts.contains("noReply: false"),
        "hook must set noReply: false to trigger AI auto-reply"
    );
    assert!(
        ts.contains(".nothrow()"),
        "hook must use .nothrow() to catch non-zero exit"
    );
}

#[test]
fn omp_hook_files_have_expected_content() {
    // Validates the content contracts of the omp hook file.
    // See `docs/omp-hook-porting.html` for the spec.
    let tmp = TempDir::new().unwrap();
    run(&[Target::OhMyPi], tmp.path()).unwrap();

    let ts =
        fs::read_to_string(tmp.path().join(".omp/hooks/pre/fl-gate.ts")).unwrap();
    assert!(ts.contains("fl gate"), "hook must call `fl gate`");
    assert!(
        ts.contains("session_stop"),
        "hook must use session_stop event"
    );
    assert!(ts.contains("continue: true"), "hook must set continue: true");
    assert!(
        ts.contains("additionalContext"),
        "hook must provide additionalContext"
    );
    assert!(ts.contains(".nothrow()"), "hook must use .nothrow()");
    assert!(
        !ts.contains(".quiet()"),
        "hook must NOT use .quiet() — suppresses stderr capture"
    );
    assert!(
        ts.contains("@oh-my-pi/pi-coding-agent"),
        "hook must import from @oh-my-pi/pi-coding-agent"
    );
}

#[test]
fn opencode_migrates_legacy_files() {
    // Pre-condition: an older `fl setup` wrote files at the legacy
    // (root-level) paths. New `fl setup` must delete them and write
    // only the plugin file (no opencode.json — auto-loaded from dir).
    let tmp = TempDir::new().unwrap();
    let old_json = tmp.path().join("opencode.json");
    let old_plugin_dir = tmp.path().join("plugin");
    let old_hook = old_plugin_dir.join("hook.ts");
    fs::write(&old_json, "{\"plugin\":[\"./plugin/hook.ts\"]}\n").unwrap();
    fs::create_dir_all(&old_plugin_dir).unwrap();
    fs::write(&old_hook, "// legacy hook\n").unwrap();

    // Also simulate the v2 error: .opencode/opencode.json with plugin entry.
    let v2_dir = tmp.path().join(".opencode");
    fs::create_dir_all(&v2_dir).unwrap();
    let v2_json = v2_dir.join("opencode.json");
    fs::write(&v2_json, "{\"plugin\":[\"./plugins/hook.ts\"]}\n").unwrap();

    run(&[Target::OpenCode], tmp.path()).unwrap();

    // Legacy files removed.
    assert!(!old_json.exists(), "legacy opencode.json must be deleted");
    assert!(!old_plugin_dir.exists(), "legacy plugin/ must be removed");
    assert!(!old_hook.exists(), "legacy plugin/hook.ts must be removed");
    // v2 error: .opencode/opencode.json must also be removed.
    assert!(!v2_json.exists(), "v2 .opencode/opencode.json must be deleted");

    // Only the plugin file is written (auto-loaded from directory).
    let new_hook = tmp.path().join(".opencode/plugins/fl.ts");
    assert!(new_hook.exists());
    assert!(
        !tmp.path().join(".opencode/opencode.json").exists(),
        "must NOT write opencode.json"
    );
}

#[test]
fn opencode_removes_v2_opencode_json() {
    // v2 of fl setup incorrectly wrote a local plugin path to
    // .opencode/opencode.json. Per OpenCode docs, local plugins
    // are auto-loaded from .opencode/plugins/ — no config entry.
    // Setup must now delete any existing .opencode/opencode.json.
    let tmp = TempDir::new().unwrap();
    let new_dir = tmp.path().join(".opencode");
    fs::create_dir_all(&new_dir).unwrap();
    let json_path = new_dir.join("opencode.json");
    fs::write(
        &json_path,
        "{\n  \"plugin\": [\"./other.ts\"],\n  \"theme\": \"dark\"\n}\n",
    )
    .unwrap();

    run(&[Target::OpenCode], tmp.path()).unwrap();

    // .opencode/opencode.json was deleted (v2 error cleanup).
    assert!(
        !json_path.exists(),
        ".opencode/opencode.json must be deleted by setup"
    );
    // Plugin file written correctly.
    assert!(tmp.path().join(".opencode/plugins/fl.ts").exists());
}

#[test]
fn opencode_setup_is_idempotent() {
    // Running `fl setup` twice in a row must produce the same
    // file set without errors (idempotent re-run).
    let tmp = TempDir::new().unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();

    // No opencode.json is ever written.
    assert!(
        !tmp.path().join(".opencode/opencode.json").exists(),
        "must never write opencode.json"
    );
    // Plugin file exists after both runs.
    assert!(tmp.path().join(".opencode/plugins/fl.ts").exists());
}
