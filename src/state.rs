use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::constants::{ERROR_LOG, FORCELOOP_DIR, STATE_FILE};
use crate::errors::{ForceLoopError, Result};

/// Pipeline state persisted at `.forceloop/state.json`.
///
/// Each boolean field represents whether the corresponding gate has
/// passed. Gates are checked in order: new → plan → audit → implement
/// → review → done. This design is idempotent: re-running
/// a passed gate simply re-sets `true` without side effects.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineState {
    #[serde(default)]
    pub new: bool,
    #[serde(default)]
    pub plan: bool,
    #[serde(default)]
    pub audit: bool,
    #[serde(default)]
    pub implement: bool,
    #[serde(default)]
    pub review: bool,
    #[serde(default)]
    pub done: bool,
}

impl PipelineState {
    /// Locate the state file by walking up from cwd looking for `.forceloop/`.
    pub fn locate_state_file() -> Result<PathBuf> {
        let cwd = std::env::current_dir().map_err(ForceLoopError::Io)?;
        let mut dir: Option<&Path> = Some(&cwd);
        while let Some(d) = dir {
            let candidate = d.join(FORCELOOP_DIR);
            if candidate.is_dir() {
                return Ok(candidate.join(STATE_FILE));
            }
            dir = d.parent();
        }
        Err(ForceLoopError::Config(
            "not in a ForceLoop project: no .forceloop/ directory found".into(),
        ))
    }

    /// Locate the `.forceloop/` directory by walking up from cwd.
    pub fn locate_forceloop_dir() -> Result<PathBuf> {
        let cwd = std::env::current_dir().map_err(ForceLoopError::Io)?;
        let mut dir: Option<&Path> = Some(&cwd);
        while let Some(d) = dir {
            let candidate = d.join(FORCELOOP_DIR);
            if candidate.is_dir() {
                return Ok(candidate);
            }
            dir = d.parent();
        }
        Err(ForceLoopError::Config(
            "not in a ForceLoop project: no .forceloop/ directory found".into(),
        ))
    }

    /// Read state from a path, returning the default (all false) if the
    /// file does not yet exist.
    ///
    /// Automatically migrates from the legacy `{"current_phase":"..."}`
    /// format to the new boolean-flag format. After migration the file
    /// is rewritten in the new format.
    pub fn read_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;

        // Try new format first (boolean flags).
        if let Ok(state) = serde_json::from_str::<Self>(&content) {
            return Ok(state);
        }

        // Fallback: legacy format with `current_phase`.
        #[derive(serde::Deserialize)]
        struct Legacy {
            current_phase: String,
        }
        if let Ok(legacy) = serde_json::from_str::<Legacy>(&content) {
            let mut state = Self::default();
            match legacy.current_phase.as_str() {
                "done" => {
                    state.new = true;
                    state.plan = true;
                    state.audit = true;
                    state.implement = true;
                    state.review = true;
                    state.done = true;
                }
                "review" => {
                    state.new = true;
                    state.plan = true;
                    state.audit = true;
                    state.implement = true;
                    state.review = true;
                }
                "implement" => {
                    state.new = true;
                    state.plan = true;
                    state.audit = true;
                    state.implement = true;
                }
                "audit" => {
                    state.new = true;
                    state.plan = true;
                    state.audit = true;
                }
                "plan" => {
                    state.new = true;
                    state.plan = true;
                }
                _ => {
                    state.new = true;
                }
            }
            // Write migrated state back.
            let migrated = serde_json::to_string_pretty(&state)
                .map_err(|e| ForceLoopError::Parse(format!("state.json migration: {e}")))?;
            let _ = fs::write(path, migrated);
            return Ok(state);
        }

        Err(ForceLoopError::Parse(format!(
            "state.json: unrecognized format: {content}"
        )))
    }

    /// Serialize and write state to the given path.
    ///
    /// On Unix, the file is made **read-only** after each write so that
    /// LLM tools (Write/Edit) cannot accidentally corrupt pipeline state.
    /// `write()` temporarily makes it writable, writes, then re-locks.
    ///
    /// This is the single entry point for persisting pipeline state, so
    /// this protection is universal — it works for all LLM platforms
    /// (Claude Code, OpenCode, oh-my-pi) without per-platform hooks.
    pub fn write(&self, path: &Path) -> Result<()> {
        make_writable(path);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ForceLoopError::Parse(format!("state.json serialization: {e}")))?;
        fs::write(path, content)?;
        make_readonly(path);
        Ok(())
    }

    /// Find the name of the first gate that hasn't been passed yet.
    pub fn next_pending(&self) -> Option<&'static str> {
        if !self.new {
            Some("new")
        } else if !self.plan {
            Some("plan")
        } else if !self.audit {
            Some("audit")
        } else if !self.implement {
            Some("implement")
        } else if !self.review {
            Some("review")
        } else {
            None
        }
    }
}


