# TODO List

## High Priority
- [ ]  Sort and flesh out keywords (u16 etc) in shared::is_valid_crate_name. (make HashSet? phf?)

## On the go
- [ ]  Theme config: for Windows:
        1. Check supports_color detection failing.
        2. Need to detect term_bg_rgb rather than or in addition to term_bg_luma.
        "Still, the crux of this bug: if COLORTERM is meant to detect color support - though what the value is set to
         doesn't seem well-defined - perhaps the more oft-supported TERM=xterm-256color is appropriate here for WT
         while COLORTERM=xterm-truecolor is appropriate in addition. TERM seems to be more general-purpose from various
         reads, while COLORTERM seems to be more specific to color support, as the name also implies."
         https://github.com/microsoft/terminal/issues/11057
- [ ]  Demo proc macro to load collection into enum at build time?
- [ ]  Add a thag feature to apply a git patch to a dependency? Consider adding
        pre-processing to toml block with support for variables.
- [ ]  Claude re REPL alternative editor to `reedline`
- [ ]  Control logging level of -x compiled code?
- [ ]  Profiling instrumentation to add toml block for thag profiling?
- [ ]  Profiling: provide an option in instrumentation for conditional instrumentation.
- [ ]  Remove writing of op (+/-) in write_memory_event_with_op, i.e. go back to write_memory_event. Not necessary.
- [ ]  Worked example - e.g. syn as dependency - no need to do this with dependencies now that we have detailed profiling.
- [ ]  profile_type arg on enable_profiling now redundant
- [ ]  Docs: should be able to #[enable_profiling] on any 1 function, and all profiled functions should be included.
       If the decorated function is not the root of all the profiled functions, then the root will be the standard "all".
- [ ]  lazy_static variable in #[enable_profiling] using backtrace to establish root. For the programmatic call to profiling::enable_profiling,
look for an alternative or cater for and put up with the overhead of not having this baseline.
- [ ]  Perhaps store async and ?method rather than desc_fn_name.
- [ ]  Debug Esc at all places in thag-analyze.
- [ ]  Consider dropping programmatic enable_profiling since only the attribute macro can run profiling code in the system allocator.
- [ ]  Shorten profile_type arg of #[profiled] to same format as for #[enable_profiling] - or rather #[profile]?
- [ ]  Re-check for profiler code not ring-fenced
- [ ]  Consider option for deallocation in detail.
- [ ]  Unbounded profiles must only go out of scope at the end of the _function_.
- [ ]  Debug thag repl: "b" == "b" || "b" == "b" not evaluated but "a" == "a" || "a" == "b" is.

I'm trying to figure out further unit testing of thag_profiler, including thag_proc_macros modules enable_profiling, profile and profiled. thag_proc_macros::lib has a fn maybe_expand_proc_macro<F> that

