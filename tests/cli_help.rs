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
