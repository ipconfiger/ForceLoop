use forceloop::compiler::{compile, Target};
use forceloop::schema::CommandSchema;

#[test]
fn end_to_end_claude() {
    static TOOLS: &[&str] = &["Read", "Grep", "Bash"];

    let s = CommandSchema {
        name: "code-review",
        description: "Review changes with severity",
        model: Some("opus"),
        argument_hint: Some("[files...]"),
        tools: TOOLS,
        agent: None, // Claude does not delegate
        prompt: "You are Code Reviewer.\n\nSeverity: CRITICAL > HIGH > MEDIUM > LOW",
    };

    let out = compile(&s, Target::Claude).unwrap();

    // Frontmatter assertions
    assert!(out.contains("description: \"Review changes with severity\""));
    assert!(out.contains("allowed-tools: [Read, Grep, Bash]"));
    assert!(out.contains("argument-hint: \"[files...]\""));
    assert!(out.contains("model: opus"));

    // Body assertion
    assert!(out.contains("Severity: CRITICAL > HIGH > MEDIUM > LOW"));

    // OpenCode-only fields should NOT appear
    assert!(!out.contains("agent:"));
}

#[test]
fn end_to_end_opencode() {
    let s = CommandSchema {
        name: "code-review",
        description: "Review changes",
        model: Some("opus"),
        argument_hint: Some("[files...]"), // OpenCode does not support
        tools: &[],                       // OpenCode command body does not support yet
        agent: Some("reviewer"),
        prompt: "Delegate to reviewer agent.",
    };

    let out = compile(&s, Target::OpenCode).unwrap();

    assert!(out.contains("description: \"Review changes\""));
    assert!(out.contains("agent: reviewer"));
    assert!(out.contains("model: opus"));

    // Fields not supported on OpenCode command body
    assert!(!out.contains("allowed-tools"));
    assert!(!out.contains("argument-hint"));
    assert!(!out.contains("tools:"));
}
