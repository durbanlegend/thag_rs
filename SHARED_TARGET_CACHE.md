# Shared Target Directory and Executable Cache

## Overview

This document describes the shared target directory and executable cache implementation in `thag_rs`, which dramatically reduces disk usage and improves build performance for multiple scripts.

## Problem Statement

Previously, `thag` created individual target directories for each script:
- Script `demo/analyze_snippet.rs` → `$TMPDIR/thag_rs/analyze_snippet/target`
- Script `demo/hello.rs` → `$TMPDIR/thag_rs/hello/target`
- And so on...

With 300+ demo scripts, this resulted in:
- **Massive disk waste**: Each script compiled dependencies separately (~500MB per script)
- **Slow builds**: Dependencies recompiled for every script
- **Estimated total**: 150GB+ for all scripts
- **Poor cleanup**: macOS `$TMPDIR` doesn't clean aggressively enough

## Solution

The new implementation uses:

1. **Shared Target Directory**: `$TMPDIR/thag_rs_shared_target`
   - All scripts share the same dependency compilation cache
   - Dependencies compiled once and reused across all scripts
   - Cargo's incremental compilation preserved

2. **Executable Cache**: `$TMPDIR/thag_rs_bins`
   - After building, only the final executable is cached
   - Each script gets one small executable file (~10MB)
   - No bulky intermediate build artifacts

## Architecture

### Directory Structure

```
$TMPDIR/
├── thag_rs_shared_target/          # Shared build cache (all scripts)
│   ├── debug/
│   │   ├── deps/                   # Shared dependencies (once!)
│   │   ├── build/                  # Build script outputs
│   │   ├── incremental/            # Incremental compilation data
│   │   ├── .fingerprint/           # Cargo fingerprints
│   │   └── <executables>           # Temporary executables
│   └── release/                    # For --executable builds
│       └── ...
├── thag_rs_bins/                   # Executable cache (per-script)
│   ├── analyze_snippet             # Just the executable
│   ├── hello_world                 # Just the executable
│   ├── test_shared_cache           # Just the executable
│   └── ...
└── thag_rs/                        # Per-script project directories
    ├── script1/                    # Cargo.toml location
    ├── script2/
    └── ...
```

### Build Flow

1. **Setup Phase**:
   - Create per-script directory for `Cargo.toml`: `$TMPDIR/thag_rs/<script_stem>/`
   - Set executable path to cache: `$TMPDIR/thag_rs_bins/<script_stem>`

2. **Build Phase**:
   - Set `CARGO_TARGET_DIR=$TMPDIR/thag_rs_shared_target`
   - Run `cargo build` (uses shared dependencies)
   - Executable built to: `$TMPDIR/thag_rs_shared_target/debug/<script_stem>`

3. **Cache Phase**:
   - Copy executable to: `$TMPDIR/thag_rs_bins/<script_stem>`
   - This is the final cached executable

4. **Run Phase**:
   - Execute from cache: `$TMPDIR/thag_rs_bins/<script_stem>`

## Benefits

### Space Savings

| Scenario | Old Approach | New Approach | Savings |
|----------|-------------|--------------|---------|
| 300 scripts | ~150GB | ~5GB | **97% reduction** |
| Per script | ~500MB | ~10MB | **98% reduction** |
| Dependencies | 300× compiled | 1× compiled | **99.7% reduction** |

### Performance Improvements

- **First build**: Same as before (dependencies must be compiled)
- **Subsequent builds**: Dramatically faster
  - Dependencies already compiled in shared cache
  - Only the script itself needs recompilation
  - Warm cache enables Cargo's incremental compilation
- **Rebuilds**: Instant if script unchanged (cached executable)

### Additional Benefits

- ✅ **Shared dependencies**: `serde`, `clap`, etc. compiled once
- ✅ **Incremental compilation**: Works across all scripts
- ✅ **Debug symbols preserved**: Full stack traces available
- ✅ **No conflicts**: Each executable has unique name
- ✅ **Easy cleanup**: Simple directory-based cache management

## Usage

### Running Scripts (No Change)

The new system is transparent to users:

```bash
# Run a script normally
thag demo/hello.rs

# First run compiles dependencies (warm cache)
thag demo/test_shared_cache.rs

# Second run is much faster (hot cache)
thag demo/test_shared_cache.rs
```

### Cleanup Commands

New `--clean` option with three modes:

```bash
# Clean everything (default)
thag --clean
thag --clean all

# Clean only cached executables (preserves dependency cache)
thag --clean bins

# Clean only shared build cache (preserves executables)
thag --clean target
```

#### When to Clean

