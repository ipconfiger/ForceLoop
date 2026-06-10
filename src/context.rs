use crate::compiler::Target;

/// Per-invocation context passed to [`crate::traits::Executable::execute`].
///
/// Holds any cross-cutting data the dispatch site ([`crate::main`]) wants
/// to forward to the executed command. Currently carries the
/// `--tool` target list, populated only when the user invokes
/// `forceloop setup --tool ...`. Other subcommands see an empty list
/// (their `execute()` methods ignore it).
pub struct Context {
    /// Targets selected via `--tool` on the CLI. Empty means "user did
    /// not specify" — each command decides its own default
    /// (e.g. `Setup` expands to all supported targets).
    pub targets: Vec<Target>,
}

impl Context {
    /// Default context with no targets. Used when dispatching
    /// subcommands that don't read `targets`.
    pub fn new() -> Self {
        Self { targets: vec![] }
    }

    /// Context populated with the user's `--tool` selections
    /// (already converted from `cli::Tool` to `compiler::Target`).
    pub fn with_targets(targets: Vec<Target>) -> Self {
        Self { targets }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_default_has_empty_targets() {
        let ctx = Context::new();
        assert!(ctx.targets.is_empty());
    }

    #[test]
    fn context_with_targets_stores_values() {
        let ctx = Context::with_targets(vec![Target::Claude]);
        assert_eq!(ctx.targets, vec![Target::Claude]);
    }
}
