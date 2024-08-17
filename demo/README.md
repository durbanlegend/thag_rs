## Running the scripts

`thag_rs` uses `clap` for a standard command-line interface. Try `thag_rs --help` (or -h) if
you get stuck.

### In its simplest form:


    thag_rs <path to script>

###### E.g.:

    thag_rs demo/hello.rs

### Passing options and arguments to a script:

Use `--` to separate options and arguments meant for the script from those meant for `thag_rs` itself.

###### E.g.:

demo/fib_dashu_snippet.rs expects to be passed an integer _n_ and will compute the _nth_ number in the
Fibonacci sequence.

     thag_rs demo/fib_dashu_snippet.rs -- 100

### Full syntax:

    thag_rs [THAG_RS OPTIONS] <path to script> [-- [SCRIPT OPTIONS] <script args>]

###### E.g.:

`demo/clap_tut_builder_01.rs` is a published example from the `clap` crate.
Its command-line signature looks like this:

    clap_tut_builder_01 [OPTIONS] [name] [COMMAND]

The arguments in their short form are:

    `-c <config_file>`      an optional configuration file
    `-d` / `-dd` / `ddd`    debug, at increasing levels of verbosity
    [name]                  an optional filename
    [COMMAND]               a command (e.g. test) to run

If we were to compile `clap_tut_builder_01` as an executable (`-x` option) and then run it, we might pass
it some parameters like this:

    clap_tut_builder_01 -dd -c my.cfg my_file test -l

and get output like this:

    Value for name: my_file
    Value for config: my.cfg
    Debug mode is on
    Printing testing lists...

Running the source from thag_rs looks similar, we just replace `clap_tut_builder_01` by `thag_rs demo/clap_tut_builder_01.rs --`:

*thag_rs demo/clap_tut_builder_01.rs --* -dd -c my.cfg my_file test -l

Any parameters for `thag_rs` should go before the `--`, e.g. we may choose use -qq to suppress `thag_rs` messages:

    thag_rs demo/clap_tut_builder_01.rs -qq -- -dd -c my.cfg my_file test -l

which will give identical output to the above.



##### Remember to use `--` to separate options and arguments that are intended for `thag_rs` from those intended for the target script.

***
## Detailed script listing


### Script: analyze_snippet_1.rs

**Description:**  Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
 to detect whether a snippet returns a value that we should print out, or whether
 it does its own printing.

 Part 1: After some back and forth with ChatGPT suggesting solutions it finally generates essentially this.


**Purpose:** Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.

**Crates:** `syn`

**Type:** Program

**Link:** [analyze_snippet_1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_1.rs)

---

### Script: analyze_snippet_2.rs

**Description:**  Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
 to detect whether a snippet returns a value that we should print out, or whether
 it does its own printing.

 Part 2: ChatGPT responds to feedback with an improved algorithm.


**Purpose:** Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.

**Crates:** `quote`, `syn`

**Type:** Program

**Link:** [analyze_snippet_2.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_2.rs)

---

### Script: analyze_snippet_3.rs

**Description:**  Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
 to detect whether a snippet returns a value that we should print out, or whether
 it does its own printing.

 Part 3: I raise the case of a function call and ChatGPT responds with essentially this.
 I've commented out ChatGPT's brute-force parse of &block.stmts and replaced it with a syn::Visit
 implementation that can handle embedded functions.


**Purpose:** Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.

**Crates:** `quote`, `syn`

**Type:** Program

**Link:** [analyze_snippet_3.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_3.rs)

---

### Script: any.rs

**Description:**  Demo determining at run-time whether an expression returns a unit value
 so that it can be handled appropriately. When using a code template this is
 short and sweet, but it has to be included in the template and thus the
 generated code, whereas using an AST is quite a mission but works with
 any arbitrary snippet and doesn't pollute the generated source code.


**Purpose:** Demo Rust's answer to dynamic typing.

**Crates:** 

**Type:** Snippet

**Link:** [any.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/any.rs)

---

### Script: bitflags.rs

**Description:**  Try out the `bitflags` crate.


**Purpose:** Explore use of `bitflags` to control processing.

**Crates:** `bitflags`

**Type:** Program

**Link:** [bitflags.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bitflags.rs)

---

### Script: borrow_wrapped.rs

**Description:**  Snippet demonstrating how to reference or clone a wrapped value without
 falling foul of the borrow checker.


**Purpose:** Demo a borrow-checker-friendly technique for accessing a wrapped value.

**Crates:** 

**Type:** Snippet

**Link:** [borrow_wrapped.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/borrow_wrapped.rs)

---

### Script: bpaf_cargo_show_asm.rs

**Description:**  Published example from `https://github.com/pacak/bpaf/src/docs2/derive_show_asm.md`


 E.g. `thag_rs demo/bpaf_cargo_show_asm.rs -- -h`


**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf`

**Type:** Program

**Link:** [bpaf_cargo_show_asm.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_cargo_show_asm.rs)

---

### Script: bpaf_cmd_chain.rs

**Description:**  Example from bpaf crate docs2/src/adjacent_command/derive.rs.

 E.g. `thag_rs demo/bpaf_cmd-chain.rs -- eat Fastfood drink --coffee sleep --time=5`


**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf_derive`

**Type:** Program

**Link:** [bpaf_cmd_chain.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_cmd_chain.rs)

---

### Script: bpaf_derive.rs

**Description:**  Example from bpaf crate docs2/src/command/derive.rs.

 E.g. `demo/bpaf_cmd_ex.rs -- --flag cmd --flag --arg=6`


