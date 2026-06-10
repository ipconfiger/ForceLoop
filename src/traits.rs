use crate::context::Context;
use crate::errors::Result;
use crate::schema::CommandSchema;

/// Trait shared by Skills and Commands
pub trait Executable {
    fn execute(&self, ctx: &Context) -> Result<()>;
}

/// Trait for top-level subcommands (setup, gate, status, archive)
pub trait Subcommand: Executable {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

/// Trait for declarative metadata shared by all Command objects.
/// Provides skill/command templates, artifact file lists, and gating logic.
pub trait CommandMetadata {
    /// Returns the ForceLoop-native Skill schema for this command.
    /// Compile to platform-native format via `crate::compiler::compile`.
    fn skill_template(&self) -> CommandSchema;

    /// Returns the ForceLoop-native Command schema for this command.
    /// Compile to platform-native format via `crate::compiler::compile`.
    fn command_template(&self) -> CommandSchema;

    /// Returns the list of artifact files this command produces.
    fn artifacts(&self) -> &[&'static str];

    /// Gate method: verifies whether the next step in the pipeline can proceed.
    /// Skeleton implementation returns Ok(()).
    fn gate(&self, ctx: &Context) -> Result<()>;
}
