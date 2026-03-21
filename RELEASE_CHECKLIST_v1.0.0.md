# Release Checklist for thag_rs v1.0.0

## Overview

This document outlines the step-by-step process for releasing thag_rs v1.0.0 and all workspace subcrates.

**Release Date:** TBD  
**Previous Version:** 0.2.2  
**Target Version:** 1.0.0

## Why v1.0.0?

The project is feature-complete, stable, and ready for production use. Moving to v1.0.0 signals:
- API stability and maturity
- Commitment to semantic versioning going forward
- Production-ready status

## Pre-Release Checklist

### 1. Code Quality & Testing

- [ ] All tests pass locally: `cargo test --all-features`
- [ ] All tests pass in CI
- [ ] No outstanding critical issues or bugs
- [ ] Code is properly formatted: `cargo fmt --all`
- [ ] Clippy passes: `cargo clippy --all-targets --all-features`
- [ ] Documentation builds: `cargo doc --all-features --no-deps`

### 2. Version Bump Preparation

- [ ] Review current state on main branch
- [ ] Ensure all Dependabot PRs are merged or postponed
- [ ] Create feature branch: `git checkout -b release-v1.0.0`

### 3. Run Version Bump Tool

**Dry run first:**
```bash
cargo run --bin thag_version_bump --features tools -- --dry-run --version 1.0.0
```

**Review the output carefully - should update:**
- 6 Cargo.toml files
- ~101 demo scripts
- ~33 tool binaries
- ~140 files total

**Apply changes:**
```bash
cargo run --bin thag_version_bump --features tools -- --version 1.0.0
```

### 4. Update Documentation

- [ ] Update main README.md:
  - [ ] Installation instructions reference v1.0
  - [ ] Any version-specific examples updated
  - [ ] Badges/shields updated if applicable

- [ ] Create/Update CHANGELOG.md:
  - [ ] Document all changes since v0.2.2
  - [ ] Highlight breaking changes (if any)
  - [ ] Credit contributors
  - [ ] Note this is the v1.0.0 stable release

- [ ] Update workspace member READMEs as needed:
  - [ ] thag_common/README.md
  - [ ] thag_demo/README.md
  - [ ] thag_proc_macros/README.md
  - [ ] thag_profiler/README.md
  - [ ] thag_styling/README.md

### 5. Local Testing with New Versions

Test with local paths to ensure everything works:

```bash
export THAG_DEV_PATH=$PWD

# Run comprehensive tests
cargo test --all-features

# Test a few demo scripts
cargo run demo/hello.rs
cargo run demo/styling_demo.rs
cargo run demo/proc_macro_category_enum.rs

# Test a few tools
cargo run --bin thag_find_demos --features tools
cargo run --bin thag_gen_readme --features tools
```

- [ ] All tests pass
- [ ] Demo scripts run successfully
- [ ] Tools work correctly

### 6. Commit and Push

```bash
git add -A
git status  # Review changes
git commit -m "chore: bump version to 1.0.0"
git push -u origin release-v1.0.0
```

- [ ] Changes committed
- [ ] Branch pushed to remote

### 7. Create Pull Request

- [ ] Create PR from `release-v1.0.0` to `main`
- [ ] Title: "Release v1.0.0"
- [ ] Description includes:
  - Summary of changes
  - Link to this checklist
  - Note about breaking changes (if any)
- [ ] Wait for CI to pass
- [ ] Get review/approval (if required)
- [ ] Merge to main

## Publication Process

**IMPORTANT:** Crates must be published in dependency order!

### Publishing Order

1. **thag_common** (no workspace dependencies)
2. **thag_proc_macros** (no workspace dependencies)
3. **thag_profiler** (depends on: thag_common, thag_proc_macros, thag_styling)
4. **thag_styling** (depends on: thag_common, thag_proc_macros)
5. **thag_demo** (depends on: thag_rs, thag_profiler)
6. **thag_rs** (depends on: thag_common, thag_proc_macros, thag_profiler, thag_styling)

**Note:** thag_profiler and thag_styling have circular-ish dependencies, but thag_profiler's dependency on thag_styling is optional. Publish thag_styling first, then thag_profiler.

### Step-by-Step Publication

Ensure you're on the main branch with the merged release PR:

```bash
git checkout main
git pull origin main
```

#### 1. Publish thag_common

```bash
cd thag_common
cargo publish --dry-run
# Review output carefully
cargo publish
cd ..
```

- [ ] thag_common v1.0.0 published to crates.io
- [ ] Verify on crates.io: https://crates.io/crates/thag_common

#### 2. Publish thag_proc_macros

```bash
cd thag_proc_macros
cargo publish --dry-run
cargo publish
cd ..
```

