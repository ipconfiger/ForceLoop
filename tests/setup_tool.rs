use std::collections::BTreeSet;
use std::fs;

use forceloop::compiler::Target;
use forceloop::context::Context;
use forceloop::setup::{default_targets, effective_targets, run, DEFAULT_TARGETS};
use tempfile::TempDir;

#[test]
fn default_targets_constant_is_both_platforms() {
    // Pins DEFAULT_TARGETS — if anyone changes the default, this fails.
    assert_eq!(default_targets(), vec![Target::Claude, Target::OpenCode]);
    assert_eq!(DEFAULT_TARGETS.len(), 2);
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
fn run_default_writes_both_targets() {
    let tmp = TempDir::new().unwrap();
    let report = run(&default_targets(), tmp.path()).unwrap();
    // 6 commands × 2 targets = 12 command files
    // + 2 OpenCode hook files (opencode.json + plugin/hook.ts)
    // = 14 total
    assert_eq!(report.written.len(), 14);
    assert!(tmp.path().join(".claude/commands/new.md").exists());
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    // OpenCode hook files
    assert!(tmp.path().join("opencode.json").exists());
    assert!(tmp.path().join("plugin/hook.ts").exists());
}

#[test]
fn claude_only_writes_claude_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 6);
    assert!(tmp.path().join(".claude/commands/new.md").exists());
    assert!(!tmp.path().join(".opencode/").exists());
    // Claude-only must NOT register the OpenCode hook.
    assert!(
        !tmp.path().join("opencode.json").exists(),
        "Claude-only setup must not write opencode.json"
    );
    assert!(
        !tmp.path().join("plugin/").exists(),
        "Claude-only setup must not create plugin/ directory"
    );
}

#[test]
fn opencode_only_writes_opencode_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::OpenCode], tmp.path()).unwrap();
    // 6 commands + 2 hook files = 8
    assert_eq!(report.written.len(), 8);
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    assert!(!tmp.path().join(".claude/").exists());
    // OpenCode-only DOES register the hook.
    assert!(tmp.path().join("opencode.json").exists());
    assert!(tmp.path().join("plugin/hook.ts").exists());
}

#[test]
fn written_files_have_valid_frontmatter() {
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    let content = fs::read_to_string(tmp.path().join(".claude/commands/new.md")).unwrap();
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
    assert!(bogus.join(".claude/commands/new.md").exists());
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
    let target_path = tmp.path().join(".claude/commands/new.md");
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
fn run_writes_all_six_commands_per_target() {
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
        "new.md",
        "plan.md",
        "audit.md",
        "implement.md",
        "review.md",
        "try_finish.md",
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
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    assert!(!tmp.path().join(".opencode/commands/").exists());
}

#[test]
fn run_claude_files_use_plural_commands_dir() {
    // Claude Code's convention is `.claude/commands/` (plural).
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    assert!(tmp.path().join(".claude/commands/new.md").exists());
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
}

#[test]
fn opencode_hook_files_have_expected_content() {
    // Validates the content contracts of both hook files:
    //   - opencode.json is valid JSON pointing to ./plugin/hook.ts
    //   - plugin/hook.ts references fl gate, session.idle, noReply: false
    //     and the 60_000 ms timeout
    // See `.omc/plans/opencode-session-idle-gate-hook.md` for rationale.
    let tmp = TempDir::new().unwrap();
    run(&[Target::OpenCode], tmp.path()).unwrap();

    // opencode.json is valid JSON pointing to our plugin
    let json = fs::read_to_string(tmp.path().join("opencode.json")).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("opencode.json must be valid JSON");
    assert_eq!(v["plugin"][0], "./plugin/hook.ts");

    // plugin/hook.ts contains the key contracts
    let ts = fs::read_to_string(tmp.path().join("plugin/hook.ts")).unwrap();
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
        ts.contains(".timeout(60_000)"),
        "hook must use 60s timeout per reference doc"
    );
}
