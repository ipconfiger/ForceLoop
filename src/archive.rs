use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::constants::{ARCHIVE_DIR, ERROR_LOG};
use crate::context::Context;
use crate::errors::{ForceLoopError, Result};
use crate::state::PipelineState;
use crate::traits::{Executable, Subcommand};

pub struct Archive;

impl Executable for Archive {
    fn execute(&self, _ctx: &Context) -> Result<()> {
        let forceloop_dir = PipelineState::locate_forceloop_dir()?;
        let report = run(&forceloop_dir)?;
        if report.count > 0 {
            println!(
                "Archived {} file(s) to {}",
                report.count,
                report.archive_path.display()
            );
        }
        Ok(())
    }
}

impl Subcommand for Archive {
    fn name(&self) -> &'static str {
        "archive"
    }
    fn description(&self) -> &'static str {
        "Archive development plan"
    }
}

/// Report from an archive run.
pub struct ArchiveReport {
    pub count: usize,
    pub archive_path: PathBuf,
}

/// Core archive logic.
///
/// Scans `.forceloop/` for:
///   - `specs/` directory (recursive, all files)
///   - `plans/` directory (recursive, all files)
///   - Root-level `.json` and `.md` files (excluding `error.log`)
///
/// Packs everything into `archive/<timestamp>.tar.gz` inside the
/// forceloop dir, then removes the originals.
pub fn run(forceloop_dir: &Path) -> Result<ArchiveReport> {
    // 1. Build file list (relative to forceloop_dir).
    let mut all_files: Vec<PathBuf> = Vec::new();
    let mut root_files: Vec<PathBuf> = Vec::new(); // root .json/.md for targeted deletion

    // specs/ — recursive, all file types.
    let specs_dir = forceloop_dir.join("specs");
    let has_specs = specs_dir.is_dir();
    if has_specs {
        collect_files_recursive(&specs_dir, forceloop_dir, &mut all_files)?;
    }

    // plans/ — recursive, all file types.
    let plans_dir = forceloop_dir.join("plans");
    let has_plans = plans_dir.is_dir();
    if has_plans {
        collect_files_recursive(&plans_dir, forceloop_dir, &mut all_files)?;
    }

    // Root-level .json and .md files (non-recursive).
    if let Ok(entries) = fs::read_dir(forceloop_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            // Exclude files that should survive archiving.
            if file_name == ERROR_LOG {
                continue;
            }

            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e,
                None => continue,
            };
            if (ext == "json" || ext == "md")
                && let Ok(rel) = path.strip_prefix(forceloop_dir)
            {
                all_files.push(rel.to_path_buf());
                root_files.push(rel.to_path_buf());
            }
        }
    }

    // 3. Nothing to archive?
    if all_files.is_empty() && !has_specs && !has_plans {
        println!("Nothing to archive.");
        return Ok(ArchiveReport {
            count: 0,
            archive_path: PathBuf::new(),
        });
    }

    // 4. Create archive directory.
    let archive_dir = forceloop_dir.join(ARCHIVE_DIR);
    fs::create_dir_all(&archive_dir)?;

    // 5. Generate timestamp.
    let timestamp = unix_timestamp_str();
    let archive_path = archive_dir.join(format!("archive_{timestamp}.tar.gz"));

    // 6. Write tar.gz.
    {
        let tar_file = File::create(&archive_path)?;
        let encoder =
            flate2::write::GzEncoder::new(tar_file, flate2::Compression::best());
        let mut tar = tar::Builder::new(encoder);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for rel_path in &all_files {
            let full_path = forceloop_dir.join(rel_path);
            if !full_path.is_file() {
                continue;
            }
            let content = fs::read(&full_path)?;
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(now);
            // Use forward-slash normalized path for the archive entry.
            let entry_path = rel_path
                .to_str()
                .unwrap_or("")
                .replace(std::path::MAIN_SEPARATOR, "/");
            tar.append_data(&mut header, &entry_path, &content[..])?;
        }

        let encoder = tar.into_inner()?;
        encoder.finish()?;
    }

    // 7. Delete originals.
    if has_specs {
        fs::remove_dir_all(&specs_dir)?;
    }
    if has_plans {
        fs::remove_dir_all(&plans_dir)?;
    }
    for rel_path in &root_files {
        let full_path = forceloop_dir.join(rel_path);
        if full_path.is_file() {
            let _ = fs::remove_file(&full_path);
        }
    }

    let total = all_files.len();
    Ok(ArchiveReport {
        count: total,
        archive_path,
    })
}

/// Recursively collect all files under `dir`, storing paths relative to `root`.
fn collect_files_recursive(
    dir: &Path,
    root: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(ForceLoopError::Io)? {
        let entry = entry.map_err(ForceLoopError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, root, files)?;
        } else if path.is_file()
            && let Ok(rel) = path.strip_prefix(root)
        {
            files.push(rel.to_path_buf());
        }
    }
    Ok(())
}