**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf_derive`

**Type:** Program

**Link:** [bpaf_derive.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_derive.rs)

---

### Script: cargo_output.rs

**Description:**  Run a command (in this case a cargo search for the `log` crate),
 and capture and print its stdout and stderr concurrently in a
 separate thread.


**Purpose:** Demo process::Command with output capture.

**Crates:** `env_logger`, `log`

**Type:** Program

**Link:** [cargo_output.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_output.rs)

---

### Script: clap_enum_strum.rs

**Description:**  Exploring using clap with an enum, in conjunction with strum.
 E.g. `thag_rs demo/clap_enum_strum.rs -- variant-num2`


**Purpose:** Simple demo of featured crates, contrasting how they expose variants.

**Crates:** `clap`, `serde`, `strum`

**Type:** Program

**Link:** [clap_enum_strum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_enum_strum.rs)

---

### Script: clap_num_arg.rs

**Description:**  `clap` with a numeric option.

 E.g. `thag_rs demo/clap_num_arg.rs -- 45`


**Purpose:** Basic demo of `clap` parsing a numeric argument

**Crates:** `clap`

**Type:** Program

**Link:** [clap_num_arg.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_num_arg.rs)

---

### Script: clap_repl_crate_rustyline.rs

**Description:**  Older version of published clap_repl crate example, modified to prototype a
 (dummy) Rust REPL.


**Purpose:** Yet another REPL demo, this time using `rustyline`.

**Crates:** `clap`, `clap_repl`, `console`, `quote`, `rustyline`, `syn`

**Type:** Program

**Link:** [clap_repl_crate_rustyline.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_repl_crate_rustyline.rs)

---

### Script: clap_repl_diy.rs

**Description:**  Example from the clap cookbook, not using the clap-repl crate.
 Can't find a keybinding to navigate history, unlike
 clap_repl_rustyline.rs and unlike clap_repl_reedline.rs.


**Purpose:** Demo building a repl using `clap` directly.

**Crates:** `clap`

**Type:** Program

**Link:** [clap_repl_diy.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_repl_diy.rs)

---

### Script: clap_tut_builder_01_quick.rs

**Description:**  Published example from `clap` tutorial (builder)

 E.g.  `thag_rs demo/clap_tut_builder_01_quick.rs -- -ddd -c dummy.cfg my_file test -l`


**Purpose:** Demonstrate `clap` CLI using the builder option

**Crates:** `clap`

**Type:** Program

**Link:** [clap_tut_builder_01_quick.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_builder_01_quick.rs)

---

### Script: clap_tut_derive_03_04_subcommands.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. thag_rs demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob


**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Link:** [clap_tut_derive_03_04_subcommands.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_03_04_subcommands.rs)

---

### Script: clap_tut_derive_04_01_enum.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. `thag_rs demo/clap_tut_derive_04_01_enum.rs -- fast`


**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Link:** [clap_tut_derive_04_01_enum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_04_01_enum.rs)

---

### Script: clap_tut_derive_04_03_relations.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. `thag_rs demo/clap_tut_derive_04_03_relations.rs -- --major -c config.toml --spec-in input.txt`


**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Link:** [clap_tut_derive_04_03_relations.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_04_03_relations.rs)

---

### Script: cli_partial_match.rs

**Description:**  Experiment with matching REPL commands with a partial match of any length.


**Purpose:** Usability: Accept a command as long as the user has typed in enough characters to identify it uniquely.

**Crates:** `clap`, `console`, `rustyline`, `strum`

**Type:** Program

**Link:** [cli_partial_match.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cli_partial_match.rs)

---

### Script: cmd_args.rs

**Description:**  A prototype of the cmd_args module of thag_rs itself.

 E.g. `thag_rs -tv demo/cmd_args.rs -- -gbrtv demo/hello.rs -- -fq Hello world`


**Purpose:** Prototype CLI.

**Crates:** `bitflags`, `clap`, `thag_rs`

**Type:** Program

**Link:** [cmd_args.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args.rs)

---

### Script: cmd_args_bpaf_gpt.rs

**Description:**  Example of a CLI using the bpaf crate instead of clap, originally generated by ChatGPT.
 See `demo/cmd_args_clap.rs` for comparison.

 E.g. `thag_rs -tv demo/cmd_args_bpaf_gpt.rs -- -gbrtv demo/hello.rs -- -fq Hello world`


**Purpose:** Demo one lighter-weight alternative to clap.

**Crates:** `bitflags`, `bpaf_derive`

**Type:** Program

**Link:** [cmd_args_bpaf_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args_bpaf_gpt.rs)

---

### Script: cmd_args_clap.rs

**Description:**  Basic CLI example using clap.

 E.g. `thag_rs -t demo/cmd_args_clap.rs -- -atv hello.sh`


**Purpose:** For comparison with `demo/cmd_args_bpaf_gpt.rs`.

**Crates:** `bitflags`, `clap`

**Type:** Program

**Link:** [cmd_args_clap.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args_clap.rs)

---

### Script: color_contrast.rs

**Description:**  Given a sample RGB colour value, determine whether it would
 contrast better with black or white (background or foreground).
 Can't recall provenance, but the luminance formula is one of
 many discussed here:
 https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color


**Purpose:** Choose black or white as a contrasting colour for a given colour.

**Crates:** 

**Type:** Program

**Link:** [color_contrast.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/color_contrast.rs)

---

### Script: colors.rs

**Description:**  A version of `thag_rs`'s `colors` module to style messages according to their type. Like the `stdin`
 module, `colors` was originally developed here as a separate script and integrated as a module later.

 Format: `nu_color_println!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);`

 E.g. `thag_rs demo/colors.rs`


**Purpose:** Demo using `thag_rs` to develop a module outside of the project.

**Crates:** `lazy_static`, `strum`, `supports_color`, `termbg`, `thag_rs`

**Type:** Program

**Link:** [colors.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/colors.rs)

---

### Script: colors_orig.rs

**Description:**  Original prototype of `thag_rs`'s `colors` module to style messages according
 to their type. I only dropped `owo-colors` because I switched from `rustyline` to
 `reedline`, which was already using `nu_ansi_term`.

 Format: `color_println!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);`


**Purpose:** Demo older alternative implementation of `colors` module using `owo-colors`.

**Crates:** `log`, `owo_colors`, `strum`, `supports_color`, `termbg`, `thag_rs`

**Type:** Program

**Link:** [colors_orig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/colors_orig.rs)

---

### Script: config.rs

**Description:**  Prototype of configuration file implementation. Delegated the grunt work to ChatGPT.


**Purpose:** Develop a configuration file implementation for `thag_rs`.

**Crates:** `serde`, `strum_macros`

**Type:** Program

**Link:** [config.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/config.rs)

---

### Script: count_main_methods.rs

**Description:**  Prototype of a function required by thag_rs to count the main methods
 in a script to decide if it's a program or a snippet. Uses the `syn`
 visitor pattern. This is more reliable than a simple source code search
 which tends to find false positives in string literals and comments.


**Purpose:** Demo prototyping with thag_rs and use of the `syn` visitor pattern to visit nodes of interest

**Crates:** `syn`

**Type:** Program

**Link:** [count_main_methods.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/count_main_methods.rs)

---

### Script: create_next_file.rs

**Description:**  Prototype of creating files named sequentially from repl_000000.rs to
 repl_999999.rs in a thag_rs/demo subdirectory of the OS's temporary
 directory. The need is to generate well-behaved and consistent human-readable
 names for temporary programs generated from REPL expressions.


**Purpose:** Demo sequential file creation and the kind of code that is well suited to generation by an LLM.

**Crates:** 

**Type:** Program

**Link:** [create_next_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/create_next_file.rs)

---

### Script: crokey_deser.rs

**Description:**  Published example of serde deserialisation from `crokey` crate.
 This is an example of a configuration structure which contains
 a map from KeyEvent to String.
 An example of what could be a configuration file


**Purpose:** Demo loading keybindings from a file.

**Crates:** `crokey`, `serde`

**Type:** Program

**Link:** [crokey_deser.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_deser.rs)

---

### Script: crokey_print_key.rs

**Description:**  Published example of combiner from `crokey` crate.


**Purpose:** Demo key combiner.

**Crates:** `crokey`, `crossterm`

**Type:** Program

**Link:** [crokey_print_key.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_print_key.rs)

---

### Script: crossbeam_channel_fibonacci.rs

**Description:**  `crossbeam-channel` published example.

 An asynchronous fibonacci sequence generator.


**Purpose:** Demo featured crate.

**Crates:** `crossbeam_channel`

**Type:** Program

**Link:** [crossbeam_channel_fibonacci.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_fibonacci.rs)

---

### Script: crossbeam_channel_matching.rs

**Description:**  `crossbeam-channel` published example
 Using `select!` to send and receive on the same channel at the same time.


**Purpose:** Demo featured crates.

**Crates:** `crossbeam_channel`, `crossbeam_utils`

**Type:** Program

**Link:** [crossbeam_channel_matching.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_matching.rs)

---

### Script: crossbeam_channel_stopwatch.rs

**Description:**  `crossbeam-channel` published example.

 Prints the elapsed time every 1 second and quits on `Ctrl+C`.
 You can reinstate the separate main method for Windows provided you
 run the script with the `--multimain (-m)` option.


**Purpose:** showcase featured crates.

**Crates:** `crossbeam_channel`, `signal_hook`

**Type:** Program

**Link:** [crossbeam_channel_stopwatch.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_stopwatch.rs)

---

### Script: crossbeam_epoch_sanitize.rs

**Description:**  The `crossbeam-epoch` crate provides epoch-based _lock-free_ memory reclamation,
 an alternative to garbage collection.

 This is the published example from the `crossbeam-epoch` crate. For a more intuitive
 example, you can try the "Canary" example from https://github.com/ericseppanen/epoch_playground.
 and the associated blog post https://codeandbitters.com/learning-rust-crossbeam-epoch/.
 (Not included here due to implicit copyright).



**Purpose:** Demo featured crate.

**Crates:** `crossbeam_epoch`, `rand`

**Type:** Program

**Link:** [crossbeam_epoch_sanitize.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_epoch_sanitize.rs)

---

### Script: crossterm.rs

**Description:**  Published example from crossterm crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/README.md


**Purpose:** Demo crossterm terminal manipulation.

**Crates:** `crossterm`

**Type:** Program

**Link:** [crossterm.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm.rs)

---

### Script: crossterm_command_macro.rs

**Description:**  Published example from crossterm crate. Macro version of the example:
 "Print a rectangle colored with magenta and use both direct execution and lazy execution."
 Direct execution with `execute` and lazy execution with `queue`.

 Url: https://docs.rs/crossterm/latest/crossterm/


**Purpose:** Demo `crossterm` command API.

**Crates:** `crossterm`

**Type:** Program

**Link:** [crossterm_command_macro.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_command_macro.rs)

---

### Script: crossterm_event_read.rs

**Description:**  Published example from crossterm crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/examples/event-read.rs
 "Demonstrates how to block read events."


**Purpose:** Demo running crate example code, `crossterm` events.

**Crates:** `crossterm`

**Type:** Program

**Link:** [crossterm_event_read.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_event_read.rs)

---

### Script: crossterm_key_events.rs

**Description:**  Published example from crossterm crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/examples/key-display.rs
 "Demonstrates the display format of key events.

 This example demonstrates the display format of key events, which is useful for displaying in
 the help section of a terminal application."


**Purpose:** Demo running crate example code, `crossterm` events.

**Crates:** `crossterm`

**Type:** Program

**Link:** [crossterm_key_events.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_key_events.rs)

---

### Script: ctrlc_demo.rs

**Description:**  Published example from `ctrlc` crate: "Cross platform handling of Ctrl-C signals."


**Purpose:** Demo one option for intercepting Ctrl-C.

**Crates:** 

**Type:** Program

**Link:** [ctrlc_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ctrlc_demo.rs)

---

### Script: curl.rs

**Description:**  Simple HTTPS GET

 This example is a Rust adaptation of the [C example of the same
 name](https://curl.se/libcurl/c/https.html).
 On Linux you may need to install `pkg-config` and `libssl-dev`.


**Purpose:** Demo `curl` implementation.

**Crates:** `curl`

**Type:** Program

**Link:** [curl.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/curl.rs)

---

### Script: dethag_re.rs

**Description:**  Unescape \n and \\" markers in a string to convert the wall of text to readable lines.
 This version using regex may be more reliable than the classic approach using .lines().
 However, at time of writing, `regex` is a 248kB crate, which makes the binary of this
 module more than 5 time larger than that of `thagomizer`.

 Tip: Regex tested using https://rustexp.lpil.uk/.


**Purpose:** Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.

**Crates:** `lazy_static`, `regex`

**Type:** Program

**Link:** [dethag_re.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/dethag_re.rs)

---

### Script: dethagomizer.rs

**Description:**  Unescape \n and \\" markers in a string to convert the wall of text to readable lines.
 This is trickier than it seems because in a compile-time literal, \n compiles to the
 true line feed character 10 (x0A), whereas a \n generated or captured as a literal
 at run time is encoded as ('\', 'n'() = (92, 110) = 0x5c6e. Not surprisingly, the two
 representations, while they look identical to the programmer, don't always behave
 the same.

 See `demo/dethagomizer.rs` for a Regex version.


**Purpose:** Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.

**Crates:** 

**Type:** Program

**Link:** [dethagomizer.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/dethagomizer.rs)

---

### Script: duration_snippet.rs

**Description:**  Minimal snippet showing how to add nice additional constructors such as `from_weeks` (and days, hours and
 minutes) to `std::time::Duration`.

 These are enabled by adding the inner attribute `#![feature(duration_constructors)]` to the script.
 I've used a snippet to illustrate that this is possible: an inner attribute (i.e. an attribute prefixed
 with `#!` (`#![...]`)) must be placed at the top of the crate it applies to, so when wrapping the snippet
 in a fn main, thag_rs pulls any inner attributes out to the top of the program.

 Notice we also have a shebang so that this script may be run as `demo/duration_snippet.rs` with execute
 permission. The shebang must be on the very first line but coexists peacefulyl with the inner attribute.

 See tracking issue https://github.com/rust-lang/rust/issues/120301 for the `Duration` constructor issue..

 E.g. `(*nix)`:

     chmod u+g demo/duration_snippet.rs      // Only requied the first time of course
     demo/duration_snippet.rs -qq
     1209600s

 Or more concisely:

     f=demo/duration_snippet.rs && chmod u+g $f && $f -qq
     1209600s