Not sure about this...
Currently the Profile registry uses the call stack from a backtrace as the key to identify and store a function Profile. This makes for a long key but is appropriate because a Profile represents one execution of a function (or section), which was called from a particular callstack (think in terms of a  backtrace) in which each entry represents a location within the span of some function that may or may not be profiled. But I would also really like a way to store the static attributes of the function, such as name, async, generics and end line, according to its span in the source file, so that when a Profile is dropped and writes out its call stack it can look up each element in its stack by file and start & end line numbers and write out the async:: prefix (or in future generics) to the .folded file. (This would also mean having to store file names and line numbers from the callstack in the Profile's path). I think file! or module_path! and start & end line numbers would be ideal. Would it be possible for an attribute macro such as #[profiled] (source attached) be able to get the line span of the function for which it constructs a profile? I'm seeing that `syn` has proc_macro2::Span, and proc_macro2 has LineColumn.line applicable to a Span.

In addition to storing each profile in the ProfileRegistry by its path (which I don't seek to change) I'd like to be able to store additional attributes - async for starters - by source file and start line number, so that when writing out the path to the .folded  files, the Profile can just

# Alternative ways to run thag-instrument without installing:
cargo run -p thag_profiler --features=instrument-tool --bin thag-instrument -- 2021 < bank/main_with_attrs.rs
cargo run --features=instrument-tool --bin thag-instrument --manifest-path thag_profiler/Cargo.toml -- 2021 < bank/main_with_attrs.rs

# Alternative ways to run thag-analyze without installing:
cargo run -p thag_profiler --features=analyze-tool --bin thag-analyze -- .

cd thag_profiler
cargo test --test profiling --features full_profiling
cargo test --lib

Worked example: TODO replace: serde
Don't use a crate that is called by other dependencies, otherwise there may be conflicts.
1. Clone repo
2. cd /home/donf/Documents/GitHub/serde
3. find . -name "*.rs"
4. d=./serde/src/de
4. find $d -name '*.rs' -exec sh -c 'temp=$(mktemp) && thag-instrument 2021 < "$1" > "$temp" && mv "$temp" "$1"' sh {} \;
5. Repeat for /serde/src/ser (& could do ./serde_derive/src if change serde_derive dep to here in ./serde/Cargo.toml and add thag_profiler as dep in serde_derive's local Cargo.toml )
5a. Undo for lib.rs
6. Add to ./serde/Cargo.toml [TODO update when published to crates.io ] thag_profiler = { path = "/home/donf/Documents/GitHub/thag_rs/thag_profiler", features = ["full_profiling"] }
7. Do same for demo/crokey_deser_profile.rs
8. Change toml path to local, e.g. `serde = { path = "/home/donf/Documents/GitHub/serde/serde", features = ["derive"] }`
9. thag demo/crokey_deser_profile.rs -ft

> Great explanation. What are the implications of replacing the thread_local IN_TRACKING with a simple static mutable bool variable in your point 3?

⏺ Replacing the thread-local IN_TRACKING with a static mutable bool has important implications:

  1. Thread safety: Static mutable variables aren't thread-safe. Multiple threads could access/modify it simultaneously, potentially causing race conditions. This works in your case because:
    - The recursion prevention is per-thread in the allocator
    - Each thread's allocation path is independent
  2. Global state vs thread-local state:
    - Before: Each thread had its own independent tracking flag
    - Now: All threads share one tracking flag
  3. Potential false positives: If two threads allocate memory simultaneously, one might set the flag and block the other thread's tracking incorrectly
  4. Performance: No more TLS lookup overhead, but potential contention between threads

  This approach works for simple use cases but might need atomic operations or a mutex for complete thread safety in a high-concurrency environment. For your specific use case, it's likely
  sufficient since allocator code runs with minimal thread contention, and recursion detection is primarily about preventing infinite recursion within the same thread.

At its very simplest, a single attribute on your `fn main` will generate a flamegraph of all the memory allocations, by function, made by your running project and its dependencies. Add `thag-profiler` to your project with the `full_profiling` feature, add `use thag_profiler::*;` to your imports, and the `#[enable_profiling(runtime)]` attribute to your main method. Then run your project with the environment variable `THAG_PROFILE=both,,announce,true`. This will default to generating .folded files to your current directory. On conclusion, run `thag-analyze .`, select `analysis type: Memory Profile - Single`, choose your project and then the timestamped `-memory_detail.folded`, and finally `Show Aggregated Memory Profile (Flamegraph)` to generate the detailed `inferno` flamegraph and show it in your default browser.


Tools classification suggestions from Claude

### Format & Conversion
- `format_dethagomize.rs` (from dethagomizer.rs) - Converts escaped text to readable format
- `convert_rust_script_to_thag.rs` (from thag_from_rust_script.rs)
- `convert_thag_to_rust_script.rs` (from thag_to_rust_script.rs)
- `convert_theme.rs` (from theme_converter.rs)

### Config & Setup
- `config_build.rs` (from thag_config_builder.rs)
- `setup_download_demos.rs` (from download_demo_dir.rs)

### Profiling
- `profile_instrument.rs` (from profile_instr.rs)
- `profile_remove.rs` (stays the same)
- `profile_analyze.rs` (from thag_profile.rs)

### Documentation & Generation
- `gen_readme.rs` (stays the same)
- `gen_error.rs` (from error_builder.rs)
- `filter_demos.rs` (stays the same)

### Frontend & UI
- `ui_cargo.rs` (from thag_cargo.rs)
- `ui_clippy.rs` (from thag_clippy.rs)
- `ui_url.rs` (from thag_url.rs)
- `ui_theme.rs` (from theme_helper.rs)

### Debugging & Analysis
- `debug_expr_to_ast.rs` (from input_expr_to_ast.rs)
- `debug_file_to_ast.rs` (from input_file_to_ast.rs)
- `analyze_terminal.rs` (from term_detection_pack.rs)


## Medium Priority
- [ ]  More unit and integration tests. Identify new functions requiring unit tests.
- [ ]  Consider releasing a copy of repl.rs as a demo script.
- [ ]  Raise clear_screen as an issue on supports-color crate?
- [ ]  Config option for formatting main?
- [ ]  Config option for stdin -d highlighting preference, like repl.rs
- [ ]  Config loading warn when defaulting to ../assets etc.
         NB: document that user should save it under ~/.config.
         Check if thag_config_builder does so, also thag -C.
- [ ]  Add conversions to and from `runner` and `cargo-script-mvs`.
- [ ]  Implement deletion of current history line with function key.
- [ ]  Look for any functions that can run at compile time.
- [ ]  Make key_handler a trait method?
        trait KeyHandler {
          fn handle_keys(
              key_event: KeyEvent,
              maybe_term: Option<&mut ManagedTerminal>,
              textarea: &mut TextArea,
              edit_data: &mut EditData,
              popup: &mut bool,
              saved: &mut bool,
              status_message: &mut String,
          ) -> ThagResult<KeyAction>;
        }
        struct ScriptContent;
        impl KeyHandler for ScriptContent {
            // (Current script_key_handler)
            fn handle_keys(
              key_event: KeyEvent,
              maybe_term: Option<&mut ManagedTerminal>,
              textarea: &mut TextArea,
              edit_data: &mut EditData,
              popup: &mut bool,
              saved: &mut bool,
              status_message: &mut String,
          ) -> ThagResult<KeyAction>;
        }
- [ ]  Add FAQ? See Usage notes in Readme.
- [ ]  Try pre-building colour mappings
- [ ]  New test for local paths in demo files and maybe even main Cargo.toml.
- [ ]  Try ThagDisplay trait and derive macro.
- [ ]  Embed dirs
- [ ]  Debug some bad crate names intermittently getting into demo/Readme.md such as xterm and self.
- [ ]  In cargo search, optionally get all features. Config option to omit unstable features.
        Add feature overrides config option default-features true/false
        Update thag_config_builder to accept dependencies inference level and default features, as well as Option<> booleans.
- [ ]  Debug: No history edit function in stdin.
- [ ]  REPL history belongs in /Users/donf/.cargo/hist_staging.txt and stdin history in /Users/donf/.cargo/rs_stdin_history.json
         (check both).
- [ ]  >>> Debug: History older than max isn't being removed even though only 25 allowed.
- [ ]  Picking up "panic", "bool", "fs" in dependency inference.
Try running tests without debug or debug_timings.
validate_state only when feature minimal not engaged - instead switched off debug and debug-assertions in Cargo.toml

- [ ]  Consider adding --guided (-G) option or a helper command like thag_url using `inquire` to capture parameters.
- [ ]  Consider "magic" substitution of latest git with say rev = "$latest" in toml block.
- [ ]  Consider a disable option?
- [ ]  Add details of --cargo (-A) option to Readme and `thag_cargo`
       - Make --expand (-X) option a helper command thag_expand.
       - Document thag_cargo and thag_clippy in the Readme.
- [ ]  Add profiling to capabilities for scripts.
- [ ]  Note possible confusion between thag --edit (uses tui editor) vs REPL edit (uses custom editor)
- [ ]  Migrate Level to Role and decommission.
- [ ]  Consider script to reverse-engineer xterm OSC sequences.

- [ ]  Upgrade all cargo.tomls

- [ ]  "Thoughts of Thag" command to spew out random stone-age Thaggisms.
- [ ]  Update Readme for new features.
- [ ]  Offer thag updates as a menu option.
- [ ]  Other front-ends: thag_test: call thag with dethag of bad output - rather fix bad output at source - done?
- [ ]  Use cargo-dist to build and distribute thag front-end commands.
- [ ]  Next: thag_expand, thag_prompt, proc macro to expose docs at runtime.
- [ ]  Documentation (cargo doc), e.g. for macros.
- [ ]  Incorporate const_gen_proc_macro into thag_rs and try to enhance?
- [ ]  ?Adapt keys display to environment: Cmd only for target macos. Or just leave it because informative?
- [ ]  Try going back to derive macro with declarative macro to expand the attributes. Problem with attrib macro is the AST isn't
        editable in the way we want, it just points to spans that get resolved later. See bank/syn_parse_mappings.rs for nice format;
- [ ]  Decide whether to decommission REPL delete function - keep list
- [ ]  Test [profile.dev] optimisation level
- [ ]  Check dead code & other #(!)[allow(...)]; look into factoring over-long gen_build_run
- [ ]  Look for code smells
- [ ]  Look into fuzzing the tests such as test_merge_manifest.
- [ ]  Consider dropping our termbg module if termbg 0.6.1 is working fine.
- [ ]  Testing fib scripts: ls -1 demo/fib_*.rs | grep -v basic | grep -v binet | while read f; do echo $f;  cargo run -- $f -qq -- 100 | grep 354224848179261915075 || echo "...failed"; done
stem=macro_lazy_static_var_advanced
stem=macro_lazy_static_var_errs
r=$stem.rs
p=demo/$r
find $TMPDIR -name $stem 2>/dev/null
d=...
f=$d/Cargo.toml
cargo expand --bin $stem --manifest-path=$f --theme=gruvbox-dark | sdiff $p - | less

### Testing without ColorSupport::None
thag -C -> change
env NO_COLOR=1 cargo run --no-default-features --features="repl,simplelog" -- -r


## Low Priority
- [ ]  Add additional popular crates
- [ ]  Paste event in Windows slow or not happening?
- [ ]  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
- [ ]  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
- [ ]  Clap aliases not working in REPL.
- [ ]  Simple demo https server
- [ ]  Conversion of Gogh themes

## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.

##  Checklist for making releases:
- [ ] Tip: disable ci.yml for Readme & similar tweaks that won't affect compilation.
- [ ] NB NB. Remember to update Cargo.toml version to the required release before tagging.
- [ ] Do a trial release build locally to check for anomalies: cargo build --release --workspace
- [ ] Don't upgrade thag versions in demo scripts to new release, because you get a
    catch-22 until it's on crates.io. If you absolutely need to, wait until you've
    released to crates.io a first time, then release all over again.
- [ ] Optional: reinstall thag_rs from path. (cargo install --path .)
- [ ] Make sure Readme images are up to date.
- [ ] Run clippy_feature_tests.sh
- [ ] Run cargo tests
- [ ] Run `gen_readme`
- [ ] Run `typos` command.
- [ ] Run `vale README.md --no-wrap` and `vale demo/README.md --no-wrap`.
- [ ] Run `cargo msrv find`, and update the MSRV in README.md.
- [ ] Check on https://deps.rs/repo/github/durbanlegend/thag_rs that all dependencies are up to date
      (can link from badge at top of README.md).
- [ ] Once you're happy that you've tested all your script changes successfully with CI.yml,
      update all bank and demo scripts using thag to use latest release instead of develop branch if appropriate.
- [ ] NB NB: If there have been any changes to thag_proc_macros since its last published release, bump its version number
      in src/proc_macros/Cargo.toml and also in its dependency entry in the main Cargo.toml. as this will be used in
      the crates.io version
- [ ] Use 'git changelog v0.1.<n-1>..HEAD' to generate raw release notes.
- [ ] Leave it to cargo-dist to make the release.
- [ ] To trigger cargo-dist:
    cargo dist init  # In case e.g. package description in Cargo.toml has changed.
    git tag v0.1.n -m "<Summary>"
    git push --tags
- [ ] To revoke and redo:
    git tag -d v0.1.n
    git push origin --delete v0.1.n
    Tag again as above when ready.
- [ ] Don't override release.yml, e.g. to try to add a workflow dispatch, as it's generated by cargo-dist.
- [ ] Edit the release notes generated by cargo-dist on Github and add in
    own change log, edited as required from raw changelog output above.
- [ ] Reinstall thag_rs from tag. (cargo install --git https://github.com/durbanlegend/thag_rs --tag v0.1.<n>)
### `Publishing to crates.io`
- [ ] Suggest give it a day to settle before publishing to crates.io.
- [ ] First publish the new version of src/proc_macros if applicable, same steps as below.
- [ ] Before publishing, dry run installation with `cargo install --path /Users/donf/projects/thag_rs/`
- [ ] First: `find . -name .DS_Store -delete`
- [ ] Test with `cargo package --no-verify`
- [ ] Publish for real: `cargo publish --no-verify`
- [ ] Reinstall updated thag_rs with cargo install.
- [ ] Keep develop branch around and bring it up to date with main branch changes such as version number in Cargo.toml
        Use a temp staging branch like staging_temp, otherwise it will merge backwards into main for some reason while creating
        the pull request.


1. **Phase 1: Parallel Structure**
   ```rust
   pub mod styling {
       // Existing code
       pub enum Level { ... }
       pub fn basic_light_style(level: Level) -> TermAttributes { ... }

       // New code (maybe in submodule?)
       pub enum MessageType { ... }
       pub enum Theme { ... }
       // ... new theme structure
   }
   ```
   - Keep existing functionality intact
   - Introduce new types without breaking changes
   - Map between old Levels and new MessageTypes

2. **Phase 2: Theme Implementation**
   - Implement the new theme system
   - Create BasicLight/Dark themes that mirror current behavior
   - Add conversion/compatibility layer:
   ```rust
   impl From<Level> for MessageType {
       fn from(level: Level) -> Self {
           match level {
               Level::Error => MessageType::Error,
               // ...
           }
       }
   }
   ```

3. **Phase 3: Gradual Migration**
   ```rust
   pub fn basic_light_style(level: Level) -> TermAttributes {
       // Use new theme system internally
       let theme = Theme::BasicLight(default_config());
       let msg_type: MessageType = level.into();
       theme.style_for(msg_type).into()
   }
   ```
   - Keep old API but use new implementation
   - Add deprecation notices
   - Document migration path for users

4. **Phase 4: New API**
   - Introduce new public API
   - Mark old API as deprecated
   - Provide migration guide

5. **Phase 5: Cleanup**
   - Remove old API in next major version
   - Complete documentation
   - Finalize theme implementations

Key Considerations:
- How to handle TermAttributes conversion
- Maintaining color support detection
- Terminal background detection
- Configuration options

Chat: Git Latest Commit Cargo Dependencies resume thag_core

As for general suggestions I've prepared:

1. "Wow" Feature Ideas:
   - "Time Machine" debugging: Integrate profiling with state tracking
   - Interactive dependency visualization
   - Smart script templates based on usage patterns
   - Real-time script optimization suggestions

2. Development Experience:
   - Simplified script development workflow
   - Better error messages for common script issues
   - More intuitive CLI interface

3. Documentation:
   - Clear separation of core vs full functionality
   - Better examples of when to use each feature
   - Migration guide for existing users

Would you like me to:
1. Detail any specific part of these suggestions?
2. Show example implementations?
3. Discuss potential challenges?
