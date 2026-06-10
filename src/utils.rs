use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::{ForceLoopError, Result};

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

// ============================================================================
// WIKI Link Validator
// ============================================================================

/// Report from wiki link validation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WikiLinkReport {
    /// All files successfully visited (canonical paths, sorted, no duplicates).
    pub visited: Vec<PathBuf>,

    /// Broken links: (source_file_canonical_path, link_target_text).
    pub missing: Vec<(PathBuf, String)>,

    /// Number of times a recursion was prevented by the visited set
    /// (includes both true cycles and duplicate references).
    pub cycles_prevented: u32,
}

/// Recursively validates all wiki links in markdown files starting from `start`.
///
/// Supports:
/// - Obsidian-style: `[[Page]]`, `[[Page.md]]`, `[[path/Page]]`, `[[Page|alias]]`, `[[Page#heading]]`
/// - Standard markdown: `[text](file.md)` or `[text](path/file.md)`
///
/// Link resolution order:
/// 1. Relative to source file's directory
/// 2. Relative to `project_root` (if provided)
/// 3. If neither exists, recorded in `report.missing`
///
/// Cycle prevention: uses a `HashSet<PathBuf>` of canonical paths to avoid
/// re-validating already-visited files.
pub fn validate_wiki_links(
    start: &Path,
    project_root: Option<&Path>,
) -> Result<WikiLinkReport> {
    if !start.exists() {
        return Err(ForceLoopError::Execution(format!(
            "start file does not exist: {}",
            start.display()
        )));
    }
    if !start.is_file() {
        return Err(ForceLoopError::Execution(format!(
            "start is not a file: {}",
            start.display()
        )));
    }

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut report = WikiLinkReport::default();

    validate_recursive(start, project_root, &mut visited, &mut report)?;

    let mut visited_vec: Vec<PathBuf> = visited.into_iter().collect();
    visited_vec.sort();
    report.visited = visited_vec;
    Ok(report)
}

fn validate_recursive(
    current: &Path,
    project_root: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
    report: &mut WikiLinkReport,
) -> Result<()> {
    let canonical = current.canonicalize().map_err(|e| {
        ForceLoopError::Execution(format!(
            "failed to canonicalize {}: {}",
            current.display(),
            e
        ))
    })?;

    if !visited.insert(canonical.clone()) {
        report.cycles_prevented += 1;
        return Ok(());
    }

    let content = fs::read_to_string(current).map_err(|e| {
        ForceLoopError::Execution(format!(
            "failed to read {}: {}",
            current.display(),
            e
        ))
    })?;

    let targets = extract_link_targets(&content);

    for target in targets {
        match resolve_link(current, &target, project_root) {
            Some(resolved) => {
                if !resolved.exists() {
                    report.missing.push((canonical.clone(), target));
                } else {
                    validate_recursive(&resolved, project_root, visited, report)?;
                }
            }
            None => {
                report.missing.push((canonical.clone(), target));
            }
        }
    }

    Ok(())
}

fn extract_link_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Obsidian wiki link: [[...]]
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '['
            && let Some(end) = find_subseq(&chars[i + 2..], &[']', ']'])
        {
            let raw: String = chars[i + 2..i + 2 + end].iter().collect();
            let target = parse_wiki_target(&raw);
            if !target.is_empty() {
                targets.push(target);
            }
            i = i + 2 + end + 2;
            continue;
        }

        // Standard markdown link: [text](url)
        if chars[i] == '['
            && let Some(close_bracket) = find_char(&chars[i + 1..], ']')
        {
            let after_text = i + 1 + close_bracket + 1;
            if after_text < chars.len() && chars[after_text] == '('
                && let Some(close_paren) = find_char(&chars[after_text + 1..], ')')
            {
                let url_start = after_text + 1;
                let url_end = after_text + 1 + close_paren;
                let url: String = chars[url_start..url_end].iter().collect();
                if let Some(cleaned) = clean_md_url(&url) {
                    targets.push(cleaned);
                }
                i = url_end + 1;
                continue;
            }
        }

        i += 1;
    }

    targets
}

fn find_subseq(haystack: &[char], needle: &[char; 2]) -> Option<usize> {
    haystack.windows(2).position(|w| w[0] == needle[0] && w[1] == needle[1])
}

fn find_char(haystack: &[char], target: char) -> Option<usize> {
    haystack.iter().position(|&c| c == target)
}

