/// ForceLoop project directory name (under project root)
pub const FORCELOOP_DIR: &str = ".forceloop";

/// State file name (inside .forceloop/)
pub const STATE_FILE: &str = "state.json";

/// Development plan file name (inside .forceloop/)
pub const PLAN_FILE: &str = "plan.json";

/// Skills subdirectory name (inside .forceloop/)
pub const SKILLS_DIR: &str = "skills";

/// Custom commands subdirectory name (inside .forceloop/)
pub const COMMANDS_DIR: &str = "commands";

/// Hooks subdirectory name (inside .forceloop/)
pub const HOOKS_DIR: &str = "hooks";

/// Archive subdirectory name (inside .forceloop/, for archived plans)
pub const ARCHIVE_DIR: &str = "archive";

/// Git directory name (project marker)
pub const GIT_DIR: &str = ".git";

/// Cargo manifest file name (project marker)
pub const CARGO_MANIFEST: &str = "Cargo.toml";

/// Environment variable name for forcing project root override
pub const ENV_PROJECT_ROOT: &str = "FORCELOOP_PROJECT_ROOT";

/// Environment variable name for debug/verbose output
pub const ENV_DEBUG: &str = "FORCELOOP_DEBUG";
