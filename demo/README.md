## Running the scripts

`thag` uses `clap` for a standard command-line interface. Try `thag --help` (or -h) if
you get stuck.

### In its simplest form:


    thag <path to script>

###### E.g.:

    thag demo/hello.rs

### Passing options and arguments to a script:

Use `--` to separate options and arguments meant for the script from those meant for `thag` itself.

###### E.g.:

demo/fib_dashu_snippet.rs expects to be passed an integer _n_ and will compute the _nth_ number in the
Fibonacci sequence.

     thag demo/fib_dashu_snippet.rs -- 100

### Full syntax:

    thag [THAG OPTIONS] <path to script> [-- [SCRIPT OPTIONS] <script args>]

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

Running the source from `thag` looks similar, we just replace `clap_tut_builder_01` by `thag demo/clap_tut_builder_01.rs --`:

*thag demo/clap_tut_builder_01.rs --* -dd -c my.cfg my_file test -l

Any parameters for `thag` should go before the `--`, e.g. we may choose use -qq to suppress `thag` messages:

    thag demo/clap_tut_builder_01.rs -qq -- -dd -c my.cfg my_file test -l

which will give identical output to the above.



##### Remember to use `--` to separate options and arguments that are intended for `thag` from those intended for the target script.

### TODO: check:
For detailed documentation on the `category_enum` procedural macro, see [category_enum](proc_macros/docs/category_enum.md).

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

**Categories:** AST, technique

