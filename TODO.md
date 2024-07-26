# TODO List

## High Priority
- [ ]  Clean up demos
- [ ]  Add additional popular crates

## Medium Priority
- [ ]  Document public APIs
- [ ]  More unit and integration tests
- [ ]  Ensure hyphens vs underscores are used correctly in rs-script command
- [ ]  Document where demo subdirectory is and how to install it
- [ ]  Simple demo https server
- [ ]  Debug egui_code_editor.rss only showing env_logger if "use egui::\*;" is included - DONE: Nothing to do with "use" statement, need to append ",egui_code_editor=debug" to $RUST_LOG
- [ ]  Exclude multimain demo scripts from integration_test or how do I selectively choose parms?
- [ ]  Put snippet doc comments above a use stmt?
- [ ]  Modify demo/fib_fast_doubling_iterative_rug.rs as per demo/fib_fast_doubling_iterative_ibig.rs
- [ ]  Debug demo/fib_fast_doubling_iterative_rug.rs!
- [ ]  Store a fib value and compare benchmark results against it for correctness?
- [ ]  Add instructions for linking scripts and pasting into rs_script -d (with [ OPTIONS ] -d [ -- args] into generated demo README.md?
= [ ]  Decide how to distribute demo readme without demo dir, maybe in main under another name.
- [ ]  Demo scripts not commented:
demo/supports_color.rs
demo/syn_ast_deps.rs
demo/syn_ast_deps2.rs
demo/syn_ast_deps_file.rs
demo/syn_dump_syntax.rs
demo/syn_file_debug.rs
demo/syn_file_debug2.rs
demo/syn_file_to_expr.rs
demo/syn_parse_quote.rs
demo/syn_parse_str.rs
demo/syn_quote.rs
demo/syn_visit_extern_crate_expr.rs
demo/syn_visit_extern_crate_file.rs
demo/syn_visit_node_type.rs
demo/syn_visit_use_path_expr.rs
demo/syn_visit_use_path_file.rs
demo/syn_visit_use_rename.rs
demo/tempfile.rs
demo/term_color_contrast.rs
demo/term_detection_pack.rs
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
