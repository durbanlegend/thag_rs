# Release Checklist: thag_rs v0.2.0

**Quick Reference** - For detailed instructions see RELEASE_PLAN.md

---

## Pre-Release Day

### Documentation (Est. 10 hours)
- [x] Create `thag_common/README.md` (brief, infrastructure crate)
- [x] Create `thag_proc_macros/README.md` (brief, proc macro crate)
- [x] Capture REPL session GIF/video
- [x] Capture TUI editor screenshot
- [ ] Capture thag_demo browse interface screenshot
- [x] Add images to main README.md
- [x] Add cross-reference links between READMEs
- [x] Update CHANGELOG.md with all v0.2.0 changes
- [ ] Run: `cargo run --bin thag_gen_readme`
- [ ] Verify demo/README.md looks correct

### Code Quality (Est. 2-3 hours)
**Note**: Integration tests now use precompiled binary (much faster than before)

#### Workspace Tests
- [ ] `cargo test --workspace` (default features with simplelog)
- [ ] `cargo test --workspace --no-default-features --features env_logger,full`
- [ ] `cargo test --workspace --features tools`
- [ ] `cargo test --workspace --features profiling`
- [ ] Optional: `cargo test --test integration_test -- --test-threads=1` (use 1 thread - script builds can still interfere)

#### Subcrate Tests and Clippy
**thag_common** (quick - minimal features)
- [ ] `cd thag_common && cargo test`
- [ ] `cd thag_common && cargo clippy -- -W clippy::pedantic`

**thag_proc_macros** (quick - no features)
- [ ] `cd thag_proc_macros && cargo test`
- [ ] `cd thag_proc_macros && cargo clippy -- -W clippy::pedantic`

**thag_styling** (test key feature combinations)
- [ ] `cd thag_styling && cargo test --features basic`
- [ ] `cd thag_styling && cargo test --features full`
- [ ] `cd thag_styling && cargo clippy --features full -- -W clippy::pedantic`

**thag_profiler** (test profiling modes)
- [ ] `cd thag_profiler && cargo test`
- [ ] `cd thag_profiler && cargo test --features full_profiling`
- [ ] `cd thag_profiler && cargo clippy --features full_profiling -- -W clippy::pedantic`

**thag_demo**
- [ ] `cd thag_demo && cargo test`
- [ ] `cd thag_demo && cargo clippy -- -W clippy::pedantic`

#### Main Workspace Quality Checks
- [ ] `cargo build --release --workspace`
- [ ] `cargo clippy --all-targets --workspace`
- [ ] `cargo clippy --workspace --no-default-features --features env_logger,core`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `typos`
- [ ] `vale README.md --no-wrap`
- [ ] `vale thag_profiler/README.md --no-wrap`
- [ ] `vale thag_styling/README.md --no-wrap`

### Version Verification
- [ ] All Cargo.toml versions are 0.2.0 (or 0.1.0 for thag_profiler)
- [ ] Main Cargo.toml dependency versions match subcrate versions
- [ ] No path/git dependencies (only version = "x.y.z")
- [ ] MSRV correct: `cargo msrv verify`
- [ ] MSRV in README.md matches

### Package Testing
- [ ] `cd thag_common && cargo package --no-verify`
- [ ] `cd thag_proc_macros && cargo package --no-verify`
- [ ] `cd thag_styling && cargo package --no-verify`
- [ ] `cd thag_profiler && cargo package --no-verify`
- [ ] `cd .. && cargo package --no-verify` (thag_rs)
- [ ] `cd thag_demo && cargo package --no-verify`
- [ ] Review package contents in target/package/
- [ ] Local install test: `cargo install --path . --force`
- [ ] Test installed binary: `thag --version` and basic commands

### Final Prep
- [ ] `find . -name .DS_Store -delete`
- [ ] Review TODO.md, update completed items
- [ ] Git status clean, all changes committed
- [ ] Final commit: `git commit -m "Prepare for v0.2.0 release"`
- [ ] Push: `git push origin main`
- [ ] Create backup: `git tag pre-v0.2.0-backup && git push origin pre-v0.2.0-backup`

---

## Publishing Day

### Phase 1: Foundation Crates (09:00)

**thag_common**
- [ ] `cd thag_common`
- [ ] `cargo publish --no-verify`
- [ ] Wait for confirmation on crates.io
- [ ] Check: https://crates.io/crates/thag_common

**thag_proc_macros** (parallel)
- [ ] `cd ../thag_proc_macros`
- [ ] `cargo publish --no-verify`
- [ ] Wait for confirmation
- [ ] Check: https://crates.io/crates/thag_proc_macros

⏱️ **WAIT 5-10 minutes for indexing**

### Phase 2: Styling (09:40)
- [ ] `cd ../thag_styling`
- [ ] `cargo publish --no-verify`
- [ ] Wait for confirmation
- [ ] Check: https://crates.io/crates/thag_styling

⏱️ **WAIT 5-10 minutes**