- [ ] thag_proc_macros v1.0.0 published
- [ ] Verify on crates.io: https://crates.io/crates/thag_proc_macros

**Wait 2-3 minutes** for crates.io to update its index before proceeding.

#### 3. Publish thag_styling

```bash
cd thag_styling
cargo publish --dry-run
cargo publish
cd ..
```

- [ ] thag_styling v1.0.0 published
- [ ] Verify on crates.io: https://crates.io/crates/thag_styling

**Wait 2-3 minutes** for crates.io to update.

#### 4. Publish thag_profiler

```bash
cd thag_profiler
cargo publish --dry-run
cargo publish
cd ..
```

- [ ] thag_profiler v1.0.0 published
- [ ] Verify on crates.io: https://crates.io/crates/thag_profiler

**Wait 2-3 minutes** for crates.io to update.

#### 5. Publish thag_demo

```bash
cd thag_demo
cargo publish --dry-run
cargo publish
cd ..
```

- [ ] thag_demo v1.0.0 published
- [ ] Verify on crates.io: https://crates.io/crates/thag_demo

**Wait 2-3 minutes** for crates.io to update.

#### 6. Publish thag_rs (main crate)

```bash
cargo publish --dry-run
# Review carefully - this is the main crate
cargo publish
```

- [ ] thag_rs v1.0.0 published
- [ ] Verify on crates.io: https://crates.io/crates/thag_rs

### Troubleshooting Publication

If you encounter "crate not found" errors during publishing:

1. **Wait longer** - crates.io index updates can take 2-5 minutes
2. **Check versions** - ensure you're referencing the correct version
3. **Verify publication** - check crates.io directly
4. **Use `cargo search`** - verify the crate is indexed:
   ```bash
   cargo search thag_common
   ```

If a publish fails:

1. **Don't panic** - you can't unpublish, but you can yank and republish with a patch version
2. **Fix the issue**
3. **Bump to v1.0.1** if necessary
4. **Try again**

## Post-Publication

### 1. Create GitHub Release

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

- [ ] Tag created and pushed

On GitHub:

- [ ] Go to Releases → Draft a new release
- [ ] Tag: `v1.0.0`
- [ ] Title: `thag_rs v1.0.0`
- [ ] Description:
  - [ ] Include CHANGELOG content
  - [ ] Highlight key features
  - [ ] Installation instructions
  - [ ] Link to documentation
- [ ] Mark as latest release
- [ ] Publish release

### 2. Test Published Versions

In a **clean directory** (not the project directory):

```bash
mkdir /tmp/test-thag-v1
cd /tmp/test-thag-v1

# Test installation
cargo install thag_rs --version 1.0.0

# Test basic functionality
thag --version
echo 'println!("Hello v1.0.0!");' | thag -

# Test with a demo script (copy one from the repo)
thag /path/to/demo/hello.rs
```

- [ ] Installation works
- [ ] Binary runs correctly
- [ ] Basic features work
- [ ] Demo scripts work without THAG_DEV_PATH

### 3. Update Documentation Sites

If applicable:

- [ ] Update docs.rs (should happen automatically)
- [ ] Update any external documentation
- [ ] Update project website if it exists

### 4. Announce Release

Consider announcing on:

- [ ] GitHub Discussions (if enabled)
- [ ] Reddit (r/rust)
- [ ] Twitter/X
- [ ] Discord/Slack communities
- [ ] This Week in Rust (submit to newsletter)

### 5. Cleanup

```bash
# Delete release branch (after PR is merged)
git branch -d release-v1.0.0
git push origin --delete release-v1.0.0
```

- [ ] Local branch deleted
- [ ] Remote branch deleted

## Post-Release Monitoring

### First 24 Hours

- [ ] Monitor crates.io download stats
- [ ] Watch for GitHub issues
- [ ] Check CI status on main branch
- [ ] Review any user feedback

### First Week

- [ ] Address any critical bugs immediately
- [ ] Document any common issues
- [ ] Update FAQ if needed

## Rollback Plan

If critical issues are discovered:

1. **Yank the problematic version:**
   ```bash
   cargo yank --vers 1.0.0 thag_rs
   ```

2. **Fix the issue**

3. **Release v1.0.1** with the fix

4. **Communicate** about the issue and fix

## Notes

- **Do not** delete or modify this checklist until release is complete
- **Do** update with actual times/dates as you progress
- **Keep** this for future releases as a template
- **Version numbers** in this checklist should match the actual versions being published

## Sign-off

- [ ] All checklist items completed
- [ ] Release successful
- [ ] Post-release testing passed
- [ ] Documentation updated

**Release Manager:** _________________  
**Date Completed:** _________________  
**Final Notes:**

---

*Generated for thag_rs v1.0.0 release*