**Purpose:** Demonstrate that some fairly subtle moves are possible even with the simplest of snippets.

**Crates:** 

**Type:** Snippet

**Link:** [duration_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/duration_snippet.rs)

---

### Script: edit.rs

**Description:**  Published example from edit crate readme.

 Will use the editor specified in VISUAL or EDITOR environment variable.

 E.g. `EDITOR=vim thag_rs demo/edit.rs`


**Purpose:** Demo of edit crate to invoke preferred editor.

**Crates:** 

**Type:** Snippet

**Link:** [edit.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/edit.rs)

---

### Script: egui_code_editor.rs

**Description:**  A prototype GUI editor with saved state and syntax highlighting.


**Purpose:** Prototype a native-mode editor using the `egui` crate.

**Crates:** 

**Type:** Program

**Link:** [egui_code_editor.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/egui_code_editor.rs)

---

### Script: enum_select.rs

**Description:**  Prototype of selecting message colours by matching against different enums
 according to the terminal's detected colour support and light or dark theme.
 (Detection itself is not part of the demo).
 This approach was rejected as it is simpler to use a single large enum and
 use the `strum` crate's `EnumString` derive macro to select the required
 variant from a composite string of the colour support, theme and message level.


**Purpose:** Demo prototyping different solutions using AI to provide the sample implementations.

**Crates:** `owo_colors`

**Type:** Program

**Link:** [enum_select.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/enum_select.rs)

---

### Script: factorial_dashu_product.rs

**Description:**  Fast factorial algorithm with arbitrary precision and avoiding recursion.
 Closures and functions are effectively interchangeable here.

  Using the `std::iter::Product` trait - if implemented - is the most concise
 factorial implementation. `dashu` implements it, so it's straightforward to use.



**Purpose:** Demo snippet, `dashu` crate, factorial using `std::iter::Product` trait.

**Crates:** `dashu`

**Type:** Snippet

**Link:** [factorial_dashu_product.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_dashu_product.rs)

---

### Script: factorial_ibig.rs

**Description:**  Fast factorial algorithms with arbitrary precision and avoiding recursion.
 A version using `std::Iterator::fold` and one using `std::iter::Successors:successors`
 are executed and compared to ensure they agree before printing out the value.
 Closures and functions are effectively interchangeable here.

 `let foo = |args| -> T {};` is equivalent to `fn foo(args) -> T {}`

 See also `demo/factorial_ibig_product.rs` for yet another version where we implement
 the `std::iter::Product` trait on a wrapped `ibig::UBig` in order to use the
 otherwise most concise, simple and approach. A very similar cross-platform implementation
 without the need for such Product scaffolding (since `dashu` implements `Product`)
 is `demo/factorial_dashu_product.rs`. The fastest by far is `demo/factorial_main_rug_product.rs`
 backed by GNU libraries, but unfortunately it does not support the Windows MSVC, although it
 may be possible to get it working with MSYS2.

 Before running any benchmarks based on these scripts, don't forget that some of them
 only run one algorithm while others are slowed down by running and comparing two different
 algorithms.


**Purpose:** Demo snippets with functions and closures, `ibig` cross-platform big-number crate.

**Crates:** `ibig`

**Type:** Snippet

**Link:** [factorial_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_ibig.rs)

---

### Script: factorial_ibig_product.rs

**Description:**  Fast factorial algorithm with arbitrary precision and avoiding recursion.
 Closures and functions are effectively interchangeable here.

 Using the `std::iter::Product` trait - if implemented - is the most concise factorial
 implementation. Unfortunately, but unlike the `dashu` and `rug` crates, `ibig` does
 not implement the Product trait, so we have to wrap the `UBig`. Which of course
 is pretty verbose in the context of a snippet, but could be useful in an app.
 The implementation is thanks to GPT-4.


**Purpose:** Demo snippet, `ibig` crate, factorial using `std::iter::Product` trait, workaround for implementing an external trait on an external crate.

**Crates:** `ibig`

**Type:** Snippet

**Link:** [factorial_ibig_product.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_ibig_product.rs)

---

### Script: factorial_main_u128_product.rs

**Description:**  Fast factorial algorithm avoiding recursion, but limited to a maximum of `34!` by using only
 Rust primitives.


**Purpose:** Demo fast limited-scale factorial using Rust primitives and std::iter::Product trait.

**Crates:** 

**Type:** Program

**Link:** [factorial_main_u128_product.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_main_u128_product.rs)

---

### Script: fib_4784969_cpp_ibig.rs

**Description:**  Rust port of C++ example from `https://github.com/ZiCog/fibo_4784969` - so named because
 F(4784969) is the first number in the Fibonacci sequence that has one million decimal
 digits. This contains 3 alternative algorithms to compare their speed, with `fibo_new`
 edging out `fibo` at this scale.

 E.g.: `thag_rs demo/fib_4784969_cpp_ibig.rs -- 4784969   // or any positive integer`



**Purpose:** Demo 3 very fast Fibonacci algorithms, though still 7-11 times slower than `rug`.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_4784969_cpp_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_4784969_cpp_ibig.rs)

---

### Script: fib_4784969_cpp_rug.rs

**Description:**  Rust port of C++ example from `https://github.com/ZiCog/fibo_4784969` - so named because
 F(4784969) is the first number in the Fibonacci sequence that has one million decimal
 digits. This contains 3 alternative algorithms to compare their speed, with `fibo_new`
 edging out `fibo` at this scale.

 The `rug` crate runs blindingly fast, but I for one found it very difficult to get this to compile.

 E.g.: `thag_rs demo/fib_4784969_cpp_ibig.rs -- 4784969   // or any positive integer`



**Purpose:** Demo 3 very fast Fibonacci algorithms (F(4784969) in 0.33 to 0.58 sec for me).

**Crates:** `rug`

**Type:** Program

**Link:** [fib_4784969_cpp_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_4784969_cpp_rug.rs)

---

### Script: fib_basic.rs

**Description:**  Fast non-recursive classic Fibonacci calculations for a specific value or an entire sequence.
 I can't recall the exact source, but see for example https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
 for a variety of alternative approaches. The various Fibonacci scripts here in the demo
 directory also show a number of approaches. `demo/fib_basic_ibig.rs` shows the use of
 the `std::iter::Successors` iterator as well as removing the limitations of Rust
 primitives. Most of the other examples explore different strategies for rapid computation of
 large Fibonacci values, and hopefully demonstrate the usefulness of `thag_rs` as a tool
 for trying out and comparing new ideas.

 As the number of Fibonacci examples here shows, this took me down a Fibonacci rabbit hole.


**Purpose:** Demo fast small-scale fibonacci using Rust primitives and `itertools` crate.

**Crates:** `itertools`

**Type:** Snippet

**Link:** [fib_basic.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_basic.rs)

---

### Script: fib_basic_ibig.rs

**Description:**  Big-number (and thus more practical) version of `demo/fib_basic.rs`.



**Purpose:** Demo using a big-number crate to avoid the size limitations of primitive integers.

**Crates:** `ibig`, `itertools`

**Type:** Snippet

**Link:** [fib_basic_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_basic_ibig.rs)

---

### Script: fib_big_clap_rug.rs

