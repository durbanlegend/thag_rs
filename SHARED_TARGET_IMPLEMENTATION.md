# Shared Target Implementation

## Overview

The `thag_rs` script runner implements a **shared build target architecture** that dramatically reduces disk usage when working with multiple Rust scripts. Instead of creating separate `target` directories for each script (which would accumulate gigabytes of redundant build artifacts), all scripts share a single build cache while maintaining individual cached executables.

## Problem Solved

Previously, running 300+ demo scripts would create 300+ individual `target` directories, each containing:
- Compiled dependencies (duplicated across scripts)
- Build artifacts and intermediate files
- Incremental compilation data

This resulted in **6GB+ of disk usage** with massive redundancy, as many scripts use the same dependencies (e.g., `serde`, `clap`, `regex`, etc.).

## Architecture

The new implementation uses three distinct directory structures:

### 1. Per-Script Manifest Directories
**Location:** `$TMPDIR/thag_rs/<script_name>/`

**Contents:**
- `Cargo.toml` - Generated manifest with dependencies
- `<script_name>.rs` - Generated or wrapped source code
- `Cargo.lock` - Dependency lock file

**Size:** ~10-20KB per script (just text files)

**Purpose:** Provides Cargo with the project structure it needs, but **no target directory**.

### 2. Shared Build Target
**Location:** `$TMPDIR/thag_rs_shared_target/`

**Contents:**
- `debug/` - Debug build artifacts
- `release/` - Release build artifacts (for `--executable` builds)
- `deps/` - **Shared compiled dependencies** (compiled once, reused by all scripts)
- `build/` - Build script outputs
- `incremental/` - Incremental compilation data
- `.fingerprint/` - Cargo's freshness tracking

**Size:** ~2-50MB (depends on number of unique dependencies used)

**Purpose:** Central build cache shared across all scripts. Dependencies are compiled once and reused.

### 3. Executable Cache
**Location:** `$TMPDIR/thag_rs_bins/`

**Contents:**
- Individual executable files, one per script
- Named after the script stem (e.g., `hello`, `fib_basic_ibig`)

**Size:** ~400KB-1MB per executable

**Purpose:** Fast execution without rebuilding. Each script's final executable is cached here.

## How It Works

### Build Process

1. **Generate Phase:**
   - Creates per-script directory: `$TMPDIR/thag_rs/<script_name>/`
   - Writes `Cargo.toml` with inferred/specified dependencies
   - Writes wrapped source code if needed

2. **Build Phase:**
   - Sets `CARGO_TARGET_DIR` environment variable to `$TMPDIR/thag_rs_shared_target/`
   - Runs `cargo build --manifest-path=$TMPDIR/thag_rs/<script_name>/Cargo.toml`
   - Cargo builds to shared target, reusing previously compiled dependencies
   - Executable appears in `$TMPDIR/thag_rs_shared_target/debug/<script_name>`

3. **Cache Phase:**
   - Copies executable from shared target to cache: `$TMPDIR/thag_rs_bins/<script_name>`
   - Updates target_path in BuildState to point to cached executable

4. **Run Phase:**
   - Executes cached executable from `$TMPDIR/thag_rs_bins/<script_name>`
   - Skips generation and build if executable is newer than source

### Freshness Checking

The system checks if the cached executable is up-to-date by comparing timestamps:
- Source file: `demo/<script_name>.rs`
- Manifest: `$TMPDIR/thag_rs/<script_name>/Cargo.toml`
- Cached executable: `$TMPDIR/thag_rs_bins/<script_name>`

If the executable is newer than both source and manifest, generation and build are skipped.

## Disk Usage Comparison

### Old Architecture (Per-Script Targets)
```
300 scripts × ~20MB target = 6GB+ total
└── High redundancy: same dependencies compiled 300 times
```

### New Architecture (Shared Target)
```
Manifests:       300 × 15KB  = 4.5MB
Shared target:   1 × 40MB    = 40MB
Executables:     300 × 600KB = 180MB
─────────────────────────────────────
Total:                         ~225MB  (97% reduction!)
```

## Performance Benefits

### First Build (Cold Cache)
- Same as before: all dependencies must be compiled
- Example: `demo/fib_basic_ibig.rs` with `ibig` dependency: ~38s

### Second Build (Warm Cache)
- **Shared dependencies are reused**: only the script itself needs compilation
- Example: `demo/fib_doubling_iterative_purge_ibig.rs` (also uses `ibig`): ~3s
- **13x faster** due to shared `ibig` compilation

### Repeated Runs (Cached Executable)
- **Near-instant**: skips generation and build entirely
- Uses cached executable directly
- Same performance as running a native binary

## Cache Management

### Viewing Cache Status

