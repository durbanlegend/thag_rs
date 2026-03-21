# Release Strategy for thag_rs v1.0.0

## Executive Summary

This document outlines the strategy and rationale for releasing thag_rs v1.0.0 and provides guidance for future releases.

## Background

### Current State (v0.2.2)
- Project is feature-complete for current scope
- Stable and well-tested
- All subcrates at v0.2.x (except thag_profiler at v0.1.1)
- No significant changes planned in foreseeable future
- Primarily receiving dependency updates from Dependabot

### The Decision

Moving from v0.2.2 to v1.0.0 signals:
1. **Stability**: The API is stable and production-ready
2. **Maturity**: The project is mature and well-tested
3. **Commitment**: Following semantic versioning going forward
4. **Confidence**: Ready for production use

## The Challenge: Circular Dependencies with thag-auto

### Understanding thag-auto

The project uses a clever `thag-auto` mechanism for dependency resolution in demo scripts and tools:

```rust
/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", features = [...] }
*/
```

The `thag-auto` keyword triggers special resolution logic:
- **In development** (`THAG_DEV_PATH` set): Uses local workspace paths
- **In CI** (`GITHUB_WORKSPACE` set): Uses checked-out code
- **In production**: Uses crates.io with specified version

### Why This Doesn't Create a Catch-22

**Initial Concern**: Scripts reference `version = "0.2"`, but we're publishing v1.0.0. Will this break?

**Answer**: No, because:

1. **During development/testing**: We use `THAG_DEV_PATH=$PWD`, which overrides version resolution
2. **In CI**: GitHub Actions uses the checked-out code, not crates.io
3. **After publication**: The version bump tool updates all scripts to `version = "1.0"`

### The Solution: Single-Step Release

We perform a **coordinated version bump** before publishing:

1. Update all Cargo.toml files to v1.0.0
2. Update all script TOML blocks to reference v1.0
3. Test with `THAG_DEV_PATH` (uses local paths)
4. Commit and merge to main
5. Publish in dependency order
6. Users get consistent v1.0.0 across all crates

**Key Insight**: The `thag-auto` mechanism with `THAG_DEV_PATH` allows us to test the new versions locally before publishing to crates.io.

## Publication Strategy

### Dependency Order

Crates must be published in this order due to workspace dependencies:

```
thag_common ─────┐
                 ├─> thag_styling ──┐
thag_proc_macros ┤                  │
                 └─> thag_profiler ─┼─> thag_rs
                                    │
                     thag_demo ─────┘
```

**Publishing sequence:**
1. `thag_common` (no workspace deps)
2. `thag_proc_macros` (no workspace deps)
3. `thag_styling` (depends on: common, proc_macros)
4. `thag_profiler` (depends on: common, proc_macros, styling)
5. `thag_demo` (depends on: thag_rs, profiler)
6. `thag_rs` (depends on: common, proc_macros, profiler, styling)

**Wait time**: 2-3 minutes between publishes for crates.io index to update.

### Automation

The `thag_version_bump` tool automates:
- Updating package versions in all Cargo.toml files
- Updating workspace dependency versions
- Updating all demo script TOML blocks
- Updating all tool binary TOML blocks

**Usage:**
```bash
# Dry run first
cargo run --bin thag_version_bump --features tools -- --dry-run --version 1.0.0

# Apply changes
cargo run --bin thag_version_bump --features tools -- --version 1.0.0
```

## Testing Strategy

### Pre-Publication Testing

Test with local workspace using `THAG_DEV_PATH`:

```bash
export THAG_DEV_PATH=$PWD

# Run all tests
cargo test --all-features

# Test demo scripts
cargo run demo/hello.rs
cargo run demo/styling_demo.rs

# Test tools
cargo run --bin thag_find_demos --features tools
```

### Post-Publication Testing

Test with published versions in a clean environment:

```bash
mkdir /tmp/test-thag-v1
cd /tmp/test-thag-v1

# Install from crates.io
cargo install thag_rs --version 1.0.0

# Test basic functionality
thag --version
echo 'println!("Hello!");' | thag -
```

## Version Numbering Strategy

### Semantic Versioning

Following SemVer (semver.org):

