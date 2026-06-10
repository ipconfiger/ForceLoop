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
    assert_eq!(report.written.len(), 20); // 10 commands × 2 targets
    assert!(tmp.path().join(".claude/commands/setup.md").exists());
    assert!(tmp.path().join(".opencode/command/setup.md").exists());
}

#[test]
fn claude_only_writes_claude_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 10);
    assert!(tmp.path().join(".claude/commands/new.md").exists());
    assert!(!tmp.path().join(".opencode/").exists());
}

#[test]
fn opencode_only_writes_opencode_dir() {
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::OpenCode], tmp.path()).unwrap();
    assert_eq!(report.written.len(), 10);
    assert!(tmp.path().join(".opencode/command/new.md").exists());
    assert!(!tmp.path().join(".claude/").exists());
}

#[test]
fn written_files_have_valid_frontmatter() {
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    let content = fs::read_to_string(tmp.path().join(".claude/commands/setup.md")).unwrap();
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
    assert!(bogus.join(".claude/commands/setup.md").exists());
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
    let target_path = tmp.path().join(".claude/commands/setup.md");
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
fn run_writes_all_ten_commands_per_target() {
    // Sanity: every Command in the static table is written.
    let tmp = TempDir::new().unwrap();
    let report = run(&[Target::Claude], tmp.path()).unwrap();
    let names: BTreeSet<_> = report
        .written
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
    let expected: BTreeSet<_> = [
        "setup.md",
        "gate.md",
        "status.md",
        "archive.md",
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
    assert!(tmp.path().join(".opencode/command/setup.md").exists());
    assert!(!tmp.path().join(".opencode/commands/").exists());
}

#[test]
fn run_claude_files_use_plural_commands_dir() {
    // Claude Code's convention is `.claude/commands/` (plural).
    let tmp = TempDir::new().unwrap();
    run(&[Target::Claude], tmp.path()).unwrap();
    assert!(tmp.path().join(".claude/commands/setup.md").exists());
    assert!(!tmp.path().join(".claude/command/").exists());
}
