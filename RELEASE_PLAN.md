# Release Plan for thag_rs v0.2.0 and Subcrates

## Overview

This document provides a coordinated release plan for `thag_rs` and all its subcrates. The plan ensures proper dependency ordering, comprehensive testing, and documentation completeness before publishing to crates.io and creating GitHub releases.

## Current State

### Version Status
- `thag_rs`: 0.2.0 (ready for release)
- `thag_common`: 0.2.0 (ready for release)
- `thag_demo`: 0.2.0 (ready for release)
- `thag_proc_macros`: 0.2.0 (ready for release)
- `thag_profiler`: 0.1.0 (ready for release)
- `thag_styling`: 0.2.0 (ready for release)

### Dependency Tree

```
thag_common (foundation - no thag deps)
    └── thag_proc_macros (foundation - no thag deps)
            ├── thag_styling (depends on common, proc_macros)
            │       └── thag_profiler (depends on common, proc_macros, styling)
            │               ├── thag_rs (depends on all)
            │               │       └── thag_demo (depends on thag_rs, thag_profiler)
            │               └── (optional for thag_rs)
            └── (used by thag_rs)
```

## Release Order

**Critical**: Subcrates must be published in dependency order because crates.io validates dependencies at publish time.

### Phase 1: Foundation Crates
1. `thag_common` (no thag dependencies)
2. `thag_proc_macros` (no thag dependencies)

### Phase 2: Styling Layer
3. `thag_styling` (depends on common, proc_macros)

### Phase 3: Profiling Layer
4. `thag_profiler` (depends on common, proc_macros, styling)

### Phase 4: Main Application
5. `thag_rs` (depends on all previous)

### Phase 5: Demo Facade
6. `thag_demo` (depends on thag_rs, thag_profiler)

**Important**: Wait 5-10 minutes between releases for crates.io to fully process and index each crate before publishing the next.

## Pre-Release Checklist

### 1. Version Verification ✓
- [x] All Cargo.toml versions set to target release versions
- [ ] Verify workspace member versions are consistent
- [ ] Check that main Cargo.toml dependency versions match subcrate versions
- [ ] Ensure no crates use git or path dependencies on each other (must use version)

### 2. Code Quality

#### Testing
- [ ] Run `cargo test --workspace` - all tests pass
- [ ] Run `cargo test --workspace --all-features` - all features tested
- [ ] Run integration tests: `cargo test --test integration_test`
- [ ] Test individual crate builds:
  ```bash
  cargo build -p thag_common
  cargo build -p thag_proc_macros
  cargo build -p thag_styling --all-features
  cargo build -p thag_profiler --all-features
  cargo build -p thag_rs --all-features
  cargo build -p thag_demo
  ```
- [ ] Run `cargo build --release --workspace` for release build verification