- **Major (x.0.0)**: Breaking API changes
- **Minor (1.x.0)**: New features, backwards-compatible
- **Patch (1.0.x)**: Bug fixes, backwards-compatible

### Future Releases

**v1.0.x (Patch releases):**
- Bug fixes
- Documentation updates
- Dependency updates (non-breaking)
- Performance improvements (non-breaking)

**v1.x.0 (Minor releases):**
- New features
- New optional dependencies
- New demo scripts
- Deprecations (with migration path)

**v2.0.0 (Major release):**
- Breaking API changes
- Removal of deprecated features
- Major architectural changes
- Minimum Rust version bumps (if breaking)

### Dependency Version Specifications

**In Cargo.toml:**
- Use `major.minor` for dependencies (per project guidelines)
- Example: `clap = "4.5"` not `clap = "4.5.60"`

**In thag-auto scripts:**
- Use `major.minor` to allow patch updates
- Example: `version = "1.0, thag-auto"`

## Communication Strategy

### Release Announcement

Announce v1.0.0 on:
1. GitHub Releases (primary)
2. crates.io (automatic)
3. Reddit r/rust
4. This Week in Rust newsletter
5. Project README badge

### Release Notes Template

```markdown
# thag_rs v1.0.0

## Overview
First stable release! 🎉

## What's Changed
- Bumped to v1.0.0 to signal stability
- All workspace crates now at v1.0.0
- [List any other changes since v0.2.2]

## Breaking Changes
[None expected for this release]

## Installation
```bash
cargo install thag_rs
```

## Documentation
- Main docs: [link]
- API docs: https://docs.rs/thag_rs/1.0.0

## Contributors
[List contributors]
```

## Risk Mitigation

### Potential Issues

1. **Publication fails mid-sequence**
   - **Mitigation**: Can continue from where it failed
   - **Recovery**: Publish remaining crates in order

2. **Critical bug discovered post-release**
   - **Mitigation**: Yank v1.0.0, fix, release v1.0.1
   - **Communication**: Immediate GitHub issue and announcement

3. **Dependency version conflicts**
   - **Mitigation**: Thorough testing with THAG_DEV_PATH before release
   - **Prevention**: Version bump tool ensures consistency

4. **Scripts fail with published versions**
   - **Unlikely**: thag-auto resolves to correct published versions
   - **Testing**: Post-publication verification in clean environment

### Rollback Procedure

If v1.0.0 has critical issues:

```bash
# 1. Yank the problematic version(s)
cargo yank --vers 1.0.0 thag_rs

# 2. Fix the issue in code

# 3. Bump to v1.0.1
cargo run --bin thag_version_bump -- --version 1.0.1

# 4. Test thoroughly

# 5. Publish v1.0.1 following normal procedure

# 6. Announce fix and apologize for inconvenience
```

## Timeline

### Recommended Release Timeline

**Day 1: Preparation**
- Run version bump tool
- Update documentation
- Create release PR
- Run all tests

**Day 2: Review**
- Review all changes
- Ensure CI passes
- Merge release PR

**Day 3: Publication**
- Publish crates in order (30-45 minutes)
- Create GitHub release
- Test published versions

**Day 4-7: Monitoring**
- Monitor for issues
- Respond to feedback
- Update documentation as needed

## Future Considerations

### Maintaining v1.x

**Principles:**
1. Stability over features
2. Backwards compatibility paramount
3. Deprecate before removing
4. Document all changes

**Release cadence:**
- Patch releases: As needed (bugs, security)
- Minor releases: When features accumulate
- Major releases: Only when necessary

### Version 2.0 Criteria

Only bump to v2.0 if:
1. Breaking API changes are necessary
2. Major refactoring improves the project significantly
3. Removing deprecated features
4. Community feedback suggests major improvements

Don't bump to v2.0 just because:
- Time has passed
- Dependencies updated
- Minor improvements made

## Conclusion

The v1.0.0 release represents:
- Confidence in stability
- Commitment to users
- Maturity of the project

The `thag-auto` mechanism and version bump tool make the release process smooth and reliable.

---

**Document Version:** 1.0  
**Last Updated:** [Date to be filled in]  
**Approved By:** [Name]  
**Status:** READY FOR EXECUTION