fn parse_wiki_target(raw: &str) -> String {
    let raw = raw.trim();
    // Strip alias: "Page|alias" -> "Page"
    let target = raw.split('|').next().unwrap_or(raw);
    // Strip heading: "Page#heading" -> "Page"
    let target = target.split('#').next().unwrap_or(target);
    target.trim().to_string()
}

fn clean_md_url(url: &str) -> Option<String> {
    let url = url.trim();
    // Skip external URLs
    if url.contains("://") {
        return None;
    }
    // Skip pure anchors
    if url.starts_with('#') {
        return None;
    }
    // Strip heading anchor: "file.md#heading" -> "file.md"
    let url = url.split('#').next().unwrap_or(url);
    // Only process .md files
    if url.ends_with(".md") {
        Some(url.to_string())
    } else {
        None
    }
}

fn resolve_link(
    source: &Path,
    target: &str,
    project_root: Option<&Path>,
) -> Option<PathBuf> {
    let target = target.trim();
    if target.is_empty() {
        return None;
    }
    // Strip any heading (defensive — should already be stripped)
    let target = target.split('#').next().unwrap_or(target);

    let source_dir = source.parent().unwrap_or_else(|| Path::new("."));

    // Build candidate list: try exact target, then with .md appended
    // (wiki link convention allows omitting extension)
    let source_candidates: [PathBuf; 2] = [
        source_dir.join(target),
        source_dir.join(format!("{}.md", target)),
    ];
    for candidate in &source_candidates {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    if let Some(root) = project_root {
        let root_candidates: [PathBuf; 2] = [
            root.join(target),
            root.join(format!("{}.md", target)),
        ];
        for candidate in &root_candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }
    }

    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_md(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_single_file_no_links() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "Just some text, no links.\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 1);
        assert!(report.missing.is_empty());
        assert_eq!(report.cycles_prevented, 0);
    }

    #[test]
    fn test_one_valid_link() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "Link: [[b]]\n");
        create_md(tmp.path(), "b.md", "Hello\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
        assert_eq!(report.cycles_prevented, 0);
    }

    #[test]
    fn test_broken_link() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "Link: [[missing]]\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 1);
        assert_eq!(report.missing.len(), 1);
        assert_eq!(report.missing[0].1, "missing");
    }

    #[test]
    fn test_cycle_detection() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "Link: [[b]]\n");
        let _b = create_md(tmp.path(), "b.md", "Link: [[a]]\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
        assert_eq!(report.cycles_prevented, 1);
    }

    #[test]
    fn test_standard_markdown_link() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "Click [here](b.md) please.\n");
        create_md(tmp.path(), "b.md", "Hello\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
    }

    #[test]
    fn test_alias_and_heading() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(
            tmp.path(),
            "a.md",
            "See [[Page|Display Text]] and [[Other#section]]\n",
        );
        create_md(tmp.path(), "Page.md", "P\n");
        create_md(tmp.path(), "Other.md", "O\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 3);
        assert!(report.missing.is_empty());
    }

    #[test]
    fn test_relative_resolution() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let a = create_md(&sub, "a.md", "Go to [[../c.md]]\n");
        create_md(tmp.path(), "c.md", "C\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
    }

    #[test]
    fn test_project_root_fallback() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let a = create_md(&sub, "a.md", "Root file: [[rootfile.md]]\n");
        create_md(tmp.path(), "rootfile.md", "R\n");
        let report = validate_wiki_links(&a, Some(tmp.path())).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
    }

    #[test]
    fn test_deduplication() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "[[b]] [[b]] [[b]]\n");
        create_md(tmp.path(), "b.md", "B\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
        // 3 references to b, 1st adds to visited, 2nd and 3rd increment cycles_prevented
        assert_eq!(report.cycles_prevented, 2);
    }

    #[test]
    fn test_nonexistent_start_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.md");
        let result = validate_wiki_links(&path, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_visited_is_sorted() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(tmp.path(), "a.md", "[[z]]\n");
        create_md(tmp.path(), "z.md", "[[m]]\n");
        create_md(tmp.path(), "m.md", "leaf\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 3);
        // Verify visited is sorted
        let mut sorted = report.visited.clone();
        sorted.sort();
        assert_eq!(report.visited, sorted);
    }

    #[test]
    fn test_external_urls_skipped() {
        let tmp = TempDir::new().unwrap();
        let a = create_md(
            tmp.path(),
            "a.md",
            "External: [google](https://google.com) and [local](b.md)\n",
        );
        create_md(tmp.path(), "b.md", "B\n");
        let report = validate_wiki_links(&a, None).unwrap();
        assert_eq!(report.visited.len(), 2);
        assert!(report.missing.is_empty());
    }
}
