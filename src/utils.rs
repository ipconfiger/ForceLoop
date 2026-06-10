use std::path::PathBuf;

use crate::errors::Result;

/// Returns the current working directory.
///
/// Wraps `std::env::current_dir()` for consistency with the rest of the API.
pub fn current_dir() -> Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

/// Returns the path to the currently running executable.
///
/// Wraps `std::env::current_exe()` for consistency.
pub fn executable_path() -> Result<PathBuf> {
    Ok(std::env::current_exe()?)
}

/// Returns the project root directory (where `.forceloop/` lives).
///
/// Skeleton: requires business decision on marker file (`.git` vs `Cargo.toml`).
pub fn project_root() -> Result<PathBuf> {
    todo!()
}

/// Returns the `.forceloop/` directory under the project root.
pub fn state_dir() -> Result<PathBuf> {
    todo!()
}

/// Returns the absolute path to the state file.
pub fn state_file() -> Result<PathBuf> {
    todo!()
}

/// Returns `true` if the current directory is inside a ForceLoop project.
pub fn is_in_project() -> bool {
    todo!()
}
