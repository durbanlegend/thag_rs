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
- [ ]  Exclude multimain demo scripts from integration_test or how do I selectively choose parms?
- [ ]  Modify demo/fib_fast_doubling_iterative_rug.rs as per demo/fib_fast_doubling_iterative_ibig.rs
- [ ]  Decide how to distribute demo readme without demo dir, maybe in main under another name.
- [ ]  Consider releasing a copy of repl.rs as a demo script.
- [ ]  Raise clear_screen as an issue on termbg crate?
- [ ]  Regression of rightward drift in testing
running 7 tests
test tests::test_color_support ... ok
                                     test tests::test_message_style_display ... ok
                                                                                  test tests::test_nu_color_get_color ... ok
                                                                                                                            test tests::test_message_style_get_style ... ok
                                                                                                                                                                           test tests::test_term_theme ... ok
                    test tests::test_nu_color_println_macro ... ok
test tests::test_nu_resolve_style ... ok
test tests::test_list ... ok
repl> test tests::test_parse_line ... ok
                                        cat: : No such file or directory
                                                                        cat: : No such file or directory
                                                                                                        q
                                                                                                         keys
                                                                                                             {<\n>use ibig::{ubig, UBig};<\n>use std::env;<\n>use std::iter::{successors, Successors, Take};<\n><\n>// Snippet accepts function or closure. This closure returns only the last value Fn.<\n>fn fib_value_n(n: usize) -> UBig {<\n>    successors(Some((ubig!(0), ubig!(1))), |(a, b)| {<\n>        Some((b.clone(), (a + b).into()))<\n>    })<\n>    .map(|(a, _b)| a)<\n>    .nth(n)<\n>    .unwrap()<\n>}<\n>fib_value_n(1000)<\n>}
                                                                                                                                                                    help
                                                                                                                                                                        history
                                                                                                                                                                               q
                                                                                                                                                                                keys
                                                                                                                                                                                    {<\n>use ibig::{ubig, UBig};<\n>use std::env;<\n>use std::iter::{successors, Successors, Take};<\n><\n>// Snippet accepts function or closure. This closure returns only the last value Fn.<\n>fn fib_value_n(n: usize) -> UBig {<\n>    successors(Some((ubig!(0), ubig!(1))), |(a, b)| {<\n>        Some((b.clone(), (a + b).into()))<\n>    })<\n>    .map(|(a, _b)| a)<\n>    .nth(n)<\n>    .unwrap()<\n>}<\n>fib_value_n(1000)<\n>}
                                                  keys
                                                      {<\n>use ibig::{ubig, UBig};<\n>use std::env;<\n>use std::iter::{successors, Successors, Take};<\n><\n>// Snippet accepts function or closure. This closure returns only the last value Fn.<\n>fn fib_value_n(n: usize) -> UBig {<\n>    successors(Some((ubig!(0), ubig!(1))), |(a, b)| {<\n>        Some((b.clone(), (a + b).into()))<\n>    })<\n>    .map(|(a, _b)| a)<\n>    .nth(n)<\n>    .unwrap():q<\n>}<\n>fib_value_n(1000)<\n>}
                                                                                                               q
                                                                                                                println ! ("result={}" , 5 + 8) ;
                                                                                                                                                 5 + 8
                                                                                                                                                      q
                                                                                                                                                       test tests::test_edit_history ... ok
  test tests::test_toml ... FAILED
                                  test tests::test_edit ... FAILED


Suspect termbg needs to be locked for single access.

clear; RUST_LOG=rs_script=debug cargo test --features=debug-logs -- --nocapture --test-threads=3

- [ ]  Add FAQ
- [ ]  Punchy intro in readme, per Zands.
- [ ]  Find a punchy name - rs-thagomizer | rs-bolt | rs-volt | rs-ares
                            rs_traction | rs_about-face | rs-backwards lol ?

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
