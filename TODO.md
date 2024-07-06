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
- [ ]  Debug egui_code_editor.rss only showing env_logger if use egui::*; is included.
- [ ]  See if with...(nu_resolve_style) methods of repl.rs can maybe use a closure to prevent lazy-static from triggering prematurely. Maybe add terminal test?
- [ ]  Consider replacing rug crate with dashu since rug not Windows-friendly?
- [ ]  Exclude multimain demo scripts from integration_test or how do I selectively choose parms?
- [ ]  Demo scripts not compiling:
  demo/syn_visit_path_exprs.rs
  demo/tokio_hello_world.rs
  demo/tui_editor.rs
  demo/tui_ta_minimal.rs


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


## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.
- [ ]  WASM - is there a worthwhile one? - maybe Leptos if it doesn't need Node.js.
