# Testing v1.0.0 with Git Dependencies

This guide shows how to test demo scripts using the `release-v1.0.0` branch via git dependencies before publishing to crates.io.

## Quick Test

You can test a demo script by setting the `THAG_GIT_REF` environment variable:

```bash
# From the thag_rs project directory
export THAG_GIT_REF=release-v1.0.0
export THAG_GIT_REPO=https://github.com/durbanlegend/thag_rs

# Test a demo script
cargo run demo/hello.rs
cargo run demo/styling_demo.rs
cargo run demo/proc_macro_category_enum.rs
```

The `thag-auto` mechanism will automatically use git dependencies instead of local paths or crates.io.

## Manual Testing

Alternatively, you can temporarily modify a demo script to explicitly use git dependencies:

### Example: Testing demo/styling_demo.rs

Original TOML block:
```rust
/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto", features = ["color_detect"] }
*/
```

Modified for testing (temporary):
```rust
/*[toml]
[dependencies]
thag_styling = { git = "https://github.com/durbanlegend/thag_rs", branch = "release-v1.0.0", features = ["color_detect"] }
*/
```

Then run:
```bash
cargo run demo/styling_demo.rs
```

## Recommended Test Scripts

Test a variety of scripts to cover different dependencies:

### 1. Basic thag_styling
```bash
export THAG_GIT_REF=release-v1.0.0
cargo run demo/styling_demo.rs
```

### 2. thag_proc_macros
```bash
cargo run demo/proc_macro_category_enum.rs
cargo run demo/proc_macro_styled.rs
```

### 3. thag_rs with features
```bash
cargo run --bin thag_find_demos --features tools
```

### 4. Multiple dependencies
```bash
cargo run demo/ratatui_integration_demo.rs
cargo run demo/test_auto_help.rs
```

### 5. thag_profiler
```bash
cargo run demo/benchmark_profile.rs
cargo run demo/thag_profile_benchmark.rs
```

## What to Look For

When testing, verify:

- ✅ Scripts compile successfully
- ✅ Scripts run without errors
- ✅ Features work as expected
- ✅ No version conflicts in Cargo output
- ✅ Dependencies resolve to the git branch correctly

## Clean Up After Testing

If you manually modified any files:

```bash
# Restore original files
git restore demo/styling_demo.rs  # or whichever you modified

# Unset environment variables
unset THAG_GIT_REF
unset THAG_GIT_REPO
```

## CI Testing

The CI will automatically test the `release-v1.0.0` branch using the checked-out code (via `GITHUB_WORKSPACE`), which is effectively the same as git dependencies.

Monitor the CI status at: https://github.com/durbanlegend/thag_rs/actions

## After Testing

Once you've confirmed everything works:

1. Create PR from `release-v1.0.0` to `main`
2. Wait for CI to pass
3. Get review/approval if needed
4. Merge to main
5. Follow publication steps in `RELEASE_COMMANDS.md`

## Troubleshooting

### "Failed to resolve patches" error

If you see this, it means Cargo can't find the git branch. Verify:
- Branch exists: `git ls-remote origin release-v1.0.0`
- Branch is pushed: Should show in GitHub

### Version conflicts

If you see version conflicts, ensure:
- All workspace crates reference version "1" (not "1.0")
- The git branch has all commits pushed

### Slow builds

First build with git dependencies will be slow (clones repository). Subsequent builds use cache.