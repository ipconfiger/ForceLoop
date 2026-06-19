# Plan: Audit subcommand + checklist gate

## Changes

### 1. `src/constants.rs`
```rust
pub const AUDIT_FILE: &str = "audit.md";
```

### 2. `src/traits.rs`
```rust
fn check_list(&self) -> bool { false }
```

### 3. `src/state.rs` — `verify_checklist()`
Scan markdown for `- [ ]` lines, require all to be `- [x]` or `- [✅]`.

### 4. `src/commands/audit.rs`
- `execute()`: check state.new && state.plan, block if not
- `check_list() -> true`
- `artifacts() -> &[".forceloop/audit.md"]`
- `gate()`: verify_artifact + verify_checklist
- Subcommand impl (already exists)
- Prompts: guide LLM to write `.forceloop/audit.md`

### 5. `src/cli.rs` + `src/main.rs` + `tests/cli_help.rs`
Already have Audit registered from previous change — just verify they're there.