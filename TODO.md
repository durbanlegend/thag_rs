# TODO List

## High Priority
- [ ]  Clean up demos
- [ ]  Document public APIs

## Medium Priority
- [ ]  Add additional popular crates
- [ ]  More unit and integration tests
- [ ]  Ensure hyphens vs underscores are used correctly in rs-script command
- [ ]  Document where demo subdirectory is and how to install it
- [ ]  Simple demo https server
- [ ]  Decide how to distribute demo readme without demo dir, maybe in main under another name.
- [ ]  Consider releasing a copy of repl.rs as a demo script.
- [ ]  Raise clear_screen as an issue on termbg and supports-color crates?
       - make demo scripts to replicate issues.
- [ ]  Add FAQ
- [ ]  Punchy intro in readme, per Zands.
- [ ]  Conversions both ways for rust-script and maybe runner?
- [ ]  Find a punchy name - rs-thagomizer or thagomizer-rs (thag)| rs-bolt | rs-volt | rs-ares
- [ ]  Mocks not working in Windows? repl::test_edit, test_edit_history and test_toml all bring up VS Code and get stuck.
- [ ]  cat demo/fizz_buzz_gpt.rs | while read l; do rs_script -qe "println!(\"{}\", \"$l\".to_uppercase());"; done
- [ ]  Readme: Document that stdin where temp saves file to. Maybe change code to print this location out when it fails.
- [ ]  ./delete-logs.sh durbanlegend/rs-script ci.yml
- [ ]  Demo scripts not commented:
demo/termbg.rs
demo/termbg_bug.rs
demo/terminal_light.rs
demo/time_snippet.rs
demo/tui_scrollview.rs
demo/type_info.rs
demo/unzip.rs
demo/verbosity.rs
demo/windows_cr_issue.rs

## Low Priority
- [ ]  Consider history support for stdin.
- [ ]  Paste event in Windows slow or not happening?
- [ ]  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
- [ ]  Decide if it's worth passing the wrapped syntax tree to gen_build_run from eval just to avoid re-parsing it for that specific use case.
- [ ]  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
- [ ]  Clap aliases not working in REPL.
- [ ]  Work on demo/reedline_clap_repl_gemini.rs
- [ ]  Consider other Rust gui packages.
- [ ]  How to navigate reedline history entry by entry instead of line by line.
- [ ]  See if with...(nu_resolve_style) methods of repl.rs can maybe use a closure to prevent lazy-static from triggering prematurely. Maybe add terminal test?

## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.
- [ ]  WASM - is there a worthwhile one? - maybe Leptos if it doesn't need Node.js.
