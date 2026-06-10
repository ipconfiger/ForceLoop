// ============================================================================
// TDD: tests first
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schema() {
        let s = CommandSchema::default();
        assert_eq!(s.name, "");
        assert_eq!(s.description, "");
        assert!(s.model.is_none());
        assert!(s.argument_hint.is_none());
        assert!(s.tools.is_empty());
        assert!(s.agent.is_none());
        assert_eq!(s.prompt, "");
    }

    #[test]
    fn test_schema_construction() {
        static TOOLS: &[&str] = &["Read", "Bash"];

        let s = CommandSchema {
            name: "code-review",
            description: "Review changed files",
            model: Some("opus"),
            argument_hint: Some("[files...]"),
            tools: TOOLS,
            agent: Some("reviewer"),
            prompt: "You are a reviewer.",
        };

        assert_eq!(s.name, "code-review");
        assert_eq!(s.description, "Review changed files");
        assert_eq!(s.model, Some("opus"));
        assert_eq!(s.argument_hint, Some("[files...]"));
        assert_eq!(s.tools.len(), 2);
        assert_eq!(s.tools[0], "Read");
        assert_eq!(s.tools[1], "Bash");
        assert_eq!(s.agent, Some("reviewer"));
        assert_eq!(s.prompt, "You are a reviewer.");
    }
}

// ============================================================================
// Production code
// ============================================================================

/// ForceLoop-native command/skill schema.
///
/// Single source of truth for command/skill declarations. Compile to
/// platform-native formats via [`crate::compiler::compile`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandSchema {
    /// Command identifier (e.g. "code-review", "implement").
    pub name: &'static str,

    /// Short human-readable description; emitted as the `description` frontmatter
    /// field on every supported platform.
    pub description: &'static str,

    /// Model identifier (e.g. "opus", "sonnet"). `None` = use platform default.
    pub model: Option<&'static str>,

    /// Argument hint for the slash command (e.g. "[file] [query]").
    /// Claude only — OpenCode does not support it.
    pub argument_hint: Option<&'static str>,

    /// Whitelist of tool names this command may use
    /// (e.g. `&["Read", "Grep", "Bash"]`).
    ///
    /// Empty slice = no restriction. Compiled to `allowed-tools` in Claude;
    /// dropped on OpenCode (command body does not accept it; future work will
    /// pass it through to the delegated agent's `permissions`).
    pub tools: &'static [&'static str],

    /// Name of the sub-agent to delegate to. OpenCode only — `None` means
    /// execute inline. Claude ignores this field.
    pub agent: Option<&'static str>,

    /// Body markdown — the actual prompt/workflow definition.
    /// Supports the `$ARGUMENTS` placeholder on both platforms.
    pub prompt: &'static str,
}
