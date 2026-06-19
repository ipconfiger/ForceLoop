use std::process::Command;

#[test]
fn help_shows_all_subcommands() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute forceloop");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("setup"), "help should mention 'setup'");
    assert!(stdout.contains("gate"), "help should mention 'gate'");
    assert!(stdout.contains("new"), "help should mention 'new'");
    assert!(stdout.contains("plan"), "help should mention 'plan'");
    assert!(stdout.contains("audit"), "help should mention 'audit'");
    assert!(stdout.contains("implement"), "help should mention 'implement'");
    assert!(stdout.contains("review"), "help should mention 'review'");
    assert!(stdout.contains("status"), "help should mention 'status'");
    assert!(stdout.contains("archive"), "help should mention 'archive'");
}

#[test]
fn setup_help_works() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--help"])
        .output()
        .expect("Failed to execute forceloop setup --help");

    assert!(output.status.success(), "setup --help should succeed");
}

#[test]
fn setup_help_mentions_tool_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--help"])
        .output()
        .expect("Failed to execute forceloop setup --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--tool"),
        "setup --help should mention --tool flag, got:\n{}",
        stdout
    );
}

#[test]
fn setup_help_lists_tool_values() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--help"])
        .output()
        .expect("Failed to execute forceloop setup --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // clap's ValueEnum output: "[possible values: claude, opencode]"
    assert!(
        stdout.contains("claude"),
        "setup --help should list 'claude' as a possible value, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("opencode"),
        "setup --help should list 'opencode' as a possible value, got:\n{}",
        stdout
    );
}

#[test]
fn setup_tool_accepts_claude_value() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--tool", "claude", "--help"])
        .output()
        .expect("Failed to execute forceloop setup --tool claude --help");

    assert!(
        output.status.success(),
        "setup --tool claude --help should succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn setup_tool_accepts_opencode_value() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--tool", "opencode", "--help"])
        .output()
        .expect("Failed to execute forceloop setup --tool opencode --help");

    assert!(output.status.success());
}

#[test]
fn setup_tool_rejects_unknown_value() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--tool", "bogus"])
        .output()
        .expect("Failed to execute forceloop setup --tool bogus");

    assert!(
        !output.status.success(),
        "setup --tool bogus should fail with non-zero exit"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bogus") || stderr.contains("invalid"),
        "error should mention the invalid value, got: {}",
        stderr
    );
}
