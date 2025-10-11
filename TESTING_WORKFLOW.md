# Testing Workflow for Release Preparation

**Problem**: Running the full `run_release_tests.sh` is slow because every failure requires massive recompilation.

**Solution**: Test incrementally at the subcrate level with `--no-fail-fast` to catch all errors at once.

---

## Workflow Strategy

1. **Test one subcrate completely** (all failures in one run)
2. **Fix all issues found**
3. **Re-test just the fixed tests** to verify
4. **Move to next subcrate**
5. **Final verification** with full workspace tests

---

## Per-Subcrate Testing Commands

### thag_common (Quick - minimal features)

```bash
cd thag_common
cargo test --no-fail-fast -- --nocapture
cargo clippy -- -W clippy::pedantic
cd ..
```

### thag_proc_macros (Quick - no features)

```bash
cd thag_proc_macros
cargo test --no-fail-fast -- --nocapture
cargo clippy -- -W clippy::pedantic
cd ..
```

### thag_styling (Test key feature combinations)

```bash
cd thag_styling
cargo test --features basic --no-fail-fast -- --nocapture
cargo test --features full --no-fail-fast -- --test-threads=1 --nocapture
cargo clippy --features full -- -W clippy::pedantic
cd ..
```

**Note**: Use `--test-threads=1` for `full` features due to integration test interference.

### thag_profiler (Test profiling modes)

```bash
cd thag_profiler
cargo test --no-fail-fast -- --test-threads=1 --nocapture
cargo test --features full_profiling --no-fail-fast -- --test-threads=1 --nocapture
cargo clippy --features full_profiling -- -W clippy::pedantic
cd ..
```

**Critical**: Always use `--test-threads=1` to avoid TLS and allocator conflicts.

### thag_demo

```bash
cd thag_demo
cargo test --no-fail-fast -- --nocapture
cargo clippy -- -W clippy::pedantic
cd ..
```

---

## Main Workspace Tests

After all subcrates pass:

```bash
# Test with default features
cargo test --workspace --no-fail-fast

# Test with env_logger variant (may conflict - that's expected)
cargo test --workspace --no-default-features --features env_logger,full --no-fail-fast || true

# Test with tools
cargo test --workspace --features tools --no-fail-fast

# Test with profiling
cargo test --workspace --features profiling --no-fail-fast

# Integration tests (fast now with precompiled binary!)
cargo test --test integration_test -- --test-threads=1 --nocapture
```

---

## Re-Running Specific Failed Tests

After fixing issues, re-run only the failed tests:

```bash
# Example: Re-run specific tests
cargo test --features full_profiling test_stack_extraction test_thread_isolation -- --test-threads=1 --nocapture

# Or re-run a specific test file
cargo test --features full_profiling --test test_profiling -- --test-threads=1 --nocapture
```

---

## Quality Checks (After Tests Pass)

```bash
# Format check
cargo fmt --all -- --check

# Build release
cargo build --release --workspace

# Documentation
cargo doc --workspace --no-deps

# Clippy workspace
cargo clippy --all-targets --workspace
cargo clippy --workspace --no-default-features --features env_logger,core || true
```

---

## Prose Quality

```bash
# Typos
typos

# Vale (if installed)
vale README.md --no-wrap
vale thag_profiler/README.md --no-wrap
vale thag_styling/README.md --no-wrap
vale thag_common/README.md --no-wrap
vale thag_proc_macros/README.md --no-wrap
```

---

## Final Verification

Once everything passes individually, run the full suite to confirm:

```bash
./run_release_tests.sh 2>&1 | tee release_test_results.log
```

---

## Common Patterns

### Capture All Failures

Always use `--no-fail-fast` for first run:
```bash
cargo test --no-fail-fast -- --nocapture 2>&1 | tee test_output.log
```

Then search the log for `FAILED` to see all failures at once.

### Test-Specific Issues

**thag_profiler**:
- Always `--test-threads=1`
- TLS tests need profiling enabled: `force_set_profiling_state(true)`
- Memory tests need `full_profiling` feature

**thag_styling**:
- `full` feature tests need `--test-threads=1`
- Integration tests (crossterm, console) can be flaky - fixed with proper imports

**Integration tests**:
- Now fast with precompiled binary!
- Still need `--test-threads=1` due to shared build artifacts

### When Tests Hang

If tests hang, likely causes:
1. **Barrier with wrong count** - check `Barrier::new(N)` matches thread count
2. **Deadlock on mutex** - use `--nocapture` to see where it stops
3. **Infinite loop** - Ctrl-C and check last output

---

## Time Estimates

| Task | Time |
|------|------|
| thag_common | 1-2 min |
| thag_proc_macros | 1-2 min |
| thag_styling | 3-5 min |
| thag_profiler | 5-10 min |
| thag_demo | 1-2 min |
| Workspace tests | 10-15 min |
| Integration tests | 5-10 min |
| Quality checks | 5-10 min |
| **Total (clean run)** | **30-45 min** |

**With fixes**: Add 5-10 min per issue depending on compilation time.

---

## Tips

1. **Use `tee`** to capture output while watching progress:
   ```bash
   cargo test --no-fail-fast -- --nocapture 2>&1 | tee test.log
   ```

2. **Filter noise** with grep:
   ```bash
   cargo test --no-fail-fast 2>&1 | grep -E "(FAILED|test result)"
   ```

3. **Check clippy pedantic issues** incrementally - they're usually quick fixes

4. **Document weird issues** - add comments explaining `--test-threads=1` requirements

5. **Keep RELEASE_CHECKLIST.md updated** as you complete each section

---

## Progress Tracking

Use RELEASE_CHECKLIST.md checkboxes, but work through subcrates in this order:

1. ✅ thag_common (easiest, builds confidence)
2. ✅ thag_proc_macros (no dependencies on others)
3. ✅ thag_styling (moderate complexity)
4. ✅ thag_profiler (most complex, save for when fresh)
5. ✅ thag_demo (depends on others)
6. ✅ Workspace tests (integrates everything)
7. ✅ Quality checks (final polish)

---

**Remember**: Fast iteration beats comprehensive but slow testing during the debug phase. Save `run_release_tests.sh` for the final verification before package/publish.