**Description:**  Fast non-recursive Fibonacci series and individual calculation with big integers.
 Won't work with default Windows 11 because of `rug` crate.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.

 The `fib_series` closure could equally be implemented as a function here,
 but closure is arguably easier as you don't have to know or figure out the
 exact return type (`impl Iterator<Item = Integer>` if you're wondering).

 Using `clap` here is complete overkill, but this is just a demo.
 On Linux you may need to install the m4 package.


**Purpose:** Demonstrate snippets, closures, `clap` builder and a fast non-recursive fibonacci algorithm using the `successors`.

**Crates:** `clap`, `rug`

**Type:** Snippet

**Link:** [fib_big_clap_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_big_clap_rug.rs)

---

### Script: fib_binet_astro_snippet.rs

**Description:**  Academic / recreational example of a closed-form (direct) calculation of a
 given number in the Fibonacci sequence using Binet's formula. This is imprecise
 above about F70, and the `dashu` crate can't help us because it does not support
 computing powers of a negative number since they may result in a complex
 number. Regardless, relying on approximations of irrational numbers lends
 itself to inaccuracy.

 Shout-out to the `expr!` macro of the `astro-float` crate, which reduces very
 complex representations back to familiar expressions.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demo closed-form Fibonacci computation and the limitations of calculations based on irrational numbers, also `astro-float` crate..

**Crates:** `astro_float`

**Type:** Snippet

**Link:** [fib_binet_astro_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_binet_astro_snippet.rs)

---

### Script: fib_binet_snippet.rs

**Description:**  Purely academic example of a closed-form (direct) calculation of an individual
 Fibonacci number using Binet's formula. This is imprecise above about F70, and
 the `dashu` crate can't help us because it does not support computing powers
 of a negative number because they may result in a complex number. Regardless,
 relying on approximations of irrational numbers lends itself to inaccuracy.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demo closed-form Fibonacci computation and the limitations of calculations based on irrational numbers..

**Crates:** 

**Type:** Snippet

**Link:** [fib_binet_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_binet_snippet.rs)

---

### Script: fib_classic_ibig.rs

**Description:**  Fast non-recursive classic Fibonacci individual calculation with big integers.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demonstrate snippets and a fast non-recursive fibonacci algorithm using the `successors` iterator.

**Crates:** `ibig`

**Type:** Snippet

**Link:** [fib_classic_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_classic_ibig.rs)

---

### Script: fib_classic_ibig_instrumented.rs

**Description:**  Same script as `demo/fib_basic_ibig.rs` with basic instrumentation added for benchmarking
 against other fibonacci scripts.
 Scripts can then be selected and run sequentially.
 E.g. an apples-with-apples comparison of different algorithms implemented using the ``ibig` crate:
 `ls -1 demo/fib*ibig*.rs | grep -v fib_basic_ibig.rs | while read f; do echo $f; thag_rs -t $f -- 10000000; done`

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demonstrate instrumenting scripts for benchmarking.

**Crates:** `ibig`

**Type:** Snippet

**Link:** [fib_classic_ibig_instrumented.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_classic_ibig_instrumented.rs)

---

### Script: fib_dashu_snippet.rs

**Description:**  Fast non-recursive Fibonacci sequence calculation with big integers.
 Should work with default Windows.

 Based on discussion https://users.rust-lang.org/t/fibonacci-sequence-fun/77495

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demonstrate snippets, a fast non-recursive fibonacci algorithm using `successors`, and zipping 2 iterators together.

**Crates:** `dashu`

**Type:** Snippet

**Link:** [fib_dashu_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_dashu_snippet.rs)

---

### Script: fib_doubling_iterative_ibig.rs

**Description:**  Very fast non-recursive calculation of an individual Fibonacci number using the
 Fibonacci doubling identity. See also `demo/fib_doubling_recursive.rs` for the
 original recursive implementation and the back story.

 This version is derived from `demo/fib_doubling_recursive_ibig.rs` with the following
 changes:

 1. Instead of calculating the `Fi` values in descending order as soon as they are
 identified, add them to a list and then calculate them from the list in ascending
 order.

 2. The list tends to end up containing strings of 3 or more commonly 4 consecutive
 `i` values for which `Fi` must be calculated. For any `i` that is the 3rd or
 subsequent entry in such a consecutive run, that is, for which Fi-2 and Fi-1 have
 already been calculated, compute Fi cheaply as Fi-2 + Fi-1 instead of using the
 normal multiplication formula.


**Purpose:** Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_doubling_iterative_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_ibig.rs)

---

### Script: fib_doubling_iterative_purge_ibig.rs

**Description:**  Very fast non-recursive calculation of an individual Fibonacci number using the
 Fibonacci doubling identity. See also `demo/fib_doubling_recursive_ibig.rs` for the
 original recursive implementation and the back story.

 This version is derived from `demo/fib_doubling_iterative.rs` with the following
 change: that we reduce bloat as best we can by purging redundant entries from the memo
 cache as soon as it's safe to do so.


**Purpose:** Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_doubling_iterative_purge_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_purge_ibig.rs)

---

### Script: fib_doubling_iterative_purge_rug.rs

**Description:**  Very fast non-recursive calculation of an individual Fibonacci number using the
 Fibonacci doubling identity. See also `demo/fib_doubling_recursive.ibig.rs` for the
 original recursive implementation and the back story.
 Won't work with default Windows 11 because of `rug` crate.
 On Linux you may need to install the m4 package.

 This version is derived from `demo/fib_doubling_iterative.rs` with the following
 change: that we reduce bloat as best we can  by purging redundant entries from the memo
 cache as soon as it's safe to do so.


**Purpose:** Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.

**Crates:** `rug`

**Type:** Program

**Link:** [fib_doubling_iterative_purge_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_purge_rug.rs)

---

### Script: fib_doubling_no_memo_ibig.rs

**Description:**  A version of `demo/fib_doubling_recursive.rs`, minus the memoization.
 This serves to prove that the memoization is significantly faster, although
 not dramatically so.



**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_doubling_no_memo_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig.rs)

---

### Script: fib_doubling_no_memo_ibig_1.rs

**Description:**  Try a version based on reverse engineering the `fibo_new / fibo_new_work` functions of `demo/fib_4784969_cpp_ibig.rs`
 This approach passes the pair `Fn, Fn+1` `(a, b)` and applies some funky calculations. I'll pay my dues here by doing
 the derivation.

 This version uses immutable arguments to the `fib` method.

 Starting with the usual formulae used by doubling methods.
 For even indices:

     F2n  = 2Fn.Fn+1 - Fn^2

          = Fn(2Fn+1 - Fn).   // i.e. a(2b - a)

 For odd indices:

     F2n+1 = Fn^2 + Fn+1^2.

 To the odd-index case we apply Cassini's identity: Fn^2 = Fn-1.Fn+1 - (-1)^n:

     F2n+1 = Fn+1^2 + Fn^2 +

           = Fn+1^2 + Fn+1Fn-1 - (-1)^n          // since by Cassini Fn^2 = Fn-1.Fn+1 - (-1)^n

           = Fn+1^2 + Fn+1(Fn+1 - Fn) - (-1)^n   // substituting for Fn-1

           = 2Fn+1^2 - Fn.Fn+1 - (-1)^n

           = Fn+1(2Fn+1 - Fn) - (-1)^n           // i.e. b(2b - a) - (-1)^n

 If n is odd, then a = F2n+1 and b = 2Fn+2, so we must derive the latter:

     F2n+2 = F2m where m = n+1 = Fm(2Fm+1 - Fm)

           = Fn+1(2F(n+2) - Fn+1)

           = Fn+1(2Fn+1 + 2Fn - Fn+1)            // Since Fn+2 = Fn + Fn+1

           = Fn+1(Fn+1 + 2Fn)                    // i.e. b(b+2a)


**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_doubling_no_memo_ibig_1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig_1.rs)

---

### Script: fib_doubling_no_memo_ibig_2.rs

**Description:**  Try a version based on reverse engineering the `fibo_new / fibo_new_work` functions of `demo/fib_4784969_cpp_ibig.rs`
 This approach passes the pair `Fn, Fn+1` `(a, b)` and applies some funky calculations. I'll pay my dues here by doing
 the derivation.

 This version uses mutable arguments to the `fib` method.

 Starting with the usual formulae used by doubling methods:
     For even indices:

     F2n  = 2Fn.Fn+1 - Fn^2

          = Fn(2Fn+1 - Fn).   // i.e. a(2b - a)

     For odd indices:

     F2n+1 = Fn^2 + Fn+1^2.


 To the odd-index case we apply Cassini's identity: Fn^2 = Fn-1.Fn+1 - (-1)^n:

     F2n+1 = Fn+1^2 + Fn^2 +

           = Fn+1^2 + Fn+1Fn-1 - (-1)^n          // since by Cassini Fn^2 = Fn-1.Fn+1 - (-1)^n

           = Fn+1^2 + Fn+1(Fn+1 - Fn) - (-1)^n   // substituting for Fn-1

           = 2Fn+1^2 - Fn.Fn+1 - (-1)^n

           = Fn+1(2Fn+1 - Fn) - (-1)^n           // i.e. b(2b - a) - (-1)^n

 If n is odd, then a = F2n+1 and b = 2Fn+2, so we must derive the latter:

     F2n+2 = F2m where m = n+1 = Fm(2Fm+1 - Fm)

           = Fn+1(2F(n+2) - Fn+1)

           = Fn+1(2Fn+1 + 2Fn - Fn+1)            // Since Fn+2 = Fn + Fn+1

           = Fn+1(Fn+1 + 2Fn)                    // i.e. b(b+2a)


**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_doubling_no_memo_ibig_2.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig_2.rs)

---

### Script: fib_doubling_recursive_ibig.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number using the
 Fibonacci doubling identity. See also `demo/fib_doubling_iterative.rs` and
 `demo/fib_doubling_iterative_purge.rs` for non-recursive variations.

 I'm sure this is old hat, but I stumbled across an apparent pattern in the
 Fibonacci sequence:
 `For m > n: Fm = Fn-1.Fm-n + Fn.Fm-n+1.`

 This has a special case when m = 2n or 2n+1, which not surprisingly turn out
 to be well-known "doubling identities". The related technique is known as
 "fast doubling".

 For even indices: `F2n = Fn x (Fn-1 + Fn+1)`.
 For odd indices: `F2n+1 = Fn^2 + Fn+1^2`.

 This allows us to compute a given Fibonacci number F2n or F2n+1 by recursively
 or indeed iteratively expressing it in terms of Fn-1, Fn and Fn+1, or any two
 of these since Fn+1 = Fn-1 + Fn.

 I suggested this to ChatGPT, as well as the idea of pre-computing and storing the
 first 10 or 100 Fibonacci numbers to save repeated recalculation. ChatGPT went
 one better by memoizing all computed numbers. As there is a great deal of repetition
 and fanning out of calls to fib(), the memoization drastically cuts down recursion.



**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and memoization, and a good job by ChatGPT.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_doubling_recursive_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_recursive_ibig.rs)

---

### Script: fib_matrix.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.
 This example is by courtesy of Gemini AI. See big-number versions
 `demo/fib_matrix_dashu.rs` and `demo/fib_matrix_ibig.rs`.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demo an alternative to the standard computation for Fibonacci numbers.

**Crates:** 

**Type:** Snippet

**Link:** [fib_matrix.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix.rs)

---

### Script: fib_matrix_dashu.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.
 This example is by courtesy of Gemini AI. For F100,000 this is the
 fastest individual calculation, 3-4 times faster than the doubling
 method, and about 10 times faster than the classic iteration. For
 F1,000,000 to F10,000,000 it's overtaken by the doubling method.
 These are not formal benchmarks and your mileage may vary. Besides,
 these are only demo scripts and come with no guarantees.

 Aside from the imports, this script is interchangeable with `demo/fib_matrix_ibig.rs`
 and performance on my setup was very similar. However, `dashu` is
 not confined to integers but also supports floating point and rational
 numbers.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demo a very fast precise computation for large individual Fibonacci numbers.

**Crates:** `dashu`

**Type:** Snippet

**Link:** [fib_matrix_dashu.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_dashu.rs)

---

### Script: fib_matrix_ibig.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.
 This example is by courtesy of Gemini AI. For F100,000 this is the
 fastest individual calculation, 3-4 times faster than the doubling
 method, and about 10 times faster than the classic iteration. For
 F1,000,000 to F10,000,000 it's overtaken by the doubling method.
 These are not formal benchmarks and your mileage may vary. Besides,
 these are only demo scripts and come with no guarantees.

 Aside from the imports, this script is interchangeable with `demo/fib_matrix_dashu.rs`
 and performance on my setup was very similar. However, `dashu` is
 not confined to integers but also supports floating point and rational
 numbers.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demo a very fast precise computation for large individual Fibonacci numbers.

**Crates:** `ibig`

**Type:** Snippet

**Link:** [fib_matrix_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_ibig.rs)

---

### Script: fib_matrix_rug.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.

 Won't work with default Windows 11 because of the `rug` crate, which is a pity becaue
 `rug` is a beast due to its access to powerful GNU libraries.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.



**Purpose:** Demo a very fast precise computation for large individual Fibonacci numbers.

**Crates:** `rug`

**Type:** Snippet

**Link:** [fib_matrix_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_rug.rs)

---

### Script: fib_quadrupling_recursive_ibig.rs

**Description:**  A curiosity: In this version I tried doubling up the doubling technique by
 deriving formulae for F4n and F4n+1 in terms of Fn and Fn+1, but it didn't
 pay off in terms of speed. It's good to test the limits, but for practical
 purposes stick to the doubling algorithm.



**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and memoization.

**Crates:** `ibig`

**Type:** Program

**Link:** [fib_quadrupling_recursive_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_quadrupling_recursive_ibig.rs)

---

### Script: fizz_buzz_blandy_orendorff.rs

**Description:**  A fun example from Programming Rust by Jim Blandy and Jason Orendorff (O’Reilly).
 Copyright 2018 Jim Blandy and Jason Orendorff, 978-1-491-92728-1.
 Described by the authors as "a really gratuitous use of iterators".


**Purpose:** Demo using `thag_rs` to try out random code snippets ... also iterators.

**Crates:** 

**Type:** Snippet

**Link:** [fizz_buzz_blandy_orendorff.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fizz_buzz_blandy_orendorff.rs)

---

### Script: fizz_buzz_gpt.rs

**Description:**  GPT-generated fizz-buzz example.


**Purpose:** Demo running random snippets in thag_rs, also AI and the art of delegation ;)

**Crates:** 

**Type:** Snippet

**Link:** [fizz_buzz_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fizz_buzz_gpt.rs)

---

### Script: flume_async.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.


**Purpose:** demo of async and channel programming and of `flume` in particular.

**Crates:** 

**Type:** Program

**Link:** [flume_async.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_async.rs)

---

### Script: flume_perf.rs

**Description:**  Published example from the `flume` channel crate.


**Purpose:** demo of channel programming and of `flume` in particular.

**Crates:** 

**Type:** Program

**Link:** [flume_perf.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_perf.rs)

---

### Script: flume_select.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.


**Purpose:** demo of async and channel programming and of `flume` in particular.

**Crates:** `flume`

**Type:** Program

**Link:** [flume_select.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_select.rs)

---

### Script: gen_names.rs

**Description:**  A very simple published example from the random name generator
 `names`.


**Purpose:** Demo a simple snippet and featured crate.

**Crates:** `names`

**Type:** Snippet

**Link:** [gen_names.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_names.rs)

---

### Script: gen_readme.rs

**Description:**  This is the actual script used to collect demo script metadata and generate
 demo/README.md.

 Strategy and grunt work thanks to ChatGPT.


**Purpose:** Document demo scripts in a demo/README.md as a guide to the user.

**Crates:** `lazy_static`, `regex`, `thag_rs`

**Type:** Program

**Link:** [gen_readme.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_readme.rs)

---

### Script: git_dependency.rs

**Description:**  Demo the use of git dependencies in the toml block. Local path dependencies
 work the same way, e.g. `thag_rs = { git = "https://github.com/durbanlegend/thag_rs" },
 but obviously the path literal will be specific to your environment.


**Purpose:** Demo `git` dependencies, explain `path` dependencies.

**Crates:** `thag_rs`

**Type:** Program

**Link:** [git_dependency.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/git_dependency.rs)

---

### Script: git_dependency_snippet.rs

**Description:**  `demo/git_dependency.rs` done as a snippet, just because.


**Purpose:** Demo `git` dependencies as a snippet.

**Crates:** `thag_rs`

**Type:** Snippet

**Link:** [git_dependency_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/git_dependency_snippet.rs)

---

### Script: gpt_clap_derive.rs

**Description:**  GPT-generated CLI using the `clap` crate.


**Purpose:** Demonstrate `clap` CLI using the derive option.

**Crates:** `clap`

**Type:** Program

**Link:** [gpt_clap_derive.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gpt_clap_derive.rs)

---

### Script: gpt_lazy_static_theme.rs

**Description:**  Prototype of detecting the light or dark theme in use, and registering it
 as a static enum value for use in message style selection. Example of using
 an LLM to generate a prototype to a simple spec. The `clear_screen` function
 was added manually later. This prototype is one of many that was incorporated
 into `thag_rs`.


**Purpose:** Demo theme detection with `termbg`, clearing terminal state with `crossterm` and setting it as a static enum value using `lazy_static`.

**Crates:** `crossterm`, `lazy_static`, `termbg`

**Type:** Program

**Link:** [gpt_lazy_static_theme.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gpt_lazy_static_theme.rs)

---

### Script: hello.rs

**Description:**  Obligatory Hello World as a snippet


**Purpose:** Demo Hello World snippet

**Crates:** 

**Type:** Snippet

**Link:** [hello.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hello.rs)

---

### Script: hello_main.rs

**Description:**  Hello World as a program (posh Winnie-the-Pooh version)


**Purpose:** Demo Hello World as a program

**Crates:** 

**Type:** Program

**Link:** [hello_main.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hello_main.rs)

---

### Script: hello_minimal.rs

**Description:**  Minimalist Hello World snippet (poor Winnie-the-Pooh version)


**Purpose:** Demo Hello World reduced to an expression

**Crates:** 

**Type:** Snippet

**Link:** [hello_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hello_minimal.rs)

---

### Script: hyper_client.rs

**Description:**  Published echo-server HTTP client example from the `hyper` crate,
 with the referenced modules `support` and `tokiort` refactored
 into the script, while respecting their original structure and
 redundancies.
 You can run the `hyper_echo_server.rs` demo as the HTTP server on
 another command line and connect to it on port 3000:
 `thag_rs demo/hyper_client.rs -- http://127.0.0.1:3000`.
 Or use any other available HTTP server.


**Purpose:** Demo `hyper` HTTP client, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `pin_project_lite`, `tokio`

**Type:** Program

**Link:** [hyper_client.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_client.rs)

---

### Script: hyper_echo_server.rs

**Description:**  Published simple echo HTTP server example from the client crate,
 with the referenced modules `support` and `tokiort` refactored
 into the script, while respecting their original structure and
 redundancies.

 "This is our service handler. It receives a Request, routes on its
 path, and returns a Future of a Response."


**Purpose:** Demo `hyper` HTTP echo server, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `pin_project_lite`, `tokio`

**Type:** Program

**Link:** [hyper_echo_server.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_echo_server.rs)

---

### Script: ibig_big_integers.rs

**Description:**  Published example from the `ibig` crate, showcasing the use of the crate.


**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `ibig`

**Type:** Snippet

**Link:** [ibig_big_integers.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ibig_big_integers.rs)

---

### Script: iced_tour.rs

**Description:**  The full tour of the `iced` crate published in the `iced` examples.


**Purpose:** Show that `thag_rs` can handle product demos.

**Crates:** `iced`

**Type:** Program

**Link:** [iced_tour.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/iced_tour.rs)

---

### Script: in_place.rs

**Description:**  Published example from `in-place crate` disemvowels the file somefile.txt.


**Purpose:** Demo editing a file in place.

**Crates:** `in_place`

**Type:** Program

**Link:** [in_place.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/in_place.rs)

---

### Script: inline_colorization.rs

**Description:**  Published simple example from `inline_colorization` crate. Simple effective inline
 styling option for text messages.


**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `inline_colorization`

**Type:** Snippet

**Link:** [inline_colorization.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/inline_colorization.rs)

---

### Script: install_demo_gpt.rs

**Description:**  Prototype downloader for the demo/ directory.


**Purpose:** Prototype a possible solution.

**Crates:** `reqwest`, `rfd`, `serde`

**Type:** Program

**Link:** [install_demo_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/install_demo_gpt.rs)

---

### Script: iter.rs

**Description:**  Demo a simple iterator


**Purpose:** Show how basic a snippet can be.

**Crates:** 

**Type:** Snippet

**Link:** [iter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/iter.rs)

---

### Script: json.rs

**Description:**  Demo of using deserializing JSON with the featured crates.


**Purpose:** Demo featured crates.

**Crates:** `serde`, `serde_json`

**Type:** Snippet

**Link:** [json.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/json.rs)

---

### Script: list_files.rs

**Description:**  Demo listing files on disk. If you want a sorted list, you will need to amend the
 program to collect the entries into a Vec and sort that.


**Purpose:** Simple demonstration.

**Crates:** 

**Type:** Program

**Link:** [list_files.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/list_files.rs)

---

### Script: loop_closure.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
 the latter since I'm not sure what the closure logic buys us. It's
 going to be checked by the compiler anyway. Compare with `demo/loop_expr.rs`.


**Purpose:** Evaluate closure logic for line processing.

**Crates:** 

**Type:** Snippet

**Link:** [loop_closure.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_closure.rs)

---

### Script: loop_expr.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
 the latter since I'm not sure what the closure logic buys us. It's
 going to be checked by the compiler anyway. Compare with `demo/loop_closure.rs`.


**Purpose:** Evaluate expression logic for line processing.

**Crates:** 

**Type:** Snippet

**Link:** [loop_expr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_expr.rs)

---

### Script: loop_pre_post.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`, but with pre-
 and post-loop logic analogous to `awk`. I got GPT to do me this
 mock-up.


**Purpose:** Evaluate expression logic for line processing.

**Crates:** 

**Type:** Program

**Link:** [loop_pre_post.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_pre_post.rs)

---

### Script: macro_print.rs

**Description:**  Proof of concept of distinguishing types that implement Display from those that implement
 Debug, and printing using the Display or Debug trait accordingly. Worked out with recourse
 to ChatGPT for suggestions and macro authoring.


**Purpose:** May be interesting or useful.

**Crates:** 

**Type:** Program

**Link:** [macro_print.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_print.rs)

---

### Script: mock_edit.rs

**Description:**  Used to debug a doctest.


**Purpose:** Debugging script.

**Crates:** `crossterm`, `mockall`, `thag_rs`

**Type:** Snippet

**Link:** [mock_edit.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/mock_edit.rs)

---

### Script: multiline_err.rs

**Description:**  LLM-provided formatting for error messages


**Purpose:** Demo of formatting error messages

**Crates:** 

**Type:** Program

**Link:** [multiline_err.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/multiline_err.rs)

---

### Script: owo_cli_color_support.rs

**Description:**  Demo the use of a command-line interface to override the colour support to be provided.
 The owo-colors "supports-colors" feature must be enabled.


**Purpose:** Demo setting colour support via a very simple CLI.

**Crates:** `clap`, `owo_colors`

**Type:** Program

**Link:** [owo_cli_color_support.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_cli_color_support.rs)

---

### Script: owo_msg_colors_1_basic_gpt.rs

**Description:**  An early exploration of message colouring, GPT-generated.
 This one uses basic Ansi 16 colours. Try it on dark vs light
 backgrounds to see how some of the colours change.


**Purpose:** May be of use to some. Demo featured crates.

**Crates:** `crossterm`, `owo_colors`, `termbg`

**Type:** Program

**Link:** [owo_msg_colors_1_basic_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_msg_colors_1_basic_gpt.rs)

---

### Script: owo_msg_colors_2_adv_gpt.rs

**Description:**  More fully worked-out prototype of colouring and styling messages based on the level of
 colour support of the current terminal and whether a light or dark theme is currently
 selected. This was the result of good deal of exploration and dialog with ChatGPT.  Try it on dark vs light
 backgrounds to see how some of the same colours "pop" when shown against a light or dark theme
 and how some virtually or literally disappear when when not well matched to the theme.
 Fully worked-out demonstration of colouring and styling display messages according
 to message level.


**Purpose:** Demo detection of terminal colour support and dark or light theme, colouring and styling of messages, use of `strum` crate to get enum variant from string, and AI-generated code.

**Crates:** `crossterm`, `enum_assoc`, `log`, `owo_colors`, `strum`, `supports_color`, `termbg`

**Type:** Program

**Link:** [owo_msg_colors_2_adv_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_msg_colors_2_adv_gpt.rs)

---

### Script: owo_styles.rs

**Description:**  An early exploration of the idea of adaptive message colouring according to the terminal theme.


**Purpose:** Demo a simple example of adaptive message colouring, and the featured crates.

**Crates:** `crossterm`, `owo_colors`, `strum`, `termbg`

**Type:** Program

**Link:** [owo_styles.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_styles.rs)

---

### Script: parse_script_rs_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code using
 basic line-by-line comparison as opposed to a regular expression. I eventually
 decided to use a regular expression as I found it less problematic (see
 `demo/regex_capture_toml.rs`).


**Purpose:** Prototype

**Crates:** 

**Type:** Program

**Link:** [parse_script_rs_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/parse_script_rs_toml.rs)

---

### Script: parse_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code by locating
 the start and end of the toml block. I eventually decided to use a regular
 expression as I found it less problematic (see `demo/regex_capture_toml.rs`).


**Purpose:** Prototype

**Crates:** 

**Type:** Program

**Link:** [parse_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/parse_toml.rs)

---

### Script: pomprt_completion.rs

**Description:**  Published example from `pomprt` crate.


**Purpose:** Demo of `pomprt` readline implementation.

**Crates:** 

**Type:** Program

**Link:** [pomprt_completion.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/pomprt_completion.rs)

---

### Script: prettyplease.rs

**Description:**  Published example from `prettyplease` Readme.


**Purpose:** Demo featured crate.

**Crates:** 

**Type:** Program

**Link:** [prettyplease.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/prettyplease.rs)

---

### Script: profiling_puffin_demo.rs

**Description:**  Published demo from the `profiling` crate using the `puffin` profiler.
 We derive Deserialize/Serialize so we can persist app state on shutdown.


**Purpose:** Demo featured crates.

**Crates:** 

**Type:** Program

**Link:** [profiling_puffin_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/profiling_puffin_demo.rs)

---

### Script: puffin_profiler_egui.rs

**Description:**  Published demo from the `puffin` crate.


**Purpose:** Demo featured crate.

**Crates:** `eframe`

**Type:** Program

**Link:** [puffin_profiler_egui.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/puffin_profiler_egui.rs)

---

### Script: py_thag.rs

**Description:**  Demo of deriving Pythagorean triples.

 Pythagorean triples are integer tuples (a, b, c) such that a^2 + b^2 = c^2).
 They represent right-angled triangles with all sides having integer length in a given unit of measure.

 They form a tree with the root at (3, 4, 5), with each triple having 3 child triples.

 Per the Wikipedia page, the standard derivation is based on the formulae:

     1. a = m^2 - n^2
     2. b = 2mn
     3. c = m^2 + n^2
     where m > n > 0 and one is always even, the other always odd.

 The next 3 values of m and n, corresponding to the 3 child triples of (3, 4, 5) are
 derived by the following 3 formulae:

     (m1, n1) = (2m - n, m)
     (m2, n2) = (2m + n, m)
     (m3, n3) = (m + 2n, n)

 So let's work out the 3 child triples of (3, 4, 5).


