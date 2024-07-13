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
- [ ]  Put snippet doc comments above a use stmt?
- [ ]  Try vi edit mode with reedline in repl.rs?
- [ ]  Demo scripts not commented:
demo/event_read.rs
demo/factorial_ibig.rs
demo/factorial_main_u128.rs
demo/fibonacci_fn.rs
demo/gen_names.rs
demo/gpt_lazy_static_theme1.rs
demo/gpt_lazy_static_theme2.rs
demo/gpt_tui_tokio.rs
demo/hello.rs
demo/hello_main.rs
demo/ibig_big_integers.rs
demo/infer_deps.rs
demo/inline_colorization.rs
demo/iter.rs
demo/list_files.rs
demo/message_colors_gpt.rs
demo/multiline_err.rs
demo/newest.rs
demo/newline_test.rs
demo/prettyplease.rs
demo/puffin_profiler_egui.rs
demo/rc1.rs
demo/reedline_basic_keybindings.rs
demo/reedline_colored_text.rs
demo/reedline_completions.rs
demo/reedline_custom_prompt.rs
demo/reedline_cwd_aware_hinter.rs
demo/reedline_highlighter.rs
demo/reedline_hinter.rs
demo/reedline_history.rs
demo/reedline_ide_completions.rs
demo/reedline_multiline.rs
demo/reedline_read_stdin.rs
demo/reedline_repl.rs
demo/reedline_stdin.rs
demo/reedline_transient_prompt.rs
demo/rug_arbitrary_precision_nums.rs
demo/rustyline.rs
demo/slog_expressions.rs
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


## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.
- [ ]  WASM - is there a worthwhile one? - maybe Leptos if it doesn't need Node.js.
