# Plan: Implement Archive Subcommand

## Requirements Summary

Implement `fl archive` — a terminal subcommand (like `fl status`, `fl gate`) that archives and then purges completed project data from `.forceloop/`.

**What it does:**
1. Locate the `.forceloop/` directory
2. Gather all files to archive:
   - `specs/` directory (recursive, all files)
   - `plans/` directory (recursive, all files)
   - All `.json` files anywhere under `.forceloop/`
   - All `.md` files anywhere under `.forceloop/`
3. Pack them into `archive_YYYYMMDD_HHMMSS.tar.gz` inside `.forceloop/archive/`
4. Delete the originals: `specs/`, `plans/`, all `.json` and `.md` files
5. Keep `state.json` (needed for pipeline state) — **exception**: if pipeline is done, include it too
6. Keep `error.log` (debugging history)
7. Keep the archive file itself, plus the `archive/` directory structure

## Acceptance Criteria

- [ ] `fl archive` creates `.forceloop/archive/archive_YYYYMMDD_HHMMSS.tar.gz` with correct contents
- [ ] `specs/` and `plans/` directories are removed after archiving
- [ ] All `.json` and `.md` files in `.forceloop/` are moved to archive then deleted
- [ ] `state.json` is **NOT** deleted (unless pipeline state is done)
- [ ] `error.log` is **NOT** deleted
- [ ] The `archive/` directory itself and its contents are not archived
- [ ] Running `fl archive` when already archived is idempotent (no-op or clear message)
- [ ] Running `fl archive` when there's nothing to archive prints a message and exits cleanly
- [ ] All existing tests pass
- [ ] `cargo clippy --all-targets` — zero warnings

## Design Decisions

### Compression: `flate2` + `tar` crates

Building a gzip archive from multiple files requires:
- **tar** — bundles multiple files into a single byte stream with headers
- **gzip (deflate)** — compresses that stream

Hand-rolling either is impractical. The gzip format requires a deflate implementation (non-trivial). The tar format is simpler but error-prone. Adding two well-maintained crates (`flate2` + `tar`) is the right tradeoff.

New `Cargo.toml` entries:
```toml
flate2 = "1"
tar = "0.4"
```

Both are pure Rust, no C dependencies, widely used, well-maintained.

### Architecture: Relative paths in tar

Files inside the tar use paths relative to `.forceloop/`:
```
specs/index.md
plans/wave-1-core.md
state.json
wave_state.md
review_result.md
```

This keeps the archive self-contained and navigable.

### Exclusion list

| Keep | Delete |
|------|--------|
| `archive/` (the archive dir itself) | `specs/` (entire dir) |
| `state.json` (pipeline state) | `plans/` (entire dir) |
| `error.log` (debug history) | All other `.json` files |
| `archive_{datetime}.tar.gz` | All other `.md` files |

### Edge: Nothing to archive

If `specs/` doesn't exist, `plans/` doesn't exist, AND there are no orphan `.json`/`.md` files → print clear message, exit 0.

### Edge: Done state

If pipeline `state.done == true`, archive `state.json` too. This allows a clean slate for a new cycle.

## Implementation Steps

### Step 1: Add dependencies

Edit `Cargo.toml` to add:
```toml
flate2 = "1"
tar = "0.4"
```

### Step 2: Add `ARCHIVE_DIR` constant if not already present

`src/constants.rs` already has `ARCHIVE_DIR = "archive"` — verify it's there, reuse it.

### Step 3: Implement `archive.rs`

The `execute()` method does:

```
1. Locate .forceloop/ dir (PipelineState::locate_forceloop_dir)
2. Check if anything to archive
   - If specs/ exists AND plans/ exists → proceed
   - Else if any orphan .json/.md files exist → proceed
   - Else → print "Nothing to archive", return Ok
3. Read pipeline state → check if done
4. Create archive/ directory if not exists
5. Generate timestamp string: YYYYMMDD_HHMMSS
6. Build file list for tar:
   - Recursively list specs/ files (if exists)
   - Recursively list plans/ files (if exists)
   - List all .json/.md files in .forceloop/ (non-recursive, excluding state.json, error.log, archive/)
   - If done: include state.json
7. Create tar.gz:
   - Create File at archive/archive_{timestamp}.tar.gz
   - Wrap in GzEncoder (flate2)
   - Wrap in tar::Builder
   - For each file in file list:
     - Read file content
     - Create tar Header
     - Append with archive path (relative to .forceloop/)
   - Finish tar
8. Delete archived originals:
   - Remove spec/ dir (fs::remove_dir_all)
   - Remove plans/ dir (fs::remove_dir_all)
   - Delete each .json/.md file from the file list
   - If done: delete state.json (already archived)
9. Print summary: how many files archived, archive path
```

### Step 4: Update `PipelineState::locate_forceloop_dir`

Already exists — no change needed.

### Step 5: Update tests

No existing tests for archive (it was `todo!()`). Add unit tests:

| Test | What it verifies |
|------|-----------------|
| `archive_creates_gzip_file` | Full integration: creates tmpdir, writes fake files, runs archive, verifies .tar.gz exists |
| `archive_preserves_state_json` | state.json not deleted unless pipeline is done |
| `archive_removes_specs_and_plans` | specs/ and plans/ deleted after archive |
| `archive_nothing_to_archive` | Clean dir → message, no error |
| `archive_idempotent` | Running twice → second run detects nothing to archive |
| `archive_includes_all_file_types` | .json + .md files included, other files (like fl.ts, error.log) excluded |

### Step 6: No other source changes needed

- `src/lib.rs` — module `pub mod archive;` already exists
- `src/cli.rs` — `Commands::Archive` already exists
- `src/main.rs` — dispatch already wired
- `src/traits.rs` — Archive already implements `Subcommand`

## File Change Summary

| File | Change |
|------|--------|
| `Cargo.toml` | Add `flate2 = "1"`, `tar = "0.4"` |
| `src/archive.rs` | Full `execute()` implementation |
| `src/constants.rs` | Verify `ARCHIVE_DIR` exists (should already) |

## Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Large archive blocks CLI | Low | Sync I/O is fine for typical project sizes (<10MB) |
| Accidental deletion of state.json | Low | Explicit exclusion check before deletion |
| Partial failure (deleted from tar but disk file remains) | Low | Tar is built in memory before any deletion — if tar fails, nothing is deleted |
| Pre-existing archive files get re-archived | Low | archive/ directory is explicitly excluded from the file scan |
| Compression OOM on huge projects | Very low | Forceloop projects are code repos, not media; typical size <5MB |

## Verification Steps

1. `cargo test` — all tests pass (including new archive tests)
2. `cargo clippy --all-targets` — zero warnings
3. Manual: create test project, run pipeline to done, run `fl archive`, verify:
   - `.forceloop/archive/archive_*.tar.gz` exists
   - `specs/` and `plans/` gone
   - `.json`/`.md` files gone
   - `state.json` still there (unless done)
   - `error.log` still there
   - `archive/` still there
4. Verify tar.gz contents with `tar -tzf archive_*.tar.gz`