**Purpose:** Recreational, educational.

**Crates:** 

**Type:** Snippet

**Link:** [py_thag.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/py_thag.rs)

---

### Script: ratatui_user_input.rs

**Description:**  Published example from the `ratatui` crate.


**Purpose:** Demo the featured crate.

**Crates:** `ratatui`

**Type:** Program

**Link:** [ratatui_user_input.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ratatui_user_input.rs)

---

### Script: readline_crossterm.rs

**Description:**  Published crossterm example.
 Demonstrates how to block read characters or a full line.
 Just note that crossterm is not required to do this and can be done with `io::stdin()`.


**Purpose:** Demo crossterm reading key events as a line or a single char.

**Crates:** `crossterm`

**Type:** Program

**Link:** [readline_crossterm.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/readline_crossterm.rs)

---

### Script: reedline_basic_keybindings.rs

**Description:**  Published example `basic.rs` from `reedline` crate.


**Purpose:** demo featured crates.

**Crates:** `crossterm`, `reedline`

**Type:** Program

**Link:** [reedline_basic_keybindings.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_basic_keybindings.rs)

---

### Script: reedline_completions.rs

**Description:**  Published example from `reedline` crate.


**Purpose:** demo featured crates.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_completions.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_completions.rs)

---

### Script: reedline_event_listener.rs