### Phase 3: Profiler (10:00)
- [ ] `cd ../thag_profiler`
- [ ] `cargo publish --no-verify`
- [ ] Wait for confirmation
- [ ] Check: https://crates.io/crates/thag_profiler

⏱️ **WAIT 5-10 minutes**

### Phase 4: Main Application (10:20)
- [ ] `cd ..` (back to root)
- [ ] `cargo publish --no-verify`
- [ ] Wait for confirmation
- [ ] Check: https://crates.io/crates/thag_rs

⏱️ **WAIT 5-10 minutes**

### Phase 5: Demo Facade (10:40)
- [ ] `cd thag_demo`
- [ ] `cargo publish --no-verify`
- [ ] Wait for confirmation
- [ ] Check: https://crates.io/crates/thag_demo

### Verify All Published (11:00)
- [ ] All 6 crates visible on crates.io
- [ ] Versions correct (0.2.0 or 0.1.0)
- [ ] READMEs displaying correctly
- [ ] No obvious errors

---

## GitHub Release (11:00)

### Prepare Changelog
- [ ] Generate: `git log v0.1.9..HEAD --pretty=format:"- %s" > /tmp/changelog.txt`
- [ ] Edit /tmp/changelog.txt for release notes

### Create Tag
- [ ] Update tag message with highlights from CHANGELOG.md
- [ ] Create tag:
```bash
git tag v0.2.0 -m "Release v0.2.0

Highlights:
- New thag_styling subcrate with 290+ terminal themes
- New thag_profiler subcrate for graphical profiling
- Enhanced dependency inference
- URL-based script execution
- Improved proc macro support

See CHANGELOG.md for full details."
```
- [ ] Push tag: `git push origin v0.2.0`

### Monitor cargo-dist
- [ ] Watch GitHub Actions: https://github.com/durbanlegend/thag_rs/actions
- [ ] Wait for cargo-dist workflow to complete
- [ ] Check release created automatically

### Edit Release Notes (12:00)
- [ ] Go to: https://github.com/durbanlegend/thag_rs/releases
- [ ] Edit auto-generated release
- [ ] Add curated changelog from CHANGELOG.md
- [ ] Add screenshots if appropriate
- [ ] Highlight breaking changes (if any)
- [ ] Save release

---

## Post-Release Verification

### Installation Tests
- [ ] From GitHub: `cargo install --git https://github.com/durbanlegend/thag_rs --tag v0.2.0 --force`
- [ ] Test: `thag --version` (should show v0.2.0)
- [ ] Test basic functionality
- [ ] Wait 1 hour, then from crates.io: `cargo install thag_rs --force`
- [ ] Test: `thag --version` (should show v0.2.0)

### Verification
- [ ] Download links work (GitHub release assets)
- [ ] docs.rs building: https://docs.rs/thag_rs
- [ ] docs.rs building: https://docs.rs/thag_profiler
- [ ] docs.rs building: https://docs.rs/thag_styling
- [ ] docs.rs building: https://docs.rs/thag_common
- [ ] docs.rs building: https://docs.rs/thag_proc_macros
- [ ] docs.rs building: https://docs.rs/thag_demo

---

## Post-Release Tasks

### Branch Management
- [ ] Merge to develop branch (use staging_temp to avoid backward merge)
- [ ] Clean up any temporary branches

### Monitoring (First 24 Hours)
- [ ] Watch GitHub issues
- [ ] Monitor crates.io downloads
- [ ] Check docs.rs build status
- [ ] Respond to any immediate feedback

### Announcements (Optional - Week 1)
- [ ] Reddit r/rust post (consider timing)
- [ ] This Week in Rust submission
- [ ] Social media (if desired)
- [ ] Update project website (if applicable)

### Next Release Prep
- [ ] Create CHANGELOG.md section for v0.2.1/v0.3.0
- [ ] Review TODO.md
- [ ] Document lessons learned from this release

---

## Emergency Rollback Procedures

### If Critical Bug Found Before GitHub Release
- [ ] Don't create GitHub tag yet
- [ ] Yank problematic versions: `cargo yank --vers 0.2.0 <crate-name>`
- [ ] Fix issue, bump to 0.2.1
- [ ] Republish

### If Critical Bug Found After GitHub Release
- [ ] Remove tag: `git tag -d v0.2.0 && git push origin --delete v0.2.0`
- [ ] Yank versions from crates.io
- [ ] Fix issue
- [ ] Start release process again

### If Minor Bug Found
- [ ] Document in GitHub issue
- [ ] Fix in patch release (0.2.1)
- [ ] No need to yank

---

## Notes

**Timing**: Allow full day for publishing with proper delays
**Backup**: pre-v0.2.0-backup tag created for safety
**Rollback**: Procedures documented above
**Questions**: See RELEASE_PLAN.md for details

**Status**:
- [ ] Pre-Release Complete
- [ ] Published to crates.io
- [ ] GitHub Release Created
- [ ] Post-Release Verified
- [ ] Announcements Made

---

**Release Date**: _____________
**Completed By**: _____________
**Issues Encountered**: _____________
**Time Taken**: _____________
**Notes for Next Time**: _____________
