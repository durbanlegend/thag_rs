# REPL to Rapid Iteration Mode Rename

## Summary

In version 0.2.x, thag's interactive mode has been renamed from "REPL mode" to "rapid iteration mode" to more accurately describe its behavior.

## Why the Change?

A traditional REPL (Read-Eval-Print Loop) preserves state line-by-line - you define a variable on one line, and it remains available on subsequent lines. Examples include `evcxr` and `irust`.

Thag's `-r` mode works differently: it allows you to edit a complete Rust file, then recompiles and runs the **entire program** from scratch. There's no line-by-line state preservation. This is better described as "rapid iteration" - a fast feedback loop for developing and testing complete Rust programs.

## What Changed

### Command-Line Interface
- **Primary flag**: `--repl` → `--rapid` (short form `-r` unchanged)
- **Backward compatibility**: `--repl` still works as a hidden alias
- **New visible alias**: `--iter` also available

### Documentation
- All user-facing documentation updated to use "rapid iteration mode"
- Feature descriptions clarified to distinguish from traditional REPLs

### Internal Changes
- Script filename: `repl_script.rs` → `iter_script.rs`
- Constant: `REPL_SCRIPT_NAME` → `ITER_SCRIPT_NAME`
- Subdirectory name: `rs_repl` (unchanged for now to avoid breaking existing workflows)
- Cargo feature name: `repl` → `iter` (with `repl` as a deprecated alias for backward compatibility)

## Backward Compatibility

✅ **Existing scripts and workflows continue to work:**
- `thag --repl` still works (hidden alias)
- `thag -r` unchanged
- Cargo feature `repl` works as an alias for `iter` (deprecated but functional)
- The `repl` field in `Cli` struct unchanged (internal implementation detail)

## Migration Guide

### For Command-Line Users
No action required! Both work:
```bash
# Old (still works)
thag --repl
thag -r

# New (recommended)
thag --rapid
thag -r

# Also available
thag --iter
```

### For Library Users
The Cargo feature has been renamed, but the old name still works:
```toml
[dependencies]
# New (recommended)
thag_rs = { version = "0.2", features = ["iter"] }

# Old (still works but deprecated)
thag_rs = { version = "0.2", features = ["repl"] }
```

The `repl` feature is now an alias for `iter` and will be removed in a future major version.

### For Documentation/Tutorial Writers
Please update references from "REPL mode" to "rapid iteration mode" when describing thag's `-r` option.

## Related Projects

For comparison, these tools are **true REPLs** with line-by-line state preservation:
- `evcxr` - Full-featured Rust REPL
- `irust` - Lightweight Rust REPL

Thag's rapid iteration mode serves a different use case: quickly developing and testing complete Rust programs with instant feedback.