/// Ensure the file at `path` is writable by the owner.
///
/// On Unix: `chmod 644` if currently read-only (mask bit 0o444).
/// On other platforms: no-op (best-effort).
/// Silently ignored on any error — never blocks pipeline progress.
fn make_writable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mode = meta.permissions().mode();
            if mode & 0o444 == 0o444 && mode & 0o200 == 0 {
                let mut perms = meta.permissions();
                perms.set_mode(mode | 0o200);
                let _ = fs::set_permissions(path, perms);
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// Make the file at `path` read-only for all.
///
/// On Unix: `chmod 444`. On other platforms: no-op (best-effort).
/// Silently ignored on any error — never blocks pipeline progress.
fn make_readonly(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o444);
            let _ = fs::set_permissions(path, perms);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// Append a detailed error message to `.forceloop/error.log`.
/// Silently ignored if not in a forceloop project.
fn append_error_log(detail: &str) {
    let cwd = std::env::current_dir().ok();
    let mut dir = cwd.as_deref();
    while let Some(d) = dir {
        let candidate = d.join(FORCELOOP_DIR);
        if candidate.is_dir() {
            let log_path = candidate.join(ERROR_LOG);
            let line = format!("{} {}\n", std::process::id(), detail);
            let _ = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .and_then(|f| {
                    use std::io::Write;
                    let mut f = f;
                    f.write_all(line.as_bytes())
                });
            return;
        }
        dir = d.parent();
    }
}

/// Verify that an artifact file exists and is semantically valid.
///
/// For `.md` files, this runs the wiki-link validator to check that all
/// internal links resolve correctly. For other files, it checks existence.
///
/// This is the single entry point for gate artifact checks — ensures that
/// markdown integrity is always validated when the artifact is a `.md` file.
pub fn verify_artifact(path: &Path) -> Result<()> {
    if !path.exists() {
        append_error_log(&format!("artifact not found: {}", path.display()));
        return Err(ForceLoopError::Execution(
            "Some design files have not been generated yet. Run the current skill first.".into(),
        ));
    }
    let extension = path.extension().and_then(|e| e.to_str());
    if extension == Some("md") {
        let report = crate::utils::validate_wiki_links(path, None)?;
        if !report.missing.is_empty() {
            let broken: Vec<String> = report
                .missing
                .iter()
                .map(|(source, target)| format!("  {} → {}", source.display(), target))
                .collect();
            append_error_log(&format!(
                "broken wiki links in {}:\n{}",
                path.display(),
                broken.join("\n")
            ));
            return Err(ForceLoopError::Execution(
                "Some design files have broken links. Regenerate them with the current skill."
                    .into(),
            ));
        }
    }
    Ok(())
}

/// Count the number of wave files (markdown files excluding `index.md`)
/// in a directory. Used to cross-check against completed waves in
/// `wave_state.md`, which tracks one entry per wave file.
pub fn count_wave_files(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            let path = e.path();
            path.extension().and_then(|e| e.to_str()) == Some("md")
                && path.file_stem().and_then(|s| s.to_str()) != Some("index")
        })
        .count()
}

/// Count total `- [ ]` / `- [x]` / `- [✅]` style checklist lines across
/// all markdown files in a directory (non-recursive, skips `index.md`).
#[deprecated(
    note = "Use count_wave_files() for wave-level comparisons. count_checklist_items_in_dir counts individual items, not wave files."
)]
pub fn count_checklist_items_in_dir(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if path.file_stem().and_then(|s| s.to_str()) == Some("index") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("- [") {
                    total += 1;
                }
            }
        }
    }
    total
}

/// Count completed (`- [x]` / `- [✅]`) checklist items in a markdown file.
pub fn count_completed_items(path: &Path) -> usize {
    let Ok(content) = fs::read_to_string(path) else {
        return 0;
    };
    let mut count = 0;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("- [") && !t.starts_with("- [ ]") {
            count += 1;
        }
    }
    count
}

