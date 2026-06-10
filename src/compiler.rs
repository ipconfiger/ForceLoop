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

    // ----------------------------------------------------------------
    // compile_agent tests
    // ----------------------------------------------------------------

    #[test]
    fn test_compile_agent_minimal() {
        let s = CommandSchema {
            name: "reviewer",
            description: "Review changes",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt: "You are a reviewer.",
        };
        let out = compile_agent("reviewer", &s).unwrap();
        assert!(out.starts_with("---\nname: reviewer\n"));
        assert!(out.contains("description: \"Review changes\""));
        assert!(!out.contains("model:"));
        assert!(!out.contains("permissions:"));
        assert!(out.ends_with("\nYou are a reviewer.\n"));
    }

    #[test]
    fn test_compile_agent_with_model() {
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: Some("opus"),
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt: "p",
        };
        let out = compile_agent("x", &s).unwrap();
        assert!(out.contains("model: opus"));
    }

    #[test]
    fn test_compile_agent_tools_to_permissions() {
        static TOOLS: &[&str] = &["Read", "Write", "Edit", "Bash", "Grep", "Glob"];
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: TOOLS,
            agent: None,
            prompt: "p",
        };
        let out = compile_agent("x", &s).unwrap();
        // Tool categories mapped to OpenCode permission keys
        assert!(out.contains("read: \"allow\""), "Read/Grep/Glob → read");
        assert!(out.contains("edit: \"allow\""), "Write/Edit → edit");
        assert!(out.contains("bash: \"allow\""), "Bash → bash");
        // Should NOT emit per-tool entries
        assert!(!out.contains("Read:"));
        assert!(!out.contains("Write:"));
    }

    #[test]
    fn test_compile_agent_no_tools_omits_permissions() {
        // Empty tools → no permissions block (agent unrestricted)
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: &[],
            agent: None,
            prompt: "p",
        };
        let out = compile_agent("x", &s).unwrap();
        assert!(!out.contains("permissions:"));
    }

    #[test]
    fn test_compile_agent_unknown_tool_defaults_to_ask() {
        static TOOLS: &[&str] = &["MysteryTool"];
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: TOOLS,
            agent: None,
            prompt: "p",
        };
        let out = compile_agent("x", &s).unwrap();
        // Unknown tool → bash: "ask" safe default
        assert!(out.contains("bash: \"ask\""));
    }

    #[test]
    fn test_compile_agent_web_tools() {
        static TOOLS: &[&str] = &["WebFetch", "WebSearch"];
        let s = CommandSchema {
            name: "x",
            description: "d",
            model: None,
            argument_hint: None,
            tools: TOOLS,
            agent: None,
            prompt: "p",
        };
        let out = compile_agent("x", &s).unwrap();
        assert!(out.contains("webfetch: \"allow\""));
        assert!(out.contains("websearch: \"allow\""));
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

/// Compile a ForceLoop schema into an OpenCode agent file.
///
/// Emits `.opencode/agent/<name>.md` with `permissions` derived from
/// `schema.tools`. The body reuses `schema.prompt` as the system prompt.
///
/// Use this when an OpenCode command delegates to an agent — the `tools`
/// whitelist is passed through to the agent's `permissions` block,
/// since OpenCode command bodies do not support `allowed-tools`.
///
/// Pure function — no IO, no side effects.
pub fn compile_agent(agent_name: &str, schema: &CommandSchema) -> Result<String> {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("name: {}", agent_name));
    parts.push(format!("description: {}", quote(schema.description)));

    if let Some(model) = schema.model {
        parts.push(format!("model: {}", model));
    }

    if !schema.tools.is_empty() {
        parts.push(format!("permissions: {}", tools_to_permissions(schema.tools)));
    }

    let front = parts.join("\n");
    Ok(format!("---\n{}\n---\n\n{}\n", front, schema.prompt))
}

/// Map tool whitelist to OpenCode v2 `permissions` block.
///
/// OpenCode permission keys (from `packages/core/src/config/agent.ts`):
/// - `bash` — shell execution
/// - `edit` — file mutation (Write/Edit/Patch/MultiEdit/NotebookEdit)
/// - `read` — file reading (Read/Glob/Grep)
/// - `webfetch` — HTTP fetch
/// - `websearch` — web search
/// - `task` — sub-agent delegation
///
/// Unknown tools fall back to `bash: "ask"` as a safe default.
fn tools_to_permissions(tools: &[&str]) -> String {
    use std::collections::BTreeMap;
    let mut perms: BTreeMap<&str, &str> = BTreeMap::new();
    for &tool in tools {
        match tool {
            "Bash" => {
                perms.insert("bash", "allow");
            }
            "Write" | "Edit" | "Patch" | "MultiEdit" | "NotebookEdit" => {
                perms.insert("edit", "allow");
            }
            "Read" | "Glob" | "Grep" => {
                perms.insert("read", "allow");
            }
            "WebFetch" => {
                perms.insert("webfetch", "allow");
            }
            "WebSearch" => {
                perms.insert("websearch", "allow");
            }
            "Task" => {
                perms.insert("task", "allow");
            }
            // Unknown tool — safe default: ask
            _ => {
                perms.insert("bash", "ask");
            }
        }
    }
    perms
        .into_iter()
        .map(|(k, v)| format!("  {}: \"{}\"", k, v))
        .collect::<Vec<_>>()
        .join("\n")
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
