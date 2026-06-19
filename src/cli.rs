use clap::{Parser, Subcommand, ValueEnum};

use crate::compiler::Target;

#[derive(Parser)]
#[command(name = "forceloop")]
#[command(about = "A CLI tool for structured development workflow")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Target tool for slash command / Skill file injection.
///
/// Values are serialized to lowercase strings (`claude`, `opencode`) for
/// the `--tool` CLI flag. `From<Tool> for Target` (defined in this module)
/// converts at the `main.rs` boundary so that internal modules depend
/// only on `Target` (the compiler-layer enum), not on `Tool` (the
/// clap-layer enum).
///
/// Note on `OpenCode` value name: clap's `ValueEnum` derives would
/// otherwise kebab-case this to `open-code`. We override with
/// `#[value(name = "opencode")]` to match the project-wide naming
/// convention agreed in the design plan.
#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum Tool {
    Claude,
    #[value(name = "opencode")]
    OpenCode,
    #[value(name = "omp")]
    OhMyPi,
}

/// Convert CLI `Tool` enum to compiler `Target` enum.
///
/// **Module layering note**: this impl lives in `cli`, NOT in `compiler`.
/// `compiler` is a dependency of `cli` (cli uses `Target`); the reverse
/// direction would create a cycle. If you need to convert the other way
/// (Target → Tool), add a separate `From` impl in `compiler` and route
/// through the `Display`/`TryFrom` boundary — do not move this impl.
impl From<Tool> for Target {
    fn from(t: Tool) -> Self {
        match t {
            Tool::Claude => Target::Claude,
            Tool::OpenCode => Target::OpenCode,
            Tool::OhMyPi => Target::OhMyPi,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize project directory structure, state, subcommands, skills, and hooks.
    /// Use --tool to restrict the injection target. Omit --tool to install
    /// to BOTH Claude Code and OpenCode (the default).
    Setup {
        /// Target tool for slash command / Skill file injection.
        /// May be repeated. Omit to install to both Claude Code and OpenCode.
        #[arg(long, value_enum)]
        tool: Vec<Tool>,
    },
    /// Gate control command, typically invoked by hooks
    Gate,
    /// Create a new development goal and design spec
    New,
    /// Create development plan (multiple waves)
    Plan,
    /// Audit design spec and development plan
    Audit,
    /// Develop the current phase (TDD)
    Implement,
    /// Regression-validate the implementation
    Review,
    /// View current status
    Status,
    /// Archive development plan
    Archive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_tool_to_target() {
        assert_eq!(Target::from(Tool::Claude), Target::Claude);
        assert_eq!(Target::from(Tool::OpenCode), Target::OpenCode);
    }
}