#### Linting
- [ ] Run `cargo clippy --all-targets --all-features --workspace`
- [ ] Run `./clippy_feature_tests.sh` (if available)
- [ ] Address any clippy warnings (or document why they're acceptable)

#### Formatting
- [ ] Run `cargo fmt --all -- --check` to verify formatting
- [ ] Fix any formatting issues with `cargo fmt --all`

### 3. Documentation

#### Code Documentation
- [ ] Check documentation completeness:
  ```bash
  cargo doc --workspace --all-features --no-deps
  ```
- [ ] Review docs for each public crate:
  ```bash
  cargo doc -p thag_common --features document-features --no-deps --open
  cargo doc -p thag_proc_macros --no-deps --open
  cargo doc -p thag_styling --all-features --no-deps --open
  cargo doc -p thag_profiler --all-features --no-deps --open
  cargo doc -p thag_rs --all-features --no-deps --open
  cargo doc -p thag_demo --no-deps --open
  ```
- [ ] Verify feature flags are documented in each Cargo.toml (document-features)
- [ ] Check that all public APIs have doc comments

#### README Files
- [ ] Review and update all README.md files (see Documentation Review section below)
- [ ] Ensure README images are up to date and accessible
- [ ] Verify all internal links work
- [ ] Check that examples in READMEs compile and run

#### Other Documentation
- [ ] Update CHANGELOG.md with all changes since last release
- [ ] Review and update TODO.md to remove completed items
- [ ] Ensure CLAUDE.md is current with development guidelines
- [ ] Check that demo/README.md accurately reflects available scripts

### 4. Dependencies

#### Version Checks
- [ ] Check deps.rs for outdated dependencies:
  - Visit https://deps.rs/repo/github/durbanlegend/thag_rs
  - Update any outdated dependencies or document why not updating
- [ ] Verify MSRV (Minimum Supported Rust Version):
  ```bash
  cargo msrv verify
  # If needed: cargo msrv set
  ```
- [ ] Update MSRV in README.md files if changed

#### Subcrate Dependencies
- [ ] Verify thag_proc_macros version in all dependent Cargo.toml files
- [ ] Verify thag_common version in all dependent Cargo.toml files
- [ ] Verify thag_styling version in thag_profiler and thag_rs
- [ ] Verify thag_profiler version in thag_rs and thag_demo
- [ ] Ensure no workspace path dependencies remain for crates.io publish

### 5. Tools and Scripts

#### Binary Tools
- [ ] Test all binary tools in src/bin/ work correctly
- [ ] Verify tools feature enables all binaries
- [ ] Test profiling tools (if applicable):
  ```bash
  cargo run --features tools --bin thag_profile -- --help
  cargo run --features tools --bin thag_instrument -- --help
  cargo run --features tools --bin thag_uninstrument -- --help
  ```

#### Demo Scripts
- [ ] Run `thag_gen_readme` to regenerate demo/README.md
- [ ] Run `thag_gen_proc_macro_readme` for proc_macros README if applicable
- [ ] Verify demo scripts still work with new version
- [ ] Test demo scripts don't reference unreleased versions

### 6. Quality Checks

#### Spelling and Grammar
- [ ] Run `typos` command on project root
- [ ] Run `vale README.md --no-wrap` on main README
- [ ] Run `vale demo/README.md --no-wrap` on demo README
- [ ] Run `vale thag_profiler/README.md --no-wrap` on profiler README
- [ ] Run `vale thag_styling/README.md --no-wrap` on styling README
- [ ] Fix any spelling/grammar issues

#### File System
- [ ] Remove .DS_Store files: `find . -name .DS_Store -delete`
- [ ] Verify .gitignore is comprehensive
- [ ] Check that no sensitive data is included

### 7. Platform Testing

#### Cross-Platform Verification
- [ ] Test on macOS (if available)
- [ ] Test on Linux (if available)
- [ ] Test on Windows (if available)
- [ ] Verify CI passes on all platforms (check GitHub Actions)

### 8. Package Verification

#### Dry Run
- [ ] Test package creation for each crate (do NOT publish yet):
  ```bash
  cd thag_common && cargo package --no-verify
  cd thag_proc_macros && cargo package --no-verify
  cd thag_styling && cargo package --no-verify
  cd thag_profiler && cargo package --no-verify
  cd .. && cargo package --no-verify  # thag_rs
  cd thag_demo && cargo package --no-verify
  ```
- [ ] Review package contents in target/package/
- [ ] Verify included files are correct (check Cargo.toml include field)

#### Local Installation Test
- [ ] Test local installation:
  ```bash
  cargo install --path . --force
  ```
- [ ] Verify installed binary works correctly
- [ ] Test with `thag --version` and basic commands

## Release Execution

### Prepare for Release

#### 1. Final Commit and Push
```bash
# Ensure all changes are committed
git status
git add -A
git commit -m "Prepare for v0.2.0 release"
git push origin main
```

#### 2. Disable CI for Documentation Tweaks (Optional)
- If making last-minute README changes, temporarily disable ci.yml
- Remember to re-enable before tagging

#### 3. Backup Current State
```bash
git tag pre-v0.2.0-backup
git push origin pre-v0.2.0-backup
```

### Publishing to crates.io

**CRITICAL**: Follow this order precisely. Wait 5-10 minutes between each publish.

#### Phase 1: Foundation (can be done in parallel)

```bash
cd thag_common
cargo publish --no-verify
# Wait for confirmation, check on crates.io

cd ../thag_proc_macros
cargo publish --no-verify
# Wait for confirmation
```

#### Phase 2: Styling
```bash
cd ../thag_styling
# Wait 5-10 minutes for dependencies to be indexed
cargo publish --no-verify
# Wait for confirmation
```

#### Phase 3: Profiler
```bash
cd ../thag_profiler
# Wait 5-10 minutes
cargo publish --no-verify
# Wait for confirmation
```

#### Phase 4: Main Application
```bash
cd ..
# Wait 5-10 minutes
cargo publish --no-verify
# Wait for confirmation
```

#### Phase 5: Demo
```bash
cd thag_demo
# Wait 5-10 minutes
cargo publish --no-verify
# Wait for confirmation
```

### Creating GitHub Release

#### 1. Initialize cargo-dist (if needed)
```bash
cargo dist init
```

#### 2. Generate Changelog
```bash
# Generate raw changelog from git
git log v0.1.9..HEAD --pretty=format:"- %s" > /tmp/raw_changelog.txt
# Review and edit for release notes
```

#### 3. Create and Push Tag
```bash
git tag v0.2.0 -m "Release v0.2.0

Highlights:
- New thag_styling subcrate with 290+ terminal themes
- New thag_profiler subcrate for graphical profiling
- Enhanced dependency inference
- URL-based script execution
- Improved proc macro support

See CHANGELOG.md for full details."

git push origin v0.2.0
```

#### 4. Monitor cargo-dist
- Watch GitHub Actions for cargo-dist workflow
- Wait for release to be created automatically

#### 5. Edit Release Notes
- Go to GitHub releases page
- Edit the auto-generated release notes
- Add curated changelog from CHANGELOG.md
- Add screenshots/examples if appropriate
- Highlight key features and breaking changes

### Verification After GitHub Release

#### 1. Test Installation from GitHub
```bash
cargo install --git https://github.com/durbanlegend/thag_rs --tag v0.2.0 --force
thag --version
# Test basic functionality
```

#### 2. Test Installation from crates.io
```bash
# Wait an hour after publishing
cargo install thag_rs --force
thag --version
# Test basic functionality
```

#### 3. Verify Download Links
- Check that release artifacts are available
- Test downloading binaries for different platforms
- Verify checksums if provided

## Post-Release Tasks

### 1. Update Branch Management
- [ ] Merge any release changes back to develop branch
- [ ] Use staging branch to avoid backward merges:
  ```bash
  git checkout develop
  git checkout -b staging_temp
  git merge main
  # Create PR from staging_temp to develop
  ```

### 2. Announcements
- [ ] Update project website (if applicable)
- [ ] Post to Reddit r/rust (consider timing and content)
- [ ] Post to This Week in Rust (submit PR)
- [ ] Update docs.rs links in documentation
- [ ] Tweet/social media announcement (if desired)

### 3. Monitor for Issues
- [ ] Watch GitHub issues for bug reports
- [ ] Monitor crates.io download stats
- [ ] Check docs.rs build status
- [ ] Respond to any community feedback

### 4. Documentation Updates
- [ ] Ensure docs.rs built successfully for all crates
- [ ] Verify documentation links in README.md point to correct version
- [ ] Update any external documentation references

### 5. Prepare for Next Release
- [ ] Create new section in CHANGELOG.md for next version
- [ ] Update version numbers to next development version (optional)
- [ ] Review TODO.md and prioritize next features

## Rollback Plan

If critical issues are discovered after release:

### Option 1: Yank and Republish
```bash
# Yank problematic version from crates.io
cargo yank --vers 0.2.0 thag_rs

# Fix issues, bump to 0.2.1, republish
```

### Option 2: Remove GitHub Tag
```bash
# Remove tag locally
git tag -d v0.2.0

# Remove tag from GitHub
git push origin --delete v0.2.0

# Fix issues, retag when ready
```

### Option 3: Patch Release
- Fix critical bug
- Release as v0.2.1 immediately
- Document the issue in CHANGELOG.md

## Documentation Review Recommendations

See [DOCUMENTATION_REVIEW.md](DOCUMENTATION_REVIEW.md) for detailed recommendations on README and documentation improvements.

Key areas requiring review:
1. Main README.md - verify examples, update feature list
2. thag_profiler README.md - excellent, minor updates only
3. thag_styling README.md - verify image links, expand examples
4. thag_demo README.md - good overview, verify links
5. demo/README.md - ensure generated content is current
6. CHANGELOG.md - complete and accurate
7. Contributing guidelines - ensure present and clear

## Timeline Recommendation

### Day 1: Pre-Release Preparation
- Morning: Run all tests, linting, documentation checks
- Afternoon: Review and update all README files
- Evening: Run package verification, local installation tests

### Day 2: Publishing (Allow Full Day)
- Morning: Final review, create backup tag
- Late Morning: Start publishing to crates.io (Phase 1)
- Afternoon: Continue publishing (Phases 2-5 with proper delays)
- Evening: Verify all crates are published and indexed

### Day 3: GitHub Release
- Morning: Create and push git tag
- Afternoon: Monitor cargo-dist, edit release notes
- Evening: Test installation, verify downloads

### Day 4: Post-Release
- Update branch management
- Begin monitoring for issues
- Start announcements

## Notes and Considerations

### Version Dependencies
- When publishing subcrates that depend on each other, ensure the dependency version in Cargo.toml matches the version being published
- crates.io requires exact version matches for workspace dependencies
- Consider using workspace inheritance for versions to ensure consistency

### Feature Flags
- Ensure features are properly documented
- Test that features work in all combinations that make sense
- Consider using `cargo hack` to test feature combinations

### Binary Distribution
- cargo-dist handles binary builds for multiple platforms
- Ensure binary names don't conflict with common commands
- Test binaries on all supported platforms if possible

### Demo Scripts
- Demo scripts should not reference unreleased versions
- Use `thag-auto` marker for development, but ensure published versions work
- Consider having a "demo compatibility" test

### Coordination
- This is a multi-crate release requiring careful coordination
- Don't rush - better to take an extra day than to publish broken crates
- Keep detailed notes during the process for next time

---

**Last Updated**: 2025-01-20
**Target Release**: v0.2.0
**Status**: Pre-Release Planning