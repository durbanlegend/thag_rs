# Quick Release Commands Reference

This is a condensed command reference for releasing thag_rs v1.0.0.
For detailed checklist, see RELEASE_CHECKLIST_v1.0.0.md

## Pre-Release

```bash
# Create release branch
git checkout -b release-v1.0.0

# Run version bump tool (dry run first)
cargo run --bin thag_version_bump --features tools -- --dry-run --version 1.0.0

# Apply version bump
cargo run --bin thag_version_bump --features tools -- --version 1.0.0

# Test everything locally
export THAG_DEV_PATH=$PWD
cargo test --all-features
cargo clippy --all-targets --all-features
cargo fmt --all

# Commit changes
git add -A
git commit -m "chore: bump version to 1.0.0"
git push -u origin release-v1.0.0
```

## After PR Merged to Main

```bash
# Switch to main and update
git checkout main
git pull origin main
```

## Publishing (in order!)

**IMPORTANT: Wait 2-3 minutes between each publish for crates.io to update**

```bash
# 1. thag_common
cd thag_common && cargo publish && cd ..

# Wait 2-3 minutes...

# 2. thag_proc_macros
cd thag_proc_macros && cargo publish && cd ..

# Wait 2-3 minutes...

# 3. thag_styling
cd thag_styling && cargo publish && cd ..

# Wait 2-3 minutes...

# 4. thag_profiler
cd thag_profiler && cargo publish && cd ..

# Wait 2-3 minutes...

# 5. thag_demo
cd thag_demo && cargo publish && cd ..

# Wait 2-3 minutes...

# 6. thag_rs (main crate)
cargo publish
```

## Create Git Tag

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

## Test Published Version

```bash
# In a clean directory
mkdir /tmp/test-thag-v1 && cd /tmp/test-thag-v1
cargo install thag_rs --version 1.0.0
thag --version
echo 'println!("Hello v1.0.0!");' | thag -
```

## Rollback (if needed)

```bash
# Yank broken version
cargo yank --vers 1.0.0 thag_rs

# Fix issue, bump to 1.0.1, and republish
```

## One-Liner for Publishing All (use with caution!)

```bash
cd thag_common && cargo publish && sleep 180 && cd .. && \
cd thag_proc_macros && cargo publish && sleep 180 && cd .. && \
cd thag_styling && cargo publish && sleep 180 && cd .. && \
cd thag_profiler && cargo publish && sleep 180 && cd .. && \
cd thag_demo && cargo publish && sleep 180 && cd .. && \
cargo publish
```

**Note:** This assumes all crates are ready and tested. Use with caution!