**Description:**  Published example from `reedline` crate.


**Purpose:** demo featured crates.

**Crates:** `crossterm`

**Type:** Program

**Link:** [reedline_event_listener.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_event_listener.rs)

---

### Script: reedline_highlighter.rs

**Description:**  Published example from `reedline` crate.


**Purpose:** Explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_highlighter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_highlighter.rs)

---

### Script: reedline_hinter.rs

**Description:**  Published example from `reedline` crate.


**Purpose:** Explore featured crate.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Link:** [reedline_hinter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_hinter.rs)

---

### Script: reedline_history.rs

**Description:**  Published example from `reedline` crate.


**Purpose:** Demo `reedline` file-backed history.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_history.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_history.rs)

---

### Script: reedline_ide_completions.rs

**Description:**  Published example from `reedline` crate. See the Vec of commands in the main method standing in for
 history. Enter a letter, e.g. "h" and press Tab to see the magic happen: all the commands starting
 with that letter will be displayed for selection with a tab, up and down arrows or Enter. Or you can
 enter subsequent letters to narrow the search. Noice.


**Purpose:** Demo `reedline` tab completions.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_ide_completions.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_ide_completions.rs)

---

### Script: reedline_list_bindings.rs

**Description:**  Published example from `reedline` crate.
 List all keybinding information


**Purpose:** Explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_list_bindings.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_list_bindings.rs)

---

### Script: reedline_multiline.rs

**Description:**  Exploratory prototype of REPL support for multi-line expressions. Based on published example
 `custom_prompt.rs` in `reedline` crate.


**Purpose:** Explore options for handling multi-line expressions in a REPL.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Link:** [reedline_multiline.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_multiline.rs)

---

### Script: reedline_read_stdin.rs

**Description:**  Basic exploration of reading a line from stdin with `reedline`.


**Purpose:** Exploring how to render prompts and read lines of input.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_read_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_read_stdin.rs)

---

### Script: reedline_repl.rs

**Description:**  Published example from `reedline-repl-rs` crate.


**Purpose:** Explore the suitability of this crate for a Rust REPL. Conclusion: it's more geared to commands.

**Crates:** `reedline_repl_rs`

**Type:** Program

**Link:** [reedline_repl.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_repl.rs)

---

### Script: reedline_repl_context.rs

**Description:**  Published example from `reedline-repl-rs` crate. This one uses the
 `clap` builder pattern; there is also one using the`clap` derive pattern.


**Purpose:** Evaluation of featured crate and of using clap to structure command input.

**Crates:** `reedline_repl_rs`

**Type:** Program

**Link:** [reedline_repl_context.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_repl_context.rs)

---

### Script: reedline_show_bindings.rs

**Description:**  Prototype of key binding display function for `reedline` REPL. This was developed
 by giving ChatGPT a simple spec which it flubbed, then repeatedly feeding back errors,
 manually corrected code and requests for changes until a nice simple display was
 achieved. This was then refined into the `keys` display of the `thag_rs` REPL, with
 the addition of command descriptions, non-edit commands such as SearchHistory, and colour-
 coding.


**Purpose:** Demo the end result of development dialog with ChatGPT.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_show_bindings.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_show_bindings.rs)

---

### Script: reedline_stdin.rs

**Description:**  Exploring `reedline` crate.


**Purpose:** explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Link:** [reedline_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_stdin.rs)

---

### Script: reedline_transient_prompt.rs

**Description:**  Published demo from `reedline` crate.


**Purpose:** Demo the use of a transient minimal prompt `! ` for returned history.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Link:** [reedline_transient_prompt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_transient_prompt.rs)

---

