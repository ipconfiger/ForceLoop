use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;
use crate::traits::{CommandMetadata, Executable};

const SKILL_PROMPT: &str = "\
# Review Skill

Regression-validate the implementation against the goal and spec.

## Steps
1. Read `.forceloop/plan.json` (goal + spec + all phases).
2. Run full test suite (`cargo test`).
3. Run lints (`cargo clippy --all-targets`).
4. Verify each phase's `acceptance_criteria` are met.
5. Check for regressions: no removed tests, coverage not regressed.
6. Output verdict: APPROVE / REQUEST CHANGES / REJECT.
";

const COMMAND_PROMPT: &str = "\
Review the implementation for regressions.

Runs tests + lints, checks each phase's acceptance criteria,
emits APPROVE / REQUEST CHANGES / REJECT verdict.
";

fn review_skill() -> CommandSchema {
    CommandSchema {
        name: "review",
        description: "Regression-validate the implementation",
        model: None,
        argument_hint: Some("[files...]"),
        tools: &["Read", "Grep", "Bash"],
        agent: None,
        prompt: SKILL_PROMPT,
    }
}

fn review_command() -> CommandSchema {
    CommandSchema {
        prompt: COMMAND_PROMPT,
        ..review_skill()
    }
}

pub struct Review;

impl Executable for Review {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        todo!()
    }
}

impl CommandMetadata for Review {
    fn skill_template(&self) -> CommandSchema {
        review_skill()
    }
    fn command_template(&self) -> CommandSchema {
        review_command()
    }
    fn artifacts(&self) -> &[&'static str] {
        &[]
    }
    fn gate(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
