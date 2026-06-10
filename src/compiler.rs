// ============================================================================
// TDD: tests first
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_claude_minimal() {
        let s = CommandSchema {
            name: "code-review",
            description: "Review changes",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt: "You are a reviewer.",
        };
        let out = compile(&s, Target::Claude).unwrap();
        assert!(out.starts_with("---\ndescription: \"Review changes\"\n"));
        assert!(out.contains("\n---\n"));
        assert!(out.ends_with("\nYou are a reviewer.\n"));
    }

    #[test]
    fn test_compile_claude_with_tools() {
        static TOOLS: &[&str] = &["Read", "Bash"];
        let s = CommandSchema {
            name: "test",
            description: "d",
            model: None,
            argument_hint: None,
            tools: TOOLS,
            agent: None,
            prompt: "p",
        };
        let out = compile(&s, Target::Claude).unwrap();
        assert!(out.contains("allowed-tools: [Read, Bash]"));
    }

    #[test]
    fn test_compile_claude_with_hint_and_model() {
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: Some("opus"),
            argument_hint: Some("[file]"),
            tools: &[],
            agent: None,
            prompt: "p",
        };
        let out = compile(&s, Target::Claude).unwrap();
        assert!(out.contains("argument-hint: \"[file]\""));
        assert!(out.contains("model: opus"));
    }

    #[test]
    fn test_compile_opencode_with_agent() {
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: Some("reviewer"),
            prompt: "p",
        };
        let out = compile(&s, Target::OpenCode).unwrap();
        assert!(out.contains("agent: reviewer"));
        assert!(!out.contains("allowed-tools"));
        assert!(!out.contains("argument-hint"));
    }

    #[test]
    fn test_compile_opencode_drops_tools() {
        // schema.tools is dropped on OpenCode (command body does not accept it).
        static TOOLS: &[&str] = &["Read"];
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: TOOLS,
            agent: Some("a"),
            prompt: "p",
        };
        let out = compile(&s, Target::OpenCode).unwrap();
        assert!(!out.contains("allowed-tools"));
        assert!(!out.contains("tools:"));
    }

    #[test]
    fn test_compile_preserves_prompt() {
        let prompt = "# Step 1\nDo thing.\n\n## Step 2\nDo other.";
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt,
        };
        let claude = compile(&s, Target::Claude).unwrap();
        let opencode = compile(&s, Target::OpenCode).unwrap();
        assert!(claude.contains(prompt));
        assert!(opencode.contains(prompt));
    }

    #[test]
    fn test_quote_description_with_special_chars() {
        // Description with colon and double quote must be escaped.
        let s = CommandSchema {
            name: "x",
            description: "He said: \"hello\"",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt: "p",
        };
        let out = compile(&s, Target::Claude).unwrap();
        assert!(out.contains("description: \"He said: \\\"hello\\\"\""));
    }
}

// ============================================================================
// Production code
// ============================================================================

use crate::errors::Result;
use crate::schema::CommandSchema;

/// Target platform for compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Anthropic Claude Code (`.claude/commands/<name>.md`).
    Claude,
    /// sst/opencode v2 (`.opencode/command/<name>.md`).
    OpenCode,
}

/// Compile a ForceLoop schema into a platform-native markdown file.
///
/// Returns the full file content: YAML frontmatter + body markdown.
/// Pure function — no IO, no side effects.
pub fn compile(schema: &CommandSchema, target: Target) -> Result<String> {
    match target {
        Target::Claude => compile_to_claude(schema),
        Target::OpenCode => compile_to_opencode(schema),
    }
}

fn compile_to_claude(schema: &CommandSchema) -> Result<String> {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("description: {}", quote(schema.description)));

    if !schema.tools.is_empty() {
        let tools = schema.tools.join(", ");
        parts.push(format!("allowed-tools: [{}]", tools));
    }

    if let Some(hint) = schema.argument_hint {
        parts.push(format!("argument-hint: {}", quote(hint)));
    }

    if let Some(model) = schema.model {
        parts.push(format!("model: {}", model));
    }

    let front = parts.join("\n");
    Ok(format!("---\n{}\n---\n\n{}\n", front, schema.prompt))
}

fn compile_to_opencode(schema: &CommandSchema) -> Result<String> {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("description: {}", quote(schema.description)));

    if let Some(agent) = schema.agent {
        parts.push(format!("agent: {}", agent));
    }

    if let Some(model) = schema.model {
        parts.push(format!("model: {}", model));
    }

    let front = parts.join("\n");
    Ok(format!("---\n{}\n---\n\n{}\n", front, schema.prompt))
}

/// Wrap a description-style string in double quotes, escaping internal `"`.
///
/// Rule: always quoted (consistent output). Internal `"` becomes `\"`.
/// Newlines inside descriptions are preserved verbatim (rare in practice;
/// tests assert on simple inputs).
fn quote(s: &str) -> String {
    let escaped = s.replace('"', "\\\"");
    format!("\"{}\"", escaped)
}