**Link:** [analyze_snippet_1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_1.rs
```

---

### Script: analyze_snippet_2.rs

**Description:**  Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
 to detect whether a snippet returns a value that we should print out, or whether
 it does its own printing.

 Part 2: ChatGPT responds to feedback with an improved algorithm.

**Purpose:** Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** AST, technique

**Link:** [analyze_snippet_2.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_2.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_2.rs
```

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

**Categories:** AST, technique

**Link:** [analyze_snippet_3.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_3.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/analyze_snippet_3.rs
```

---

### Script: any.rs

**Description:**  Demo determining at run-time whether an expression returns a unit value
 so that it can be handled appropriately. When using a code template this is
 short and sweet, but it has to be included in the template and thus the
 generated code, whereas using an AST is quite a mission but works with
 any arbitrary snippet and doesn't pollute the generated source code.

**Purpose:** Demo Rust's answer to dynamic typing.

**Type:** Snippet

**Categories:** type_identification, technique

**Link:** [any.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/any.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/any.rs
```

---

### Script: bitflags.rs

**Description:**  Try out the `bitflags` crate.

**Purpose:** Explore use of `bitflags` to control processing.

**Crates:** `bitflags`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [bitflags.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bitflags.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/bitflags.rs
```

---

### Script: borrow_wrapped.rs

**Description:**  Snippet demonstrating how to reference or clone a wrapped value without
 falling foul of the borrow checker.

**Purpose:** Demo a borrow-checker-friendly technique for accessing a wrapped value.

**Type:** Snippet

**Categories:** technique

**Link:** [borrow_wrapped.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/borrow_wrapped.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/borrow_wrapped.rs
```

---

### Script: bpaf_cargo_show_asm.rs

**Description:**  Published example from `https://github.com/pacak/bpaf/src/docs2/derive_show_asm.md`

 E.g. `thag demo/bpaf_cargo_show_asm.rs -- -h`

**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [bpaf_cargo_show_asm.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_cargo_show_asm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_cargo_show_asm.rs
```

---

### Script: bpaf_cmd_chain.rs

**Description:**  Example from bpaf crate docs2/src/adjacent_command/derive.rs.

 E.g. `thag demo/bpaf_cmd-chain.rs -- eat Fastfood drink --coffee sleep --time=5`

**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf_derive`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [bpaf_cmd_chain.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_cmd_chain.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_cmd_chain.rs
```

---

### Script: bpaf_derive.rs

**Description:**  Example from bpaf crate docs2/src/command/derive.rs.

 E.g. `demo/bpaf_cmd_ex.rs -- --flag cmd --flag --arg=6`

**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf_derive`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [bpaf_derive.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_derive.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/bpaf_derive.rs
```

---

### Script: cargo_capture_output.rs

**Description:**  Run a command (in this case a cargo search for the `log` crate),
 and capture and print its stdout and stderr concurrently in a
 separate thread.

**Purpose:** Demo process::Command with output capture.

**Crates:** `env_logger`, `log`

**Type:** Program

**Categories:** technique

**Link:** [cargo_capture_output.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_capture_output.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_capture_output.rs
```

---

### Script: cargo_lookup.rs

**Description:**  Explore querying crates.io information for a crate.

 Format: `thag demo/cargo_lookup.rs -- <crate_name>`

**Purpose:** Proof of concept

**Crates:** `cargo_lookup`

**Type:** Program

**Categories:** crates, technique

**Link:** [cargo_lookup.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_lookup.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_lookup.rs -- serde
```

---

### Script: cargo_output.rs

**Description:**  Run a command (in this case a cargo search for the `log` crate),
 and capture and print its stdout and stderr concurrently in a
 separate thread.

**Purpose:** Demo process::Command with output capture.

**Crates:** `env_logger`, `log`

**Type:** Program

**Categories:** technique

**Link:** [cargo_output.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_output.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/cargo_output.rs
```

---

### Script: clap_enum_strum.rs

**Description:**  Exploring using clap with an enum, in conjunction with strum.
 E.g. `thag demo/clap_enum_strum.rs -- variant-num2`

**Purpose:** Simple demo of featured crates, contrasting how they expose variants.

**Crates:** `clap`, `serde`, `strum`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_enum_strum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_enum_strum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_enum_strum.rs
```

---

### Script: clap_num_arg.rs

**Description:**  `clap` with a numeric option.

 E.g. `thag demo/clap_num_arg.rs -- 45`

**Purpose:** Basic demo of `clap` parsing a numeric argument

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_num_arg.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_num_arg.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_num_arg.rs
```

---

### Script: clap_repl_crate_rustyline.rs

**Description:**  Older version of published clap_repl crate example, modified to prototype a
 (dummy) Rust REPL.

**Purpose:** Yet another REPL demo, this time using `rustyline`.

**Crates:** `clap`, `clap_repl`, `console`, `quote`, `rustyline`, `syn`

**Type:** Program

**Categories:** REPL, technique

**Link:** [clap_repl_crate_rustyline.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_repl_crate_rustyline.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_repl_crate_rustyline.rs
```

---

### Script: clap_repl_diy.rs

**Description:**  Example from the clap cookbook, not using the `clap-repl` crate.
 Can't find a keybinding to navigate history, unlike `clap_repl_crate_rustyline.rs`.

**Purpose:** Demo building a repl using `clap` directly.

**Crates:** `clap`, `shlex`

**Type:** Program

**Categories:** REPL, technique

**Link:** [clap_repl_diy.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_repl_diy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_repl_diy.rs
```

---

### Script: clap_tut_builder_01_quick.rs

**Description:**  Published example from `clap` tutorial (builder)

 E.g.  `thag demo/clap_tut_builder_01_quick.rs -- -ddd -c dummy.cfg my_file test -l`

**Purpose:** Demonstrate `clap` CLI using the builder option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_builder_01_quick.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_builder_01_quick.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_builder_01_quick.rs
```

---

### Script: clap_tut_derive_03_04_subcommands.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. thag demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_derive_03_04_subcommands.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_03_04_subcommands.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob
```

---

### Script: clap_tut_derive_04_01_enum.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. `thag demo/clap_tut_derive_04_01_enum.rs -- fast`

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_derive_04_01_enum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_04_01_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_04_01_enum.rs -- fast
```

---

### Script: clap_tut_derive_04_03_relations.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. `thag demo/clap_tut_derive_04_03_relations.rs -- --major -c config.toml --spec-in input.txt`

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_derive_04_03_relations.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_04_03_relations.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/clap_tut_derive_04_03_relations.rs -- --major -c config.toml --spec-in input.txt
```

---

### Script: cmd_args.rs

**Description:**  A prototype of the `cmd_args` module of thag_rs itself.

 E.g. `thag -tv demo/cmd_args.rs -- -gbrtv demo/hello.rs -- -fq Hello world`

**Purpose:** Prototype CLI.

**Crates:** `bitflags`, `clap`

**Type:** Program

**Categories:** CLI, crates, prototype, technique

**Link:** [cmd_args.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args.rs -- -gbrtv demo/hello.rs -- -fq Hello world
```

---

### Script: cmd_args_bpaf_gpt.rs

**Description:**  Example of a CLI using the bpaf crate instead of clap, originally generated by ChatGPT.
 See `demo/cmd_args_clap.rs` for comparison.

 E.g. `thag -tv demo/cmd_args_bpaf_gpt.rs -- -gbrtv demo/hello.rs -- -fq Hello world`

**Purpose:** Demo one lighter-weight alternative to clap.

**Crates:** `bitflags`, `bpaf_derive`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [cmd_args_bpaf_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args_bpaf_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args_bpaf_gpt.rs -- -gbrtv demo/hello.rs -- -fq Hello world
```

---

### Script: cmd_args_clap.rs

**Description:**  Basic CLI example using clap.

 E.g. `thag -t demo/cmd_args_clap.rs -- -atv hello.sh`

**Purpose:** For comparison with `demo/cmd_args_bpaf_gpt.rs`.

**Crates:** `bitflags`, `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [cmd_args_clap.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args_clap.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/cmd_args_clap.rs -- -atv hello.sh
```

---

### Script: color_contrast.rs

**Description:**  Given a sample RGB colour value, determine whether it would
 contrast better with black or white (background or foreground).
 Can't recall provenance, but the luminance formula is one of
 many discussed here:
 https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color

**Purpose:** Choose black or white as a contrasting colour for a given colour.

**Type:** Program

**Categories:** technique

**Link:** [color_contrast.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/color_contrast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/color_contrast.rs
```

---

### Script: colors.rs

**Description:**  Runner for current version of `src/colors.rs`, as it's become too enmeshed with other modules to split out nicely.
 We just borrow the main method here and add all the necessary dependencies and imports.

 E.g. `thag demo/colors.rs`

**Purpose:** Test the look of the various colours.

**Crates:** `nu_ansi_term`, `strum`, `termbg`, `thag_rs`

**Type:** Program

**Categories:** testing

**Link:** [colors.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/colors.rs)

**Not suitable to be run from a URL.**


---

### Script: colors_old.rs

**Description:**  An older version of `thag_rs`'s `colors` module to style messages according to their type. Like the `stdin`
 module, `colors` was originally developed here as a separate script and integrated as a module later.

 E.g. `thag demo/colors_old.rs`

**Purpose:** Demo using `thag_rs` to develop a module outside of the project.

**Crates:** `lazy_static`, `nu_ansi_term`, `strum`, `supports_color`, `termbg`, `thag_rs`

**Type:** Program

**Categories:** prototype, technique

**Link:** [colors_old.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/colors_old.rs)

**Not suitable to be run from a URL.**


---

### Script: colors_orig.rs

**Description:**  Original prototype of `thag_rs`'s `colors` module to style messages according
 to their type. I only dropped `owo-colors` because I switched from `rustyline` to
 `reedline`, which was already using `nu_ansi_term`.


**Purpose:** Demo older alternative implementation of `colors` module using `owo-colors`.

**Crates:** `log`, `owo_colors`, `strum`, `supports_color`, `termbg`, `thag_rs`

**Type:** Program

**Categories:** prototype, technique

**Link:** [colors_orig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/colors_orig.rs)

**Not suitable to be run from a URL.**


---

### Script: config.rs

**Description:**  Prototype of configuration file implementation. Delegated the grunt work to ChatGPT.
 Initializes and returns the configuration.
 A struct for use in normal execution, as opposed to use in testing.
 Open the configuration file in an editor.
 # Errors
 Will return `Err` if there is an error editing the file.
 # Panics
 Will panic if it can't create the parent directory for the configuration.
 Main function for use by testing or the script runner.

**Purpose:** Develop a configuration file implementation for `thag_rs`.

**Crates:** `edit`, `firestorm`, `home`, `mockall`, `nu_ansi_term`, `serde`, `serde_with`, `thag_rs`, `toml`

**Type:** Program

**Categories:** prototype, technique

**Link:** [config.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/config.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/config.rs
```

---

### Script: count_main_methods.rs

**Description:**  Prototype of a function required by thag_rs to count the main methods
 in a script to decide if it's a program or a snippet. Uses the `syn`
 visitor pattern. This is more reliable than a simple source code search
 which tends to find false positives in string literals and comments.

**Purpose:** Demo prototyping with thag_rs and use of the `syn` visitor pattern to visit nodes of interest

**Crates:** `syn`

**Type:** Program

**Categories:** AST, prototype, technique

**Link:** [count_main_methods.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/count_main_methods.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/count_main_methods.rs
```

---

### Script: create_next_file.rs

**Description:**  Prototype of creating files named sequentially from repl_000000.rs to
 repl_999999.rs in a thag_rs/demo subdirectory of the OS's temporary
 directory. The need is to generate well-behaved and consistent human-readable
 names for temporary programs generated from REPL expressions.

**Purpose:** Demo sequential file creation and the kind of code that is well suited to generation by an LLM.

**Type:** Program

**Categories:** technique

**Link:** [create_next_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/create_next_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/create_next_file.rs
```

---

### Script: crokey_deser.rs

**Description:**  Published example of serde deserialisation from `crokey` crate.

**Purpose:** Demo loading keybindings from a file.

**Crates:** `crokey`, `serde`, `toml`

**Type:** Program

**Categories:** crates, technique

**Link:** [crokey_deser.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_deser.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_deser.rs
```

---

### Script: crokey_print_key.rs

**Description:**  Published example of combiner from `crokey` crate.

**Purpose:** Demo key combiner.

**Crates:** `crokey`, `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crokey_print_key.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_print_key.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_print_key.rs
```

---

### Script: crokey_print_key_no_combiner.rs

**Description:**  Published example of KeyCombination from `crokey` crate.

**Purpose:** Demo key combination without Combiner.

**Crates:** `crokey`, `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crokey_print_key_no_combiner.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_print_key_no_combiner.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crokey_print_key_no_combiner.rs
```

---

### Script: crossbeam_channel_fibonacci.rs

**Description:**  `crossbeam-channel` published example.

 An asynchronous fibonacci sequence generator.

**Purpose:** Demo featured crate.

**Crates:** `crossbeam_channel`

**Type:** Program

**Categories:** crates

**Link:** [crossbeam_channel_fibonacci.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_fibonacci.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_fibonacci.rs
```

---

### Script: crossbeam_channel_matching.rs

**Description:**  `crossbeam-channel` published example
 Using `select!` to send and receive on the same channel at the same time.

**Purpose:** Demo featured crates.

**Crates:** `crossbeam_channel`, `crossbeam_utils`

**Type:** Program

**Categories:** crates

**Link:** [crossbeam_channel_matching.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_matching.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_matching.rs
```

---

### Script: crossbeam_channel_stopwatch.rs

**Description:**  `crossbeam-channel` published example.

 Prints the elapsed time every 1 second and quits on `Ctrl+C`. You can reinstate the separate main method for
 Windows provided you run the script with the `--multimain (-m)` option.

**Purpose:** showcase featured crates.

**Crates:** `crossbeam_channel`, `signal_hook`

**Type:** Program

**Categories:** crates

**Link:** [crossbeam_channel_stopwatch.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_stopwatch.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_channel_stopwatch.rs
```

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

**Categories:** crates, technique

**Link:** [crossbeam_epoch_sanitize.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_epoch_sanitize.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossbeam_epoch_sanitize.rs
```

---

### Script: crossterm.rs

**Description:**  Published example from crossterm crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/README.md

**Purpose:** Demo crossterm terminal manipulation.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm.rs
```

---

### Script: crossterm_alternate_screen.rs

**Description:**  Published example from crossterm crate. Macro version of the example:
 "Print a rectangle colored with magenta and use both direct execution and lazy execution."
 Direct execution with `execute` and lazy execution with `queue`.

 Url: https://docs.rs/crossterm/latest/crossterm/

**Purpose:** Demo `crossterm` command API.

**Crates:** `ratatui`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm_alternate_screen.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_alternate_screen.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_alternate_screen.rs -- true
```

---

### Script: crossterm_command_macro.rs

**Description:**  Published example from crossterm crate. Macro version of the example:
 "Print a rectangle colored with magenta and use both direct execution and lazy execution."
 Direct execution with `execute` and lazy execution with `queue`.

 Url: https://docs.rs/crossterm/latest/crossterm/

**Purpose:** Demo `crossterm` command API.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm_command_macro.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_command_macro.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_command_macro.rs
```

---

### Script: crossterm_event_read.rs

**Description:**  Published example from crossterm crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/examples/event-read.rs
 "Demonstrates how to block read events."

**Purpose:** Demo running crate example code, `crossterm` events.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm_event_read.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_event_read.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_event_read.rs
```

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

**Categories:** crates, technique

**Link:** [crossterm_key_events.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_key_events.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/crossterm_key_events.rs
```

---

### Script: ctrlc_demo.rs

**Description:**  Published example from `ctrlc` crate: "Cross platform handling of Ctrl-C signals."

**Purpose:** Demo one option for intercepting Ctrl-C.

**Crates:** `ctrlc`

**Type:** Program

**Categories:** crates, technique

**Link:** [ctrlc_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ctrlc_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/ctrlc_demo.rs
```

---

### Script: curl.rs

**Description:**  Simple HTTPS GET

 This example is a Rust adaptation of the [C example of the same
 name](https://curl.se/libcurl/c/https.html).
 On Linux you may need to install `pkg-config` and `libssl-dev`.

**Purpose:** Demo `curl` implementation.

**Crates:** `curl`

**Type:** Program

**Categories:** crates, technique

**Link:** [curl.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/curl.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/curl.rs
```

---

### Script: darling_consume_fields.rs

**Description:**  Published example from `darling` crate showing parsing for derive input.
 Extended to show formatted version of emitted code.

**Purpose:** Explore `darling` crate.

**Crates:** `darling`, `proc_macro2`, `quote`, `syn`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [darling_consume_fields.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/darling_consume_fields.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/darling_consume_fields.rs
```

---

### Script: darling_struct.rs

**Description:**  Published example from `darling` crate showing parsing for derive input.
 Extended to show formatted version of emitted code.

**Purpose:** Explore `darling` crate.

**Crates:** `darling`, `proc_macro2`, `quote`, `syn`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [darling_struct.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/darling_struct.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/darling_struct.rs
```

---

### Script: derive_deftly.rs

**Description:**  Introductory example from the `derive-deftly` user guide.

**Purpose:** Explore proc macro alternatives.

**Crates:** `derive_deftly`

**Type:** Snippet

**Categories:** crates, exploration, technique

**Link:** [derive_deftly.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/derive_deftly.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/derive_deftly.rs
```

---

### Script: dethag_re.rs

**Description:**  Unescape `\n` and `\\` markers in a string to convert the wall of text to readable lines.
 This version using regex may be more reliable than the classic approach using .lines().
 However, at time of writing, `regex` is a 248kB crate, which makes the binary of this
 module more than 5 time larger than that of `thagomizer`.

 Tip: Regex tested using https://rustexp.lpil.uk/.

**Purpose:** Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.

**Crates:** `lazy_static`, `regex`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [dethag_re.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/dethag_re.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/dethag_re.rs
```

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

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [dethagomizer.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/dethagomizer.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/dethagomizer.rs
```

---

### Script: documented.rs

**Description:**  Published example from the `documented` crate.
 Trying is the first step to failure.

**Purpose:** Explore making docs available at runtime.

**Crates:** `documented`

**Type:** Snippet

**Categories:** crates, exploration, technique

**Link:** [documented.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/documented.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/documented.rs
```

---

### Script: documented_dependencies.rs

**Description:**  Use the `documented` crate to iterate through struct fields and their docs at runtime.
 Dependency handling

**Purpose:** Prototype for `thag_config_builder`.

**Crates:** `documented`, `phf`, `serde`, `serde_with`

**Type:** Snippet

**Categories:** crates, prototype, technique

**Link:** [documented_dependencies.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/documented_dependencies.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/documented_dependencies.rs
```

---

### Script: download_demo_dir.rs

**Description:**  Downloader for the `demo` directory. Basics courtesy of GPT.

**Purpose:** Download the demo directory from Github main.

**Crates:** `reqwest`, `rfd`, `serde`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [download_demo_dir.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/download_demo_dir.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/download_demo_dir.rs
```

---

### Script: duration_snippet.rs

**Description:**  Minimal snippet showing how to add nice additional constructors such as `from_weeks` (and days, hours and
 minutes) to `std::time::Duration`.

 These are enabled by adding the inner attribute `#![feature(duration_constructors)]` to the script.
 I've used a snippet to illustrate that this is possible: an inner attribute (i.e. an attribute prefixed
 with `#!` (`#![...]`)) must be placed at the top of the crate it applies to, so when wrapping the snippet
 in a fn main, thag_rs pulls any inner attributes out to the top of the program.

 Notice we also have a shebang so that this script may be run as `demo/duration_snippet.rs` with execute
 permission. The shebang must be on the very first line but coexists peacefully with the inner attribute.

 See tracking issue https://github.com/rust-lang/rust/issues/120301 for the `Duration` constructor issue..

 E.g. `(*nix)`:

     chmod u+g demo/duration_snippet.rs      // Only required the first time of course
     demo/duration_snippet.rs -qq
     1209600s

 Or more concisely:

     f=demo/duration_snippet.rs && chmod u+g $f && $f -qq
     1209600s


**Purpose:** Demonstrate that some fairly subtle moves are possible even with the simplest of snippets.

**Type:** Snippet

**Categories:** technique

**Link:** [duration_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/duration_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/duration_snippet.rs
```

---

### Script: edit.rs

**Description:**  Published example from edit crate readme.

 Will use the editor specified in VISUAL or EDITOR environment variable.

 E.g. `EDITOR=vim thag_rs demo/edit.rs`

**Purpose:** Demo of edit crate to invoke preferred editor.

**Crates:** `edit`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [edit.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/edit.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/edit.rs
```

---

### Script: egui_code_editor.rs

**Description:**  A prototype GUI editor with saved state and syntax highlighting.

**Purpose:** Prototype a native-mode editor using the `egui` crate.

**Crates:** `eframe`, `egui`, `egui_extras`, `env_logger`

**Type:** Program

**Categories:** crates, prototype

**Link:** [egui_code_editor.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/egui_code_editor.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/egui_code_editor.rs
```

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

**Categories:** crates, prototype, technique

**Link:** [enum_select.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/enum_select.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/enum_select.rs
```

---

### Script: factorial_dashu_product.rs

**Description:**  Fast factorial algorithm with arbitrary precision and avoiding recursion.
 Closures and functions are effectively interchangeable here.

  Using the `std::iter::Product` trait - if implemented - is the most concise
 factorial implementation. `dashu` implements it, so it's straightforward to use.


**Purpose:** Demo snippet, `dashu` crate, factorial using `std::iter::Product` trait.

**Crates:** `dashu`

**Type:** Snippet

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [factorial_dashu_product.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_dashu_product.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_dashu_product.rs -- 50
```

---

### Script: factorial_ibig.rs

**Description:**  Fast factorial algorithms with arbitrary precision and avoiding recursion.
 A version using `std::Iterator::fold` and one using `std::iter::Successors:successors`
 are executed and compared to ensure they agree before printing out the value.
 Closures and functions are effectively interchangeable here.

 `let foo = |args| -> T {};` is equivalent to `fn foo(args) -> T {}`.

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [factorial_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_ibig.rs -- 50
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [factorial_ibig_product.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_ibig_product.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_ibig_product.rs -- 50
```

---

### Script: factorial_main_u128_product.rs

**Description:**  Fast factorial algorithm avoiding recursion, but limited to a maximum of `34!` by using only
 Rust primitives.

**Purpose:** Demo fast limited-scale factorial using Rust primitives and std::iter::Product trait.

**Type:** Program

**Categories:** educational, math, recreational, technique

**Link:** [factorial_main_u128_product.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_main_u128_product.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/factorial_main_u128_product.rs -- 34
```

---

### Script: fib_4784969_cpp_ibig.rs

**Description:**  Rust port of C++ example from `https://github.com/ZiCog/fibo_4784969` - so named because
 F(4784969) is the first number in the Fibonacci sequence that has one million decimal
 digits. This contains 3 alternative algorithms to compare their speed, with `fibo_new`
 edging out `fibo` at this scale.

 E.g.: `thag demo/fib_4784969_cpp_ibig.rs -- 4784969   // or any positive integer`


**Purpose:** Demo 3 very fast Fibonacci algorithms, though still 7-11 times slower than `rug`.

**Crates:** `ibig`

**Type:** Program

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_4784969_cpp_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_4784969_cpp_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_4784969_cpp_ibig.rs -- 50
```

---

### Script: fib_4784969_cpp_rug.rs

**Description:**  Rust port of C++ example from `https://github.com/ZiCog/fibo_4784969` - so named because
 F(4784969) is the first number in the Fibonacci sequence that has one million decimal
 digits. This contains 3 alternative algorithms to compare their speed, with `fibo_new`
 edging out `fibo` at this scale.

 The `rug` crate runs blindingly fast, but I for one found it very difficult to get this to compile.

 E.g.: `thag demo/fib_4784969_cpp_ibig.rs -- 4784969   // or any positive integer`


**Purpose:** Demo 3 very fast Fibonacci algorithms (F(4784969) in 0.33 to 0.58 sec for me).

**Crates:** `rug`

**Type:** Program

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_4784969_cpp_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_4784969_cpp_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_4784969_cpp_rug.rs -- 50
```

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

**Categories:** educational, math, recreational, technique

**Link:** [fib_basic.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_basic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_basic.rs -- 90
```

---

### Script: fib_basic_ibig.rs

**Description:**  Big-number (and thus more practical) version of `demo/fib_basic.rs`.


**Purpose:** Demo using a big-number crate to avoid the size limitations of primitive integers.

**Crates:** `ibig`, `itertools`

**Type:** Snippet

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_basic_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_basic_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_basic_ibig.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_big_clap_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_big_clap_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_big_clap_rug.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_binet_astro_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_binet_astro_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_binet_astro_snippet.rs -- 100
```

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

**Type:** Snippet

**Categories:** educational, math, recreational, technique

**Link:** [fib_binet_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_binet_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_binet_snippet.rs -- 100
```

---

### Script: fib_classic_ibig.rs

**Description:**  Fast non-recursive classic Fibonacci individual calculation with big integers.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.


**Purpose:** Demonstrate snippets and a fast non-recursive fibonacci algorithm using the `successors` iterator.

**Crates:** `ibig`

**Type:** Snippet

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_classic_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_classic_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_classic_ibig.rs -- 100
```

---

### Script: fib_classic_ibig_instrumented.rs

**Description:**  Same script as `demo/fib_basic_ibig.rs` with basic instrumentation added for benchmarking
 against other fibonacci scripts.
 Scripts can then be selected and run sequentially.

 E.g. an apples-with-apples comparison of different algorithms implemented using the `ibig` crate:

 `ls -1 demo/fib*ibig*.rs | grep -v fib_basic_ibig.rs | while read f; do echo $f; thag_rs -t $f -- 10000000; done`

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.


**Purpose:** Demonstrate instrumenting scripts for benchmarking.

**Crates:** `ibig`

**Type:** Snippet

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_classic_ibig_instrumented.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_classic_ibig_instrumented.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_classic_ibig_instrumented.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_dashu_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_dashu_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_dashu_snippet.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_iterative_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_ibig.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_iterative_purge_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_purge_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_purge_ibig.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_iterative_purge_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_purge_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_iterative_purge_rug.rs -- 100
```

---

### Script: fib_doubling_no_memo_ibig.rs

**Description:**  A version of `demo/fib_doubling_recursive.rs`, minus the memoization.
 This serves to prove that the memoization is significantly faster, although
 not dramatically so.


**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_no_memo_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_no_memo_ibig_1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig_1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig_1.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_no_memo_ibig_2.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig_2.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_no_memo_ibig_2.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_doubling_recursive_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_recursive_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_doubling_recursive_ibig.rs -- 100
```

---

### Script: fib_matrix.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.
 This example is by courtesy of Gemini AI. See big-number versions
 `demo/fib_matrix_dashu.rs` and `demo/fib_matrix_ibig.rs`.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.


**Purpose:** Demo an alternative to the standard computation for Fibonacci numbers.

**Type:** Snippet

**Categories:** educational, math, recreational, technique

**Link:** [fib_matrix.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_matrix_dashu.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_dashu.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_dashu.rs -- 100
```

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

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_matrix_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_ibig.rs -- 100
```

---

### Script: fib_matrix_rug.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.

 Won't work with default Windows 11 because of the `rug` crate, which is a pity because
 `rug` is a beast due to its access to powerful GNU libraries.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.


**Purpose:** Demo a very fast precise computation for large individual Fibonacci numbers.

**Crates:** `rug`

**Type:** Snippet

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_matrix_rug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_matrix_rug.rs -- 100
```

---

### Script: fib_quadrupling_recursive_ibig.rs

**Description:**  A curiosity: In this version I tried doubling up the doubling technique by
 deriving formulae for F4n and F4n+1 in terms of Fn and Fn+1, but it didn't
 pay off in terms of speed. It's good to test the limits, but for practical
 purposes stick to the doubling algorithm.


**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and memoization.

**Crates:** `ibig`

**Type:** Program

**Categories:** big_numbers, educational, math, recreational, technique

**Link:** [fib_quadrupling_recursive_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_quadrupling_recursive_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fib_quadrupling_recursive_ibig.rs -- 100
```

---

### Script: filter_demos.rs

**Description:**  Select demo scripts and generate and serve HTML report.

 Strategy and grunt work thanks to ChatGPT.

**Purpose:** Allow user to select scripts by category.

**Crates:** `edit`, `inquire`, `thag_demo_proc_macros`, `thag_rs`, `tokio`, `warp`

**Type:** Program

**Categories:** technique, tools

**Link:** [filter_demos.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/filter_demos.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/filter_demos.rs
```

---

### Script: fizz_buzz_blandy_orendorff.rs

**Description:**  A fun example from Programming Rust by Jim Blandy and Jason Orendorff (O’Reilly).
 Copyright 2018 Jim Blandy and Jason Orendorff, 978-1-491-92728-1.
 Described by the authors as "a really gratuitous use of iterators".

**Purpose:** Demo using `thag_rs` to try out random code snippets ... also iterators.

**Type:** Snippet

**Categories:** educational, technique

**Link:** [fizz_buzz_blandy_orendorff.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fizz_buzz_blandy_orendorff.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fizz_buzz_blandy_orendorff.rs
```

---

### Script: fizz_buzz_gpt.rs

**Description:**  GPT-generated fizz-buzz example.

**Purpose:** Demo running random snippets in thag_rs, also AI and the art of delegation ;)

**Type:** Snippet

**Categories:** educational, technique

**Link:** [fizz_buzz_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/fizz_buzz_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/fizz_buzz_gpt.rs
```

---

### Script: flume_async.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.

**Purpose:** demo of async and channel programming and of `flume` in particular.

**Crates:** `async_std`, `flume`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [flume_async.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_async.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_async.rs
```

---

### Script: flume_perf.rs

**Description:**  Published example from the `flume` channel crate.

**Purpose:** demo of channel programming and of `flume` in particular.

**Crates:** `flume`

**Type:** Program

**Categories:** crates, technique

**Link:** [flume_perf.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_perf.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_perf.rs
```

---

### Script: flume_select.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.

**Purpose:** demo of async and channel programming and of `flume` in particular.

**Crates:** `flume`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [flume_select.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_select.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/flume_select.rs
```

---

### Script: gen_names.rs

**Description:**  A very simple published example from the random name generator
 `names`.

**Purpose:** Demo a simple snippet and featured crate.

**Crates:** `names`

**Type:** Snippet

**Categories:** technique

**Link:** [gen_names.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_names.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_names.rs
```

---

### Script: gen_readme.rs

**Description:**  This is the actual script used to collect demo script metadata and generate
 demo/README.md.

 Strategy and grunt work thanks to ChatGPT.

**Purpose:** Document demo scripts in a demo/README.md as a guide to the user.

**Crates:** `convert_case`, `thag_demo_proc_macros`, `thag_rs`

**Type:** Program

**Categories:** technique, tools

**Link:** [gen_readme.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_readme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/gen_readme.rs
```

---

### Script: git_dependency.rs

**Description:**  Demo the use of git dependencies in the toml block. Local path dependencies
 work the same way, e.g. `thag_rs = { path = "<path/to-project>/thag_rs" },
 but obviously the path literal will be specific to your environment.

**Purpose:** Demo `git` dependencies, explain `path` dependencies.

**Crates:** `thag_rs`

**Type:** Program

**Categories:** technique

**Link:** [git_dependency.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/git_dependency.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/git_dependency.rs
```

---

### Script: git_dependency_snippet.rs

**Description:**  `demo/git_dependency.rs` done as a snippet, just because.

**Purpose:** Demo `git` dependencies as a snippet.

**Crates:** `thag_rs`

**Type:** Snippet

**Categories:** technique

**Link:** [git_dependency_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/git_dependency_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/git_dependency_snippet.rs
```

---

### Script: gpt_clap_derive.rs

**Description:**  GPT-generated CLI using the `clap` crate.

**Purpose:** Demonstrate `clap` CLI using the derive option.

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [gpt_clap_derive.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gpt_clap_derive.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/gpt_clap_derive.rs -- -bgtv dummy_script.rs
```

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

**Categories:** crates, technique

**Link:** [gpt_lazy_static_theme.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/gpt_lazy_static_theme.rs)

**Not suitable to be run from a URL.**


---

### Script: hello.rs

**Description:**  Obligatory Hello World as a snippet

**Purpose:** Demo Hello World snippet

**Type:** Snippet

**Categories:** basic

**Link:** [hello.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hello.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/hello.rs
```

---

### Script: hello_main.rs

**Description:**  Hello World as a program (posh Winnie-the-Pooh version)

**Purpose:** Demo Hello World as a program

**Type:** Program

**Categories:** basic

**Link:** [hello_main.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hello_main.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/hello_main.rs
```

---

### Script: hello_minimal.rs

**Description:**  Minimalist Hello World snippet (poor Winnie-the-Pooh version)

**Purpose:** Demo Hello World reduced to an expression

**Type:** Snippet

**Categories:** basic

**Link:** [hello_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hello_minimal.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/hello_minimal.rs
```

---

### Script: history_debug.rs

**Description:**  Debug the history handling logic of the `stdin` module and display the effects.
 Using this abstraction because displays don't work nicely in a TUI editor.

**Purpose:** Debug and demo history ordering.

**Crates:** `regex`, `serde`, `serde_json`

**Type:** Snippet

**Categories:** testing

**Link:** [history_debug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/history_debug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/history_debug.rs
```

---

### Script: hyper_client.rs

**Description:**  Published echo-server HTTP client example from the `hyper` crate,
 with the referenced modules `support` and `tokiort` refactored
 into the script, while respecting their original structure and
 redundancies.
 You can run the `hyper_echo_server.rs` demo as the HTTP server on
 another command line and connect to it on port 3000:
 `thag demo/hyper_client.rs -- http://127.0.0.1:3000`.
 Or use any other available HTTP server.

**Purpose:** Demo `hyper` HTTP client, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `pin_project_lite`, `pretty_env_logger`, `tokio`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [hyper_client.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_client.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_client.rs -- http://127.0.0.1:3000
```

---

### Script: hyper_echo_server.rs

**Description:**  Published simple echo HTTP server example from the `hyper` crate,
 with the referenced modules `support` and `tokiort` refactored
 into the script, while respecting their original structure and
 redundancies.

 "This is our service handler. It receives a Request, routes on its
 path, and returns a Future of a Response."

**Purpose:** Demo `hyper` HTTP echo server, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `pin_project_lite`, `tokio`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [hyper_echo_server.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_echo_server.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_echo_server.rs
```

---

### Script: hyper_hello_server.rs

**Description:**  Published simple hello HTTP server example from the `hyper` crate,
 with the referenced modules `support` and `tokiort` refactored
 into the script, while respecting their original structure and
 redundancies.

**Purpose:** Demo `hyper` HTTP hello server, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `pin_project_lite`, `pretty_env_logger`, `tokio`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [hyper_hello_server.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_hello_server.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/hyper_hello_server.rs
```

---

### Script: ibig_big_integers.rs

**Description:**  Published example from the `ibig` crate, showcasing the use of the crate.

**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `ibig`

**Type:** Snippet

**Categories:** big_numbers, crates, technique

**Link:** [ibig_big_integers.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ibig_big_integers.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/ibig_big_integers.rs
```

---

### Script: iced_tour.rs

**Description:**  The full tour of the `iced` crate published in the `iced` examples.

**Purpose:** Show that `thag_rs` can handle product demos.

**Crates:** `console_error_panic_hook`, `console_log`, `iced`, `tracing_subscriber`

**Type:** Program

**Categories:** crates, technique

**Link:** [iced_tour.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/iced_tour.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/iced_tour.rs
```

---

### Script: in_place.rs

**Description:**  Published example from `in-place crate` disemvowels the file somefile.txt.

**Purpose:** Demo editing a file in place.

**Crates:** `in_place`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [in_place.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/in_place.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/in_place.rs
```

---

### Script: infer_deps.rs

**Description:**  Interactively test dependency inferency. This script was arbitrarily copied from
 `demo/repl_partial_match.rs`.
 Experiment with matching REPL commands with a partial match of any length.

**Purpose:** Usability: Accept a command as long as the user has typed in enough characters to identify it uniquely.

**Crates:** `clap`, `console`, `rustyline`, `shlex`, `strum`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [infer_deps.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/infer_deps.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/infer_deps.rs
```

---

### Script: inline_colorization.rs

**Description:**  Published simple example from `inline_colorization` crate. Simple effective inline
 styling option for text messages.

**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `inline_colorization`

**Type:** Snippet

**Categories:** async, crates, technique

**Link:** [inline_colorization.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/inline_colorization.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/inline_colorization.rs
```

---

### Script: input_expr_to_ast.rs

**Description:**  Tries to convert input to a `syn` abstract syntax tree (syn::Expr).

**Purpose:** Debugging

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, technique, tools

**Link:** [input_expr_to_ast.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/input_expr_to_ast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/input_expr_to_ast.rs
```

---

### Script: input_file_to_ast.rs

**Description:**  Tries to convert input to a `syn` abstract syntax tree (syn::File).

**Purpose:** Debugging

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, technique, tools

**Link:** [input_file_to_ast.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/input_file_to_ast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/input_file_to_ast.rs
```

---

### Script: interactive_clap_adv_struct.rs

**Description:**  Published example from the `interactive-clap` crate. I've adapted the run instractions below for use with `thag_rs`:

 This example shows additional functionality of the "interactive-clap" macro for parsing command-line data into a structure using macro attributes.

```
 thag demo/interactive_clap_adv_struct.rs (without parameters) => entered interactive mode
 thag demo/interactive_clap_adv_struct.rs -- --age-full-years 30 --first-name QWE --second-name QWERTY --favorite-color red
                                    => cli_args: CliArgs { age: Some(30), first_name: Some("QWE"), second_name: Some("QWERTY"), favorite_color: Some(Red) }
 thag demo/interactive_clap_adv_struct.rs -- --first-name QWE --second-name QWERTY --favorite-color red
                                    => cli_args: CliArgs { age: None, first_name: Some("QWE"), second_name: Some("QWERTY"), favorite_color: Some(Red) }
```

 To learn more about the parameters, use "help" flag:

```
  thag demo/interactive_clap_adv_struct.rs -- --help
```


**Purpose:** Demo featured crate.

**Crates:** `clap`, `color_eyre`, `inquire`, `interactive_clap`, `shell_words`, `strum`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [interactive_clap_adv_struct.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/interactive_clap_adv_struct.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/interactive_clap_adv_struct.rs
```

---

### Script: iter.rs

**Description:**  Demo a simple iterator

**Purpose:** Show how basic a snippet can be.

**Type:** Snippet

**Categories:** basic

**Link:** [iter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/iter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/iter.rs
```

---

### Script: json.rs

**Description:**  Demo of deserialising JSON with the featured crates.

**Purpose:** Demo featured crates.

**Crates:** `serde`, `serde_json`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [json.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/json.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/json.rs
```

---

### Script: json_parse.rs

**Description:**  Demo of deserialising JSON with the featured crates.
 This version prompts for JSON input.

**Purpose:** Demo featured crates.

**Crates:** `serde`, `serde_json`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [json_parse.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/json_parse.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/json_parse.rs
```

---

### Script: just_a_test_expression.rs

**Description:**  This is an arbitrary expression for use by scripts like `demo/syn_visit_extern_crate_expr.rs`
 and `demo/syn_visit_use_path_expr.rs`.
 Don't remove the surrounding braces, because those serve to make it an expression.

**Purpose:** Testing.

**Crates:** `syn`

**Type:** Snippet

**Categories:** testing

**Link:** [just_a_test_expression.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/just_a_test_expression.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/just_a_test_expression.rs
```

---

### Script: list_files.rs

**Description:**  Demo listing files on disk. If you want a sorted list, you will need to amend the
 program to collect the entries into a Vec and sort that.

**Purpose:** Simple demonstration.

**Type:** Program

**Categories:** basic, technique

**Link:** [list_files.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/list_files.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/list_files.rs
```

---

### Script: loop_closure.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
 the latter since I'm not sure what the closure logic buys us. It's
 going to be checked by the compiler anyway. Compare with `demo/loop_expr.rs`.
 P.S.: This was since implemented as `--loop`.

**Purpose:** Evaluate closure logic for line processing.

**Type:** Snippet

**Categories:** exploration, technique

**Link:** [loop_closure.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_closure.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_closure.rs
```

---

### Script: loop_expr.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
 the latter since I'm not sure what the closure logic buys us. It's
 going to be checked by the compiler anyway. Compare with `demo/loop_closure.rs`.
 P.S.: This was since implemented as `--loop`.

**Purpose:** Evaluate expression logic for line processing.

**Type:** Snippet

**Categories:** exploration, technique

**Link:** [loop_expr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_expr.rs
```

---

### Script: loop_pre_post.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`, but with pre-
 and post-loop logic analogous to `awk`. I got GPT to do me this
 mock-up.
 P.S.: This was since implemented as `--loop`.

**Purpose:** Evaluate expression logic for line processing.

**Type:** Program

**Categories:** exploration, technique

**Link:** [loop_pre_post.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_pre_post.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/loop_pre_post.rs -- 'dummy prelude' 'dummy main' 'dummy post' # ... and hit Enter then Ctrl-d
```

---

### Script: macro_fn_lazy_static.rs

**Description:**  Demo of a generic macro to generate lazy static variables without the `lazy_static` crate.

**Purpose:** Demonstrate a technique

**Type:** Program

**Categories:** educational, technique

**Link:** [macro_fn_lazy_static.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_fn_lazy_static.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_fn_lazy_static.rs
```

---

### Script: macro_gen_enum.rs

**Description:**  First prototype of building an enum from a macro and using it thereafter, thanks to SO user DK.
 `https://stackoverflow.com/questions/37006835/building-an-enum-inside-a-macro`

**Purpose:** explore a technique for resolving mappings from a message level enum to corresponding

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_gen_enum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_gen_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_gen_enum.rs
```

---

### Script: macro_gen_styles_enum.rs

**Description:**  Second prototype of building an enum from a macro and using it thereafter.

**Purpose:** explore a technique for resolving mappings from a message level enum to corresponding

**Type:** Snippet

**Categories:** macros, technique

**Link:** [macro_gen_styles_enum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_gen_styles_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_gen_styles_enum.rs
```

---

### Script: macro_lazy_static_var_advanced.rs

**Description:**  Demo of an advanced generic macro to generate lazy static variables.
 See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.
 match my_lazy_var {
     Ok(value) => println!("Initialized value: {}", value),
     Err(e) => eprintln!("Failed to initialize: {}", e),
 }
 ```

**Purpose:** Demonstrate a handy alternative to the `lazy_static` crate.

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_lazy_static_var_advanced.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_advanced.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_advanced.rs
```

---

### Script: macro_lazy_static_var_advanced_alt.rs

**Description:**  Demo of an advanced generic macro to generate lazy static variables.
 See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.
 A generic macro for lazily initializing a static variable using `OnceLock`.

 # Parameters
 - `$static_var`: The static variable name.
 - `$init_fn`: The initialization function, which is only called once.
 - $name: todo()

 # Example
 ```rust
 let my_lazy_var = lazy_static_var!(HashMap<usize, &'static str>, { /* initialization */ });
 ```

**Purpose:** Demonstrate a handy alternative to the `lazy_static` crate.

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_lazy_static_var_advanced_alt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_advanced_alt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_advanced_alt.rs
```

---

### Script: macro_lazy_static_var_error_handling.rs

**Description:**  Demo of an advanced generic macro to generate lazy static variables.
 See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.

**Purpose:** Demonstrate a handy alternative to the `lazy_static` crate.

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_lazy_static_var_error_handling.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_error_handling.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_error_handling.rs
```

---

### Script: macro_lazy_static_var_errs.rs

**Description:**  Demo of a generic macro to generate lazy static variables.
 Sometimes you need to call a function repeatedly and it makes sense for it to lazily initialise a
 variable that it will use each time. I got you fam!

 See also `demo/macro_lazy_static_var_advanced.rs` for a more advanced form of the macro.

**Purpose:** Demonstrate a handy alternative to the `lazy_static` crate.

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_lazy_static_var_errs.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_errs.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_lazy_static_var_errs.rs
```

---

### Script: macro_print.rs

**Description:**  Proof of concept of distinguishing types that implement Display from those that implement
 Debug, and printing using the Display or Debug trait accordingly. Worked out with recourse
 to ChatGPT for suggestions and macro authoring.

**Purpose:** May be interesting or useful.

**Type:** Program

**Categories:** macros, technique, type_identification

**Link:** [macro_print.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_print.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/macro_print.rs
```

---

### Script: merge_toml.rs

**Description:**  Prototype of comprehensive merge of script toml metadata with defaults.

**Purpose:** Develop for inclusion in main project.

**Crates:** `cargo_toml`, `serde_merge`

**Type:** Program

**Categories:** crates, prototype, technique

**Link:** [merge_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/merge_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/merge_toml.rs
```

---

### Script: mock_edit.rs

**Description:**  Used to debug a doctest.

**Purpose:** Debugging script.

**Crates:** `crossterm`, `mockall`, `thag_rs`

**Type:** Snippet

**Categories:** crates, technique, testing

**Link:** [mock_edit.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/mock_edit.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/mock_edit.rs
```

---

### Script: multiline_err.rs

**Description:**  LLM-provided formatting for error messages

**Purpose:** Demo of formatting error messages

**Type:** Program

**Categories:** error_handling, technique

**Link:** [multiline_err.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/multiline_err.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/multiline_err.rs
```

---

### Script: owo_cli_color_support.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. thag demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [owo_cli_color_support.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_cli_color_support.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_cli_color_support.rs -- add patrick
```

---

### Script: owo_msg_colors_1_basic_gpt.rs

**Description:**  An early exploration of message colouring, GPT-generated.
 This one uses basic Ansi 16 colours. Try it on dark vs light
 backgrounds to see how some of the colours change.

**Purpose:** May be of use to some. Demo featured crates.

**Crates:** `crossterm`, `owo_colors`, `termbg`

**Type:** Program

**Categories:** crates, exploration

**Link:** [owo_msg_colors_1_basic_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_msg_colors_1_basic_gpt.rs)

**Not suitable to be run from a URL.**


---

### Script: owo_msg_colors_2_adv_gpt.rs

**Description:**  More fully worked-out prototype of colouring and styling messages based on the level of
 colour support of the current terminal and whether a light or dark theme is currently
 selected. This was the result of good deal of exploration and dialog with ChatGPT.  Try it on dark vs light
 backgrounds to see how some of the same colours "pop" when shown against a light or dark theme
 and how some virtually or literally disappear when not well matched to the theme.
 Fully worked-out demonstration of colouring and styling display messages according
 to message level.

**Purpose:** Demo detection of terminal colour support and dark or light theme, colouring and styling of messages, use of `strum` crate to get enum variant from string, and AI-generated code.

**Crates:** `enum_assoc`, `log`, `owo_colors`, `strum`, `supports_color`, `termbg`

**Type:** Program

**Categories:** crates, prototype, technique

**Link:** [owo_msg_colors_2_adv_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_msg_colors_2_adv_gpt.rs)

**Not suitable to be run from a URL.**


---

### Script: owo_styles.rs

**Description:**  An early exploration of the idea of adaptive message colouring according to the terminal theme.

**Purpose:** Demo a simple example of adaptive message colouring, and the featured crates.

**Crates:** `crossterm`, `owo_colors`, `strum`, `termbg`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [owo_styles.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/owo_styles.rs)

**Not suitable to be run from a URL.**


---

### Script: parse_script_rs_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code using
 basic line-by-line comparison as opposed to a regular expression. I eventually
 decided to use a regular expression as I found it less problematic (see
 `demo/regex_capture_toml.rs`).

**Purpose:** Prototype

**Type:** Program

**Categories:** prototype, technique

**Link:** [parse_script_rs_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/parse_script_rs_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/parse_script_rs_toml.rs
```

---

### Script: parse_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code by locating
 the start and end of the toml block. I eventually decided to use a regular
 expression as I found it less problematic (see `demo/regex_capture_toml.rs`).

**Purpose:** Prototype

**Type:** Program

**Categories:** prototype

**Link:** [parse_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/parse_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/parse_toml.rs
```

---

### Script: pomprt_completion.rs

**Description:**  Published example from `pomprt` crate.

 Not suitable for running from a URL. Run locally and enter simple shell commands like `ls -l` at the prompt.
 `Ctrl-d` to terminate.

**Purpose:** Demo of `pomprt` readline implementation.

**Crates:** `pomprt`

**Type:** Program

**Categories:** crates

**Link:** [pomprt_completion.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/pomprt_completion.rs)

**Not suitable to be run from a URL.**


---

### Script: prettyplease.rs

**Description:**  Published example from `prettyplease` Readme.

**Purpose:** Demo featured crate.

**Crates:** `prettyplease`, `syn`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [prettyplease.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/prettyplease.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/prettyplease.rs
```

---

### Script: proc_macro_attribute_basic.rs

**Description:**  Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
 `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.

**Purpose:** Sample model of a basic attribute proc macro.

**Crates:** `thag_demo_proc_macros`

**Type:** Snippet

**Categories:** proc_macros, technique

**Link:** [proc_macro_attribute_basic.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_attribute_basic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_attribute_basic.rs
```

---

### Script: proc_macro_category_enum.rs

**Description:**  Try generating category enum.
 Testing the `category_enum` proc macro for use with `demo/gen_readme.rs` and `demo/filter_demos.rs`/

**Purpose:** Test the proof of concept and potentially the implementation.

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** missing

**Link:** [proc_macro_category_enum.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_category_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_category_enum.rs
```

---

### Script: proc_macro_const_demo.rs

**Description:**  Recycled test suite from `https://github.com/redmcg/const_gen_proc_macro`

**Purpose:** Demo the use of proc macros to generate constants at compile time

**Crates:** `thag_demo_proc_macros`

**Type:** Snippet

**Categories:** proc_macros, technique

**Link:** [proc_macro_const_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_const_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_const_demo.rs
```

---

### Script: proc_macro_const_demo_debug.rs

**Description:**  Exploring integrated macro expansion, based on `demo/proc_macro_const_demo.rs`.

**Purpose:** Second working prototype of expanding proc macros for debugging purposes. See also `demo/proc_macro_const_demo_expand.rs`.

**Crates:** `thag_demo_proc_macros`

**Type:** Snippet

**Categories:** proc_macros, technique

**Link:** [proc_macro_const_demo_debug.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_const_demo_debug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_const_demo_debug.rs
```

---

### Script: proc_macro_const_demo_expand.rs

**Description:**  Exploring integrated macro expansion, based on `demo/proc_macro_const_demo.rs`.
 Recycled test suite from `https://github.com/redmcg/const_gen_proc_macro`.

**Purpose:** First working prototype of expanding proc macros for debugging purposes. See also `demo/proc_macro_const_demo_debug.rs`.

**Crates:** `thag_demo_proc_macros`

**Type:** Snippet

**Categories:** proc_macros, technique

**Link:** [proc_macro_const_demo_expand.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_const_demo_expand.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_const_demo_expand.rs
```

---

### Script: proc_macro_derive_basic.rs

**Description:**  Exploring expansion

**Purpose:** explore proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_derive_basic.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_basic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_basic.rs
```

---

### Script: proc_macro_derive_custom_model.rs

**Description:**  Published example from `https://github.com/anshulsanghi-blog/macros-handbook`

**Purpose:** explore derive proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_derive_custom_model.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_custom_model.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_custom_model.rs
```

---

### Script: proc_macro_derive_doc_comment.rs

**Description:**  Exploring exposing doc comments at runtime.
 Example from https://www.reddit.com/r/rust/comments/pv5v3x/looking_for_a_minimal_example_on_how_to_parse_doc/

**Purpose:** explore proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_derive_doc_comment.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_doc_comment.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_doc_comment.rs
```

---

### Script: proc_macro_derive_key_map_list.rs

**Description:**  Use a derive proc macro to implement a table. from a base with additions and deletions.
 Not very useful currently: the dream is to generate a constant and get mappings as a variable.

**Purpose:** explore derive proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_derive_key_map_list.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_key_map_list.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_derive_key_map_list.rs
```

---

### Script: proc_macro_expander_demo.rs

**Description:**  Published example from crate `expander`

**Purpose:** debug proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_expander_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_expander_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_expander_demo.rs
```

---

### Script: proc_macro_functionlike_basic.rs

**Description:**  Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
 `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.

**Purpose:** Sample model of a basic function-like proc macro.

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_functionlike_basic.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_functionlike_basic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_functionlike_basic.rs
```

---

### Script: proc_macro_host_port_const.rs

**Description:**  Demo example generated by ChatGPT, followed by intensive debugging of the `syn` logic in the proc macro.

**Purpose:** Demo the use of proc macros to generate constants at compile time

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_host_port_const.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_host_port_const.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_host_port_const.rs
```

---

### Script: proc_macro_organizing_code.rs

**Description:**  Published example from `https://github.com/tdimitrov/rust-proc-macro-post`

**Purpose:** explore proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_organizing_code.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_organizing_code.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_organizing_code.rs
```

---

### Script: proc_macro_organizing_code_const.rs

**Description:**  Experimental - work in progress

**Purpose:** investigate the possibility of generating a useful constant.

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_organizing_code_const.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_organizing_code_const.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_organizing_code_const.rs
```

---

### Script: proc_macro_organizing_code_tokenstream.rs

**Description:**  Published example from `https://github.com/tdimitrov/rust-proc-macro-post`

**Purpose:** explore proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_organizing_code_tokenstream.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_organizing_code_tokenstream.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_organizing_code_tokenstream.rs
```

---

### Script: proc_macro_repeat_dash.rs

**Description:**  Exploring expansion: function-like proc macro.

**Purpose:** explore proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_repeat_dash.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_repeat_dash.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_repeat_dash.rs
```

---

### Script: proc_macro_string_concat.rs

**Description:**  Published example from `https://github.com/redmcg/const_gen_proc_macro`

**Purpose:** Use proc macros to generate constants at compile time

**Crates:** `thag_demo_proc_macros`

**Type:** Snippet

**Categories:** proc_macros, technique

**Link:** [proc_macro_string_concat.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_string_concat.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/proc_macro_string_concat.rs
```

---

### Script: profiling_puffin_demo.rs

**Description:**  Published demo from the `profiling` crate using the `puffin` profiler.
 We derive Deserialize/Serialize so we can persist app state on shutdown.

**Purpose:** Demo featured crates.

**Crates:** `eframe`, `egui`, `env_logger`, `log`, `profiling`, `puffin`, `puffin_egui`

**Type:** Program

**Categories:** crates

**Link:** [profiling_puffin_demo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/profiling_puffin_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/profiling_puffin_demo.rs
```

---

### Script: puffin_profiler_egui.rs

**Description:**  Published demo from the `puffin` crate.

**Purpose:** Demo featured crate.

**Crates:** `eframe`, `puffin`, `puffin_egui`

**Type:** Program

**Categories:** crates

**Link:** [puffin_profiler_egui.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/puffin_profiler_egui.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/puffin_profiler_egui.rs
```

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

**Crates:** `io`

**Type:** Snippet

**Categories:** educational, math, recreational

**Link:** [py_thag.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/py_thag.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/py_thag.rs
```

---

### Script: ratatui_user_input.rs

**Description:**  Published example from the `ratatui` crate.

**Purpose:** Demo the featured crate.

**Crates:** `ratatui`

**Type:** Program

**Categories:** crates

**Link:** [ratatui_user_input.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ratatui_user_input.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/ratatui_user_input.rs
```

---

### Script: readline_crossterm.rs

**Description:**  Published crossterm example.
 Demonstrates how to block read characters or a full line.
 Just note that crossterm is not required to do this and can be done with `io::stdin()`.

**Purpose:** Demo crossterm reading key events as a line or a single char.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates

**Link:** [readline_crossterm.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/readline_crossterm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/readline_crossterm.rs
```

---

### Script: reedline_basic_keybindings.rs

**Description:**  Published example `basic.rs` from `reedline` crate.

**Purpose:** demo featured crates.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_basic_keybindings.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_basic_keybindings.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_basic_keybindings.rs
```

---

### Script: reedline_completions.rs

**Description:**  Published example from `reedline` crate.

**Purpose:** demo featured crates.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_completions.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_completions.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_completions.rs
```

---

### Script: reedline_event_listener.rs

**Description:**  Published example from `reedline` crate.

**Purpose:** demo featured crates.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_event_listener.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_event_listener.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_event_listener.rs
```

---

### Script: reedline_highlighter.rs

**Description:**  Published example from `reedline` crate.

**Purpose:** Explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_highlighter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_highlighter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_highlighter.rs
```

---

### Script: reedline_hinter.rs

**Description:**  Published example from `reedline` crate.

**Purpose:** Explore featured crate.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_hinter.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_hinter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_hinter.rs
```

---

### Script: reedline_history.rs

**Description:**  Published example from `reedline` crate.

**Purpose:** Demo `reedline` file-backed history.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_history.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_history.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_history.rs
```

---

### Script: reedline_ide_completions.rs

**Description:**  Published example from `reedline` crate. See the Vec of commands in the main method standing in for
 history. Enter a letter, e.g. "h" and press Tab to see the magic happen: all the commands starting
 with that letter will be displayed for selection with a tab, up and down arrows or Enter. Or you can
 enter subsequent letters to narrow the search. Noice.

**Purpose:** Demo `reedline` tab completions.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_ide_completions.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_ide_completions.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_ide_completions.rs
```

---

### Script: reedline_list_bindings.rs

**Description:**  Published example from `reedline` crate.
 List all keybinding information

**Purpose:** Explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_list_bindings.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_list_bindings.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_list_bindings.rs
```

---

### Script: reedline_multiline.rs

**Description:**  Exploratory prototype of REPL support for multi-line expressions. Based on published example
 `custom_prompt.rs` in `reedline` crate.

**Purpose:** Explore options for handling multi-line expressions in a REPL.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_multiline.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_multiline.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_multiline.rs
```

---

### Script: reedline_read_stdin.rs

**Description:**  Basic exploration of reading a line from stdin with `reedline`.

**Purpose:** Exploring how to render prompts and read lines of input.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_read_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_read_stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_read_stdin.rs
```

---

### Script: reedline_repl.rs

**Description:**  Published example from `reedline-repl-rs` crate.

**Purpose:** Explore the suitability of this crate for a Rust REPL. Conclusion: it's more geared to commands.

**Crates:** `reedline_repl_rs`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_repl.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_repl.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_repl.rs
```

---

### Script: reedline_repl_context.rs

**Description:**  Published example from `reedline-repl-rs` crate. This one uses the
 `clap` builder pattern; there is also one using the`clap` derive pattern.

**Purpose:** Evaluation of featured crate and of using clap to structure command input.

**Crates:** `reedline_repl_rs`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_repl_context.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_repl_context.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_repl_context.rs
```

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

**Categories:** crates, REPL, technique

**Link:** [reedline_show_bindings.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_show_bindings.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_show_bindings.rs
```

---

### Script: reedline_stdin.rs

**Description:**  Exploring `reedline` crate.

**Purpose:** explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_stdin.rs
```

---

### Script: reedline_transient_prompt.rs

**Description:**  Published demo from `reedline` crate.

**Purpose:** Demo the use of a transient minimal prompt `! ` for returned history.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [reedline_transient_prompt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_transient_prompt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/reedline_transient_prompt.rs
```

---

### Script: regex_capture_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code using
 a regular expression. I ended up choosing this approach as being less
 problematic than line-by-line parsing (see `demo/parse_script_rs_toml.rs`)
 See also `demo/regex_capture_toml.rs`.

**Purpose:** Prototype, technique

**Crates:** `regex`

**Type:** Program

**Categories:** prototype

**Link:** [regex_capture_toml.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/regex_capture_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/regex_capture_toml.rs
```

---

### Script: repl_block.rs

**Description:**  Early proof of concept of using a different line editor for repl.rs.

**Purpose:** Exploration

**Crates:** `clap`, `lazy_static`, `regex`, `repl_block`, `strum`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [repl_block.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/repl_block.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/repl_block.rs
```

---

### Script: repl_partial_match.rs

**Description:**  Experiment with matching REPL commands with a partial match of any length.

**Purpose:** Usability: Accept a command as long as the user has typed in enough characters to identify it uniquely.

**Crates:** `clap`, `console`, `rustyline`, `shlex`, `strum`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [repl_partial_match.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/repl_partial_match.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/repl_partial_match.rs
```

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

**Categories:** crates, technique

**Link:** [rug_arbitrary_precision_nums.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rug_arbitrary_precision_nums.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/rug_arbitrary_precision_nums.rs
```

---

### Script: rustfmt.rs

**Description:**  Prototype of invoking the Rust formatter programmatically, with the addition of an `rfd`
 (`Rusty File Dialogs`) cross-platform file chooser to select the file to format. The code
 for both was AI-generated because I find AI very handy for this kind of grunt work.

**Purpose:** Demo file chooser and calling an external program, in this case the Rust formatter.

**Crates:** `rfd`

**Type:** Program

**Categories:** crates, technique

**Link:** [rustfmt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustfmt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/rustfmt.rs
```

---

### Script: rustfmt_stdin.rs

**Description:**  Read Rust source code from stdin and display the output as formatted by `rustfmt`.

**Purpose:** Format arbitrary Rust code. Does no more than `rustfmt --`.

**Type:** Program

**Categories:** crates, technique

**Link:** [rustfmt_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustfmt_stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/rustfmt_stdin.rs
```

---

### Script: rustlings_smart_pointers_rc1.rs

**Description:**  Published exercise solution from the `rustlings` crate.

**Purpose:** Demo one way to preserve your `rustlings` solutions, for reference or as katas.

**Type:** Program

**Categories:** educational

**Link:** [rustlings_smart_pointers_rc1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustlings_smart_pointers_rc1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/rustlings_smart_pointers_rc1.rs
```

---

### Script: rustyline_compl.rs

**Description:**  Published example from the `rustyline` crate.

**Purpose:** Demo using `thag_rs` to run a basic REPL as a script.

**Crates:** `env_logger`, `rustyline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [rustyline_compl.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustyline_compl.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/rustyline_compl.rs
```

---

### Script: rustyline_full.rs

**Description:**  Example from `rustyline` crate readme.
 MatchingBracketValidator uses matching brackets to decide between single- and multi-line
 input.

**Purpose:** Explore `rustyline` crate.

**Crates:** `env_logger`, `rustyline`

**Type:** Program

**Categories:** crates, REPL, technique

**Link:** [rustyline_full.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/rustyline_full.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/rustyline_full.rs
```

---

### Script: semver_exclude_prerelease.rs

**Description:**  Prototype of excluding pre-release crates from cargo queries.

**Purpose:** Prototype technique for `thag_rs`.

**Crates:** `semver`

**Type:** Program

**Categories:** prototype, technique

**Link:** [semver_exclude_prerelease.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/semver_exclude_prerelease.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/semver_exclude_prerelease.rs
```

---

### Script: side_by_side_diff.rs

**Description:**  Published example from `side-by-side-diff` crate.

**Purpose:** Explore integrated side by side diffs.

**Crates:** `side_by_side_diff`

**Type:** Program

**Categories:** crates, exploration

**Link:** [side_by_side_diff.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/side_by_side_diff.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/side_by_side_diff.rs
```

---

### Script: slog_expressions.rs

**Description:**  Published example from `slog` crate (misc/examples/expressions.rs).

**Purpose:** Demo a popular logging crate.

**Crates:** `slog`, `slog_term`

**Type:** Program

**Categories:** crates

**Link:** [slog_expressions.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/slog_expressions.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/slog_expressions.rs
```

---

### Script: snippet_import_scope.rs

**Description:**  Demo scope of import statements.

**Purpose:** Prototype to confirm leaving imports in situ when wrapping snippets.

**Crates:** `ibig`

**Type:** Snippet

**Categories:** crates, educational

**Link:** [snippet_import_scope.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/snippet_import_scope.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/snippet_import_scope.rs
```

---

### Script: snippet_name_clash.rs

**Description:**  Demo scope of import statements. Two conflicting imports with the same name
 coexisting in the same `println!` invocation. Demonstrates that when wrapping
 a snippet we can't assume it's OK to pull the imports up to the top level.

**Purpose:** Prototype to confirm leaving imports in situ when wrapping snippets.

**Type:** Snippet

**Categories:** crates, educational

**Link:** [snippet_name_clash.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/snippet_name_clash.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/snippet_name_clash.rs
```

---

### Script: stdin.rs

**Description:**  A version of `thag_rs`'s `stdin` module to handle standard input editor input. Like the `colors`
 module, `stdin` was originally developed here as a separate script and integrated as a module later.

 E.g. `thag demo/stdin.rs`

**Purpose:** Demo using `thag_rs` to develop a module outside of the project.

**Crates:** `anyhow`, `lazy_static`, `ratatui`, `regex`, `tui_textarea`

**Type:** Program

**Categories:** crates, prototype, technique

**Link:** [stdin.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/stdin.rs
```

---

### Script: stdin_main.rs

**Description:**  A version of `thag_rs`'s `stdin` module from the `main` `git` branch for the purpose of comparison
 with the `develop` branch version being debugged.

 E.g. `thag demo/stdin_main.rs`
 Apply highlights to the text depending on the light or dark theme as detected, configured
 or defaulted, or as toggled by the user with Ctrl-t.

**Purpose:** Debugging.

**Crates:** `crossterm`, `lazy_static`, `mockall`, `ratatui`, `regex`, `scopeguard`, `serde`, `serde_json`, `thag_rs`, `tui_textarea`

**Type:** Program

**Categories:** testing

**Link:** [stdin_main.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/stdin_main.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/stdin_main.rs
```

---

### Script: structopt_cli_gpt.rs

**Description:**  Basic demo of GPT-generated CLI using the `structopt` crate. This
 crate is in maintenance mode, its features having been integrated
 into `clap`.

**Purpose:** Demonstrate `structopt` CLI.

**Crates:** `structopt`

**Type:** Snippet

**Categories:** CLI, crates, technique

**Link:** [structopt_cli_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/structopt_cli_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/structopt_cli_gpt.rs -- -- -Vt dummy.rs 1 2 3
```

---

### Script: supports_color.rs

**Description:**  Demo of crate `supports-color` that `thag_rs` uses to detect the level of
 colour support of the terminal in use.
 Caution: from testing I suspect that `supports-color` may mess with the terminal
 settings. Obviously that doesn't matter in a demo that exists before doing
 serious work, but it can wreak havoc with your program's output.

**Purpose:** Demo featured crate doing its job.

**Crates:** `supports_color`

**Type:** Snippet

**Categories:** crates

**Link:** [supports_color.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/supports_color.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/supports_color.rs
```

---

### Script: supports_color_win.rs

**Description:**  Windows-friendly logic extracted from crate `supports-color`.


**Purpose:** Proof of concept for Windows environment

**Type:** Snippet

**Categories:** crates, prototype

**Link:** [supports_color_win.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/supports_color_win.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/supports_color_win.rs
```

---

### Script: syn_dump_syntax.rs

**Description:**  Published example from the `syn` crate. Description "Parse a Rust source file
 into a `syn::File` and print out a debug representation of the syntax tree."

 Pass it the absolute or relative path of any Rust PROGRAM source file, e.g. its own
 path that you passed to the script runner to invoke it.

 NB: Pick a script that is a valid program (containing `fn main()` as opposed to a snippet).

**Purpose:** show off the power of `syn`.

**Crates:** `colored`, `syn`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [syn_dump_syntax.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_dump_syntax.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_dump_syntax.rs -- demo/hello_main.rs
```

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
 expression (in this case the number 13). Or you can do the same with the input (5 + 8)
 and it will do the same because `thag_rs` will detect and evaluate an expression in
 essentially the same way as this script does.

**Purpose:** demo expression evaluation (excluding compilation and execution) using the `syn` and `quote` crates.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_quote.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_quote.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_quote.rs
```

---

### Script: syn_remove_attributes.rs

**Description:**  Prototype of removing an inner attribute (`#![...]`) from a syntax tree. Requires the `visit-mut'
 feature of `syn`.

**Purpose:** Demonstrate making changes to a `syn` AST.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_remove_attributes.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_remove_attributes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_remove_attributes.rs
```

---

### Script: syn_visit_extern_crate_expr.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_item_extern_crate`
 to process the nodes representing `extern crate` statements and fn `visit_expr` to initiate the tree
 traversal. This version expects the script contents to consist of a Rust expression.

**Purpose:** Prototype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_extern_crate_expr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_extern_crate_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_extern_crate_expr.rs -- demo/just_a_test_expression.rs
```

---

### Script: syn_visit_extern_crate_file.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_item_extern_crate`
 to process the nodes representing `extern crate` statements and fn `visit_file` to initiate the tree
 traversal. This version expects the script contents to consist of a full-fledged Rust program.

**Purpose:** Prototype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_extern_crate_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_extern_crate_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_extern_crate_file.rs -- demo/syn_visit_extern_crate_file.rs
```

---

### Script: syn_visit_node_type.rs

**Description:**  Demo of selectively modifying source code using `syn` and `quote`. This is from a solution posted by user Yandros on the Rust Playground
 in answer to a question asked on the Rust users forum. The discussion and Playground link are to be found here:
 https://users.rust-lang.org/t/writing-proc-macros-with-syn-is-there-a-way-to-visit-parts-of-the-ast-that-match-a-given-format/54733/4
 (This content is dual-licensed under the MIT and Apache 2.0 licenses according to the Rust forum terms of service.)
 I've embellished it to show how it can be formatted with `prettyplease` if parsed as a `syn::File`.

**Purpose:** Demo programmatically modifying Rust source code using `syn` and `quote`.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [syn_visit_node_type.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_node_type.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_node_type.rs
```

---

### Script: syn_visit_use_path_expr.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source expression passed to the script. Specifically the combination of fn `visit_use_path`
 to process the nodes representing `use` statements and fn `visit_expr` to initiate the tree
 traversal. This version expects the script contents to consist of a Rust expression.

**Purpose:** Prototype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_use_path_expr.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_path_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_path_expr.rs -- demo/just_a_test_expression.rs
```

---

### Script: syn_visit_use_path_file.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_use_path`
 to process the nodes representing `use` statements and fn `visit_file` to initiate the tree
 traversal. This version expects the script contents to consist of a full-fledged Rust program.

**Purpose:** Protorype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_use_path_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_path_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_path_file.rs -- demo/syn_visit_use_path_file.rs
```

---

### Script: syn_visit_use_rename.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to identify `use` statements that exist
 for the purpose of renaming a dependency so that we don't go looking for the temporary in the registry.
 Specifically the combination of fn `visit_use_rename` to process the nodes representing `extern crate`
 statements and fn `visit_file` to initiate the tree traversal. This version expects the script contents
 to consist of a full-fledged Rust program.

**Purpose:** Prototype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_use_rename.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_rename.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_rename.rs -- demo/crossbeam_epoch_sanitize.rs
```

---

### Script: syn_visit_use_tree_file.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_use_tree`
 to process the nodes representing `use` statements and fn `visit_file` to initiate the tree
 traversal. This version expects the script contents to consist of a full-fledged Rust program.

**Purpose:** Develop improved algorithm for `thag_rs` that accepts imports of the form `use <crate>;` instead of requiring `use <crate>::...`.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_use_tree_file.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_tree_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/syn_visit_use_tree_file.rs -- demo/syn_visit_use_tree_file.rs
```

---

### Script: tempfile.rs

**Description:**  Published example from the `tempfile` readme.

**Purpose:** Demo featured crate.

**Crates:** `tempfile`

**Type:** Program

**Categories:** crates

**Link:** [tempfile.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tempfile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/tempfile.rs
```

---

### Script: term_detection_pack.rs

**Description:**  A basic tool I cobbled together that uses different crates to a) test terminal
 types on different platforms, b) determine and cross-check if a light or dark
 theme is in use and c) determine the level of colour supported reported by
 the terminal.

**Purpose:** Allow checking of terminals on platforms to be supported, also test reliability of different crates.

**Crates:** `crossterm`, `log`, `simplelog`, `supports_color`, `termbg`, `terminal_light`

**Type:** Snippet

**Categories:** crates, tools

**Link:** [term_detection_pack.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/term_detection_pack.rs)

**Not suitable to be run from a URL.**


---

### Script: termbg.rs

**Description:**  Published example from `termbg` readme.

 Detects the light or dark theme in use, as well as the colours in use.

**Purpose:** Demo theme detection with `termbg`

**Crates:** `simplelog`, `termbg`

**Type:** Program

**Categories:** crates

**Link:** [termbg.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/termbg.rs)

**Not suitable to be run from a URL.**


---

### Script: terminal_light.rs

**Description:**  Demo of `terminal_light`, a crate that "answers the question "Is the terminal dark
 or light?".

**Purpose:** Demo terminal-light interrogating the background color. Results will vary with OS and terminal type.

**Crates:** `terminal_light`

**Type:** Snippet

**Categories:** crates

**Link:** [terminal_light.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light.rs
```

---

### Script: terminal_light_fading.rs

**Description:**  A fun published example from the `terminal-light` crate. "Demonstrate mixing
 any ANSI color with the background."

**Purpose:** Mostly recreational.

**Crates:** `coolor`, `crossterm`, `terminal_light`

**Type:** Program

**Categories:** crates, recreational

**Link:** [terminal_light_fading.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light_fading.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light_fading.rs
```

---

### Script: terminal_light_skins.rs

**Description:**  A published example from the `terminal-light` crate. A simple example of
 choosing an appropriate skin based on the terminal theme.

**Purpose:** Demo of the `terminal-light` crate.

**Crates:** `coolor`, `crossterm`, `terminal_light`

**Type:** Program

**Categories:** crates

**Link:** [terminal_light_skins.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light_skins.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/terminal_light_skins.rs
```

---

### Script: test_clap_4707.rs

**Description:**  Minimal reproducible code posted by user `mkeeter` to demonstrate `clap` issue 4707
 which we are experiencing at time of creation of this script.
 https://github.com/clap-rs/clap/issues/4707

 To reproduce the error, run `cargo run demo/test_clap_4707.rs -- --write --show-hex`
 Correct behaviour would be:
 error: the following required arguments were not provided:
  --read
 Incorrect behaviour is that the command runs without an error.

**Purpose:** test if the error exists, then periodically to see if it persists.

**Crates:** `clap`

**Type:** Program

**Categories:** crates, testing

**Link:** [test_clap_4707.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/test_clap_4707.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/test_clap_4707.rs -- --write --show-hex
```

---

### Script: thag_cargo.rs

**Description:**  `thag` prompted front-end command to run Cargo commands on scripts. It is recommended to compile this to an executable with -x.
 Prompts the user to select a Rust script and a cargo command to run against the script's generated project, and
 and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option.

**Crates:** `atty`, `inquire`, `rustix`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_cargo.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_cargo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_cargo.rs
```

---

### Script: thag_clippy.rs

**Description:**  `thag` prompted front-end command to run `clippy` on scripts. It is recommended to compile this to an executable with -x.
 Prompts the user to select a Rust script and one or more Clippy lints to run against the script's generated project, and
 and invokes `thag` with the --cargo option to run it.

**Purpose:** A user-friendly interface to the `thag` `--cargo` option specifically for running `cargo clippy` on a script.

**Crates:** `atty`, `colored`, `inquire`, `rustix`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_clippy.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_clippy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_clippy.rs
```

---

### Script: thag_config_builder.rs

**Description:**  Prompted config file builder for `thag`, intended to be saved as a command with `-x`.

**Purpose:** Handy configuration file builder.

**Crates:** `colored`, `convert_case`, `dirs`, `documented`, `inquire`, `strum`, `syn`, `thag_rs`, `toml`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [thag_config_builder.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_config_builder.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_config_builder.rs
```

---

### Script: thag_crokey_print_key.rs

**Description:**  Published example of KeyCombination from `crokey` crate, modified to use
 basic `crokey` key combos embedded in `thag_rs` under MIT licence.

**Purpose:** Test for stability and consistency across different platforms and terminals.

**Crates:** `crossterm`, `thag_rs`

**Type:** Program

**Categories:** crates, testing

**Link:** [thag_crokey_print_key.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_crokey_print_key.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_crokey_print_key.rs
```

---

### Script: thag_from_rust_script.rs

**Description:**  Converts embedded manifest format from `rust-script` to `thag`.

 E.g. `cat <path_to_rust_script_file> | thag -qq demo/thag_from_rust_script.rs | thag -s [-- [options] [args] ...]`

 Place any command-line options and/or arguments for the script at the end after a -- as shown.


**Purpose:** Convenience for any `rust-script` user who wants to try out `thag`.

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_from_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_from_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_from_rust_script.rs
```

---

### Script: thag_profile.rs

**Description:** 
**Purpose:** 

**Crates:** `inferno`, `inquire`, `thag_core`

**Type:** Program

**Categories:** missing

**Link:** [thag_profile.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_profile.rs
```

---

### Script: thag_profile_save.rs

**Description:** 
**Purpose:** 

**Crates:** `inferno`, `inquire`

**Type:** Program

**Categories:** missing

**Link:** [thag_profile_save.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_profile_save.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_profile_save.rs
```

---

### Script: thag_prompt.rs

**Description:**  Early prototype of prompting front-end for `thag`.

**Purpose:** Ultimately, to provide a prompt-driven front-end to the `thag` command.

**Crates:** `inquire`

**Type:** Program

**Categories:** prototype, thag_front_ends, tools

**Link:** [thag_prompt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_prompt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_prompt.rs
```

---

### Script: thag_to_rust_script.rs

**Description:**  Converts embedded manifest format from `thag` to `rust-script`.

**Purpose:** Convenience for any `thag` user who wants to try out `rust-script`.

**Type:** Program

**Categories:** crates, tools

**Link:** [thag_to_rust_script.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_to_rust_script.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_to_rust_script.rs
```

---

### Script: thag_url.rs

**Description:**  `thag` front-end command to run scripts from URLs. It is recommended to compile this with -x.

**Purpose:** A front-end to allow thag to run scripts from URLs while offloading network dependencies from `thag` itself.

**Crates:** `syn`, `tempfile`, `tinyget`, `url`

**Type:** Program

**Categories:** technique, thag_front_ends, tools

**Link:** [thag_url.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_url.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/thag_url.rs
```

---

### Script: time_cookbook.rs

**Description:**  Simple time demo pasted directly from Rust cookbook. Run without -q to show how
 `thag_rs` will find the missing `chrono` manifest entry and display a specimen
 toml block you can paste in at the top of the script.

**Purpose:** Demo cut and paste from a web source with Cargo search and specimen toml block generation.

**Crates:** `chrono`

**Type:** Program

**Categories:** basic

**Link:** [time_cookbook.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/time_cookbook.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/time_cookbook.rs
```

---

### Script: tlborm_callbacks.rs

**Description:**  `Callbacks` example from `The Little Book of Rust Macros`

**Purpose:** Demo macro callbacks.

**Type:** Program

**Categories:** educational, technique

**Link:** [tlborm_callbacks.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tlborm_callbacks.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/tlborm_callbacks.rs
```

---

### Script: tokio_hello_short.rs

**Description:**  Published example from `tokio` crate, with comments removed to work with `thag_rs` `repl` feature.
 Before running, start a server: `ncat -l 6142` in another terminal.

**Purpose:** Demo running `tokio` from `thag_rs`.

**Crates:** `tokio`

**Type:** Program

**Categories:** async, educational, technique

**Link:** [tokio_hello_short.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tokio_hello_short.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/tokio_hello_short.rs
```

---

### Script: tokio_hello_world.rs

**Description:**  Published example from `tokio` crate. Before running, start a server: `ncat -l 6142`
 in another terminal.

**Purpose:** Demo running `tokio` from `thag_rs`.

**Crates:** `tokio`

**Type:** Program

**Categories:** async, educational, technique

**Link:** [tokio_hello_world.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tokio_hello_world.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/tokio_hello_world.rs
```

---

### Script: tui_scrollview.rs

**Description:**  Published example from `tui-scrollview` crate. Toml entries from crate's Cargo.toml.

 Not suitable for running from a URL.

**Purpose:** Explore TUI editing

**Crates:** `color_eyre`, `lipsum`, `ratatui`, `tui_scrollview`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [tui_scrollview.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_scrollview.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_editor.rs

**Description:**  Demo a TUI (text user interface) editor based on the featured crates. This editor is locked
 down to two files at a time, because it was developed to allow editing of generated code and
 cargo.toml from the REPL, but was eventually dropped in favour of leaving the user to choose
 or default to a standard editor. A more minimalist version is used to edit stdin input in
 the `--edit (-d)` option of `thag_rs`.

 Not suitable for running from a URL.

**Purpose:** Demo and explore TUI editor and featured crates, including `crossterm`.

**Crates:** `ratatui`, `tui_textarea`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [tui_ta_editor.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_ta_editor.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_editor_profile.rs

**Description:**  The same script as `demo/tui_ta_editor.rs`, but with `firestorm` profiling.

 Not suitable for running from a URL.
 To see the profiling flamegraph after exiting the program, look in dir `flames` under the `env::temp_dir()`
 for your operating system. Note that due to an apparent bug in `firestorm`, the `Editor::run` method currently
 executes twice, so it will need to be closed a second time.

**Purpose:** Demo featured crates, but `firestorm` profiler in particular.

**Crates:** `firestorm`, `ratatui`, `tui_textarea`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [tui_ta_editor_profile.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_ta_editor_profile.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_minimal.rs

**Description:**  Demo a very minimal and not very useful TUI (text user interface) editor based on the featured crates.

 Not suitable for running from a URL.

**Purpose:** Demo TUI editor and featured crates, including `crossterm`, and the use of the `scopeguard` crate to reset the terminal when it goes out of scope.

**Crates:** `ratatui`, `scopeguard`, `tui_textarea`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [tui_ta_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_ta_minimal.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_vim.rs

**Description:**  Published basic `vim` editor example from crate `tui-textarea`. Mildly tweaked
 to use `ratatui::crossterm` re-exports instead of `crossterm` directly.

 Not suitable for running from a URL.

**Purpose:** Demo TUI `vim` editor and featured crates, including `crossterm`.

**Crates:** `ratatui`, `tui_textarea`

**Type:** Program

**Categories:** crates

**Link:** [tui_ta_vim.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_ta_vim.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_tokio_editor_gpt.rs

**Description:**  GPT-provided demo of a very basic TUI (terminal user interface) editor using
 `tokio` and the `crossterm` / `ratatui` / `tui-textarea` stack. provides a blank editor
 screen on which you can capture lines of data. `Ctrl-D` closes the editor and simply
 prints the captured data.

 Not suitable for running from a URL.

**Purpose:** Exploring options for editing input. e.g. for a REPL.

**Crates:** `ratatui`, `tokio`, `tui_textarea`

**Type:** Program

**Categories:** async, crates, educational, exploration, technique

**Link:** [tui_tokio_editor_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/tui_tokio_editor_gpt.rs)

**Not suitable to be run from a URL.**


---

### Script: type_of_at_compile_time_1.rs

**Description:**  Use a trait to determine the type of an expression at compile time, provided all cases are known in advance.

 This is a slightly embellished version of user `phicr`'s answer on `https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable-in-rust`.

 See also `demo/type_of_at_compile_time_2.rs` for an alternative implementation.

**Purpose:** Demo expression type determination for static dispatch.

**Type:** Program

**Categories:** type_identification, technique

**Link:** [type_of_at_compile_time_1.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_compile_time_1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_compile_time_1.rs
```

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

**Categories:** type_identification, technique

**Link:** [type_of_at_compile_time_2.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_compile_time_2.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_compile_time_2.rs
```

---

### Script: type_of_at_run_time.rs

**Description:**  Typical basic (runtime) solution to expression type identification. See also `demo/determine_if_known_type_trait.rs`
 for what may be a better (compile-time) solution depending on your use case.

**Purpose:** Demo of runtime type identification.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** type_identification, technique

**Link:** [type_of_at_run_time.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_run_time.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/type_of_at_run_time.rs
```

---

### Script: ubig_product_gpt.rs

**Description:**  Implement trait std::iter::Product for `ibig::UBig`. Example provided by GPT.

**Purpose:** Educational / reference.

**Crates:** `ibig`

**Type:** Program

**Categories:** big_numbers, educational, reference, technique

**Link:** [ubig_product_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/ubig_product_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/ubig_product_gpt.rs
```

---

### Script: unzip.rs

**Description:**  Very simple demo of the `unzip` iterator function.

**Purpose:** Demo

**Type:** Snippet

**Categories:** technique

**Link:** [unzip.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/unzip.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/unzip.rs
```

---

### Script: win_test_control.rs

**Description:**  This is the "control" test for the `demo/win_test_*.rs` scripts. It seems to reliably NOT swallow the first character.

**Purpose:** Show how crates *not* sending an OSC to the terminal in Windows will *not* the first character you enter to be swallowed.

**Type:** Program

**Categories:** testing

**Link:** [win_test_control.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_control.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_control.rs
```

---

### Script: win_test_supports_color.rs

**Description:**  This seems to intermittently swallow the very first character entered in Windows, prior to `termbg` 0.6.0.

**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `supports_color`

**Type:** Program

**Categories:** testing

**Link:** [win_test_supports_color.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_supports_color.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_supports_color.rs
```

---

### Script: win_test_termbg.rs

**Description:**  This seems to "reliably" swallow the very first character entered in Windows, prior to `termbg` 0.6.0.

**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `termbg`

**Type:** Program

**Categories:** testing

**Link:** [win_test_termbg.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_termbg.rs)

**Not suitable to be run from a URL.**


---

### Script: win_test_terminal_light.rs

**Description:**  This seems to "reliably" swallow the very first character entered in Window, prior to `termbg` 0.6.0..

**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `terminal_light`

**Type:** Program

**Categories:** testing

**Link:** [win_test_terminal_light.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_terminal_light.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_terminal_light.rs
```

---

### Script: win_test_vt.rs

**Description:**  Exploration of `Windows Terminal` virtual terminal processing with respect to the `termbg` crate.
 `termbg` comment states: "Windows Terminal is Xterm-compatible"
 https://github.com/microsoft/terminal/issues/3718.
 Unfortunately it turns out that this is only partially true and misleading, because
 this compatibility excludes OSC 10/11 colour queries until Windows Terminal 1.22,
 which was only released in preview in August 2024.
 https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release/:
 "Applications can now query ... the default foreground (OSC 10 ?) [and] background (OSC 11 ?)"
 Another finding is that WT_SESSION is not recommended as a marker for VT capabilities:
 https://github.com/Textualize/rich/issues/140.
 Also, but out of scope of this script, there is no good fallback detection method provided by Windows,
 as per my comments in the adapted module `thag_rs::termbg`. Unless you have WT 1.22 or higher as above,
 the best bet for supporting colour schemes other than the default black is to fall back to using a
 configuration file (as we do) or allowing the user to specify the theme in real time.
 Finally, the `termbg` crate was swallowing the first character of input in Windows and causing a
 "rightward march" of log output due to suppression of carriage returns in all environments. I've
 addressed the former by using non-blocking `crossterm` event polling instead of `stdin`, and also
 introduced a

**Purpose:** Debug `termbg`

**Crates:** `crossterm`, `winapi`

**Type:** Program

**Categories:** testing

**Link:** [win_test_vt.rs](https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_vt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/master/demo/win_test_vt.rs
```

---

