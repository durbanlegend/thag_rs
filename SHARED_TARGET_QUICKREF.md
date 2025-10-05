# Shared Target Quick Reference

## What Changed?

**Before:** Each script had its own `target/` directory → 6GB+ for 300 scripts
**Now:** All scripts share one build cache → ~200MB total (97% reduction!)

## Directory Structure

```
$TMPDIR/
├── thag_rs/                      # Per-script manifests (tiny)
│   ├── hello/
│   │   ├── Cargo.toml           # ~10KB
│   │   ├── hello.rs
│   │   └── Cargo.lock
│   └── fib_basic_ibig/
│       ├── Cargo.toml
│       └── fib_basic_ibig.rs
│
├── thag_rs_shared_target/        # Shared build cache (~40MB)
│   ├── debug/                    # All compiled dependencies
│   ├── deps/                     # Shared across scripts
│   └── .fingerprint/
│
└── thag_rs_bins/                 # Cached executables (~200MB)
    ├── hello
    └── fib_basic_ibig
```

## Normal Usage

**Nothing changes!** Use thag exactly as before:

```bash
thag demo/hello.rs
thag demo/fib_basic_ibig.rs -- 100
thag -e "println!(\"Hello\")"
```

The shared target is automatic and transparent.

## Cache Management

### View Cache Size
```bash
du -sh $TMPDIR/thag_rs_shared_target   # Build cache
du -sh $TMPDIR/thag_rs_bins             # Executables
du -sh $TMPDIR/thag_rs                  # Manifests
```

### Clean Commands

```bash
# Clean only executables (keeps build cache)
thag --clean bins

# Clean only build cache (keeps executables)
thag --clean target

# Clean everything (default)
thag --clean all
thag --clean
```

## Benefits

### Space Savings
- **Old:** 300 scripts = 6GB (20MB each)
- **New:** 300 scripts = ~225MB (shared deps)
- **Saved:** ~5.8GB (97% reduction)

### Speed Improvements
- **First build:** Same as before (deps must compile)
- **Related scripts:** 10-15x faster (shared deps)
- **Repeated runs:** Instant (cached executable)

### Example
```bash
# First script: compiles ibig + script (~38s)
thag demo/fib_basic_ibig.rs -- 10

# Second script: reuses ibig, only compiles script (~3s)
thag demo/fib_doubling_iterative_purge_ibig.rs -- 10

# Run again: uses cache, instant
thag demo/fib_basic_ibig.rs -- 10
```

## When to Clean

| Scenario | Command | Why |
|----------|---------|-----|
| Need space quickly | `thag --clean bins` | Frees ~200MB, keeps build cache |
| Dependencies corrupted | `thag --clean target` | Fresh rebuild of all deps |
| Major cleanup | `thag --clean all` | Complete reset |
| After Rust update | `thag --clean target` | Rebuild with new compiler |

## Troubleshooting

### Build Fails After Clean
```bash
thag --clean all
thag -f demo/script.rs    # Force rebuild
```

### Executable Not Found
```bash
thag --clean bins
thag -f demo/script.rs    # Rebuild & recache
```

### Want to Force Rebuild
```bash
thag -f demo/script.rs    # Always works
```

## Migration from Old System

If you have old per-script targets:

```bash
# Check old structure size
du -sh $TMPDIR/thag_rs

# Remove old target directories (safe!)
rm -rf $TMPDIR/thag_rs

# New system will recreate without target dirs
thag demo/hello.rs
```

**Safe to delete:** The old `thag_rs/*/target/` directories are no longer used.

## Tips

1. **Don't worry about the cache growing** - it's still tiny compared to per-script targets
2. **Clean `bins` if low on space** - fastest to rebuild
3. **Clean `target` after major updates** - ensures fresh dependencies
4. **Use `-f` to force rebuild** - if output seems stale

## Technical Details

- Sets `CARGO_TARGET_DIR=$TMPDIR/thag_rs_shared_target`
- Copies executables to `$TMPDIR/thag_rs_bins/` after build
- Checks timestamps to skip unnecessary rebuilds
- Fully automatic, no configuration needed

## Running Integration Tests

The integration test suite compiles all ~340 demo scripts. Due to cargo lock contention when each test runs `cargo run`, use sequential execution:

```bash
# Recommended: Run tests sequentially
cargo test --features=simplelog -- --nocapture --test-threads=1

# Faster but may have occasional lock failures
cargo test --features=simplelog -- --nocapture --test-threads=3
```

**Note:** With the shared target implementation, multiple concurrent `cargo run` invocations can contend for the project's cargo lock. Using `--test-threads=1` ensures reliable test execution.

## See Also

- Full documentation: `SHARED_TARGET_IMPLEMENTATION.md`
- Development guide: `CLAUDE.md`
