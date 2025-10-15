# TODO List

## High Priority

## On the go
- [ ]  Mention `thag_demo` in thag README.md. Mention "co-releases".

Ref 0042 5193

printf "\x1b]4;{};?\x07"
printf "\x1b]4;01;?\x07"
Mintty:
^[]7704;index;?^G
printf "\x1b]7704;01;?\x07"

terminal_sample="\x1b[38;5;15m████\x1b[0m"
thag_display="\x1b[38;2;91;116;116m████ #5b7474 ( 91,116,116)\x1b[0m"

# Query mintty ANSI slot 0–15 and print fg bg hex
mintty_color() {
  local idx=$1 resp colors out=()

  stty raw -echo < /dev/tty
  printf '\033]7704;%d;?\a' "$idx" > /dev/tty
  IFS= read -r -d $'\a' -t 1 resp < /dev/tty || true
  stty sane < /dev/tty

  # resp looks like: ^[]7704;rgb:5c5c/3f3f/1515;rgb:xxxx/yyyy/zzzz
  colors=$(echo "$resp" | sed -E 's/.*7704;//; s/^[0-9]+;//; s/\x1b.*//')

  for c in ${colors//;/ }; do
    if [[ $c =~ rgb:([0-9a-fA-F]+)/([0-9a-fA-F]+)/([0-9a-fA-F]+) ]]; then
      # take the first two hex digits of each component
      out+=("#${BASH_REMATCH[1]:0:2}${BASH_REMATCH[2]:0:2}${BASH_REMATCH[3]:0:2}")
    fi
  done

  if ((${#out[@]})); then
    echo "${out[@]}"
  else
    echo "No match: $resp"
    return 1
  fi
}

mintty_color 2
# → #5c3f15   (or two colors if fg/bg differ)

And to see the whole 0–15 palette:

for i in {0..15}; do
  printf '%2d: %s\n' "$i" "$(mintty_color $i)"
done


https://github.com/base16-project/base16

donf@MacBook-Air thag_rs % thag bank/styling.rs -fb
[src/bin/thag_rs.rs:32:5]
[src/bin/thag_rs.rs:34:5]
cargo_manifest_dir=Err(NotPresent)
[src/bin/thag_rs.rs:36:5]
env::current_dir()=Ok("/Users/donf/projects/thag_rs")
[src/bin/thag_rs.rs:38:5]

Fix most recent git commit message:

git commit --amend -m "Your new, corrected commit message"
git push --force origin develop

To find snippets with many functions:
grep -c fn demo/*.rs | egrep -v ':0' | egrep -v ':1$' | grep -v '2' | sort -t: -k2rn,2rn | while read x; do sed 's/:/ /'; done | while read f n; do grep -L "fn main" $f; done

Syntax for changing background color using OSC:
All (iterm2, Wezterm, terminal):
bg=24273a
printf "\x1b]11;#$bg\x1b\\"
Reset: printf "\x1b]104;\x1b\\"

bg=1c2023
fg=c7ccd1
bg=262626
fg=333366
bg=000000; fg=c6c6c6
fg=cccccc, bg=1c2023
printf '\x1b]10;#c7ccd1\x07\x1b]11;#1c2023\x07'
printf "\x1b]10;#${fg}\x07\x1b]11;#${bg}\x07"
printf "\x1b]10;#${fg}\x07\\x1b]12;#${fg}\x07\x1b]11;#${bg}\x07\x1b[0 q\07"
# Set cursor color (code 12) and cursor to block ([0 q)
printf "\x1b]12;#${fg}\x07\x1b[0 q"

# Set cursor visible
printf "\x1b[?25h"

    eprintln!("\tfg={fg}, bg={bg}");
    eprintln!(
        r#"
        OSC string=
        printf "\x1b]10;{fg}\x07\x1b]11;{bg}\x07""#
    );

base_16_gruvbox_light_hard

iterm only:
bg=f9f5d7 # Gruvbox light hard
bg=24273a # Catppuccin Macchiato
echo -e "\033]1337;SetColors=bg=$bg\a"

# iterm2 change prompt
autoload -Uz vcs_info
precmd() { vcs_info }

zstyle ':vcs_info:git:*' formats '%b '

setopt PROMPT_SUBST
PROMPT='%F{green}%*%f %F{blue}%~%f %F{red}${vcs_info_msg_0_}%f$ '


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
- [ ]  Raise clear_screen as an issue on supports-color crate?
- [ ]  DROP: Config option for formatting main?
- [ ]  Config option for stdin -d highlighting preference, like repl.rs
- [ ]  DROP: Config loading warn when defaulting to ../assets etc.
         NB: document that user should save it under ~/.config.
         Check if thag_config_builder does so, also thag -C.
- [ ]  Implement deletion of current history line with function key.
- [ ]  Look for any functions that can run at compile time.
- [ ]  DROP: Make key_handler a trait method?
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
- [ ]  OBSOLETE: Try pre-building colour mappings
- [ ]  New test for local paths in demo files and maybe even main Cargo.toml.
- [ ]  Try ThagDisplay trait and derive macro.
- [ ]  Embed dirs
- [ ]  DROP: Debug some bad crate names intermittently getting into demo/Readme.md such as xterm and self.
- [ ]  DROP: In cargo search, optionally get all features. Config option to omit unstable features.
        Add feature overrides config option default-features true/false
        Update thag_config_builder to accept dependencies inference level and default features, as well as Option<> booleans.
- [ ]  Debug: No history edit function in stdin.
- [ ]  REPL history belongs in /Users/donf/.cargo/hist_staging.txt and stdin history in /Users/donf/.cargo/rs_stdin_history.json
         (check both).
- [ ]  >>> Debug: History older than max isn't being removed even though only 25 allowed.
- [ ]  Picking up "panic", "bool", "fs" in dependency inference.
Try running tests without debug or debug_timings.
validate_state only when feature minimal not engaged - instead switched off debug and debug-assertions in Cargo.toml

- [ ]  Consider "magic" substitution of latest git with say rev = "$latest" in toml block.
- [ ]  Consider a disable option?
- [ ]  Add details of --cargo (-A) option to Readme and `thag_cargo`
- [ ]  DROP: Add profiling to capabilities for scripts.
- [ ]  OBSOLETE: Note possible confusion between thag --edit (uses tui editor) vs REPL edit (uses custom editor)
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
- [ ]  RYO cargo-lookup?.
- [ ]  Think of ways to run thag remotely or with minimal effort.
- [ ]  Demo readmes: Give thag_url alternative options for crate demos - test first of course.
- [ ]  Tool for comparing / ?merging? line ranges of different files, or clipboard paste to line range of file.
- [ ]  Tool for running tests for all feature sets?
- [ ]  Consider thag --altedit(-D) option to use built-in editor, and/or key option to open TextArea in better editor.
- [ ]  ?Use curl to download a compiled binary of a profiling demo.
- [ ]  Thag tool for invoking thag as a library and running a remote source file.
- [ ]  Upgrade demo graphs headers to be same quality as thag_profile.
- [ ]  Consider a tool to show the current theme and switch via OSC?
- [ ]  ?Improve filtering algo in thag_demo browse (inquire Scorer).
- [ ]  Don't check features in crates.io when using local or git version of thag_rs.

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
- [ ]  If thag or thag_demo doesn't find demo scripts, offer to install them?. Make the logic in src/bin/thag_get_demo_dir.rs and demo/download_demos.rs a library function (where?) or a proc macro.
- [ ]  Feature-gated impls of styling integration for owo-colors and nu_ansi_term in thag_styling ... others?


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
- [ ]  Add further attributes such as reversed to Style?
- [ ]  Add conversions to and from `runner` and `cargo-script-mvs`.


## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.

##  Checklist for making releases:
- [ ] Tip: disable ci.yml for Readme & similar tweaks that won't affect compilation.
- [ ] NB NB. Remember to update Cargo.toml version to the required release before tagging.
- [ ] Do a trial release build locally to check for anomalies: cargo build --release --workspace
- [ ] Don't upgrade thag versions in demo scripts to new release, because you get a catch-22 until it's on crates.io. If you absolutely need to, wait until you've released to crates.io a first time, then release all over again.
- [ ] Check local docs with e.g. cargo doc -p thag_common --features document-features --no-deps (thag_rs)
- [ ] For whole workspace with all features enabled: `cargo doc-all`
 (thag_profiler public)
 [Internal API: cargo doc --features document-features,full_profiling,debug_logging,internal_docs --no-deps
]
[Comprehensive: cargo doc --features document-features,full_profiling,debug_logging,internal_docs --no-deps --document-private-items
]
- [ ] Optional: reinstall thag_rs from path. (cargo install --path .)
- [ ] Make sure Readme images are up to date.
- [ ] Run clippy_feature_tests.sh
- [ ] Run cargo tests
- [ ] Run `gen_readme` for demo, src/bin.
- [ ] Run `typos` command.
- [ ] Run `find . -name "*.md" | while read f; do vale $f --no-wrap; done`.
- [ ] Run `cargo msrv set/verify`, and update the MSRV in README.md.
- [ ] Check on https://deps.rs/repo/github/durbanlegend/thag_rs that all dependencies are up to date
      (can link from badge at top of README.md).
- [ ] NB NB: If there have been any changes to thag_proc_macros or thag_profiler since their last published releases, bump their version numbers in their respective Cargo.tomls and also in their dependency entries in the main Cargo.toml. as these will be used in the crates.io version
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

ANSI Color           Semantic Role
───────────────────────────────
Black (0)            Background
Red (1)              Error
Green (2)            Success
Yellow (3)           Warning
Blue (4)             Info
Magenta (5)          Heading1
Cyan (6)             Heading3
White (7)            Normal
Bright Black (8)     Subtle
Bright Red (9)       Trace
Bright Green (10)    Debug
Bright Yellow (11)   Emphasis
Bright Blue (12)     Info (brighter)
Bright Magenta (13)  Heading1 (brighter)
Bright Cyan (14)     Hint
Bright White (15)    Normal (brighter)


export feature_sets=()

# thag_common
export feature_sets=(
    "",
    "config",
    "color_detect",
    "debug_logging",
    "config,debug_logging",
    "color_detect,debug_logging",
)

# thag_styling
export feature_sets=(
    "basic",
    "config",
    "color_detect",
    "inquire_theming"
    "console_support",
    "crossterm_support",
    "full"
    "image_themes",
    "inquire_theming",
    "nu_ansi_term_support",
    "owo_colors_support",
    "ratatui_support",
    "tools",
)

## thag_profiler
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
    "tools"
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

asciinema rec demo.cast

{
let nums = vec![1, 2, 3, 4];
nums.iter().sum::<i32>()
}

printf '\e[8;30;100t'

# I'll use the `tui` command to show you what I just changed in the external editor

{
println!("The `edit command` took me to my preferred editor, Zed.");
println!("There I replaced the algo with `(1..=10).product::<u32>()`);
(1..=10).product::<u32>()
}

Should you update them?

**Yes, definitely!** Here's why:
- ✅ Consistency with v0.2 branding - themed look is a major feature
- ✅ Shows off the new styling system visually
- ✅ More appealing/professional appearance
- ✅ Demonstrates the theming in action

## Should you rename them?

**No, keep the current names.** Here's why:
- ✅ Already embedded in the README in multiple places
- ✅ Generic names (`edit1t.png`, `edit2t.png`) are clear and simple
- ✅ No need to update references if you keep the same filenames
- ✅ The "t" suffix probably means "themed" or "terminal" - still appropriate

## What to capture:

**edit1t.png** (The editor itself):
- Show the TUI editor with some code
- Demonstrate syntax highlighting with your theme
- Make sure the UI elements are visible (status bar, key hints)

**edit2t.png** (The output):
- Show the result after running (Ctrl-R)
- Demonstrate successful compilation and output
- Keep it simple but impressive

## Recommendations:

1. **Keep filenames** - Just overwrite the existing files
2. **Use the same theme** as the asciinema demo for consistency
3. **Keep them simple** - Don't need to show everything, just enough to be impressive
4. **Consider adding better captions** to the README to explain what each shows

Would you like me to improve the captions in the README once you've updated the screenshots? Something like:

```markdown
![TUI Editor](assets/edit1t.png)
*Built-in TUI editor with syntax highlighting and themed interface*

![TUI Output](assets/edit2t.png)
*Running code directly from the editor with Ctrl-R*

Here's my video suggestion:
1. Copy a script into the editor using `thag -d < demo/fizz_buzz_gpt.rs`
2. Show some edits, viz strip off the comment lines and change 1..=100 to 1..=16 to show the most concise complete fizz-buzz result.
3. Ctrl-l to show the kep mappings and find the Save key combo.
4. Ctrl-s to bring up the Save file dialog. Scroll down to demo dir, tab to filename box, type in fizz_buzz_demo.rs and enter.
5. This takes us back to the main TUI editor with a message in the status line to say where the file has been saved to.
6. Before hitting Ctrl-d to submit, I could even show F9 to remove the line numbers and set TUI mouse capture off, so as to do normal mouse selection and Cmd-c to copy (for no particular purpose admittedly), and the F10 feature to restore it.
7. After submitting, let it visibly compile and display the result before ending the capture at that point.

[0:00-0:04] Load existing script into TUI editor
Command: thag -d < demo/fizz_buzz_gpt.rs

[0:04-0:18] Edit the script
- Remove doc comment lines
- Change range from 1..=100 to 1..=16

[0:18-0:26] View available key bindings
Press: Ctrl-L

[0:26-0:50] Save to new file
Press: Ctrl-S
Navigate to: demo/
Filename: fizz_buzz_demo.rs

[0:50-0:53] Submit and run
Press: Ctrl-D

[0:53-0:56] Watch compilation and execution
Result: FizzBuzz output for 1-16

head -51 src/bin/thag_palette.rs | tail -4 | thag_copy

/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

## Successful thag_profiler test:
cargo test --no-fail-fast --features full_profiling -- --test-threads=1 --nocapture

-p thag_profiler --test test_tls_vs_global

ffmpeg -i thag_profile_demo.mov -vf "fps=15,scale=1200:-1:flags=lanczos" -c:v gif thag_profile_demo.gif

![Flamegraph interaction](../docs/thag_profiler/assets/thag_profile_demo.gif)
*Interactive flamegraph showing async and sync operations - click bars to zoom, use search to find functions*

[![Flamegraph interaction](../docs/thag_profiler/assets/thag_profile_demo.gif)](https://durbanlegend.github.io/thag_rs/thag_profiler/assets/thag_profile_demo.svg)
*Click to open interactive SVG with full flamegraph features*

asciinema rec -I -i2 --window-size 203x30 --overwrite demo.cast