### Script: regex_capture_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code using
 a regular expression. I ended up choosing this approach as being less
 problematic than line-by-line parsing (see `demo/parse_script_rs_toml.rs`)
 See also `demo/regex_capture_toml.rs`.


**Purpose:** Prototype

**Crates:** `regex`

**Type:** Program

**Link:** [regex_capture_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/regex_capture_toml.rs)

---

### Script: rug_arbitrary_precision_nums.rs

**Description:**  Published example from the `rug` crate, showcasing the use of the crate. I added the
 last line to return a tuple of the state of the values of interest, as a quick way
 of displaying them.
 
 Won't work with default Windows 11 because of `rug` crate.
 On Linux you may need to install the m4 package.



**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `rug`

**Type:** Snippet

**Link:** [rug_arbitrary_precision_nums.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rug_arbitrary_precision_nums.rs)

---

### Script: rustfmt.rs

**Description:**  Prototype of invoking the Rust formatter programmatically, with the addition of an `rfd`
 (`Rusty File Dialogs`) cross-platform file chooser to select the file to format. The code
 for both was AI-generated because I find AI very handy for this kind of grunt work.


**Purpose:** Demo file chooser and calling an external program, in this case the Rust formatter.

**Crates:** `rfd`

**Type:** Program

**Link:** [rustfmt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustfmt.rs)

---

### Script: rustlings_smart_pointers_rc1.rs

**Description:**  Published exercise solution from the `rustlings` crate.


**Purpose:** Demo one way to preserve your `rustlings` solutions, for reference or as katas.

**Crates:** `super`

**Type:** Program

**Link:** [rustlings_smart_pointers_rc1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustlings_smart_pointers_rc1.rs)

---

### Script: rustyline_compl.rs

**Description:**  Published example from the `rustyline` crate.


**Purpose:** Demo using `thag_rs` to run a basic REPL as a script.

**Crates:** `rustyline`

**Type:** Program

**Link:** [rustyline_compl.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustyline_compl.rs)

---

### Script: rustyline_full.rs

**Description:**  Example from `rustyline` crate readme.
 MatchingBracketValidator uses matching brackets to decide between single- and multi-line
 input.


**Purpose:** Explore `rustyline` crate.

**Crates:** `rustyline`

**Type:** Program

**Link:** [rustyline_full.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustyline_full.rs)

---

### Script: slog_expressions.rs

**Description:**  Published example from `slog` crate (misc/examples/expressions.rs).


**Purpose:** Demo a popular logging crate.

**Crates:** `slog`

**Type:** Program

**Link:** [slog_expressions.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/slog_expressions.rs)

---

### Script: snippet_import_scope.rs

**Description:**  Demo scope of import statements.


**Purpose:** Prototype to confirm leaving imports in situ when wrapping snippets.

**Crates:** `ibig`

**Type:** Snippet

**Link:** [snippet_import_scope.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/snippet_import_scope.rs)

---

### Script: snippet_name_clash.rs

**Description:**  Demo scope of import statements. Two conflicting imports with the same name
 coexisting in the same println! invocation. Demonstrates that when wrapping
 a snippet we can't assume it's OK to pull the imports up to the top level.


**Purpose:** Prototype to confirm leaving imports in situ when wrapping snippets.

**Crates:** 

**Type:** Snippet

**Link:** [snippet_name_clash.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/snippet_name_clash.rs)

---

### Script: stdin.rs

**Description:**  A version of `thag_rs`'s `stdin` module to handle standard input editor input. Like the `colors`
 module, `stdin` was originally developed here as a separate script and integrated as a module later.
///
 E.g. `thag_rs demo/stdin.rs`


**Purpose:** Demo using `thag_rs` to develop a module outside of the project.

**Crates:** `lazy_static`, `ratatui`, `regex`, `thag_rs`, `tui_textarea`

**Type:** Program

**Link:** [stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/stdin.rs)

---

### Script: structopt_cli_gpt.rs

**Description:**  Basic demo of GPT-generated CLI using the `structopt` crate. This
 crate is in maintenance mode, its features having been integrated
 into `clap`.


**Purpose:** Demonstrate `structopt` CLI.

**Crates:** `structopt`

**Type:** Snippet

**Link:** [structopt_cli_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/structopt_cli_gpt.rs)

---

### Script: supports_color.rs

**Description:**  Demo of crate `supports-color` that `thag_rs` uses to detect the level of
 colour support of the terminal in use. I've added the `clear_screen` method
 because from testing I suspect that `supports-color` may mess with the terminal
 settings. Obviously that doesn't matter in a demo that exists before doing
 serious work, but it can wreak havoc with your program's output.


**Purpose:** Demo featured crate doing its job.

**Crates:** `crossterm`, `supports_color`

**Type:** Snippet

**Link:** [supports_color.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/supports_color.rs)

---

### Script: supports_color_win.rs

**Description:**  Windows-friendly logic extracted from crate `supports-color`.



**Purpose:** Proof of concept for Windows environment

**Crates:** 

**Type:** Snippet

**Link:** [supports_color_win.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/supports_color_win.rs)

---

### Script: syn_dump_syntax.rs

**Description:**  Published example from the `syn` crate. Description "Parse a Rust source file
 into a `syn::File` and print out a debug representation of the syntax tree."
 Pass it the absolute or relative path of any Rust source file, e.g. its own
 path that you passed to the script runner to invoke it.


**Purpose:** show off the power of `syn`.

**Crates:** `colored`

**Type:** Program

**Link:** [syn_dump_syntax.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_dump_syntax.rs)

---

### Script: syn_quote.rs

**Description:**  Prototype of a simple partial expression evaluator. It solicits a Rust expression and embeds
 it in a `println!` statement for use in generated code.

 E.g.:
 ```
 Enter an expression (e.g., 2 + 3):
 5 + 8
 rust_code=println ! ("result={}" , 5 + 8) ;
 ```
 Fun fact: you can paste the output into any of the `expr`, `edit`, `repl` or `stdin`
 modes of `thag_rs`, or even into a .rs file, and it will print out the value of the
 expression (in this case 13). Or you can do the same with the input (5 + 8) and it
 will do the same because `thag_rs` will detect and evaluate an expression in
 essentially the same way as this script does.


**Purpose:** demo expression evaluation (excluding compilation and execution) using the `syn` and `quote` crates.

**Crates:** `quote`, `syn`

**Type:** Program

**Link:** [syn_quote.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_quote.rs)

---

### Script: syn_remove_attributes.rs

**Description:**  Prototype of removing an inner attribute (`#![...]`) from a syntax tree. Requires the `visit-mut'
 feature of `syn`.


**Purpose:** Demonstrate making changes to a `syn` AST.

**Crates:** `quote`, `syn`

**Type:** Program

**Link:** [syn_remove_attributes.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_remove_attributes.rs)

---

### Script: syn_visit_extern_crate_expr.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_item_extern_crate`
 to process the nodes representing `extern crate` statements and fn `visit_expr` to initiate the tree
 traversal. This version expects the script contents to consist of a Rust expression.


**Purpose:** Demo featured crate.

**Crates:** `syn`

**Type:** Program

**Link:** [syn_visit_extern_crate_expr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_extern_crate_expr.rs)

---

### Script: syn_visit_extern_crate_file.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_item_extern_crate`
 to process the nodes representing `extern crate` statements and fn `visit_expr` to initiate the tree
 traversal. This version expects the script contents to consist of a full-fledged Rust program.


**Purpose:** Demo featured crate.

**Crates:** `syn`

**Type:** Program

**Link:** [syn_visit_extern_crate_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_extern_crate_file.rs)

---

### Script: syn_visit_node_type.rs

**Description:**  Demo of selectively modifying source code using `syn` and `quote`. This is from a solution posted by user Yandros on the Rust Playground
 in answer to a question asked on the Rust users forum. The discussion and Playground link are to be found here:
 https://users.rust-lang.org/t/writing-proc-macros-with-syn-is-there-a-way-to-visit-parts-of-the-ast-that-match-a-given-format/54733/4
 (This content is dual-licensed under the MIT and Apache 2.0 licenses according to the Rust forum terms of service.)


**Purpose:** Demo programmatically modifying Rust source code using `syn` and `quote`.

**Crates:** `quote`, `syn`

**Type:** Program

**Link:** [syn_visit_node_type.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_node_type.rs)

---

### Script: syn_visit_use_path_expr.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source expression passed to the script. Specifically the combination of fn `visit_use_path`
 to process the nodes representing `use` statements and fn `visit_expr` to initiate the tree
 traversal. This version expects the script contents to consist of a Rust expression.


**Purpose:** Demo featured crate.

**Crates:** `syn`

**Type:** Program

**Link:** [syn_visit_use_path_expr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_path_expr.rs)

---

### Script: syn_visit_use_path_file.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_use_path`
 to process the nodes representing `extern crate` statements and fn `visit_expr` to initiate the tree
 traversal. This version expects the script contents to consist of a full-fledged Rust program.


**Purpose:** Demo featured crate.

**Crates:** `syn`

**Type:** Program

**Link:** [syn_visit_use_path_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_path_file.rs)

---

### Script: syn_visit_use_rename.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to identify `use` statements that exist
 for the purpose of renaming a dependency so that we don't go looking for the temporary in the registry.
 Specifically the combination of fn `visit_use_rename` to process the nodes representing `extern crate`
 statements and fn `visit_expr` to initiate the tree traversal. This version expects the script contents
 to consist of a full-fledged Rust program.


**Purpose:** Demo featured crate.

**Crates:** `syn`

**Type:** Program

**Link:** [syn_visit_use_rename.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_rename.rs)

---

### Script: tempfile.rs

**Description:**  Published example from the `tempfile` readme.


**Purpose:** Demo featured crate.

**Crates:** 

**Type:** Program

**Link:** [tempfile.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tempfile.rs)

---

### Script: term_detection_pack.rs

**Description:**  A basic tool I cobbled together that uses different crates to a) test terminal
 types on different platforms, b) determine and cross-check if a light or dark
 theme is in use and c) determine the level of colour supported reported by
 the terminal.