/// Return the current Unix timestamp as a decimal string.
fn unix_timestamp_str() -> String {
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    dur.as_secs().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_forceloop(tmp: &TempDir) -> PathBuf {
        let forceloop = tmp.path().join(".forceloop");
        fs::create_dir_all(forceloop.join("archive")).unwrap();
        fs::create_dir_all(forceloop.join("specs")).unwrap();
        fs::create_dir_all(forceloop.join("plans")).unwrap();

        fs::write(
            forceloop.join("state.json"),
            r#"{"new":true,"plan":true,"done":false}"#,
        )
        .unwrap();
        fs::write(forceloop.join("wave_state.md"), "- [x] wave-1\n- [x] wave-2\n")
            .unwrap();
        fs::write(forceloop.join("review_result.md"), "# Review\n\n- [x] done\n")
            .unwrap();
        fs::write(forceloop.join("error.log"), "some error\n").unwrap();
        fs::write(forceloop.join("specs").join("index.md"), "# Specs\n").unwrap();
        fs::write(forceloop.join("plans").join("wave-1.md"), "# Wave 1\n").unwrap();
        forceloop
    }

    #[test]
    fn archive_creates_gzip_file() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let report = run(&forceloop).unwrap();

        assert!(
            report.archive_path.exists(),
            "archive file should exist: {}",
            report.archive_path.display()
        );
        assert!(
            report.archive_path.to_string_lossy().contains("archive_"),
            "archive filename should contain archive_"
        );
        assert!(
            report.archive_path
                .extension()
                .and_then(|e| e.to_str())
                == Some("tar.gz")
                || report
                    .archive_path
                    .extension()
                    .and_then(|e| e.to_str())
                    == Some("gz"),
            "archive file should end with .tar.gz"
        );
    }

    #[test]
    fn archive_removes_specs_and_plans() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let _ = run(&forceloop).unwrap();

        assert!(
            !forceloop.join("specs").exists(),
            "specs/ should be removed"
        );
        assert!(
            !forceloop.join("plans").exists(),
            "plans/ should be removed"
        );
    }

    #[test]
    fn archive_deletes_state_json() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let _ = run(&forceloop).unwrap();

        assert!(
            !forceloop.join("state.json").exists(),
            "state.json should be deleted (it is a .json file)"
        );
    }

    #[test]
    fn archive_preserves_error_log() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let _ = run(&forceloop).unwrap();

        assert!(
            forceloop.join("error.log").exists(),
            "error.log should be preserved"
        );
    }

    #[test]
    fn archive_removes_root_md_and_json() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let _ = run(&forceloop).unwrap();

        assert!(
            !forceloop.join("wave_state.md").exists(),
            "wave_state.md should be deleted"
        );
        assert!(
            !forceloop.join("review_result.md").exists(),
            "review_result.md should be deleted"
        );
    }

    #[test]
    fn archive_nothing_to_archive() {
        let tmp = TempDir::new().unwrap();
        let forceloop = tmp.path().join(".forceloop");
        fs::create_dir_all(forceloop.join("archive")).unwrap();
        // Nothing except error.log and empty archive dir.
        fs::write(forceloop.join("error.log"), "").unwrap();

        // Only error.log exists (excluded) — nothing to archive.
        let report = run(&forceloop).unwrap();

        assert_eq!(report.count, 0);
        // Archive file should NOT be created.
        let archive_dir = forceloop.join(ARCHIVE_DIR);
        let entry_count = fs::read_dir(&archive_dir)
            .map(|d| d.flatten().count())
            .unwrap_or(0);
        assert_eq!(entry_count, 0, "archive dir should be empty");
    }

    #[test]
    fn archive_all_files_in_tar_have_correct_names() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let report = run(&forceloop).unwrap();

        // Read back the tar and list entry names.
        let file = File::open(&report.archive_path).unwrap();
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        let entries: Vec<String> = archive
            .entries()
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        assert!(entries.contains(&"specs/index.md".to_string()));
        assert!(entries.contains(&"plans/wave-1.md".to_string()));
        assert!(entries.contains(&"wave_state.md".to_string()));
        assert!(entries.contains(&"review_result.md".to_string()));
        // state.json is a .json file, included.
        assert!(entries.contains(&"state.json".to_string()));
        // error.log should be excluded.
        assert!(!entries.contains(&"error.log".to_string()));
    }

    #[test]
    fn archive_idempotent_second_run() {
        let tmp = TempDir::new().unwrap();
        let forceloop = setup_forceloop(&tmp);

        let r1 = run(&forceloop).unwrap();
        assert!(r1.count > 0, "first run should archive something");

        // Second run: nothing left to archive.
        let r2 = run(&forceloop).unwrap();
        assert_eq!(r2.count, 0, "second run should have nothing to archive");
    }

    #[test]
    fn archive_collects_specs_files_recursively() {
        let tmp = TempDir::new().unwrap();
        let forceloop = tmp.path().join(".forceloop");
        fs::create_dir_all(forceloop.join("archive")).unwrap();
        fs::create_dir_all(forceloop.join("specs").join("sub")).unwrap();
        fs::create_dir_all(forceloop.join("plans")).unwrap();
        fs::write(
            forceloop.join("state.json"),
            r#"{"new":true,"done":true}"#,
        )
        .unwrap();
        fs::write(forceloop.join("specs").join("a.md"), "a").unwrap();
        fs::write(forceloop.join("specs").join("sub").join("b.md"), "b").unwrap();
        fs::write(forceloop.join("specs").join("data.json"), "{}").unwrap();
        fs::write(forceloop.join("specs").join("image.png"), "fake").unwrap();

        let report = run(&forceloop).unwrap();

        let file = File::open(&report.archive_path).unwrap();
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        let entries: Vec<String> = archive
            .entries()
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        // All file types from specs/ are included (not just .md/.json).
        assert!(entries.contains(&"specs/a.md".to_string()));
        assert!(entries.contains(&"specs/sub/b.md".to_string()));
        assert!(entries.contains(&"specs/data.json".to_string()));
        assert!(entries.contains(&"specs/image.png".to_string()));
        // state.json included because pipeline is done.
        assert!(entries.contains(&"state.json".to_string()));
    }
}