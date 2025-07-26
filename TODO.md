# TODO List

## High Priority

## On the go
- [ ]  Think of ways to run thag remotely or with minimal effort.
- [ ]  Demo readmes: Give thag_url alternative options for crate demos - test first of course.
- [ ]  Tool for comparing / ?merging? line ranges of different files, or clipboard paste to line range of file.
- [ ]  Tool for running tests for all feature sets?
- [ ]  Consider thag --altedit(-D) option to use built-in editor, and/or key option to open TextArea in better editor.
- [ ]  ?Use curl to download a compiled binary of a profiling demo.
- [ ]  Thag tool for invoking thag as a library and running a remote source file.
- [ ]  "Playground" naming. Multifaceted playground.
- [ ]  Upgrade demo graphs headers to be same quality as thag_profile.
- [ ]  If thag or thag_demo doesn't find demo scripts, offer to install them?. Make the logic in src/bin/thag_get_demo_dir.rs and demo/download_demos.rs a library function (where?) or a proc macro.
- [ ]  Give file_navigator can't find typed.

To find snippets with many functions:
grep -c fn demo/*.rs | egrep -v ':0' | egrep -v ':1$' | grep -v '2' | sort -t: -k2rn,2rn | while read x; do sed 's/:/ /'; done | while read f n; do grep -L "fn main" $f; done

Syntax for changing background color using OSC:
echo -e "\033]1337;SetColors=bg=f4f4f4\a"

curl -sL https://raw.githubusercontent.com/durbanlegend/thag_rs/main/thag_demo/install_and_demo.sh | bash

thag --loop 'if line.len() > 3 { count += 1; true } else { false }' --begin 'let mut count = 0;' --end 'println!("Total: {}", count);' --toml '[dependencies]
regex = "1.0"'

thag -vv -B 'let mut max = 0; let mut longest = String::new();' -l '{let l = line.len(); if l > max { max = l; longest = line.to_string(); true } else { false }}' -E 'println!("Longest line is: {longest} of length {max}");' < demo/hello.rs

thag -vv -B 'let mut min = usize::MAX; let mut shortest = String::new();' -l '{let l = line.len(); if l < min { min = l; shortest = line.to_string(); true } else { false }}' -E 'println!("shortest line is: {shortest} of length {min}");' < demo/hello.rs

cargo run -- -vv --loop 'let gt = if line.len() > 3 { count += 1; true } else { false }; let _ = writeln!(io::stdout(), "{gt}");' --begin 'let mut count = 0;' --end 'println!("Total: {}", count);' < demo/hello.rs

# Alternative ways to run thag_instrument without installing:
cargo run -p thag_profiler --features=instrument-tool --bin thag_instrument -- 2021 < bank/main_with_attrs.rs
cargo run --features=instrument-tool --bin thag_instrument --manifest-path thag_profiler/Cargo.toml -- 2021 < bank/main_with_attrs.rs

# Alternative ways to run thag_profile without installing:
cargo run -p thag_profiler --features=analyze-tool --bin thag_profile -- .

cd thag_profiler
# cargo test --test profiling --features full_profiling
cargo test --lib
cargo test --package thag_profiler --test test_profiled_behavior

THAG_PROFILER=both,,announce cargo test --package thag_profiler --test test_profiled_behavior --features=full_profiling -- --nocapture

  cargo test --features=full_profiling logging::tests::test_logging_functionality -- --nocapture

cargo test --features=analyze-tool,time_profiling errors::tests  -- --nocapture

Worked example: TODO replace: serde
Don't use a crate that is called by other dependencies, otherwise there may be conflicts.
1. Clone repo
2. cd /home/donf/Documents/GitHub/serde
3. find . -name "*.rs"
4. d=./serde/src/de
4. find $d -name '*.rs' -exec sh -c 'temp=$(mktemp) && thag_instrument 2021 < "$1" > "$temp" && mv "$temp" "$1"' sh {} \;
5. Repeat for /serde/src/ser (& could do ./serde_derive/src if change serde_derive dep to here in ./serde/Cargo.toml and add thag_profiler as dep in serde_derive's local Cargo.toml )
5a. Undo for lib.rs
6. Add to ./serde/Cargo.toml [TODO update when published to crates.io ] thag_profiler = { path = "/home/donf/Documents/GitHub/thag_rs/thag_profiler", features = ["full_profiling"] }
7. Do same for demo/crokey_deser_profile.rs
8. Change toml path to local, e.g. `serde = { path = "/home/donf/Documents/GitHub/serde/serde", features = ["derive"] }`
9. thag demo/crokey_deser_profile.rs -ft

> Great explanation. What are the implications of replacing the thread_local IN_TRACKING with a simple static mutable bool variable in your point 3?


At its very simplest, a single attribute on your `fn main` will generate a flamegraph of all the memory allocations, by function, made by your running project and its dependencies. Add `thag_profiler` to your project with the `full_profiling` feature, add `use thag_profiler::*;` to your imports, and the `#[enable_profiling(runtime)]` attribute to your main method. Then run your project with the environment variable `THAG_PROFILER=both,,announce,true`. This will default to generating .folded files to your current directory. On conclusion, run `thag_profile .`, select `analysis type: Memory Profile - Single`, choose your project and then the timestamped `-memory_detail.folded`, and finally `Show Aggregated Memory Profile (Flamegraph)` to generate the detailed `inferno` flamegraph and show it in your default browser.

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
- [ ]  Add profiling to capabilities for scripts.
- [ ]  Note possible confusion between thag --edit (uses tui editor) vs REPL edit (uses custom editor)
- [ ]  Consider script to reverse-engineer xterm OSC sequences.

- [ ]  Upgrade all cargo.tomls

- [ ]  "Thoughts of Thag" command to spew out random stone-age Thaggisms.
- [ ]  Update Readme for new features.
- [ ]  Offer thag updates as a menu option.
- [ ]  Other front-ends: thag_test: call thag with dethag of bad output - rather fix bad output at source - done?
- [ ]  Use cargo-dist to build and distribute thag front-end commands.- [ ]  Documentation (cargo doc), e.g. for macros.
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
- [ ]  Sort and flesh out keywords (u16 etc) in shared::is_valid_crate_name. (make HashSet? phf?)


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

- [ ]  Theme config: for Windows:
        1. Check supports_color detection failing.
        2. Need to detect term_bg_rgb rather than or in addition to term_bg_luma.
        "Still, the crux of this bug: if COLORTERM is meant to detect color support - though what the value is set to
         doesn't seem well-defined - perhaps the more oft-supported TERM=xterm-256color is appropriate here for WT
         while COLORTERM=xterm-truecolor is appropriate in addition. TERM seems to be more general-purpose from various
         reads, while COLORTERM seems to be more specific to color support, as the name also implies."
         https://github.com/microsoft/terminal/issues/11057
- [ ]  Demo proc macro to load collection into enum at build time?
- [ ]  Add a thag feature to apply a git patch to a dependency? Consider adding pre-processing to toml block with support for variables.
- [ ]  Consider removing Peak from summary flamegraphs and flamecharts due to inaccuracy?


## Low Priority
- [ ]  Add additional popular crates
- [ ]  Paste event in Windows slow or not happening?
- [ ]  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
- [ ]  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
- [ ]  Clap aliases not working in REPL.
- [ ]  Simple demo https server
- [ ]  Conversion of Gogh themes
- [ ]  Claude re REPL alternative editor to `reedline`
- [ ]  Control logging level of -x compiled code?
- [ ]  Profiling instrumentation to add toml block for thag profiling?
- [ ]  Profiling: provide an option in instrumentation for conditional instrumentation.

## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.

##  Checklist for making releases:
- [ ] Tip: disable ci.yml for Readme & similar tweaks that won't affect compilation.
- [ ] NB NB. Remember to update Cargo.toml version to the required release before tagging.
- [ ] Do a trial release build locally to check for anomalies: cargo build --release --workspace
- [ ] Don't upgrade thag versions in demo scripts to new release, because you get a catch-22 until it's on crates.io. If you absolutely need to, wait until you've released to crates.io a first time, then release all over again.
- [ ] cargo doc --features document-features --no-deps (thag_rs)
- [ ] cargo doc --features document-features,full_profiling,debug_logging --no-deps
 (thag_profiler public)
 [Internal API: cargo doc --features document-features,full_profiling,debug_logging,internal_docs --no-deps
]
[Comprehensive: cargo doc --features document-features,full_profiling,debug_logging,internal_docs --no-deps --document-private-items
]
- [ ] Optional: reinstall thag_rs from path. (cargo install --path .)
- [ ] Make sure Readme images are up to date.
- [ ] Run clippy_feature_tests.sh
- [ ] Run cargo tests
- [ ] Run `gen_readme`
- [ ] Run `typos` command.
- [ ] Run `vale README.md --no-wrap` and `vale demo/README.md --no-wrap`.
- [ ] Run `cargo msrv set/verify`, and update the MSRV in README.md.
- [ ] Check on https://deps.rs/repo/github/durbanlegend/thag_rs that all dependencies are up to date
      (can link from badge at top of README.md).
- [ ] Once you're happy that you've tested all your script changes successfully with CI.yml,
      update all bank and demo scripts using thag to use latest release instead of develop branch if appropriate.
- [ ] NB NB: If there have been any changes to thag_proc_macros or thag_profiler since their last published releases, bump their version numbers
      in their respective Cargo.tomls and also in their dependency entries in the main Cargo.toml. as these will be used in the crates.io version
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


export feature_sets=(
    "demo,"
    "demo,debug_logging"
    "demo,full_profiling"
    "demo,full_profiling,debug_logging"
    "demo,time_profiling"
    "demo,time_profiling,debug_logging"
    ""
    "debug_logging"
    "full_profiling"
    "full_profiling,debug_logging"
    "time_profiling"
    "time_profiling,debug_logging"
)

failures=()

for f in "${feature_sets[@]}"; do
    echo
    echo "===> Running: cargo clippy --features=${f:-<none>} ..."
    if ! cargo clippy --features="$f" -- -W clippy::pedantic -W clippy::nursery; then
        echo "ERROR: cargo clippy failed for feature set '${f:-<none>}'"
        failures+=("$f")
    fi
done

feature_sets=(
    ""
    "core"
    "build"
    "ast"
    "tui"
    "repl"
    "full"
    "default"
)

for f in "${feature_sets[@]}"; do
    echo
    echo "===> Running: cargo test --lib --features=${f:-<none>} ..."
    if ! cargo test --lib --features="$f"; then
        echo "ERROR: cargo test --lib failed for feature set '${f:-<none>}'"
        failures+=("$f")
    fi
done

no_default_feature_sets=(
    "env_logger,core"
    "env_logger,build"
    "env_logger,ast"
    "env_logger,tui"
    "env_logger,repl"
    "env_logger,full"
)

for f in "${no_default_feature_sets[@]}"; do
    echo
    echo "===> Running: cargo test --lib --features=${f:-<none>} ..."
    if ! cargo test --lib --no-default-features --features="$f"; then
        echo "ERROR: cargo test --lib failed for feature set '${f:-<none>}'"
        failures+=("$f")
    fi
done

echo
if [ ${#failures[@]} -eq 0 ]; then
    echo "All feature sets passed successfully"
    # exit 0
else
    echo "The following feature sets FAILED:"
    for f in "${failures[@]}"; do
        echo "  - ${f:-<none>}"
    done
    # exit 1
fi


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