- **`--clean bins`**: Free up space while keeping dependencies compiled
- **`--clean target`**: Force dependency recompilation (after updates)
- **`--clean all`**: Fresh start (maximum space recovery)

### Development Workflow

```bash
# Normal development - leverage the cache
thag demo/my_script.rs

# Force rebuild (e.g., after dependency updates)
thag -f demo/my_script.rs

# Or clean and rebuild
thag --clean target
thag demo/my_script.rs

# Check cache sizes
du -sh $TMPDIR/thag_rs_bins
du -sh $TMPDIR/thag_rs_shared_target
```

## Implementation Details

### Key Files Modified

1. **`src/lib.rs`**: Added constants
   - `SHARED_TARGET_SUBDIR = "thag_rs_shared_target"`
   - `EXECUTABLE_CACHE_SUBDIR = "thag_rs_bins"`

2. **`src/cmd_args.rs`**: Added cleanup functionality
   - New `--clean` option with argument
   - New `ProcFlags::CLEAN` flag
   - Updated validation logic

3. **`src/builder.rs`**: Core implementation
   - `clean_cache()`: Cache cleanup function
   - `cache_executable()`: Copy executable to cache
   - `set_up_paths()`: Updated to use cache paths
   - `create_cargo_command()`: Sets `CARGO_TARGET_DIR` env var
   - `deploy_executable()`: Updated for `--executable` builds

### Environment Variables

The build process sets:

```rust
CARGO_TARGET_DIR=$TMPDIR/thag_rs_shared_target
```

This tells Cargo to use the shared directory instead of the per-project default.

### Platform Support

Works on all platforms:
- **Linux/macOS**: Executables cached without extension
- **Windows**: Executables cached with `.exe` extension, `.pdb` for debug symbols

## Testing

### Manual Testing

1. **Test shared dependencies**:
   ```bash
   # Build multiple scripts with common dependencies
   thag demo/test_shared_cache.rs
   thag demo/another_script.rs
   
   # Verify shared target contains dependencies only once
   ls $TMPDIR/thag_rs_shared_target/debug/deps/
   ```

2. **Test executable cache**:
   ```bash
   # Build and run
   thag demo/hello.rs
   
   # Verify cached executable exists
   ls $TMPDIR/thag_rs_bins/hello
   
   # Run again (should be instant - no rebuild)
   thag demo/hello.rs
   ```

3. **Test cleanup**:
   ```bash
   # Clean bins only
   thag --clean bins
   ls $TMPDIR/thag_rs_bins/  # Should be empty
   
   # Rebuild (fast - dependencies still cached)
   thag demo/hello.rs
   
   # Clean everything
   thag --clean all
   ```

### Demo Script

Use `demo/test_shared_cache.rs` to verify functionality:

```bash
# First run: compiles serde_json and script
time thag demo/test_shared_cache.rs

# Second run: should be much faster
time thag demo/test_shared_cache.rs

# Clean and compare
thag --clean all
time thag demo/test_shared_cache.rs
```

## Troubleshooting

### Issue: Script not rebuilding after dependency update

**Solution**: Clean the shared target cache
```bash
thag --clean target
# or
thag -f demo/script.rs  # Force rebuild
```

### Issue: Cache directory growing too large

**Cause**: Many scripts compiled over time

**Solution**: Periodic cleanup
```bash
# Remove old executables
thag --clean bins

# Or clean everything
thag --clean all
```

### Issue: Different scripts with same name

**Behavior**: Scripts with identical stems share cached executable

**Impact**: Usually benign (executable overwritten by latest build)

**Solution**: Use descriptive, unique script names

## Future Enhancements

Potential improvements:

1. **Smart cache invalidation**: Track script hashes, invalidate on change
2. **Size-based cleanup**: Auto-clean when cache exceeds threshold
3. **Time-based expiry**: Remove executables not used in N days
4. **Cache statistics**: `thag --cache-info` to show sizes and usage
5. **Per-script target option**: Flag to opt out of shared cache

## References

- ChatGPT conversation that inspired this implementation
- Cargo documentation on `CARGO_TARGET_DIR`
- Rust incremental compilation design

## Migration Notes

### For Users

- **No changes required**: Existing scripts work without modification
- **Cleanup recommended**: Run `thag --clean` to clear old per-script caches
- **Performance**: Expect faster builds after first run

### For Developers

- The `target_path` in `BuildState` now points to cache location
- `CARGO_TARGET_DIR` set automatically in build commands
- Cleanup logic uses standard `fs::remove_dir_all()`

## Conclusion

The shared target and executable cache implementation provides massive space savings (97%+) and significant performance improvements while maintaining full compatibility with existing scripts. The transparent design means users benefit automatically without changing their workflow.