/// Scan a markdown file for checklist items and verify all are completed.
///
/// Accepts `- [x]`, `- [X]` and `- [✅]` (or any non-space character) as
/// completed. Rejects any `- [ ]` (unchecked) item.
pub fn verify_checklist(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let mut pending: Vec<(usize, String)> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]") {
            pending.push((i + 1, trimmed.to_string()));
        }
    }
    if !pending.is_empty() {
        let items: Vec<String> = pending
            .iter()
            .map(|(n, l)| format!("  line {}: {}", n, l))
            .collect();
        append_error_log(&format!(
            "uncompleted checklist items in {}:\n{}",
            path.display(),
            items.join("\n")
        ));
        return Err(ForceLoopError::Execution(
            "Some checklist items are not yet completed. Finish them and try again.".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_all_false() {
        let state = PipelineState::default();
        assert!(!state.new);
        assert!(!state.plan);
        assert!(!state.audit);
        assert!(!state.implement);
        assert!(!state.review);
        assert!(!state.done);
    }

    #[test]
    fn test_next_pending_sequence() {
        let mut state = PipelineState::default();
        assert_eq!(state.next_pending(), Some("new"));

        state.new = true;
        assert_eq!(state.next_pending(), Some("plan"));

        state.plan = true;
        assert_eq!(state.next_pending(), Some("audit"));

        state.audit = true;
        assert_eq!(state.next_pending(), Some("implement"));

        state.implement = true;
        assert_eq!(state.next_pending(), Some("review"));

        state.review = true;
        assert_eq!(state.next_pending(), None);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.json");

        let state = PipelineState {
            new: true,
            plan: true,
            ..Default::default()
        };
        state.write(&path).unwrap();
        assert!(path.exists());

        // After write, file should be readable
        let loaded = PipelineState::read_or_default(&path).unwrap();
        assert!(loaded.new);
        assert!(loaded.plan);
        assert!(!loaded.audit);
    }

    #[test]
    #[cfg(unix)]
    fn test_write_makes_file_readonly() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.json");

        let state = PipelineState {
            new: true,
            plan: true,
            audit: true,
            ..Default::default()
        };
        state.write(&path).unwrap();

        let meta = fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode();
        // After write, file should be read-only (0o444)
        assert_eq!(
            mode & 0o444,
            0o444,
            "file should be read-only after write"
        );
        // Should NOT have owner write bit (0o200)
        assert_eq!(
            mode & 0o200,
            0,
            "file should NOT be writable after write"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_write_is_idempotent_on_readonly() {
        // Writing twice should succeed: make_writable + write + make_readonly
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("state.json");

        let state = PipelineState {
            new: true,
            ..Default::default()
        };
        state.write(&path).unwrap();
        state.write(&path).unwrap(); // second write on read-only file

        let loaded = PipelineState::read_or_default(&path).unwrap();
        assert!(loaded.new);
    }

    #[test]
    fn test_read_or_default_initial_when_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.json");
        assert!(!path.exists());

        let state = PipelineState::read_or_default(&path).unwrap();
        assert!(!state.new);
    }

    #[test]
    fn test_verify_artifact_json_exists() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("data.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(verify_artifact(&path).is_ok());
    }

    #[test]
    fn test_verify_artifact_json_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("missing.json");
        let result = verify_artifact(&path);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("not been generated yet"));
    }

    #[test]
    fn test_verify_artifact_md_valid_links() {
        let tmp = TempDir::new().unwrap();
        let a_path = tmp.path().join("a.md");
        let b_path = tmp.path().join("b.md");
        std::fs::write(&a_path, "Link to [[b]]\n").unwrap();
        std::fs::write(&b_path, "Hello\n").unwrap();
        assert!(verify_artifact(&a_path).is_ok());
    }

    #[test]
    fn test_verify_artifact_md_broken_links() {
        let tmp = TempDir::new().unwrap();
        let a_path = tmp.path().join("a.md");
        std::fs::write(&a_path, "Link to [[missing]]\n").unwrap();
        let result = verify_artifact(&a_path);
        assert!(result.is_err());
    }

    // ---------- verify_checklist tests ----------

    #[test]
    fn test_checklist_all_completed_passes() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("report.md");
        std::fs::write(
            &p,
            "- [x] Tests written\n- [✅] Implementation done\n- [X] Reviewed\n",
        )
        .unwrap();
        assert!(verify_checklist(&p).is_ok());
    }

    #[test]
    fn test_checklist_uncompleted_fails() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("report.md");
        std::fs::write(
            &p,
            "- [x] Done\n- [ ] Not done yet\n- [✅] Also done\n",
        )
        .unwrap();
        let result = verify_checklist(&p);
        assert!(result.is_err());
    }

    #[test]
    fn test_checklist_no_items_passes() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("plain.md");
        std::fs::write(&p, "Just a regular file.\nNo checklist items.\n").unwrap();
        assert!(verify_checklist(&p).is_ok());
    }

    #[test]
    fn test_checklist_non_existent_fails() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("missing.md");
        let result = verify_checklist(&p);
        assert!(result.is_err());
    }
}