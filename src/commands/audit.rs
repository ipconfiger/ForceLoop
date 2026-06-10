use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

const SKILL_PROMPT: &str = "\
# Audit Skill

Review the design spec and development plan for completeness.

## Steps
1. Read `.forceloop/plan.json` (spec + phases).
2. Check spec for:
   - Clear problem statement
   - Measurable success criteria
   - Explicit non-goals
   - Identified risks
3. Check phases for:
   - Verifiable acceptance criteria
   - Reasonable phase size
   - No circular dependencies
4. Output a structured review with severity-rated issues.

## Severity Levels
- CRITICAL: blocker, cannot proceed
- HIGH: significant gap, fix before impl
- MEDIUM: improvement recommended
- LOW: nitpick
";

const COMMAND_PROMPT: &str = "\
Audit the design spec and development plan.

Emits a severity-rated review (CRITICAL/HIGH/MEDIUM/LOW).
Read-only — no code changes.
";

fn audit_skill() -> CommandSchema {
    CommandSchema {
        name: "audit",
        description: "Audit design spec and development plan",
        model: None,
        argument_hint: Some("[files...]"),
        tools: &["Read", "Grep", "Glob"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn audit_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..audit_skill()
    }
}

pub struct Audit;

impl Executable for Audit {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for Audit {
    fn skill_template(&self) -> CommandSchema {
        audit_skill()
    }
    fn command_template(&self) -> CommandSchema {
        audit_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