**Purpose:** Allow checking of terminals on platforms to be supported, also test reliability of different crates.

**Crates:** `crossterm`, `supports_color`

**Type:** Snippet

**Link:** [term_detection_pack.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/term_detection_pack.rs)

---

### Script: termbg.rs

**Description:**  Published example from `termbg` readme, only I've added the `clear_screen` method
 because `termbg` messes with the terminal settings. Obviously that doesn't matter
 in a demo that exists before doing serious work, but having struggled with a
 persistent issue of rightward drift in println! output that turned out to be
 caused by an overlooked termbg call buried in the message colour resolution logic,
 I think it's important to make it a habit.

 Detects the light or dark theme in use, as well as the colours in use.


**Purpose:** Demo theme detection with `termbg` and clearing terminal state with `crossterm`.

**Crates:** `crossterm`

**Type:** Program

**Link:** [termbg.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/termbg.rs)

---

### Script: terminal_light.rs

**Description:**  Demo of `terminal_light`, a crate that "answers the question "Is the terminal dark
 or light?". I've added the `clear_screen` method because as is common, `terminal_light`
 interrogates the terminal with an escape sequence which may mess with its settings
 and compromise the program's output.


**Purpose:** Demo terminal-light interrogating the background color. Results will vary with OS and terminal type.

**Crates:** `crossterm`

**Type:** Snippet

**Link:** [terminal_light.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light.rs)

---

### Script: terminal_light_fading.rs

**Description:**  A fun published example from the `terminal-light` crate. "Demonstrate mixing
 any ANSI color with the background." I've added the `clear_screen` method
 because as is common, `terminal_light` interrogates the terminal with an
 escape sequence which may mess with its settings and compromise the
 program's output.


**Purpose:** Mostly recreational.

**Crates:** `coolor`, `crossterm`

**Type:** Program

**Link:** [terminal_light_fading.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light_fading.rs)

---

### Script: terminal_light_skins.rs

**Description:**  A published example from the `terminal-light` crate. A simple example of
 choosing an appropriate skin based on the terminal theme.


**Purpose:** Demo of the `terminal-light` crate.

**Crates:** `coolor`, `crossterm`

**Type:** Program

**Link:** [terminal_light_skins.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light_skins.rs)

---

### Script: thag_from_rust_script.rs

**Description:**  Converts embedded manifest format from `rust-script` to `thag`.


**Purpose:** Convenience for any `rust-script` user who wants to try out `thag`.

**Crates:** 

**Type:** Program

**Link:** [thag_from_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_from_rust_script.rs)

---

### Script: thag_to_rust_script.rs

**Description:**  Converts embedded manifest format from `thag` to `rust-script`.


**Purpose:** Convenience for any `thag` user who wants to try out `rust-script`.

**Crates:** 

**Type:** Program

**Link:** [thag_to_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_to_rust_script.rs)

---

### Script: time_cookbook.rs

**Description:**  Simple time demo pasted directly from Rust cookbook. Run without -q to show how
 `thag_rs` will find the missing chrono manifest entry and display a specimen
 toml block you can paste in at the top of the script.


**Purpose:** Demo cut and paste from a web source with Cargo search and specimen toml block generation.

**Crates:** `chrono`

**Type:** Program

**Link:** [time_cookbook.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/time_cookbook.rs)

---

### Script: tokio_hello_short.rs

**Description:**  Published example from `tokio` crate, with comments removed to work with `thag_rs` `repl` feature.
 Before running, start a server: `ncat -l 6142` in another terminal.


**Purpose:** Demo running `tokio` from `thag_rs`.

**Crates:** `tokio`

**Type:** Program

**Link:** [tokio_hello_short.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tokio_hello_short.rs)

---

### Script: tokio_hello_world.rs

**Description:**  Published example from `tokio` crate. Before running, start a server: `ncat -l 6142`
 in another terminal.


**Purpose:** Demo running `tokio` from `thag_rs`.

**Crates:** `tokio`

**Type:** Program

**Link:** [tokio_hello_world.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tokio_hello_world.rs)

---

### Script: tui_editor.rs

**Description:**  Demo a TUI (text user interface) editor based on the featured crates. This editor is locked
 down to two files at a time, because it was developed to allow editing of generated code and
 cargo.toml from the REPL, but was eventually dropped in favour of leaving the user to choose
 or default to a standard editor. A more minimalist version is used to edit stdin input in
 the `--edit (-d)` option of `thag_rs`.


**Purpose:** Demo TUI editor and featured crates, including `crossterm`.

**Crates:** `ratatui`, `tui_textarea`

**Type:** Program

**Link:** [tui_editor.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_editor.rs)

---

### Script: tui_scrollview.rs

**Description:**  Published example from `tui-scrollview` crate. Toml entries from crate's Cargo.toml.


**Purpose:** Explore TUI editing

**Crates:** `color_eyre`, `ratatui`, `tui_scrollview`

**Type:** Program

**Link:** [tui_scrollview.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_scrollview.rs)

---

### Script: tui_ta_editor.rs

**Description:**  Demo a TUI (text user interface) editor based on the featured crates. This editor is locked
 down to two files at a time, because it was developed to allow editing of generated code and
 cargo.toml from the REPL, but was eventually dropped in favour of leaving the user to choose
 or default to a standard editor. A more minimalist version is used to edit stdin input in
 the `--edit (-d)` option of `thag_rs`.


**Purpose:** Demo TUI editor and featured crates, including `crossterm`.

**Crates:** `ratatui`, `tui_textarea`

**Type:** Program

**Link:** [tui_ta_editor.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_ta_editor.rs)

---

### Script: tui_ta_minimal.rs

**Description:**  Demo a very minimal and not very useful TUI (text user interface) editor based on the featured crates.


**Purpose:** Demo TUI editor and featured crates, including `crossterm`, and the use of the `scopeguard` crate to reset the terminal when it goes out of scope.

**Crates:** `ratatui`, `tui_textarea`

**Type:** Program

**Link:** [tui_ta_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_ta_minimal.rs)

---

### Script: tui_tokio_editor_gpt.rs

**Description:**  GPT-provided demo of a very basic TUI (terminal user interface) editor using
 `tokio` and the `crossterm` / `ratatui` / `tui-textarea` stack. provides a blank editor
 screen on which you can capture lines of data. `Ctrl-D` closes the editor and simply
 prints the captured data.


**Purpose:** Exploring options for editing input. e.g. for a REPL.

**Crates:** `ratatui`, `tokio`, `tui_textarea`

**Type:** Program

**Link:** [tui_tokio_editor_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_tokio_editor_gpt.rs)

---

### Script: type_of_at_compile_time_1.rs

**Description:**  Use a trait to determine the type of an expression at compile time, provided all cases are known in advance.

 This is a slightly embellished version of user `phicr`'s answer on `https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable-in-rust`.

 See also `demo/type_of_at_compile_time_2.rs` for an alternative implementation.


**Purpose:** Demo expression type determination for static dispatch.

**Crates:** 

**Type:** Program

**Link:** [type_of_at_compile_time_1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_compile_time_1.rs)

---

### Script: type_of_at_compile_time_2.rs

**Description:**  Use a trait to determine the type of an expression at compile time, provided all cases are known in advance.

 Most upvoted and recommended answer on Stack Overflow page:
 https://stackoverflow.com/questions/34214136/how-do-i-match-the-type-of-an-expression-in-a-rust-macro/34214916#34214916

 Credit to Stack Overflow user `Francis Gagné`.

 See also `demo/type_of_at_compile_time_1.rs` for an alternative implementation.

 Seems to work very well provided all the types encountered are anticipated.


**Purpose:** Demo expression type determination for static dispatch.

**Crates:** `dashu`

**Type:** Program

**Link:** [type_of_at_compile_time_2.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_compile_time_2.rs)

---

### Script: type_of_at_run_time.rs

**Description:**  Typical basic (runtime) solution to expression type identification. See also `demo/determine_if_known_type_trait.rs`
 for what may be a better (compile-time) solution depending on your use case.


**Purpose:** Demo of runtime type identification.

**Crates:** `quote`, `syn`

**Type:** Program

**Link:** [type_of_at_run_time.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_run_time.rs)

---

### Script: ubig_product_gpt.rs

**Description:**  Implement trait std::iter::Product for `ibig::UBig`. Example provided by GPT.


**Purpose:** Educational / reference.

**Crates:** `ibig`

**Type:** Program

**Link:** [ubig_product_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ubig_product_gpt.rs)

---

### Script: unzip.rs

**Description:**  Very simple demo of the `unzip` iterator function.


**Purpose:** Demo

**Crates:** 

**Type:** Snippet

**Link:** [unzip.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/unzip.rs)

---

### Script: win_test_control.rs

**Description:**  This is the "control" test for the `demo/win_test_*.rs` scripts. It seems to reliably NOT swallow the first character.


**Purpose:** Show how crates *not* sending an OSC to the terminal in Windows will *not* the first character you enter to be swallowed.

**Crates:** 

**Type:** Program

**Link:** [win_test_control.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_control.rs)

---

### Script: win_test_supports_color.rs

**Description:**  This seems to intermittently swallow the very first character entered in Windows.


**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `supports_color`

**Type:** Program

**Link:** [win_test_supports_color.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_supports_color.rs)

---

### Script: win_test_termbg.rs

**Description:**  This seems to "reliably" swallow the very first character entered in Windows.


**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** 

**Type:** Program

**Link:** [win_test_termbg.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_termbg.rs)

---

### Script: win_test_termbg_reset.rs

**Description:**  This still seems to "reliably" swallow the very first character entered in Windows.
 The `crossterm` reset doesn't seem to help. My disappointment is immeasurable and
 my day is ruined.


**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `crossterm`

**Type:** Program

**Link:** [win_test_termbg_reset.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_termbg_reset.rs)

---

### Script: win_test_terminal_light.rs

**Description:**  This seems to "reliably" swallow the very first character entered in Windows.


**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** 

**Type:** Program

**Link:** [win_test_terminal_light.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_terminal_light.rs)

---

