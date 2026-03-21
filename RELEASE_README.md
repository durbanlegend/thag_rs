# Release Documentation

This directory contains comprehensive documentation and tooling for releasing thag_rs v1.0.0.

## Quick Start

### Just want to release? Start here:

1. **Review the strategy first:**
   ```bash
   cat RELEASE_STRATEGY_v1.md
   ```

2. **Run the version bump tool in dry-run mode:**
   ```bash
   cargo run --bin thag_version_bump --features tools -- --dry-run --version 1.0.0
   ```

3. **Read the summary:**
   ```bash
   cat RELEASE_v1_SUMMARY.md
   ```

4. **Follow the checklist:**
   ```bash
   cat RELEASE_CHECKLIST_v1.0.0.md
   ```

5. **Use the command reference:**
   ```bash
   cat RELEASE_COMMANDS.md
   ```

## Documentation Files

### 📋 [RELEASE_v1_SUMMARY.md](RELEASE_v1_SUMMARY.md)
**Start here!** Overview of what's been prepared, why v1.0.0 makes sense, and next steps.

- What we've done
- Key insights about the release strategy
- Why there's no catch-22 problem
- Three options for how to proceed
- Quick reference for what gets updated

### 📖 [RELEASE_STRATEGY_v1.md](RELEASE_STRATEGY_v1.md)
**For understanding the approach.** Detailed explanation of the release strategy and rationale.

- Why v1.0.0 is the right move
- How the thag-auto mechanism works
- Publication strategy and dependency order
- Testing strategy
- Future versioning guidelines
- Risk mitigation

### ✅ [RELEASE_CHECKLIST_v1.0.0.md](RELEASE_CHECKLIST_v1.0.0.md)
**The complete guide.** Step-by-step checklist with every detail needed for the release.

- Pre-release checklist (code quality, testing, etc.)
- Version bump procedure
- Documentation updates
- Local testing steps
- Publication process (in correct order!)
- Post-publication verification
- Announcement strategy
- Rollback procedures

### ⚡ [RELEASE_COMMANDS.md](RELEASE_COMMANDS.md)
**Quick reference.** All commands in one place for easy copy-paste.

- Pre-release commands
- Version bump commands
- Publication commands (in order)
- Testing commands
- Rollback commands
- One-liner for publishing all (use with caution!)

## Tools

### 🔧 `thag_version_bump`
Automated version bumping tool that updates version numbers across the entire workspace.

**Location:** `src/bin/thag_version_bump.rs`

**Features:**
- Updates all Cargo.toml package versions
- Updates all workspace dependency versions
- Updates all demo script TOML blocks
- Updates all tool binary TOML blocks
- Supports dry-run mode
- Updates 140+ files automatically

**Usage:**
```bash
# Dry run (safe, shows what would change)
cargo run --bin thag_version_bump --features tools -- --dry-run --version 1.0.0

# Apply changes
cargo run --bin thag_version_bump --features tools -- --version 1.0.0

# Interactive mode (prompts for version)
cargo run --bin thag_version_bump --features tools
```

**What it updates:**
- 6 Cargo.toml files (all workspace members)
- 101 demo scripts with thag-auto dependencies
- 33 tool binaries with thag-auto dependencies
- Total: 140 files

## The Release Process (TL;DR)

```bash
# 1. Create release branch
git checkout -b release-v1.0.0

# 2. Bump versions
cargo run --bin thag_version_bump --features tools -- --version 1.0.0

# 3. Test
export THAG_DEV_PATH=$PWD
cargo test --all-features

# 4. Commit and create PR
git add -A
git commit -m "chore: bump version to 1.0.0"
git push -u origin release-v1.0.0

# 5. After PR merged, publish in order
cd thag_common && cargo publish && cd ..
# (wait 2-3 minutes between each)
cd thag_proc_macros && cargo publish && cd ..
cd thag_styling && cargo publish && cd ..
cd thag_profiler && cargo publish && cd ..
cd thag_demo && cargo publish && cd ..
cargo publish

# 6. Create git tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# 7. Test published version
mkdir /tmp/test-thag && cd /tmp/test-thag
cargo install thag_rs --version 1.0.0
thag --version
```

## Why v1.0.0?

The project is:
- ✅ Feature-complete for current scope
- ✅ Stable and well-tested
- ✅ Production-ready
- ✅ No major changes planned
- ✅ Receiving only maintenance updates

v1.0.0 signals stability and production-readiness to users.

## The thag-auto Solution

The `thag-auto` mechanism elegantly handles version resolution:

- **Development** (`THAG_DEV_PATH` set): Uses local workspace paths
- **CI** (`GITHUB_WORKSPACE` set): Uses checked-out code  
- **Production**: Uses crates.io with specified version

This means we can test v1.0.0 locally before publishing, and there's no circular dependency problem.

## Publication Order (CRITICAL!)

**Must publish in this exact order:**

1. `thag_common` (no workspace deps)
2. `thag_proc_macros` (no workspace deps)
3. `thag_styling` (depends on: common, proc_macros)
4. `thag_profiler` (depends on: common, proc_macros, styling)
5. `thag_demo` (depends on: thag_rs, profiler)
6. `thag_rs` (depends on: common, proc_macros, profiler, styling)

**Wait 2-3 minutes between each publish** for crates.io index to update.

## Rollback Plan

If critical issues are discovered:

```bash
# Yank the problematic version
cargo yank --vers 1.0.0 thag_rs

# Fix the issue
# Bump to v1.0.1
# Republish following normal procedure
```

## Questions?

- Read `RELEASE_STRATEGY_v1.md` for detailed rationale
- Check `RELEASE_CHECKLIST_v1.0.0.md` for step-by-step guide
- Review `RELEASE_COMMANDS.md` for quick command reference
- See `RELEASE_v1_SUMMARY.md` for overview and next steps

## Document Versions

These documents are versioned for future reference:
- Can be used as templates for future releases
- Update version numbers for subsequent releases
- Keep for historical reference

---

**Ready to release?** Start with:
```bash
cargo run --bin thag_version_bump --features tools -- --dry-run --version 1.0.0
```

This shows what would change without actually changing anything.