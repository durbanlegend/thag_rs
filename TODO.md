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
- [ ]  Get builder ignored tests working
- [ ]  Debug egui_code_editor.rss only showing env_logger if use egui::*; is included.
- [ ]  See if with...(nu_resolve_style) methods of repl.rs can maybe use a closure to prevent lazy-static from triggering prematurely. Maybe add terminal test?
- [ ]  Cargo config in [toml] block to exclude Windows for rug files?
- [ ]  Demo scripts not compiling:
    test_clap_default_value_ifs_rs
    test_code_completion_rs
    test_create_next_file_rs
    test_edit_rs
    test_egui_code_editor_rs
    test_filter_lines_rs
    test_flume_async_rs
    test_flume_select_rs
    test_gpt_clap_derive_rs
    test_gpt_tui_no_ast_rs
    test_inline_colorization_rs
    test_json_rs
    test_msg_colors_gpt_rs
    test_msg_colors_rs
    test_owo_styles_rs
    test_parse_script_rs_toml_rs
    test_parse_toml_rs
    test_path_dependency_rs
    test_pomprt_completion_rs
    test_puffin_debug_rs
    test_syn_visit_path_exprs_rs
    test_tokio_hello_world_rs
    test_tui_editor_rs
    test_tui_ta_minimal_rs



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
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs.
- [ ]  Option to cat files before delete.
- [ ]  WASM - is there a worthwhile one? - maybe Leptos if it doesn't need Node.js.