```bash
# Check disk usage
du -sh $TMPDIR/thag_rs_shared_target  # Shared build artifacts
du -sh $TMPDIR/thag_rs_bins            # Cached executables
du -sh $TMPDIR/thag_rs                 # Per-script manifests
```

### Cleaning Caches

The `--clean` option supports three modes:

#### Clean Executables Only
```bash
thag --clean bins
```
- Removes: `$TMPDIR/thag_rs_bins/`
- Keeps: Shared target (build cache)
- Next run: Rebuild from source, but reuse compiled dependencies

#### Clean Shared Target Only
```bash
thag --clean target
```
- Removes: `$TMPDIR/thag_rs_shared_target/`
- Keeps: Cached executables
- Next run: Full rebuild of dependencies, executables recached

#### Clean Everything
```bash
thag --clean all
thag --clean        # 'all' is the default
```
- Removes: Both shared target and executable cache
- Keeps: Per-script manifests (regenerated as needed)
- Next run: Complete rebuild from scratch

### When to Clean

**Clean executables (`bins`):**
- Free up space quickly (~few hundred MB)
- When you want to force a rebuild but keep dependency cache

**Clean target (`target`):**
- After updating Rust toolchain
- When dependencies seem corrupted
- When you want a completely fresh build

**Clean all:**
- Major cleanup (reclaims hundreds of MB)
- After significant changes to many scripts
- When disk space is critically low

## Implementation Details

### Key Files

**`thag_rs/src/builder.rs`:**
- `set_up_paths()`: Configures paths, sets `target_path` to executable cache
- `create_cargo_command()`: Sets `CARGO_TARGET_DIR` environment variable
- `cache_executable()`: Copies built executable to cache
- `clean_cache()`: Implements cache cleanup

**`thag_rs/src/lib.rs`:**
- `SHARED_TARGET_SUBDIR`: Constant for shared target directory name
- `EXECUTABLE_CACHE_SUBDIR`: Constant for executable cache directory name

**`thag_rs/src/code_utils.rs`:**
- `modified_since_compiled()`: Checks if cached executable is fresh

### Environment Variables

**`CARGO_TARGET_DIR`:**
- Set automatically by `create_cargo_command()`
- Points to: `$TMPDIR/thag_rs_shared_target/`
- Tells Cargo where to place build artifacts
- Enables dependency sharing across all scripts

### Platform Considerations

**Windows:**
- Executables have `.exe` extension
- Paths adjusted automatically in `set_up_paths()` and `cache_executable()`

**macOS/Linux:**
- No executable extension
- Standard Unix paths

## Migration Notes

### For Existing Users

If you have the old per-script structure:

1. **The new system is already active** - new builds use shared target
2. **Old data remains** in `$TMPDIR/thag_rs/*/target/` until manually cleaned
3. **To reclaim space:**
   ```bash
   # Check current usage
   du -sh $TMPDIR/thag_rs
   
   # Remove old structure (safe - will be recreated without target dirs)
   rm -rf $TMPDIR/thag_rs
   
   # Or use the built-in clean command
   thag --clean all
   ```

### Backward Compatibility

The implementation is fully backward compatible:
- Old per-script directories are ignored if they exist
- New builds use shared target automatically
- No changes needed to existing scripts
- All command-line options work as before

## Troubleshooting

### "Build failed" after cleaning

**Problem:** Cleaned caches, now builds fail

**Solution:** This is usually a transient issue. Try:
```bash
thag --clean target  # Clean shared target
thag -f <script>     # Force rebuild
```

### Disk space still high

**Problem:** `$TMPDIR/thag_rs` still large after cleaning

**Solution:** Old target directories may remain:
```bash
find $TMPDIR/thag_rs -name "target" -type d -exec du -sh {} \;
rm -rf $TMPDIR/thag_rs  # Remove old structure
```

### Executable not found

**Problem:** "Built executable not found at expected location"

**Solution:** Cache may be out of sync:
```bash
thag --clean bins    # Clean executable cache
thag -f <script>     # Force rebuild
```

### Dependencies not shared

**Problem:** Each script seems to rebuild dependencies

**Solution:** Check that `CARGO_TARGET_DIR` is set:
```bash
# Run with verbose mode
thag -v <script>

# Should see: "cargo_command" with CARGO_TARGET_DIR in environment
```

## Future Enhancements

Potential improvements for consideration:

1. **Size-based cleanup**: Automatically clean when shared target exceeds threshold
2. **LRU cache**: Remove least-recently-used executables when cache is full
3. **Statistics**: Show cache hit rate, space saved, dependency reuse
4. **Parallel builds**: Leverage Cargo's locking to build multiple scripts concurrently
5. **Global cache**: Option to use a persistent cache outside `$TMPDIR`

## References

- **ChatGPT discussion**: Original problem analysis and solution proposal
- **Cargo documentation**: [CARGO_TARGET_DIR](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- **Project guide**: See `CLAUDE.md` for development guidelines