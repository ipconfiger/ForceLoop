use forceloop::commands::{Audit, Implement, New, Plan, Review};
use forceloop::compiler::{compile, compile_agent, Target};
use forceloop::schema::CommandSchema;
use forceloop::traits::CommandMetadata;

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

// ============================================================================
// Real schemas — verify all 10 Command objects have populated fields
// (not just CommandSchema::default())
// ============================================================================

fn assert_populated(name: &str, s: CommandSchema) {
    assert!(!s.name.is_empty(), "{}: name should be set", name);
    assert!(
        !s.description.is_empty(),
        "{}: description should be set",
        name
    );
    assert!(!s.prompt.is_empty(), "{}: prompt should be set", name);
}

#[test]
fn all_5_commands_have_populated_schemas() {
    // Only the 5 Skill / Custom Command objects in `src/commands/`
    // implement `CommandMetadata` and have a skill/command template.
    // The 4 top-level subcommands (Setup, Gate, Status, Archive) are
    // terminal CLI subcommands and intentionally do NOT implement
    // `CommandMetadata` — see `.omc/plans/command-metadata-narrow-to-commands.md`.

    assert_populated("New", New.skill_template());
    assert_populated("New", New.command_template());
    assert_populated("Plan", Plan.skill_template());
    assert_populated("Plan", Plan.command_template());
    assert_populated("Audit", Audit.skill_template());
    assert_populated("Audit", Audit.command_template());
    assert_populated("Implement", Implement.skill_template());
    assert_populated("Implement", Implement.command_template());
    assert_populated("Review", Review.skill_template());
    assert_populated("Review", Review.command_template());
}

#[test]
fn skill_and_command_schemas_share_metadata_but_differ_in_prompt() {
    // Skill = detailed workflow; Command = short invocation.
    // They share name/description/argument-hint, but prompt body differs.
    let skill = New.skill_template();
    let command = New.command_template();

    assert_eq!(skill.name, command.name);
    assert_eq!(skill.description, command.description);
    assert_eq!(skill.argument_hint, command.argument_hint);
    assert_eq!(skill.tools, command.tools);
    assert_ne!(
        skill.prompt, command.prompt,
        "skill and command bodies should differ"
    );
    assert!(
        skill.prompt.len() > command.prompt.len(),
        "skill body should be more detailed than command body"
    );
}

#[test]
fn each_command_has_appropriate_tools() {
    // Sanity: the tool whitelist reflects what the Skill / Custom Command
    // actually does. Only the 6 objects in `src/commands/` are checked.
    assert!(Implement.skill_template().tools.contains(&"Edit"));
    assert!(Implement.skill_template().tools.contains(&"Bash"));
    assert!(Review.skill_template().tools.contains(&"Grep"));
    assert!(Audit.skill_template().tools.contains(&"Read"));
    // Read-only commands should NOT have Write/Edit
    assert!(!Audit.skill_template().tools.contains(&"Edit"));
}

#[test]
fn all_commands_compile_to_valid_claude_markdown() {
    // Sanity: every Skill / Custom Command produces a parseable frontmatter
    // + body when compiled to Claude format.
    let schemas = [
        New.skill_template(),
        Plan.skill_template(),
        Audit.skill_template(),
        Implement.skill_template(),
        Review.skill_template(),
    ];
    for s in &schemas {
        let out = compile(s, Target::Claude).unwrap();
        assert!(out.starts_with("---\n"), "{}: must start with ---", s.name);
        assert!(out.contains("---"), "{}: must close frontmatter", s.name);
        assert!(
            out.contains(&format!("description: \"{}", s.description)),
            "{}: description should be quoted",
            s.name
        );
    }
}

#[test]
fn end_to_end_compile_agent_with_real_implement_schema() {
    // Implement has Edit + Bash + Grep + Glob + Write — exercises full mapping.
    let s = Implement.skill_template();
    let out = compile_agent("fl-implement", &s).unwrap();

    assert!(out.starts_with("---\nname: fl-implement\n"));
    assert!(out.contains("permissions:"));
    // All Implement tool categories should map
    assert!(out.contains("bash: \"allow\"")); // Bash
    assert!(out.contains("edit: \"allow\"")); // Write, Edit
    assert!(out.contains("read: \"allow\"")); // Read, Grep, Glob (via Grep/Glob in tools)
    // No per-tool entries
    assert!(!out.contains("Edit:"));
    assert!(!out.contains("Bash:"));
}
