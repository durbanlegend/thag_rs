## Running the scripts in `demo` and `src/bin`

### Commonality of scripts and tools

The scripts in src/bin are integrated `thag_rs` tools, in other words they are declared in Cargo.toml and are normally installed as commands. However, if you have cloned the `thag_rs` project you can run them like any other `thag` script.

Conversely, you can make your own tools by using the `thag -x` option to do a release build of any script, even a snippet. You could even use `thag -xe ` to build a command, as described in the main [README.md](../README.md).

---

`thag_rs` uses `clap` for a standard command-line interface. Try `thag --help` (or -h) if
you get stuck.

### In its simplest form:

  ```
  thag <path to script>
  ```

#### For example:

  ```
  thag demo/hello.rs
  ```

#### Full syntax:

  ```
  thag [THAG OPTIONS] <path to script> [-- [SCRIPT OPTIONS]   <script args>]
  ```

### Passing options and arguments to a script:

Use `--` to separate options and arguments meant for the script from those meant for `thag` itself.

#### Example 1:

`demo/fib_dashu_snippet.rs` expects to be passed an integer _n_ and will compute the _nth_ number in the Fibonacci sequence.

  thag demo/fib_dashu_snippet.rs -- 100

---

#### Example 2:

`demo/clap_tut_builder_01.rs` is a published example from the `clap` crate.
Its command-line signature looks like this:

##### Signature:

  ```
  clap_tut_builder_01 [OPTIONS] [name] [COMMAND]
  ```

This `clap` example takes the following arguments (in their short form):

##### Arguments:

  ```
  # Arguments to demo/clap_tut_builder_01.rs
  [OPTIONS]
    `-c <config_file>`      an optional configuration file
    `-d` / `-dd` / `-ddd`   debug, at increasing levels of verbosity
  [name]                    an optional filename
  [COMMAND]                 a command (e.g. test) to run
  ```

If we were to compile `clap_tut_builder_01` as an executable (`-x` option) and then run it, we might pass
it some parameters like this:

##### Running compiled script:

  ```
  clap_tut_builder_01 -dd -c my.cfg my_file test -l
  ```

and get output like this:

##### Output:

  ```
  Value for name: my_file
  Value for config: my.cfg
  Debug mode is on
  Printing testing lists...
  ```

##### Running the script source from `thag`:

Running the source from `thag` looks similar. We just replace `clap_tut_builder_01` by `thag demo/clap_tut_builder_01.rs --`:

  ```
  thag demo/clap_tut_builder_01.rs -- -dd -c my.cfg my_file test -l
  ```

##### Separating `thag` parameters from script parameters:

Any parameters for `thag` itself should go before the `--`.

For example, if we choose to use -qq to suppress `thag` messages:

  ```
  thag demo/clap_tut_builder_01.rs -qq -- -dd -c my.cfg my_file test -l
  ```

which will give identical output to the compiled script above.

---

***Remember to use `--` to separate options and arguments that are intended for `thag` from those intended for the target script.***

***
## Detailed script listing

### Script: alloc_proto_atomic.rs

**Description:**  Prototype of ring-fenced memory allocators for `thag_profiler`.

 The `global_allocator` attribute flags a `Dispatcher` which dispatches each
 memory allocation, deallocation and reallocation requests to one of two allocators
 according to the designated current allocator (held in an atomic boolean) at the
 moment that it receives the request. The default allocator is `TaskAware` and is
 used for user code, while the regular system allocator `System` handles requests
 from profiler code. The role of the `TaskAware` allocator is to record the details
 of the user code allocation events before passing them to the system allocator.

 To invoke the system allocator directly, profiler code must call a function or
 closure with fn `with_sys_alloc`, which checks the current allocator, and if it
 finds it to be `TaskAware`, changes it to `System` and runs the function or closure,
 with a guard to restore the default to `TaskAware`. If the current allocator is
 already `System`, `with_sys_alloc` concludes that it must be running nested under
 another `with_sys_alloc` call, so does nothing except run the function or closure.

 The flaw in this design is its vulnerability to race conditions, e.g. user code
 in another thread could fail to go through `TaskAware` if `with_sys_alloc` is
 running concurrently, or conversely an outer `with_sys_alloc` ending in one thread
 could prematurely reset the current allocator to  `TaskAware` while another
 instance is still running in another thread. We can and do build in a check in
 the TaskAware branch to detect and ignore profiler code, but in practice there is
 little sign of such races being a problem.

 Attempts to resolve this issue with thread-local storage have not borne fruit.
 For instance async tasks are by no means guaranteed to resume in the same thread
 after suspension.
 The ideal would seem to be a reentrant Mutex or RwLock with mutability - so far tried
 without success, but a subject for another prototype.
 Dispatcher that routes allocation requests to the active allocator
 according to the USING_SYSTEM_ALLOCATOR variable for the current thread.
 Task-aware allocator that tracks memory allocations

**Purpose:** Prototype of a ring-fenced allocator for memory profiling.

**Type:** Program

**Categories:** profiling, prototype

**Link:** [alloc_proto_atomic.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/alloc_proto_atomic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/alloc_proto_atomic.rs
```

---

### Script: alloc_proto_rwlock.rs

**Description:**  Prototype of ring-fenced memory allocators for `thag_profiler`.

 The `global_allocator` attribute flags a `Dispatcher` which dispatches each
 memory allocation, deallocation and reallocation requests to one of two allocators
 according to the designated current allocator at the moment that it receives
 the request. The default allocator is `TaskAware` and is used for user code,
 while the regular system allocator `System` handles requests from profiler code.
 The role of the `TaskAware` allocator is to record the details of the user code
 allocation events before passing them to the system allocator.

 To invoke the system allocator directly, profiler code must call a function or
 closure with fn `with_sys_alloc`, which checks the current allocator, and if it
 finds it to be `TaskAware`, changes it to `System` and runs the function or closure,
 with a guard to restore the default to `TaskAware`. If the current allocator is
 already `System`, `with_sys_alloc` concludes that it must be running nested under
 another `with_sys_alloc` call, so does nothing except run the function or closure.

 The flaw in this design is its vulnerability to race conditions, e.g. user code
 in another thread could fail to go through `TaskAware` if `with_sys_alloc` is
 running concurrently, or conversely an outer `with_sys_alloc` ending in one thread
 could prematurely reset the current allocator to  `TaskAware` while another
 instance is still running in another thread. We can and do build in a check in
 the TaskAware branch to detect and ignore profiler code, but in practice there is
 little sign of such races being a problem.

 Attempts to resolve this issue with thread-local storage have not borne fruit.
 For instance async tasks are by no means guaranteed to resume in the same thread
 after suspension.
 The ideal would seem to be a reentrant Mutex with mutability - so far tried
 without success, but a subject for another prototype.
 Dispatcher that routes allocation requests to the desired allocator
 Task-aware allocator that tracks memory allocations

**Purpose:** Prototype of a ring-fenced allocator for memory profiling.

**Crates:** `parking_lot`

**Type:** Program

**Categories:** profiling, prototype

**Link:** [alloc_proto_rwlock.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/alloc_proto_rwlock.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/alloc_proto_rwlock.rs
```

---

### Script: analyze_snippet_1.rs

**Description:**  Guided ChatGPT-generated prototype of using a `syn` abstract syntax tree (AST)
 to detect whether a snippet returns a value that we should print out, or whether
 it does its own printing.

 Part 1: After some back and forth with ChatGPT suggesting solutions it finally generates essentially this.

**Purpose:** Demo use of `syn` AST to analyse code and use of AI LLM dialogue to flesh out ideas and provide code.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, technique

**Link:** [analyze_snippet_1.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/analyze_snippet_1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/analyze_snippet_1.rs
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

**Link:** [analyze_snippet_2.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/analyze_snippet_2.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/analyze_snippet_2.rs
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

**Link:** [analyze_snippet_3.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/analyze_snippet_3.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/analyze_snippet_3.rs
```

---

### Script: append_option_to_iter.rs

**Description:**  Demo: Optionally append one item to an iterator.
 The trick is that `Option` implements the `IntoIterator` trait.

**Purpose:** demo a handy trick.

**Type:** Program

**Categories:** learning, technique

**Link:** [append_option_to_iter.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/append_option_to_iter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/append_option_to_iter.rs
```

---

### Script: attribute_reset_replacement_test.rs

**Description:**  Test enhanced reset replacement with proper text attribute handling

 This demonstrates the fix for text attributes (bold/dim, italic, underline) that
 were previously leaking from inner styled content when using reset replacement.

 The enhanced system:
 1. Analyzes the outer style's ANSI codes
 2. Only resets attributes that won't be reapplied
 3. Optimizes the reset sequence to avoid redundant operations
 4. Maintains perfect context preservation across all nesting levels

**Purpose:** Test enhanced reset replacement with text attribute handling

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ansi, styling, terminal, testing

**Link:** [attribute_reset_replacement_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/attribute_reset_replacement_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/attribute_reset_replacement_test.rs
```

---

### Script: benchmark.rs

**Description:**  ChagtGPT-generated profiling synchronous time profiling benchmark: base code.
 See `demo/benchmark*.rs` for `firestorm` and `thag_profiler` implementations.


**Purpose:** For checking and comparison of profiling tools

**Crates:** `rand`, `rayon`, `regex`, `serde_json`, `sha2`

**Type:** Program

**Categories:** benchmark, profiling

**Link:** [benchmark.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/benchmark.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/benchmark.rs
```

---

### Script: benchmark_firestorm.rs

**Description:**  ChagtGPT-generated profiling synchronous time profiling benchmark: `firestorm` implementation`.
 See `demo/benchmark*.rs` for base code and `thag_profiler` implementation.


**Purpose:** For checking and comparison of profiling tools

**Crates:** `firestorm`, `rand`, `rayon`, `regex`, `serde_json`, `sha2`

**Type:** Program

**Categories:** benchmark, profiling

**Link:** [benchmark_firestorm.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/benchmark_firestorm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/benchmark_firestorm.rs
```

---

### Script: benchmark_profile.rs

**Description:**  ChagtGPT-generated profiling synchronous time profiling benchmark: `thag_profiler` implementation`.
 See `demo/benchmark*.rs` for base code and `firestorm` implementation.


**Purpose:** For checking and comparison of profiling tools

**Crates:** `rand`, `rayon`, `regex`, `serde_json`, `sha2`, `thag_profiler`

**Type:** Program

**Categories:** benchmark, profiling

**Link:** [benchmark_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/benchmark_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/benchmark_profile.rs
```

---

### Script: bg_detect.rs

**Description:**  Background Color Detection

 This script investigates querying palette color 0 as a proxy for background color
 detection. This approach might work where OSC 11 queries
 fail, particularly in PowerShell and regular Windows terminals.

 Based on the observation that demo/truecolor*.rs files work in PowerShell,
 suggesting we can interrogate palette colors from Rust instead of shell scripts.
 RGB color representation
 Query background color using standard OSC 11
 Query palette color 0 using OSC 4 (potential background proxy)
 Generic OSC color query function
 Read terminal response with timeout
 Parse OSC color response (handles both OSC 11 and OSC 4 responses)
 Parse hex component from OSC response
 Compare different detection methods
 Calculate color distance (simple Euclidean)

**Purpose:** Test background color detection via palette color 0 query

**Crates:** `crossterm`, `thag_common`

**Type:** Program

**Categories:** ansi, color, terminal, windows, xterm

**Link:** [bg_detect.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/bg_detect.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/bg_detect.rs
```

---

### Script: bitflags.rs

**Description:**  Try out the `bitflags` crate.

**Purpose:** Explore use of `bitflags` to control processing.

**Crates:** `bitflags`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [bitflags.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/bitflags.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/bitflags.rs
```

---

### Script: borrow_wrapped.rs

**Description:**  Snippet demonstrating how to reference or clone a wrapped value without
 falling foul of the borrow checker.

**Purpose:** Demo a borrow-checker-friendly technique for accessing a wrapped value.

**Type:** Snippet

**Categories:** technique

**Link:** [borrow_wrapped.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/borrow_wrapped.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/borrow_wrapped.rs
```

---

### Script: bpaf_cargo_show_asm.rs

**Description:**  Published example from the `bpaf` crate.

 E.g. `thag demo/bpaf_cargo_show_asm.rs -- -h`

**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [bpaf_cargo_show_asm.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/bpaf_cargo_show_asm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/bpaf_cargo_show_asm.rs -- -h
```

---

### Script: bpaf_cmd_chain.rs

**Description:**  Example from bpaf crate docs2/src/adjacent_command/derive.rs.

 E.g. `thag demo/bpaf_cmd-chain.rs -- eat Fastfood drink --coffee sleep --time=5`

**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf_derive`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [bpaf_cmd_chain.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/bpaf_cmd_chain.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/bpaf_cmd_chain.rs -- eat Fastfood drink --coffee sleep --time=5
```

---

### Script: bpaf_derive.rs

**Description:**  Example from the `bpaf` crate docs2/src/command/derive.rs.

 E.g. `thag demo/bpaf_cmd_ex.rs -- --flag cmd --flag --arg=6`

**Purpose:** Demo CLI alternative to clap crate

**Crates:** `bpaf_derive`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [bpaf_derive.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/bpaf_derive.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/bpaf_derive.rs -- --flag cmd --flag --arg=6
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

**Link:** [cargo_capture_output.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_capture_output.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_capture_output.rs
```

---

### Script: cargo_debug_test_case.rs

**Description:**  Run a command (in this case an integration test case to be debugged),
 and capture and print its stdout and stderr concurrently in a
 separate thread.

**Purpose:** Demo process::Command with output capture, debugging unit tests.

**Crates:** `env_logger`, `log`

**Type:** Program

**Categories:** debugging, technique, testing

**Link:** [cargo_debug_test_case.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_debug_test_case.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_debug_test_case.rs
```

---

### Script: cargo_lookup_highest.rs

**Description:**  Updated prototype of getting the highest valid release of a crate via `cargo-lookup`.
 The crate in its raw state only gets the latest. `thag_rs` was picking up `inquire`
 8.1 because it was released after 9.1 to fix the same issue on the previous version.

**Purpose:** Originally used to debug and then prototype crate lookup, now brought up to date.

**Crates:** `cargo_lookup`

**Type:** Program

**Categories:** debugging, prototype

**Link:** [cargo_lookup_highest.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_lookup_highest.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_lookup_highest.rs
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

**Link:** [cargo_output.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_output.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cargo_output.rs
```

---

### Script: clap_demo.rs

**Description:**  Published example from the `clap` crate.

 The latest version of this example is available in the [examples] folder in the `clap` repository.
 At time of writing you can run it successfully just by invoking its URL with the `thag_url` tool
 and passing the required arguments as normal, like this:

 ```bash
 thag_url https://github.com/clap-rs/clap/blob/master/examples/demo.rs -- --name "is this the Krusty Krab?"
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.

 Original `clap` crate comments:

 Simple program to greet a person
 Simple program to greet a person

**Purpose:** Demo building a repl using `clap` directly.

**Crates:** `clap`

**Type:** Program

**Categories:** REPL, technique

**Link:** [clap_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_demo.rs -- --name "is this the Krusty Krab?"
```

---

### Script: clap_enum_strum.rs

**Description:**  Exploring using clap with an enum, in conjunction with strum.
 E.g. `thag demo/clap_enum_strum.rs -- variant-num2`

**Purpose:** Simple demo of featured crates, contrasting how they expose variants.

**Crates:** `clap`, `serde`, `strum`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_enum_strum.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_enum_strum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_enum_strum.rs -- variant-num2
```

---

### Script: clap_num_arg.rs

**Description:**  `clap` with a numeric option.

 E.g. `thag demo/clap_num_arg.rs -- 45`

**Purpose:** Basic demo of `clap` parsing a numeric argument

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_num_arg.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_num_arg.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_num_arg.rs -- 45
```

---

### Script: clap_repl_crate_rustyline.rs

**Description:**  Older version of published clap_repl crate example, modified to prototype a
 (dummy) Rust REPL.

**Purpose:** Yet another REPL demo, this time using `rustyline`.

**Crates:** `clap`, `clap_repl`, `console`, `quote`, `rustyline`, `syn`

**Type:** Program

**Categories:** REPL, technique

**Link:** [clap_repl_crate_rustyline.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_repl_crate_rustyline.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_repl_crate_rustyline.rs
```

---

### Script: clap_repl_diy.rs

**Description:**  Example from the clap cookbook, not using the `clap-repl` crate.

 The latest version of this example is `repl-derive.rs` in the [examples] folder
  in the `clap` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/clap-rs/clap/blob/master/examples/repl-derive.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.

 Can't find a keybinding to navigate history, unlike `clap_repl_crate_rustyline.rs`.

**Purpose:** Demo building a repl using `clap` directly.

**Crates:** `clap`, `shlex`

**Type:** Program

**Categories:** REPL, technique

**Link:** [clap_repl_diy.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_repl_diy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_repl_diy.rs
```

---

### Script: clap_tut_builder_01_quick.rs

**Description:**  Published example from `clap` tutorial (builder)

 E.g.  `thag demo/clap_tut_builder_01_quick.rs -- -ddd -c dummy.cfg my_file test -l`

**Purpose:** Demonstrate `clap` CLI using the builder option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_builder_01_quick.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_builder_01_quick.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_builder_01_quick.rs -- -ddd -c dummy.cfg my_file test -l
```

---

### Script: clap_tut_derive_03_04_subcommands.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. thag demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_derive_03_04_subcommands.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_derive_03_04_subcommands.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob
```

---

### Script: clap_tut_derive_04_01_enum.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. `thag demo/clap_tut_derive_04_01_enum.rs -- fast`

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_derive_04_01_enum.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_derive_04_01_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_derive_04_01_enum.rs -- fast
```

---

### Script: clap_tut_derive_04_03_relations.rs

**Description:**  Published example from `clap` tutorial (derive), with added displays.

 E.g. `thag demo/clap_tut_derive_04_03_relations.rs -- --major -c config.toml --spec-in input.txt`

**Purpose:** Demonstrate `clap` CLI using the derive option

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [clap_tut_derive_04_03_relations.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_derive_04_03_relations.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/clap_tut_derive_04_03_relations.rs -- --major -c config.toml --spec-in input.txt
```

---

### Script: cmd_args.rs

**Description:**  A prototype of the `cmd_args` module of thag_rs itself.

 E.g. `thag -tv demo/cmd_args.rs -- -tv demo/hello.rs -- -fq Hello world`

**Purpose:** Prototype CLI.

**Crates:** `bitflags`, `clap`

**Type:** Program

**Categories:** CLI, crates, prototype, technique

**Link:** [cmd_args.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cmd_args.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cmd_args.rs -- -tv demo/hello.rs -- -fq Hello world
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

**Link:** [cmd_args_bpaf_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cmd_args_bpaf_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cmd_args_bpaf_gpt.rs -- -gbrtv demo/hello.rs -- -fq Hello world
```

---

### Script: cmd_args_clap.rs

**Description:**  Basic CLI example using clap.

 E.g. `thag -t demo/cmd_args_clap.rs -- -atv hello.sh`

**Purpose:** For comparison with `demo/cmd_args_bpaf_gpt.rs`.

**Crates:** `bitflags`, `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [cmd_args_clap.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/cmd_args_clap.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/cmd_args_clap.rs -- -atv hello.sh
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

**Link:** [color_contrast.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/color_contrast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/color_contrast.rs
```

---

### Script: colors_old.rs

**Description:**  A version of `thag_rs`'s  now defunct `colors` module to style messages according to their type. Like the `stdin`
 module, `colors` was originally developed here as a separate script and integrated as a module later.

 The `colors` module was superseded by `styling`. See `demo/styling_demo.rs`

 E.g. `thag demo/colors_old.rs`

**Purpose:** Demo using `thag_rs` to develop a module outside of the project.

**Crates:** `lazy_static`, `log`, `nu_ansi_term`, `strum`, `supports_color`, `termbg`, `thag_rs`

**Type:** Program

**Categories:** prototype, reference, testing

**Link:** [colors_old.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/colors_old.rs)

**Not suitable to be run from a URL.**


---

### Script: colors_orig.rs

**Description:**  Original prototype of `thag_rs`'s `colors` module to style messages according
 to their type. I only dropped `owo-colors` because I switched from `rustyline` to
 `reedline`, which was already using `nu_ansi_term`.

 The `colors` module was superseded by `styling`. See `demo/styling_demo.rs`

 See also `demo/colors_old.rs`.


**Purpose:** Demo older alternative implementation of `colors` module using `owo-colors`.

**Crates:** `log`, `owo_colors`, `strum`, `supports_color`, `termbg`, `thag_rs`

**Type:** Program

**Categories:** prototype, reference, testing

**Link:** [colors_orig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/colors_orig.rs)

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

**Purpose:** Develop a configuration file implementation for `thag_rs`.

**Crates:** `edit`, `firestorm`, `home`, `log`, `mockall`, `serde`, `serde_with`, `thag_rs`, `toml`

**Type:** Program

**Categories:** prototype, technique

**Link:** [config.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/config.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/config.rs
```

---

### Script: config_with_tests.rs

**Description:**  Demo of unit testing a non-snippet source file such as a library module using `thag --test-only` `(thag -T)`.

 In this case this demo file is the one we're testing.

 The unit tests must be in mod `tests` in the file.

 `thag` will leave the file as is, but generate a temporary Cargo.toml for it in the usual way as a prerequisite for running `cargo test`.

 `thag` will then invoke `cargo test` on the file, specifying the Cargo.toml location via `--manifest-path`.

 `thag <filepath> -T [-- <cargo test options>]`

 E.g.:

 `TEST_CONFIG_PATH=~/.config/thag_rs/config.toml thag demo/config_with_tests.rs -Tv -- --nocapture --show-output`

**Purpose:** Demonstrate unit testing a file in situ without wrapping it if it doesn't have a main method.

**Crates:** `documented`, `edit`, `log`, `mockall`, `serde`, `serde_with`, `simplelog`, `strum`, `tempfile`, `thag_rs`, `toml`, `toml_edit`

**Type:** Program

**Categories:** technique, testing

**Link:** [config_with_tests.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/config_with_tests.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/config_with_tests.rs TEST_CONFIG_PATH=~/.config/thag_rs/config.toml thag demo/config_with_tests.rs -Tv -- --nocapture --show-output
```

---

### Script: context_demo.rs

**Description:**  TermAttributes context pattern demo.

**Purpose:** Demonstrate TermAttributes context pattern for testing and temporary overrides

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, terminal, testing

**Link:** [context_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/context_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/context_demo.rs
```

---

### Script: correct_reset_replacement.rs

**Description:**  Correct implementation of reset replacement approach for multi-level nesting

 This implements the approach where each level replaces all reset sequences (\x1b[0m)
 in its content with its own ANSI color codes, ensuring that outer context is
 always restored after inner styled content.

**Purpose:** Demonstrate correct reset replacement for perfect context preservation

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ansi, prototype, styling

**Link:** [correct_reset_replacement.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/correct_reset_replacement.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/correct_reset_replacement.rs
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

**Link:** [count_main_methods.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/count_main_methods.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/count_main_methods.rs
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

**Link:** [create_next_file.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/create_next_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/create_next_file.rs
```

---

### Script: crokey_deser.rs

**Description:**  Published example of serde deserialisation from the `crokey` crate.

 The latest version of this example is available in the [examples] folder
  in the `crokey` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/Canop/crokey/blob/main/examples/deser_keybindings/src/main.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Demo loading keybindings from a file.

**Crates:** `crokey`, `serde`, `toml`

**Type:** Program

**Categories:** crates, technique

**Link:** [crokey_deser.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crokey_deser.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crokey_deser.rs
```

---

### Script: crokey_print_key.rs

**Description:**  Published example of combiner from the `crokey` crate.

 The latest version of this example is available in the [examples] folder
  in the `crokey` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/Canop/crokey/blob/main/examples/print_key/src/main.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.

**Purpose:** Demo key combiner.

**Crates:** `crokey`, `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crokey_print_key.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crokey_print_key.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crokey_print_key.rs
```

---

### Script: crokey_print_key_no_combiner.rs

**Description:**  Published example of KeyCombination from the `crokey` crate.

 The latest version of this example is available in the [examples] folder
  in the `crokey` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/Canop/crokey/blob/main/examples/print_key_no_combiner/src/main.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Demo key combination without Combiner.

**Crates:** `KeyCombinationFormat`, `crokey`, `key`

**Type:** Program

**Categories:** crates, technique

**Link:** [crokey_print_key_no_combiner.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crokey_print_key_no_combiner.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crokey_print_key_no_combiner.rs
```

---

### Script: crossbeam_channel_fibonacci.rs

**Description:**  Published example from the `crossbeam-channel` crate.

 The latest version of this example is available in the [examples] folder
  in the `crossbeam-channel` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/crossbeam-rs/crossbeam/blob/master/crossbeam-channel/examples/fibonacci.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Demo featured crate.

**Crates:** `crossbeam_channel`

**Type:** Program

**Categories:** crates

**Link:** [crossbeam_channel_fibonacci.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_channel_fibonacci.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_channel_fibonacci.rs
```

---

### Script: crossbeam_channel_matching.rs

**Description:**  `crossbeam-channel` published example
 Using `select!` to send and receive on the same channel at the same time.

**Purpose:** Demo featured crates.

**Crates:** `crossbeam_channel`, `crossbeam_utils`

**Type:** Program

**Categories:** crates

**Link:** [crossbeam_channel_matching.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_channel_matching.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_channel_matching.rs
```

---

### Script: crossbeam_channel_stopwatch.rs

**Description:**  Published example from the `crossbeam-channel` crate.

 Prints the elapsed time every 1 second and quits on `Ctrl+C`. You can reinstate the separate main method for
 Windows provided you run the script with the `--multimain (-m)` option.

**Purpose:** showcase featured crates.

**Crates:** `crossbeam_channel`, `signal_hook`

**Type:** Program

**Categories:** crates

**Link:** [crossbeam_channel_stopwatch.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_channel_stopwatch.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_channel_stopwatch.rs
```

---

### Script: crossbeam_epoch_sanitize.rs

**Description:**  The `crossbeam-epoch` crate provides epoch-based _lock-free_ memory reclamation,
 an alternative to garbage collection.

 This is the published example from the `crossbeam-epoch` crate. For a more intuitive
 example, you can try the "Canary" example from https://github.com/ericseppanen/epoch_playground.
 and the associated blog post https://codeandbitters.com/learning-rust-crossbeam-epoch/.
 (Not included here due to implicit copyright). This will need at least a change from
 `rng.gen_range(0, bc_size)` to `rng.gen_range(0..bc_size)`, and optional updates to function naming.


**Purpose:** Demo featured crate.

**Crates:** `crossbeam_epoch`, `rand`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossbeam_epoch_sanitize.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_epoch_sanitize.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossbeam_epoch_sanitize.rs
```

---

### Script: crossterm.rs

**Description:**  Published example from the `crossterm` crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/README.md

**Purpose:** Demo crossterm terminal manipulation.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm.rs
```

---

### Script: crossterm_alternate_screen.rs

**Description:**  Published example from the `crossterm` crate. Macro version of the example:
 "Print a rectangle colored with magenta and use both direct execution and lazy execution."
 Direct execution with `execute` and lazy execution with `queue`.

 Url: https://docs.rs/crossterm/latest/crossterm/

**Purpose:** Demo `crossterm` command API.

**Crates:** `ratatui`

**Type:** Program

**Categories:** crates, technique, tui

**Link:** [crossterm_alternate_screen.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_alternate_screen.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_alternate_screen.rs -- true
```

---

### Script: crossterm_command_macro.rs

**Description:**  Published example from the `crossterm` crate. Macro version of the example:
 "Print a rectangle colored with magenta and use both direct execution and lazy execution."
 Direct execution with `execute` and lazy execution with `queue`.

 Url: https://docs.rs/crossterm/latest/crossterm/

**Purpose:** Demo `crossterm` command API.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm_command_macro.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_command_macro.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_command_macro.rs
```

---

### Script: crossterm_event_read.rs

**Description:**  Published example from the `crossterm` crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/examples/event-read.rs
 "Demonstrates how to block read events."

**Purpose:** Demo running crate example code, `crossterm` events.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm_event_read.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_event_read.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_event_read.rs
```

---

### Script: crossterm_key_events.rs

**Description:**  Published example from the `crossterm` crate.

 Url: https://github.com/crossterm-rs/crossterm/blob/master/examples/key-display.rs
 "Demonstrates the display format of key events.

 This example demonstrates the display format of key events, which is useful for displaying in
 the help section of a terminal application."

**Purpose:** Demo running crate example code, `crossterm` events.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, technique

**Link:** [crossterm_key_events.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_key_events.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/crossterm_key_events.rs
```

---

### Script: ctrlc_demo.rs

**Description:**  Published example from the `ctrlc` crate: "Cross platform handling of Ctrl-C signals."

**Purpose:** Demo one option for intercepting Ctrl-C.

**Crates:** `ctrlc`

**Type:** Program

**Categories:** crates, technique

**Link:** [ctrlc_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ctrlc_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ctrlc_demo.rs
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

**Link:** [curl.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/curl.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/curl.rs
```

---

### Script: darling_consume_fields.rs

**Description:**  Published example from the `darling` crate showing parsing for derive input.
 Extended to show formatted version of emitted code.

**Purpose:** Explore `darling` crate.

**Crates:** `darling`, `proc_macro2`, `quote`, `syn`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [darling_consume_fields.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/darling_consume_fields.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/darling_consume_fields.rs
```

---

### Script: debug_ansi_color_mismatch.rs

**Description:**  Debug ANSI color generation mismatch

 This script investigates why RGB values don't match the displayed colors
 by examining the ANSI codes being generated for specific RGB values.

**Purpose:** Diagnose ANSI color generation mismatch in dynamic color system

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, debugging, terminal

**Link:** [debug_ansi_color_mismatch.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_ansi_color_mismatch.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_ansi_color_mismatch.rs
```

---

### Script: debug_ansi_parsing.rs

**Description:**  Debug ANSI code generation and parsing

 This script helps debug how ANSI codes are generated by the styling system
 and tests the parsing logic for detecting attributes in ANSI sequences.

**Purpose:** Debug ANSI code generation and attribute detection

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, debugging, testing

**Link:** [debug_ansi_parsing.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_ansi_parsing.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_ansi_parsing.rs
```

---

### Script: debug_ansi_parsing_alt.rs

**Description:**  Debug ANSI code generation and parsing

 This script helps debug how ANSI codes are generated by the styling system
 and tests the parsing logic for detecting attributes in ANSI sequences.

**Purpose:** Debug ANSI code generation and attribute detection

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, debugging, testing

**Link:** [debug_ansi_parsing_alt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_ansi_parsing_alt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_ansi_parsing_alt.rs
```

---

### Script: debug_colors.rs

**Description:**  Debug demo to check theme loading and color values

 This demonstrates:
 1. Current theme information
 2. Color values for each role
 3. RGB values and indices
 4. Comparison with expected palette colors

**Purpose:** Debug theme and color loading issues

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, debugging, styling

**Link:** [debug_colors.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_colors.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_colors.rs
```

---

### Script: debug_colors_specific_theme.rs

**Description:**  Debug demo to check specific theme loading and color values

 This demonstrates:
 1. Explicit theme selection instead of auto-detection
 2. Color values for each role with specific theme
 3. RGB values and indices for the monet theme
 4. Comparison with auto-detected theme

**Purpose:** Debug specific theme loading vs auto-detection

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, debugging, styling, theming

**Link:** [debug_colors_specific_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_colors_specific_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_colors_specific_theme.rs
```

---

### Script: debug_detection.rs

**Description:**  Simple diagnostic script to debug the terminal corruption detection
 Simple corruption detection without synchronization
 Test what happens with normal println
 Test manual cursor positioning
 Test raw mode behavior
 Test the specific sequence that our detection function uses
 Visual alignment test

**Purpose:** Debug why we're getting false positives in corruption detection

**Crates:** `crossterm`

**Type:** Program

**Categories:** debugging, diagnosis, terminal

**Link:** [debug_detection.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_detection.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_detection.rs
```

---

### Script: debug_light_theme.rs

**Description:**  Minimal debug script to isolate light theme processing issues

 This script tests light theme generation with minimal parameters
 to identify where the infinite loop or color issues are occurring.

**Purpose:** Debug light theme generation issues

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [debug_light_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_light_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_light_theme.rs
```

---

### Script: debug_replace_logic.rs

**Description:**  Debug the replace logic to find why colors are wrong and resets aren't removed

 This demonstrates step-by-step what the replace logic is doing wrong
 and provides a corrected implementation

**Purpose:** Debug and fix replace logic for multi-level nesting

**Crates:** `thag_styling`

**Type:** Program

**Categories:** debugging, prototype, styling

**Link:** [debug_replace_logic.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_replace_logic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_replace_logic.rs
```

---

### Script: debug_styled_issue.rs

**Description:**  Debug styled! duplication issue

 Minimal test to isolate the double-printing problem

**Purpose:** Debug styled! macro duplication issue

**Crates:** `thag_proc_macros`

**Type:** Program

**Categories:** debugging, styling, testing

**Link:** [debug_styled_issue.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_styled_issue.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/debug_styled_issue.rs
```

---

### Script: derive_deftly.rs

**Description:**  Introductory example from the `derive-deftly` user guide.

**Purpose:** Explore proc macro alternatives.

**Crates:** `derive_deftly`

**Type:** Snippet

**Categories:** crates, exploration, technique

**Link:** [derive_deftly.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/derive_deftly.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/derive_deftly.rs
```

---

### Script: dethag_re.rs

**Description:**  Unescape `\n` and `\\` markers in a string to convert the wall of text to readable lines.
 This is an alternative approach to the original script that ended up as `src/bin/thag_legible.rs`.
 This version using regex may be more reliable than the classic approach using .lines().
 However, at time of writing, `regex` is a 248kB crate, which makes the binary of this
 module almost 7 times larger than that of `thag_legible` for debug builds and 4 times
 larger for release builds.

 Tip: Regex tested using https://rustexp.lpil.uk/.

**Purpose:** Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.

**Crates:** `lazy_static`, `regex`

**Type:** Program

**Categories:** crates, technique, tools

**Link:** [dethag_re.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/dethag_re.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/dethag_re.rs
```

---

### Script: document_pipeline.rs

**Description:**  Test async program (uninstrumented base / control version) for `thag_profiler` testing.
 See also `demo/document_pipeline_profile.rs` and `demo/document_pipeline_profile_minimal.rs`.


**Purpose:** Test auto-instrumentation using `thag_profiler`'s `thag-instrument` resulting in `demo/document_pipeline_profile.rs`.

**Crates:** `futures`, `tokio`

**Type:** Program

**Categories:** prototype, testing

**Link:** [document_pipeline.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline.rs
```

---

### Script: document_pipeline_profile.rs

**Description:**  Test async program (instrumented version) for `thag_profiler` testing.
 See also `demo/document_pipeline.rs` and `demo/document_pipeline_profile_minimal.rs`.

 Busy-wait for approximately `duration` without calling `.await`.
 Await was taking 200ms+ in tokio overhead

**Purpose:** Test profiling using `thag_profiler`.

**Crates:** `futures`, `thag_profiler`, `tokio`

**Type:** Program

**Categories:** prototype, testing

**Link:** [document_pipeline_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile.rs
```

---

### Script: document_pipeline_profile_minimal.rs

**Description:**  Test async program (minimalist instrumented version) for `thag_profiler` debugging.
 See also `demo/document_pipeline.rs` and `demo/document_pipeline_profile.rs`.


**Purpose:** Test and debug profiling using `thag_profiler`.

**Crates:** `thag_profiler`, `tokio`

**Type:** Program

**Categories:** prototype, testing

**Link:** [document_pipeline_profile_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_minimal.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_minimal.rs
```

---

### Script: document_pipeline_profile_minimal_alt.rs

**Description:**  Test async program (minimalist instrumented version) for `thag_profiler` debugging.
 See also `demo/document_pipeline.rs` and `demo/document_pipeline_profile.rs`.


**Purpose:** Test and debug profiling using `thag_profiler`.

**Crates:** `thag_profiler`, `tokio`

**Type:** Program

**Categories:** prototype, testing

**Link:** [document_pipeline_profile_minimal_alt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_minimal_alt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_minimal_alt.rs
```

---

### Script: document_pipeline_profile_sync.rs

**Description:**  Test sync program instrumented version for `thag_profiler` testing. You can use this script and
 `demo/document_pipeline_profile_sync_firestorm.rs` to compare `thag_profiler` with `firestorm`.
 Use the `-t` flag to get timings.

 Note that `thag_profiler`'s `Individual Sequential Execution Timeline` option is equivalent to `firestorm`'s `Time Axis`
 option, while `thag_profiler`'s `Aggregated Execution Timeline` option is equivalent to `firestorm`'s `Merged` option.
 `thag_profiler`'s `Show Statistics By Total Time` report is equivalent to  `firestorm`'s `Own Time` option.

 E.g.:

 `thag demo/document_pipeline_profile_sync.rs -t`


 See all `demo/document_pipeline*.rs` and in particular `demo/document_pipeline_profile_sync_firestorm.rs`.


**Purpose:** Test profiling using `thag_profiler`.

**Crates:** `thag_profiler`

**Type:** Program

**Categories:** prototype, testing

**Link:** [document_pipeline_profile_sync.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_sync.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_sync.rs
```

---

### Script: document_pipeline_profile_sync_firestorm.rs

**Description:**  `demo/document_pipeline_profile_sync.rs` to compare `firestorm` with `thag_profiler`.
 Use the `-t` flag to get timings.

 Note that `thag_profiler`'s `Individual Sequential Execution Timeline` option is equivalent to `firestorm`'s `Timeline`
 option, while `thag_profiler`'s `Aggregated Execution Timeline` option is equivalent to `firestorm`'s `Merged` option.
 `thag_profiler`'s `Show Statistics By Total Time` report is equivalent to  `firestorm`'s `Own Time` option.

 Firestorm does an internal warm-up AFAICS, so runs twice, and therefore almost twice as long. So is it apples with apples?
 Discuss.

 E.g.:

 `thag demo/document_pipeline_profile_sync_firestorm.rs -t`


 See all `demo/document_pipeline*.rs` and in particular `demo/document_pipeline_profile_sync.rs`.


**Purpose:** Test profiling using `firestorm`.

**Crates:** `firestorm`

**Type:** Program

**Categories:** prototype, testing

**Link:** [document_pipeline_profile_sync_firestorm.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_sync_firestorm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/document_pipeline_profile_sync_firestorm.rs
```

---

### Script: documented.rs

**Description:**  Published example from the `documented` crate.
 Trying is the first step to failure.

**Purpose:** Explore making docs available at runtime.

**Crates:** `documented`

**Type:** Snippet

**Categories:** crates, exploration, technique

**Link:** [documented.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/documented.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/documented.rs
```

---

### Script: documented_dependencies.rs

**Description:**  Use the `documented` crate to iterate through struct fields and their docs at runtime.
 Dependency handling

**Purpose:** Prototype for `thag_config_builder`.

**Crates:** `documented`, `phf`, `serde`, `serde_with`

**Type:** Snippet

**Categories:** crates, prototype, technique

**Link:** [documented_dependencies.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/documented_dependencies.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/documented_dependencies.rs
```

---

### Script: download_demos.rs

**Description:**  Prototype script for `thag_get_demo_dir` - fast replacement for `thag_get_demo`
 with subdirectory support. Git `sparse-checkout` approach suggested and written
 by ChatGPT, local directory handling assisted by Claude.

 `thag_styling` included

**Purpose:** Prototype for `thag_get_demo_dir`.

**Crates:** `colored`, `inquire`, `thag_styling`

**Type:** Program

**Categories:** crates, prototype, technique

**Link:** [download_demos.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/download_demos.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/download_demos.rs
```

---

### Script: duration_snippet.rs

**Description:**  Minimal snippet showing how to add further nice constructors such as `from_weeks` (and days, hours and
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

**Link:** [duration_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/duration_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/duration_snippet.rs
```

---

### Script: edit.rs

**Description:**  Published example from the `edit` crate readme.

 Will use the editor specified in VISUAL or EDITOR environment variable.

 E.g. `VISUAL="zed --wait" thag demo/edit.rs`

**Purpose:** Demo of edit crate to invoke preferred editor.

**Crates:** `edit`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [edit.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/edit.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/edit.rs
```

---

### Script: edit_profile.rs

**Description:**  Profiled version of published example from the `edit` crate readme.

 Will use the editor specified in VISUAL or EDITOR environment variable.

 E.g. `EDITOR="zed --wait" thag demo/edit_profile.rs`

**Purpose:** Demo of edit crate to invoke preferred editor.

**Crates:** `edit`, `thag_profiler`

**Type:** Program

**Categories:** crates, profiling, technique

**Link:** [edit_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/edit_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/edit_profile.rs
```

---

### Script: egui_code_editor.rs

**Description:**  A prototype GUI editor with saved state and syntax highlighting.

**Purpose:** Prototype a native-mode editor using the `egui` crate.

**Crates:** `eframe`, `egui`, `egui_extras`, `env_logger`

**Type:** Program

**Categories:** crates, gui, prototype

**Link:** [egui_code_editor.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/egui_code_editor.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/egui_code_editor.rs
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

**Link:** [enum_select.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/enum_select.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/enum_select.rs
```

---

### Script: env_var_debug.rs

**Description:**  Environment Variable Debug

 This script directly tests the environment variable parsing for color support
 to debug why THAG_COLOR_MODE=256 isn't working as expected.
 Direct implementation of check_env_color_support for testing

**Purpose:** Debug environment variable parsing for color support

**Crates:** `thag_common`

**Type:** Program

**Categories:** terminal, color, debugging, environment

**Link:** [env_var_debug.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/env_var_debug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/env_var_debug.rs
```

---

### Script: exclusify.rs

**Description:**  Process a folded file to calculate exclusive times

 This function converts inclusive time profiling data to exclusive time:
 - Inclusive time: total time spent in a function including all child calls
 - Exclusive time: time spent only in the function itself, excluding child calls

**Purpose:** Prototype converting inclusive elapsed times to exclusive for flamegraphs in order to avoid double counting.

**Type:** Program

**Categories:** profiling, prototype

**Link:** [exclusify.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/exclusify.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/exclusify.rs
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [factorial_dashu_product.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_dashu_product.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_dashu_product.rs -- 50
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [factorial_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_ibig.rs -- 50
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [factorial_ibig_product.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_ibig_product.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_ibig_product.rs -- 50
```

---

### Script: factorial_ibig_product_instr.rs

**Description:**  A version of `demo/factorial_ibig_product.rs` instrumented for profiling.

 Run this version in the normal way, then run `tools/thag_profile.rs` to analyse the profiling data.

**Purpose:** Demo `thag_rs` execution timeline and memory profiling.

**Crates:** `ibig`, `thag_profiler`

**Type:** Snippet

**Categories:** profiling

**Link:** [factorial_ibig_product_instr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_ibig_product_instr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_ibig_product_instr.rs -- 50
```

---

### Script: factorial_main_u128_product.rs

**Description:**  Fast factorial algorithm avoiding recursion, but limited to a maximum of `34!` by using only
 Rust primitives.

**Purpose:** Demo fast limited-scale factorial using Rust primitives and std::iter::Product trait.

**Type:** Program

**Categories:** learning, math, recreational, technique

**Link:** [factorial_main_u128_product.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_main_u128_product.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/factorial_main_u128_product.rs -- 34
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_4784969_cpp_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_4784969_cpp_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_4784969_cpp_ibig.rs -- 50
```

---

### Script: fib_4784969_cpp_rug.rs

**Description:**  Rust port of C++ example from `https://github.com/ZiCog/fibo_4784969` - so named because
 F(4784969) is the first number in the Fibonacci sequence that has one million decimal
 digits. This contains 3 alternative algorithms to compare their speed, with `fibo_new`
 edging out `fibo` at this scale.

 **Not compatible with Windows MSVC.**

 The `rug` crate runs blindingly fast, but be aware the rug dependency `gmp-mpfr-sys` may
 take several minutes to compile on first use or a version change.

 E.g.: `thag demo/fib_4784969_cpp_ibig.rs -- 4784969   // or any positive integer`


**Purpose:** Demo 3 very fast Fibonacci algorithms (F(4784969) in 0.33 to 0.58 sec for me).

**Crates:** `rug`

**Type:** Program

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_4784969_cpp_rug.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_4784969_cpp_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_4784969_cpp_rug.rs -- 50
```

---

### Script: fib_basic.rs

**Description:**  Fast non-recursive classic Fibonacci calculations for a specific value or an entire sequence.
 I can't recall the exact source, but see for example https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
 for a variety of alternative approaches. The various Fibonacci scripts here in the demo
 directory also show a range of approaches. `demo/fib_basic_ibig.rs` shows the use of
 the `std::iter::Successors` iterator as well as removing the limitations of Rust
 primitives. Most of the other examples explore different strategies for rapid computation of
 large Fibonacci values, and hopefully demonstrate the usefulness of `thag_rs` as a tool
 for trying out and comparing new ideas.

 As the number of Fibonacci examples here shows, this took me down a Fibonacci rabbit hole.

**Purpose:** Demo fast small-scale fibonacci using Rust primitives and `itertools` crate.

**Crates:** `itertools`

**Type:** Snippet

**Categories:** learning, math, recreational, technique

**Link:** [fib_basic.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_basic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_basic.rs -- 90
```

---

### Script: fib_basic_ibig.rs

**Description:**  Big-number (and thus more practical) version of `demo/fib_basic.rs`.


**Purpose:** Demo using a big-number crate to avoid the size limitations of primitive integers.

**Crates:** `ibig`, `itertools`

**Type:** Snippet

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_basic_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_basic_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_basic_ibig.rs -- 100
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

 **Not compatible with Windows MSVC.**

 The `rug` crate runs blindingly fast, but be aware the rug dependency `gmp-mpfr-sys` may
 take several minutes to compile on first use or a version change.


**Purpose:** Demonstrate snippets, closures, `clap` builder and a fast non-recursive fibonacci algorithm using the `successors`.

**Crates:** `clap`, `rug`

**Type:** Snippet

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_big_clap_rug.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_big_clap_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_big_clap_rug.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_binet_astro_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_binet_astro_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_binet_astro_snippet.rs -- 100
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

**Categories:** learning, math, recreational, technique

**Link:** [fib_binet_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_binet_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_binet_snippet.rs -- 100
```

---

### Script: fib_classic_ibig.rs

**Description:**  Fast non-recursive classic Fibonacci individual calculation with big integers.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.


**Purpose:** Demonstrate snippets and a fast non-recursive fibonacci algorithm using the `successors` iterator.

**Crates:** `ibig`

**Type:** Snippet

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_classic_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_classic_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_classic_ibig.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_classic_ibig_instrumented.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_classic_ibig_instrumented.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_classic_ibig_instrumented.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_dashu_snippet.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_dashu_snippet.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_dashu_snippet.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_iterative_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_iterative_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_iterative_ibig.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_iterative_purge_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_iterative_purge_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_iterative_purge_ibig.rs -- 100
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

 **Not compatible with Windows MSVC.**

 The `rug` crate runs blindingly fast, but be aware the rug dependency `gmp-mpfr-sys` may
 take several minutes to compile on first use or a version change.


**Purpose:** Demo fast efficient Fibonacci with big numbers, no recursion, and memoization, and ChatGPT implementation.

**Crates:** `rug`

**Type:** Program

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_iterative_purge_rug.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_iterative_purge_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_iterative_purge_rug.rs -- 100
```

---

### Script: fib_doubling_no_memo_ibig.rs

**Description:**  A version of `demo/fib_doubling_recursive.rs`, minus the memoization.
 This serves to prove that the memoization is faster, although
 not dramatically so.


**Purpose:** Demo fast efficient Fibonacci with big numbers, limited recursion, and no memoization, and ChatGPT implementation.

**Crates:** `ibig`

**Type:** Program

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_no_memo_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_no_memo_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_no_memo_ibig.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_no_memo_ibig_1.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_no_memo_ibig_1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_no_memo_ibig_1.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_no_memo_ibig_2.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_no_memo_ibig_2.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_no_memo_ibig_2.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_doubling_recursive_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_recursive_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_doubling_recursive_ibig.rs -- 100
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

**Categories:** learning, math, recreational, technique

**Link:** [fib_matrix.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_matrix_dashu.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix_dashu.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix_dashu.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_matrix_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix_ibig.rs -- 100
```

---

### Script: fib_matrix_rug.rs

**Description:**  Very fast recursive calculation of an individual Fibonacci number
 using the matrix squaring technique.

 Won't work with default Windows 11 because of the `rug` crate, which is a pity because
 `rug` is a beast due to its access to powerful GNU libraries.

 See https://en.wikipedia.org/wiki/Fibonacci_sequence.
 F0 = 0, F1 = 1, Fn = F(n-1) + F(n-2) for n > 1.

 **Not compatible with Windows MSVC.**

 The `rug` crate runs blindingly fast, but be aware the rug dependency `gmp-mpfr-sys` may
 take several minutes to compile on first use or a version change.


**Purpose:** Demo a very fast precise computation for large individual Fibonacci numbers.

**Crates:** `rug`

**Type:** Snippet

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_matrix_rug.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix_rug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_matrix_rug.rs -- 100
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

**Categories:** big_numbers, learning, math, recreational, technique

**Link:** [fib_quadrupling_recursive_ibig.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_quadrupling_recursive_ibig.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fib_quadrupling_recursive_ibig.rs -- 100
```

---

### Script: file_dialog_gui.rs

**Description:**  Demo of invoking the Rust formatter programmatically, with the addition of an `rfd`
 (`Rusty File Dialogs`) cross-platform file chooser to select the file to format.
 Compare with `demo/file_dialog_thag.rs`, which uses `thag_proc_macros` text-based
 file navigator and `inquire` to choose the file.

**Purpose:** Demo file chooser and calling an external program, in this case the Rust formatter.

**Crates:** `rfd`

**Type:** Program

**Categories:** crates, technique

**Link:** [file_dialog_gui.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/file_dialog_gui.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/file_dialog_gui.rs
```

---

### Script: file_dialog_thag.rs

**Description:**  Demo of invoking the Rust formatter programmatically, using `thag_proc_macros`
 cross-platform file chooser to select the file to format.
 Compare with `demo/file_dialog_gui.rs`, which uses the platform's native gui.

**Purpose:** Demo file chooser and calling an external program, in this case the Rust formatter.

**Crates:** `inquire`, `thag_proc_macros`, `thag_styling`

**Type:** Program

**Categories:** crates, technique

**Link:** [file_dialog_thag.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/file_dialog_thag.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/file_dialog_thag.rs
```

---

### Script: final_original_problem_test.rs

**Description:**  Final test recreating the exact original problem scenario

 This demonstrates the complete solution to your original embedding issue:
 - Multi-level nesting with different styles and attributes
 - Perfect context preservation using reset replacement
 - Direct comparison with the original broken approach

**Purpose:** Final verification that original embedding problem is completely solved

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, testing, validation

**Link:** [final_original_problem_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/final_original_problem_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/final_original_problem_test.rs
```

---

### Script: fizz_buzz_blandy_orendorff.rs

**Description:**  A fun example from Programming Rust by Jim Blandy and Jason Orendorff (O’Reilly).
 Copyright 2018 Jim Blandy and Jason Orendorff, 978-1-491-92728-1.
 Described by the authors as "a really gratuitous use of iterators".

**Purpose:** Demo using `thag_rs` to try out random code snippets ... also iterators.

**Type:** Snippet

**Categories:** learning, technique

**Link:** [fizz_buzz_blandy_orendorff.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fizz_buzz_blandy_orendorff.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fizz_buzz_blandy_orendorff.rs
```

---

### Script: fizz_buzz_gpt.rs

**Description:**  GPT-generated fizz-buzz example.

**Purpose:** Demo running random snippets in thag_rs, also AI and the art of delegation ;)

**Type:** Snippet

**Categories:** learning, technique

**Link:** [fizz_buzz_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/fizz_buzz_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/fizz_buzz_gpt.rs
```

---

### Script: flume_async.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.

**Purpose:** demo of async and channel programming and of `flume` in particular.

**Crates:** `async_std`, `flume`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [flume_async.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_async.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_async.rs
```

---

### Script: flume_async_profile.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.

 Refactored and profiled to test and demonstrate profiling of non-tokio
 async functions with `thag_profiler`.

**Purpose:** demo and test profiling of non-tokio async functions with `thag_profiler`.

**Crates:** `async_std`, `flume`, `thag_profiler`

**Type:** Program

**Categories:** async, crates, proc_macros, profiling, technique

**Link:** [flume_async_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_async_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_async_profile.rs
```

---

### Script: flume_perf.rs

**Description:**  Published example from the `flume` channel crate.

**Purpose:** demo of channel programming and of `flume` in particular.

**Crates:** `flume`

**Type:** Program

**Categories:** crates, technique

**Link:** [flume_perf.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_perf.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_perf.rs
```

---

### Script: flume_select.rs

**Description:**  Published example from the `flume` channel crate.
 Must be run with --multimain (-m) option to allow multiple main methods.

**Purpose:** demo of async and channel programming and of `flume` in particular.

**Crates:** `flume`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [flume_select.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_select.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/flume_select.rs
```

---

### Script: gen_names.rs

**Description:**  A very simple published example from the random name generator
 `names`. See also `demo/hyper_name_server.rs`.

**Purpose:** Demo a simple snippet and featured crate.

**Crates:** `names`

**Type:** Snippet

**Categories:** technique

**Link:** [gen_names.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/gen_names.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/gen_names.rs
```

---

### Script: generate_theme_from_image.rs

**Description:**  Demo of generating a `thag_styling` theme from an image.

**Purpose:** Demo making your own themes

**Crates:** `thag_styling`

**Type:** Snippet

**Categories:** color, styling, technique, xterm

**Link:** [generate_theme_from_image.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/generate_theme_from_image.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/generate_theme_from_image.rs
```

---

### Script: gpt_clap_derive.rs

**Description:**  GPT-generated CLI using the `clap` crate.

**Purpose:** Demonstrate `clap` CLI using the derive option.

**Crates:** `clap`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [gpt_clap_derive.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/gpt_clap_derive.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/gpt_clap_derive.rs -- -bgtv dummy_script.rs
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

**Link:** [gpt_lazy_static_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/gpt_lazy_static_theme.rs)

**Not suitable to be run from a URL.**


---

### Script: hello.rs

**Description:**  Obligatory Hello World as a snippet

**Purpose:** Demo Hello World snippet

**Type:** Snippet

**Categories:** basic

**Link:** [hello.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hello.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hello.rs
```

---

### Script: hello_main.rs

**Description:**  Hello World as a program (posh Winnie-the-Pooh version)

**Purpose:** Demo Hello World as a program

**Type:** Program

**Categories:** basic

**Link:** [hello_main.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hello_main.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hello_main.rs
```

---

### Script: hello_minimal.rs

**Description:**  Minimalist Hello World snippet (poor Winnie-the-Pooh version)

**Purpose:** Demo Hello World reduced to an expression

**Type:** Snippet

**Categories:** basic

**Link:** [hello_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hello_minimal.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hello_minimal.rs
```

---

### Script: history_debug.rs

**Description:**  Debug the history handling logic of the `stdin` module and display the effects.
 Using this abstraction because stdout/stderr displays don't work nicely in a TUI editor.

**Purpose:** Debug and demo history ordering.

**Crates:** `regex`, `serde`, `serde_json`

**Type:** Snippet

**Categories:** debugging, testing

**Link:** [history_debug.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/history_debug.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/history_debug.rs
```

---

### Script: hyper_client.rs

**Description:**  Published echo-server HTTP client example from the `hyper` crate,
 with the referenced modules `support` and `tokiort` refactored
 into the script, while respecting their original structure and
 redundancies. I've also synchronised the printing of the response,
 which was displaying out of sequence.
 You can run one of the hyper demo servers as the HTTP server on
 another command line and connect to it on port 3000.
 I prefer `hyper_name_server.rs` for variety, but `hyper_hello_server.rs`
 or `hyper_echo_server.rs` will work.
 Or use any other available HTTP server.

 ```bash
 thag demo/hyper_client.rs -- http://127.0.0.1:3000
 ```


**Purpose:** Demo `hyper` HTTP client, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `pin_project_lite`, `pretty_env_logger`, `tokio`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [hyper_client.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_client.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_client.rs -- http://127.0.0.1:3000
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

**Link:** [hyper_echo_server.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_echo_server.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_echo_server.rs
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

**Link:** [hyper_hello_server.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_hello_server.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_hello_server.rs
```

---

### Script: hyper_name_server.rs

**Description:**  An adaptation of `demo/hyper_hello_server.rs` that uses a thread-local name generator
 to show that each call to the server legitimately generates a new response.

**Purpose:** Demo `hyper` HTTP hello server, and incorporating separate modules into the script.

**Crates:** `bytes`, `http_body_util`, `hyper`, `names`, `pin_project_lite`, `pretty_env_logger`, `tokio`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [hyper_name_server.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_name_server.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/hyper_name_server.rs
```

---

### Script: ibig_big_integers.rs

**Description:**  Published example from the `ibig` crate, showcasing the use of the crate.

**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `ibig`

**Type:** Snippet

**Categories:** big_numbers, crates, technique

**Link:** [ibig_big_integers.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ibig_big_integers.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ibig_big_integers.rs
```

---

### Script: iced_tour.rs

**Description:**  The full tour of the `iced` crate published in the `iced` examples.

**Purpose:** Show that `thag_rs` can handle product demos.

**Crates:** `console_error_panic_hook`, `console_log`, `iced`, `tracing_subscriber`

**Type:** Program

**Categories:** crates, technique

**Link:** [iced_tour.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/iced_tour.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/iced_tour.rs
```

---

### Script: image_to_multi_format_theme.rs

**Description:**  Demo of generating multi-format terminal themes from images

 This example demonstrates the complete workflow:
 1. Generate a theme from an image using image analysis
 2. Export that theme to all supported terminal emulator formats
 3. Provide installation instructions for each format
 Extract RGB values from a style for display purposes

**Purpose:** Generate multi-format terminal themes from an image

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ansi, color, demo, styling, technique, terminal, xterm

**Link:** [image_to_multi_format_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/image_to_multi_format_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/image_to_multi_format_theme.rs
```

---

### Script: in_place.rs

**Description:**  Published example from `in-place crate` disemvowels the file somefile.txt.

**Purpose:** Demo editing a file in place.

**Crates:** `in_place`

**Type:** Program

**Categories:** async, crates, technique

**Link:** [in_place.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/in_place.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/in_place.rs
```

---

### Script: include_str.rs

**Description:**  Simple demo of `std::include_str` macro showing how to includes other files in demo or neighboring
 directories.

 This requires a main method so that `thag` won't move the snippet to a location under temp_dir().

 Not suitable for running from a URL.

**Purpose:** demo technique

**Type:** Program

**Categories:** basic, learning, technique

**Link:** [include_str.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/include_str.rs)

**Not suitable to be run from a URL.**


---

### Script: infer_deps.rs

**Description:**  Interactively test dependency inferency. This script was arbitrarily copied from
 `demo/repl_partial_match.rs`.
 Experiment with matching REPL commands with a partial match of any length.

**Purpose:** Usability: Accept a command as long as the user has typed in enough characters to identify it uniquely.

**Crates:** `clap`, `console`, `rustyline`, `shlex`, `strum`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [infer_deps.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/infer_deps.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/infer_deps.rs
```

---

### Script: inline_colorization.rs

**Description:**  Published simple example from `inline_colorization` crate. Simple effective inline
 styling option for text messages.

**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `inline_colorization`

**Type:** Snippet

**Categories:** async, crates, technique

**Link:** [inline_colorization.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/inline_colorization.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/inline_colorization.rs
```

---

### Script: interactive_clap_adv_struct.rs

**Description:**  Published example from the `interactive-clap` crate. I've adapted the run instructions below for use with `thag_rs`, and added theming of the `inquire::Select` UI:

 This example shows further functionality of the "interactive-clap" macro for parsing command-line data into a structure using macro attributes.

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

**Crates:** `clap`, `color_eyre`, `inquire`, `interactive_clap`, `shell_words`, `strum`, `thag_styling`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [interactive_clap_adv_struct.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/interactive_clap_adv_struct.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/interactive_clap_adv_struct.rs
```

---

### Script: is_unit_expr.rs

**Description:**  Demo determining at run-time whether an expression returns a unit value
 so that it can be handled appropriately.

 `thag` needs to know whether an expression returns a unit type or a value
 that we should display. When using a code template this approach using `Any`
 is short and sweet, but it has to be included in the template and thus the
 generated code, whereas the alternative of using an AST is quite a mission
 but works with any arbitrary snippet and doesn't pollute the generated
 source code, so `thag` went with the latter.

**Purpose:** Demo Rust's answer to dynamic typing.

**Type:** Snippet

**Categories:** exploration, type_identification, technique

**Link:** [is_unit_expr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/is_unit_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/is_unit_expr.rs
```

---

### Script: iter.rs

**Description:**  Demo a simple iterator

**Purpose:** Show how basic a snippet can be.

**Type:** Snippet

**Categories:** basic

**Link:** [iter.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/iter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/iter.rs
```

---

### Script: json.rs

**Description:**  Demo of deserialising JSON with the featured crates.

**Purpose:** Demo featured crates.

**Crates:** `serde`, `serde_json`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [json.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/json.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/json.rs
```

---

### Script: json_parse.rs

**Description:**  Demo of deserialising JSON with the featured crates.
 This version prompts for JSON input.

**Purpose:** Demo featured crates.

**Crates:** `serde`, `serde_json`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [json_parse.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/json_parse.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/json_parse.rs
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

**Link:** [just_a_test_expression.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/just_a_test_expression.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/just_a_test_expression.rs
```

---

### Script: konsole_export_demo.rs

**Description:**  Demo script showing Konsole colorscheme export functionality

**Purpose:** Demonstrate exporting thag themes to KDE Konsole .colorscheme format

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, terminal, theming, tools

**Link:** [konsole_export_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/konsole_export_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/konsole_export_demo.rs
```

---

### Script: list_files.rs

**Description:**  Demo listing files on disk. If you want a sorted list, you will need to amend the
 program to collect the entries into a Vec and sort that.

**Purpose:** Simple demonstration.

**Type:** Program

**Categories:** basic, technique

**Link:** [list_files.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/list_files.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/list_files.rs
```

---

### Script: loop_closure.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
 the latter since I'm not sure what the closure logic buys us. It's
 going to be checked by the compiler anyway. Compare with `demo/loop_expr.rs`.

 P.S.: The `--loop` option has since been implemented in `thag(_rs)`, without closure logic.

**Purpose:** Evaluate closure logic for line processing.

**Type:** Snippet

**Categories:** exploration, technique

**Link:** [loop_closure.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/loop_closure.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/loop_closure.rs
```

---

### Script: loop_expr.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
 the latter since I'm not sure what the closure logic buys us. It's
 going to be checked by the compiler anyway. Compare with `demo/loop_closure.rs`.

 P.S.: This has since been implemented in `thag(_rs)` as `--loop`.

**Purpose:** Evaluate expression logic for line processing.

**Type:** Snippet

**Categories:** exploration, technique

**Link:** [loop_expr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/loop_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/loop_expr.rs
```

---

### Script: loop_pre_post.rs

**Description:**  Exploring the possibility of incorporating a line processor similar
 to `rust-script`'s `--loop` or `runner`'s `--lines`, but with pre-
 and post-loop logic analogous to `awk`. I got GPT to do me this
 mock-up.
 P.S.: This has since been implemented in `thag(_rs)` as `--loop`.

**Purpose:** Evaluate expression logic for line processing.

**Type:** Program

**Categories:** exploration, technique

**Link:** [loop_pre_post.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/loop_pre_post.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/loop_pre_post.rs -- 'dummy prelude' 'dummy main' 'dummy post' # ... and hit Enter then Ctrl-d
```

---

### Script: mac_rgb_investigation.rs

**Description:**  Mac RGB Color Investigation

 This script investigates the specific issue on Mac where:
 - Palette-indexed colors (ESC[38;5;Nm) display correctly
 - RGB truecolor sequences (ESC[38;2;R;G;Bm) display incorrectly as washed-out colors

 The script tests various color output methods to understand what's happening
 with RGB color interpretation on macOS terminals.
 Test color struct for our investigations
 Test colors - specifically chosen to be distinctive
 Display a color using different methods for comparison
 Find the closest 256-color palette index for an RGB color
 Calculate color distance (simple Manhattan distance)
 Test OSC sequence color setting and querying
 Query terminal capabilities using OSC sequences
 Read a terminal response with timeout
 Display environment information that might affect color handling

**Purpose:** Investigate Mac RGB color display issues with different escape sequence methods

**Crates:** `crossterm`

**Type:** Program

**Categories:** color, debugging, mac_os, terminal

**Link:** [mac_rgb_investigation.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/mac_rgb_investigation.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/mac_rgb_investigation.rs
```

---

### Script: macro_fn_lazy_static.rs

**Description:**  Demo of a generic macro to generate lazy static variables without the `lazy_static` crate.

**Purpose:** Demonstrate a technique

**Type:** Program

**Categories:** learning, technique

**Link:** [macro_fn_lazy_static.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_fn_lazy_static.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_fn_lazy_static.rs
```

---

### Script: macro_gen_enum.rs

**Description:**  First prototype of building an enum from a macro and using it thereafter, thanks to SO user DK.
 `https://stackoverflow.com/questions/37006835/building-an-enum-inside-a-macro`

**Purpose:** explore a technique for resolving mappings from a message level enum to corresponding

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_gen_enum.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_gen_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_gen_enum.rs
```

---

### Script: macro_gen_styles_enum.rs

**Description:**  Second prototype of building an enum from a macro and using it thereafter.

**Purpose:** explore a technique for resolving mappings from a message level enum to corresponding

**Type:** Snippet

**Categories:** macros, technique

**Link:** [macro_gen_styles_enum.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_gen_styles_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_gen_styles_enum.rs
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

**Link:** [macro_lazy_static_var_advanced.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_advanced.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_advanced.rs
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

**Link:** [macro_lazy_static_var_advanced_alt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_advanced_alt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_advanced_alt.rs
```

---

### Script: macro_lazy_static_var_error_handling.rs

**Description:**  Demo of an advanced generic macro to generate lazy static variables.
 See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.

**Purpose:** Demonstrate a handy alternative to the `lazy_static` crate.

**Type:** Program

**Categories:** macros, technique

**Link:** [macro_lazy_static_var_error_handling.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_error_handling.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_error_handling.rs
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

**Link:** [macro_lazy_static_var_errs.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_errs.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_lazy_static_var_errs.rs
```

---

### Script: macro_print.rs

**Description:**  Proof of concept of distinguishing types that implement Display from those that implement
 Debug, and printing using the Display or Debug trait accordingly. Worked out with recourse
 to ChatGPT for suggestions and macro authoring.

**Purpose:** May be interesting or useful.

**Type:** Program

**Categories:** macros, technique, type_identification

**Link:** [macro_print.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_print.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/macro_print.rs
```

---

### Script: merge_toml.rs

**Description:**  Prototype of comprehensive merge of script toml metadata with defaults.

**Purpose:** Develop for inclusion in main project.

**Crates:** `cargo_toml`, `serde_merge`

**Type:** Program

**Categories:** crates, prototype, technique

**Link:** [merge_toml.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/merge_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/merge_toml.rs
```

---

### Script: mintty_color_detect.rs

**Description:**  Mintty Color Detection Test

 This script tests mintty's special OSC 7704 sequence for querying palette colors.
 Mintty uses a non-standard but more reliable method for color queries compared
 to standard OSC sequences. This can help detect background colors and verify
 palette colors in mintty terminals.

 Based on the shell script in TODO.md lines 49-74, this implements the same
 functionality in Rust to query mintty ANSI slots 0-15.
 RGB color representation
 Color pair for foreground and background
 Check if running in mintty
 Test specific palette color indices
 Test the full palette (0-15)
 Query mintty palette color using OSC 7704
 Read terminal response with timeout
 Parse mintty OSC 7704 response
 Expected format: ESC]7704;rgb:RRRR/GGGG/BBBB;rgb:RRRR/GGGG/BBBBBEL
 or: ESC]7704;{index};rgb:RRRR/GGGG/BBBB;rgb:RRRR/GGGG/BBBBBEL
 Parse a single rgb: component from mintty response
 Parse hex component (mintty uses 4-digit hex, we want the high byte)
 Analyze background detection possibilities
 Show suggestions for integrating mintty detection

**Purpose:** Test mintty-specific color detection using OSC 7704 sequences

**Crates:** `crossterm`, `thag_common`

**Type:** Program

**Categories:** color, detection, mintty, terminal, windows

**Link:** [mintty_color_detect.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/mintty_color_detect.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/mintty_color_detect.rs
```

---

### Script: mock_edit.rs

**Description:**  Used to debug a doctest.

**Purpose:** Debugging script.

**Crates:** `crossterm`, `mockall`, `thag_rs`

**Type:** Snippet

**Categories:** crates, technique, testing

**Link:** [mock_edit.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/mock_edit.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/mock_edit.rs
```

---

### Script: multi_format_theme_export.rs

**Description:**  Demo of multi-format theme export functionality

 This example demonstrates how to export a thag theme to multiple terminal emulator formats
 including Alacritty, WezTerm, iTerm2, Kitty, and Windows Terminal.

**Purpose:** Export thag themes to multiple terminal emulator formats

**Crates:** `env`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming

**Link:** [multi_format_theme_export.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/multi_format_theme_export.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/multi_format_theme_export.rs
```

---

### Script: multiline_err.rs

**Description:**  LLM-provided formatting for error messages

**Purpose:** Demo of formatting error messages

**Type:** Program

**Categories:** error_handling, technique

**Link:** [multiline_err.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/multiline_err.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/multiline_err.rs
```

---

### Script: owo_cli_color_support.rs

**Description:**  Demo the use of a command-line interface to override the colour support to be provided.
 The owo-colors "supports-colors" feature must be enabled.

**Purpose:** Demo setting colour support via a very simple CLI.

**Crates:** `clap`, `owo_colors`

**Type:** Program

**Categories:** CLI, crates, technique

**Link:** [owo_cli_color_support.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_cli_color_support.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_cli_color_support.rs
```

---

### Script: owo_colors_integration_demo.rs

**Description:**  Demo and test script for owo-colors integration with `thag_styling`.


**Purpose:** Demonstrate and test the owo-colors integration with `thag`'s theming system.

**Crates:** `owo_colors`, `thag_styling`

**Type:** Program

**Categories:** color, demo, styling, terminal, testing

**Link:** [owo_colors_integration_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_colors_integration_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_colors_integration_demo.rs
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

**Link:** [owo_msg_colors_1_basic_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_msg_colors_1_basic_gpt.rs)

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

**Link:** [owo_msg_colors_2_adv_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_msg_colors_2_adv_gpt.rs)

**Not suitable to be run from a URL.**


---

### Script: owo_styles.rs

**Description:**  An early exploration of the idea of adaptive message colouring according to the terminal theme.

**Purpose:** Demo a simple example of adaptive message colouring, and the featured crates.

**Crates:** `owo_colors`, `strum`, `termbg`

**Type:** Program

**Categories:** crates, exploration, technique

**Link:** [owo_styles.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/owo_styles.rs)

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

**Link:** [parse_script_rs_toml.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/parse_script_rs_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/parse_script_rs_toml.rs
```

---

### Script: parse_toml.rs

**Description:**  Prototype of extracting Cargo manifest metadata from source code by locating
 the start and end of the toml block. I eventually decided to use a regular
 expression as I found it less problematic (see `demo/regex_capture_toml.rs`).

**Purpose:** Prototype

**Type:** Program

**Categories:** prototype

**Link:** [parse_toml.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/parse_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/parse_toml.rs
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

**Link:** [pomprt_completion.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/pomprt_completion.rs)

**Not suitable to be run from a URL.**


---

### Script: prettyplease.rs

**Description:**  Published example from `prettyplease` Readme.

**Purpose:** Demo featured crate.

**Crates:** `prettyplease`, `syn`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [prettyplease.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/prettyplease.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/prettyplease.rs
```

---

### Script: proc_macro_cached.rs

**Description:**  Demo of the cached attribute macro that adds automatic memoization to functions.

 This macro demonstrates advanced attribute macro techniques by wrapping functions
 with caching logic. It automatically stores function results and returns cached
 values for repeated calls with the same parameters, providing significant
 performance improvements for expensive computations.
 Expensive computation that benefits from caching
 Expensive string processing that benefits from caching
 Mathematical computation with multiple parameters
 Prime number checking (expensive operation)

**Purpose:** Demonstrate automatic function memoization with caching

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, attribute_macros, performance, caching

**Link:** [proc_macro_cached.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_cached.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_cached.rs
```

---

### Script: proc_macro_category_enum.rs

**Description:**  Try generating category enum.
 Testing the `category_enum` proc macro for use with `demo/gen_readme.rs` and `demo/filter_demos.rs`/

**Purpose:** Test the proof of concept and potentially the implementation.

**Crates:** `thag_proc_macros`

**Type:** Program

**Categories:** missing

**Link:** [proc_macro_category_enum.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_category_enum.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_category_enum.rs
```

---

### Script: proc_macro_compile_time_assert.rs

**Description:**  Demo of the compile_time_assert function-like macro for compile-time validation.

 This macro demonstrates function-like macro parsing with multiple parameters
 and compile-time validation techniques. It generates assertions that are
 checked at compile time, causing compilation to fail if conditions are not met.

**Purpose:** Demonstrate compile-time assertions and validation

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, function_like_macros, compile_time, validation

**Link:** [proc_macro_compile_time_assert.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_compile_time_assert.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_compile_time_assert.rs
```

---

### Script: proc_macro_derive_builder.rs

**Description:**  Demo of the `DeriveBuilder` proc macro that generates builder pattern implementations.

 This macro demonstrates advanced derive macro techniques by generating a complete
 builder pattern implementation including:
 - A separate builder struct with optional fields
 - Fluent API with method chaining
 - Build-time validation with comprehensive error handling
 - Default trait implementation
 - Documentation generation

**Purpose:** Demonstrate builder pattern generation with validation

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, derive_macros, builder_pattern

**Link:** [proc_macro_derive_builder.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_builder.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_builder.rs
```

---

### Script: proc_macro_derive_constructor.rs

**Description:**  Basic "derive" macro generates a constructor (`new()`) for the struct it annotates.

 It also demonstrates how we can configure an attribute to expand the macro from the
 caller.

**Purpose:** explore proc macros

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** proc_macros, technique

**Link:** [proc_macro_derive_constructor.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_constructor.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_constructor.rs
```

---

### Script: proc_macro_derive_display.rs

**Description:**  Demo of the DeriveDisplay proc macro that generates Display trait implementations.

 This macro demonstrates advanced trait implementation generation by automatically
 creating Display implementations for various types:
 - Structs with named fields
 - Tuple structs
 - Unit structs
 - Enums with all variant types
 - Proper formatting with separators and type-aware output

**Purpose:** Demonstrate automatic Display trait implementation generation

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, derive_macros, trait_implementation

**Link:** [proc_macro_derive_display.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_display.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_display.rs
```

---

### Script: proc_macro_derive_doc_comment.rs

**Description:**  Demo of the enhanced DeriveDocComment proc macro that extracts documentation from multiple types.

 This macro demonstrates advanced derive macro techniques by extracting documentation
 comments from various Rust items and making them available at runtime:
 - Enum variants with their documentation
 - Struct fields with their documentation
 - The items themselves (struct/enum level docs)
 - Different struct types (named fields, tuple, unit)
 Represents the current status of a task or operation
 A comprehensive user configuration structure

 This struct holds all the necessary configuration
 for connecting to and managing a server.
 A simple point in 3D space
 A marker struct indicating successful initialization
 Different types of network protocols

**Purpose:** Demonstrate comprehensive documentation extraction across item types

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, derive_macros, documentation, attribute_parsing

**Link:** [proc_macro_derive_doc_comment.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_doc_comment.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_doc_comment.rs
```

---

### Script: proc_macro_derive_getters.rs

**Description:**  Demo of the DeriveGetters proc macro that automatically generates getter methods.

 This macro generates getter methods for all fields in a struct, returning references
 to avoid unnecessary moves. It's a simpler but still useful teaching example that
 demonstrates:
 - Derive macro syntax and parsing
 - Field iteration and type analysis
 - Method generation with documentation
 - Error handling for unsupported types

**Purpose:** Demonstrate automatic getter generation

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, derive_macros

**Link:** [proc_macro_derive_getters.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_getters.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_derive_getters.rs
```

---

### Script: proc_macro_env_or_default.rs

**Description:**  Demo of the env_or_default function-like macro for compile-time environment variable access.

 This macro demonstrates compile-time environment variable processing with fallback
 defaults. It reads environment variables during compilation and generates string
 literals, providing a zero-overhead configuration management pattern.

**Purpose:** Demonstrate compile-time environment variable access with defaults

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, function_like_macros, configuration, environment

**Link:** [proc_macro_env_or_default.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_env_or_default.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_env_or_default.rs
```

---

### Script: proc_macro_file_navigator.rs

**Description:**  Enhanced file navigator demo with editing and saving capabilities.

 This demo showcases the file_navigator proc macro by:
 1. Selecting a file using an interactive file browser
 2. Reading and displaying the file content
 3. Opening the file in an external editor for modification
 4. Saving the modified content to a new file
 5. Demonstrating all generated methods from the file_navigator macro

**Purpose:** Comprehensive demo of file_navigator macro with full workflow

**Crates:** `edit`, `inquire`, `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, file_handling, interactive

**Link:** [proc_macro_file_navigator.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_file_navigator.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_file_navigator.rs
```

---

### Script: proc_macro_generate_tests.rs

**Description:**  Demo of the generate_tests function-like macro for automatic test generation.

 This macro demonstrates repetitive code generation patterns by creating multiple
 test functions from a list of test data. It reduces boilerplate in test suites
 and shows how macros can automate common development tasks.

 Note that the expansion is not picked up by `cargo expand`, for reasons unknown.
 To compensate, the proc macro `generate_tests` prints the test source to `stderr`.

 Also, the expansions of the individual `generate_tests!` invocations are visible
 if the `expand` argument of the call to fn `maybe_expand_proc_macro` from the proc
 macro function fn `generate_tests` in `lib.rs` iis set to `true`. So if you prefer
 to use this, you can remove the hard-coded debugging from `generate_tests.rs`.

 To perform the tests and see the results, simply run:

 ```bash
 thag demo/proc_macro_generate_tests.rs --testing   # Short form: -T

 ```

 # Alternatively: you can run the tests via `thag_cargo`. Choose the script and the `test` subcommand.

 See also: `demo/test_profile_extract_timestamp.rs`

**Purpose:** Demonstrate automatic test case generation from data

**Crates:** `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, function_like_macros, testing, automation

**Link:** [proc_macro_generate_tests.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_generate_tests.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_generate_tests.rs
```

---

### Script: proc_macro_retry.rs

**Description:**  Demo of the retry attribute macro that adds automatic retry logic to functions.

 This macro demonstrates attribute macro parameter parsing and error handling
 patterns by wrapping functions with retry logic. It automatically retries
 failed function calls with configurable attempts and backoff delays.
 Unreliable network operation that fails randomly
 Custom retry count - try 5 times
 File operation that might fail due to permissions
 API call with authentication that might fail
 Resource allocation that might fail under load
 Service health check with retry

**Purpose:** Demonstrate automatic retry logic with configurable parameters

**Crates:** `rand`, `thag_demo_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, attribute_macros, error_handling, resilience

**Link:** [proc_macro_retry.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_retry.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_retry.rs
```

---

### Script: proc_macro_styled.rs

**Description:**  Testing the `styled` proc macro with `ansi_styling_support`

**Purpose:** Test the styled! macro with generated ANSI styling support.

**Crates:** `thag_proc_macros`, `thag_styling`

**Type:** Program

**Categories:** ansi, color, demo, macros, proc_macros, styling, terminal

**Link:** [proc_macro_styled.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_styled.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_styled.rs
```

---

### Script: proc_macro_timing.rs

**Description:**  Demo of the timing attribute macro that adds automatic execution time measurement.

 This macro demonstrates simple but effective attribute macro patterns by wrapping
 functions with timing logic. It automatically measures and displays execution time
 for any function, making it invaluable for performance analysis and optimization.
 Fast computation - should show minimal timing
 Medium-speed computation with visible timing
 Slow computation with artificial delay
 Recursive function with timing at each level
 Function that might fail, showing timing regardless
 Complex data processing with multiple steps
 Function with generic parameters
 Async-like simulation (using blocking operations)

**Purpose:** Demonstrate automatic function timing and performance measurement

**Crates:** `thag_proc_macros`

**Type:** Program

**Categories:** technique, proc_macros, attribute_macros, performance, timing

**Link:** [proc_macro_timing.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_timing.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/proc_macro_timing.rs
```

---

### Script: process_results.rs

**Description:**  Trait for processing results of an iterator.
 From Chaim freedman's answer to https://stackoverflow.com/questions/69746026/how-to-convert-an-iterator-of-results-into-a-result-of-an-iterator,
 combined with an example from the `itertools` crate at https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.process_results.


**Purpose:** RYO iterator result processor

**Type:** Program

**Categories:** learning, technique

**Link:** [process_results.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/process_results.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/process_results.rs
```

---

### Script: production_palette_query.rs

**Description:**  Production-Ready Terminal Palette Query

 This script demonstrates a production-ready implementation of OSC 4 palette
 querying using the crossterm method, which has been proven to work reliably
 across all major macOS terminals (Zed, WezTerm, Apple Terminal, iTerm2,
 Alacritty, and Kitty).

 Unlike the experimental version, this focuses on the reliable crossterm
 approach and includes proper error handling, caching, and integration
 patterns suitable for use in the `thag_styling` subcrate.
 RGB color representation
 Error types for palette querying
 Cached palette query results
 Production-ready palette color query using crossterm threading
 Parse OSC 4 response from accumulated buffer
 Parse hex component (2 or 4 digits)
 Get terminal identifier for caching
 Production-ready palette detection with caching
 Compare palette colors with current thag theme
 Extract RGB from a thag Style
 Display palette colors in a formatted table

**Purpose:** Production-ready palette querying with crossterm

**Crates:** `crossterm`, `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal

**Link:** [production_palette_query.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/production_palette_query.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/production_palette_query.rs
```

---

### Script: profile_file.rs

**Description:**  An early profiling prototype that tries to profile a file with macros via injection
 into its `syn` abstract syntax tree. The drawback is that this technique discards
 valuable information like comments and formatting.

 Note that the injected profiling code is no longer valid. this is a demonstration only

 E.g.: `thag demo/profile_file.rs < demo/hello_main.rs > $TMPDIR/hello_main_profiled.rs`


**Purpose:** Debugging

**Crates:** `prettyplease`, `quote`, `syn`

**Type:** Program

**Categories:** AST, crates, demo, learning, profiling, technique

**Link:** [profile_file.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/profile_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/profile_file.rs
```

---

### Script: profiling_puffin_demo.rs

**Description:**  Published demo from the `profiling` crate using the `puffin` profiler.
 We derive Deserialize/Serialize so we can persist app state on shutdown.

**Purpose:** Demo featured crates.

**Crates:** `eframe`, `egui`, `env_logger`, `log`, `profiling`, `puffin`, `puffin_egui`

**Type:** Program

**Categories:** crates

**Link:** [profiling_puffin_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/profiling_puffin_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/profiling_puffin_demo.rs
```

---

### Script: puffin_egui.rs

**Description:**  Published demo from the `puffin` crate. See `demo/puffin_egui_29.rs` for a newer version.

**Purpose:** Demo featured crate.

**Crates:** `eframe`, `puffin`, `puffin_egui`

**Type:** Program

**Categories:** crates

**Link:** [puffin_egui.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/puffin_egui.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/puffin_egui.rs
```

---

### Script: puffin_egui_28.rs

**Description:**  Published demo from the `puffin` profiling crate. The only change is to add a toml block
 entry to prevent a more recent `eframe` version from clashing with `puffin`.

**Purpose:** Demo featured crate.

**Crates:** `eframe`, `puffin`, `puffin_egui`

**Type:** Program

**Categories:** crates

**Link:** [puffin_egui_28.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/puffin_egui_28.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/puffin_egui_28.rs
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

**Purpose:** Recreational, learning.

**Crates:** `io`

**Type:** Snippet

**Categories:** learning, math, recreational

**Link:** [py_thag.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/py_thag.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/py_thag.rs
```

---

### Script: ra_ap_syntax_tree.rs

**Description:**  Parse and display the `rust-analyzer` (not `syn`) format syntax tree of a Rust source file.

 Assumes the input is a valid Rust program and that its Rust edition is 2021


**Purpose:** examine a `ra_ap_syntax` syntax tree.

**Crates:** `ra_ap_syntax`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [ra_ap_syntax_tree.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ra_ap_syntax_tree.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ra_ap_syntax_tree.rs
```

---

### Script: ratatui_integration_demo.rs

**Description:**  Simple Ratatui + thag_styling Integration Demo

 This demo shows how to create a basic themed TUI application using ratatui
 and thag_styling's semantic role system.

 E.g.:
 ```
 thag demo/ratatui_integration_demo.rs
 ```

**Purpose:** Basic demonstration of ratatui integration with thag_styling

**Crates:** `crossterm`, `ratatui`, `thag_styling`

**Type:** Program

**Categories:** demo, gui, theming, tui

**Link:** [ratatui_integration_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_integration_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_integration_demo.rs
```

---

### Script: ratatui_theming_showcase.rs

**Description:**  Comprehensive Ratatui Theming Showcase

 This example demonstrates how to build a themed TUI application using ratatui
 and thag_styling. It showcases various UI components styled with semantic roles
 and demonstrates both the ThemedStyle trait and extension methods.

 ```Rust
 E.g. `thag demo/ratatui_theming_showcase`
 ```

**Purpose:** Comprehensive showcase of ratatui integration with thag_styling themes

**Crates:** `crossterm`, `ratatui`, `thag_styling`

**Type:** Program

**Categories:** demo, theming, tui

**Link:** [ratatui_theming_showcase.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_theming_showcase.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_theming_showcase.rs
```

---

### Script: ratatui_user_input.rs

**Description:**  Published example from the `ratatui` crate.

 The latest version of this example is available in the [examples] folder in the "latest"
 branch of the `ratatui` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/ratatui/ratatui/blob/latest/examples/user_input.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Demo the featured crate.

**Crates:** `ratatui`

**Type:** Program

**Categories:** crates, tui

**Link:** [ratatui_user_input.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_user_input.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_user_input.rs
```

---

### Script: ratatui_user_input_profile.rs

**Description:**  Profiling the published example from the `ratatui` crate (`demo/ratatui_user_input.rs`)
 with `thag_profiler`.


**Purpose:** Demo the featured crate.

**Crates:** `ratatui`, `thag_profiler`

**Type:** Program

**Categories:** crates, profiling, tui

**Link:** [ratatui_user_input_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_user_input_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ratatui_user_input_profile.rs
```

---

### Script: readline_crossterm.rs

**Description:**  Published example from `crossterm` crate.

 The latest version of this example is available in the [examples] folder in the `crossterm`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/crossterm-rs/crossterm/blob/master/examples/event-read-char-line.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.

 Original `crossterm` crate comments:

 Demonstrates how to block read characters or a full line.
 Just note that crossterm is not required to do this and can be done with `io::stdin()`.

**Purpose:** Demo crossterm reading key events as a line or a single char.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates

**Link:** [readline_crossterm.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/readline_crossterm.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/readline_crossterm.rs
```

---

### Script: reedline_basic_keybindings.rs

**Description:**  Published example `basic.rs` from `reedline` crate.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/basic.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** demo featured crates.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_basic_keybindings.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_basic_keybindings.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_basic_keybindings.rs
```

---

### Script: reedline_completions.rs

**Description:**  Published example from `reedline` crate.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/completions.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** demo featured crates.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_completions.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_completions.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_completions.rs
```

---

### Script: reedline_event_listener.rs

**Description:**  Published example from `reedline` crate.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/event_listener.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** demo featured crates.

**Crates:** `crossterm`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_event_listener.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_event_listener.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_event_listener.rs
```

---

### Script: reedline_highlighter.rs

**Description:**  Published example from `reedline` crate.

 Try typing - among others - the known commands "test", "hello world", "hello world reedline", "this is the reedline crate".

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/highlighter.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_highlighter.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_highlighter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_highlighter.rs
```

---

### Script: reedline_hinter.rs

**Description:**  Published example from `reedline` crate.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/hinter.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Explore featured crate.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_hinter.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_hinter.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_hinter.rs
```

---

### Script: reedline_history.rs

**Description:**  Published example from `reedline` crate.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/history.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Demo `reedline` file-backed history.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_history.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_history.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_history.rs
```

---

### Script: reedline_ide_completions.rs

**Description:**  Published example from `reedline` crate. See the Vec of commands in the main method standing in for
 history. Enter a letter, e.g. "h" and press Tab to see the magic happen: all the commands starting
 with that letter will be displayed for selection with a tab, up and down arrows or Enter. Or you can
 enter subsequent letters to narrow the search. Noice.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/ide_completions.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Demo `reedline` tab completions.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_ide_completions.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_ide_completions.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_ide_completions.rs
```

---

### Script: reedline_list_bindings.rs

**Description:**  Published example from `reedline` crate.

 The latest version of this example is available in the [examples] folder in the `reedline`
 repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/list_bindings.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.

 List all keybinding information

**Purpose:** Explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_list_bindings.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_list_bindings.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_list_bindings.rs
```

---

### Script: reedline_multiline.rs

**Description:**  Exploratory prototype of REPL support for multi-line expressions. Loosely based on the
 published example `custom_prompt.rs` in `reedline` crate.

 The latest version of the original `custom_prompt.rs` is available in the [examples] folder
 in the `reedline` repository. At time of writing you can run it successfully just
 by invoking its URL with the `thag_url` tool, like this:

 ```bash
 thag_url https://github.com/nushell/reedline/blob/main/examples/custom_prompt.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Explore options for handling multi-line expressions in a REPL.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_multiline.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_multiline.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_multiline.rs
```

---

### Script: reedline_read_stdin.rs

**Description:**  Basic exploration of reading a line from stdin with `reedline`.

**Purpose:** Exploring how to render prompts and read lines of input.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_read_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_read_stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_read_stdin.rs
```

---

### Script: reedline_repl.rs

**Description:**  Published example from `reedline-repl-rs` crate.

 Sample invocation and dialogue:

 ```bash
 thag demo/reedline_repl.rs
 Welcome to MyApp
 MyApp〉say hello World!
 Hello, World!
 MyApp〉say goodbye --spanish                                                                                                                                06/30/2025 02:13:40 PM
 Adiós!
 MyApp〉[Ctrl-D]
 $
 ```

 The latest version of this example is available in the [examples] folder in the `reedline-repl-rs` repository.
 At time of writing you can run it successfully just by invoking its URL with the `thag_url` tool
 and passing the required arguments as normal, like this:

 ```bash
 thag_url https://github.com/arturh85/reedline-repl-rs/blob/main/examples/subcommands.rs
 ```

 This requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Explore the suitability of this crate for a Rust REPL. Conclusion: it's more geared to commands.

**Crates:** `reedline_repl_rs`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_repl.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_repl.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_repl.rs
```

---

### Script: reedline_repl_context.rs

**Description:**  Published example from `reedline-repl-rs` crate. This one uses the
 `clap` builder pattern; there is also one using the`clap` derive pattern.

 The latest version of this example is available in the [examples] folder in the `reedline-repl-rs` repository.
 At time of writing you can run it successfully just by invoking its URL with the `thag_url` tool
 and passing the required arguments as normal, like this:

 ```bash
 thag_url https://github.com/arturh85/reedline-repl-rs/blob/main/examples/with_context.rs
 ```

 Obviously this requires you to have first installed `thag_rs` with the `tools` feature.


**Purpose:** Evaluation of featured crate and of using clap to structure command input.

**Crates:** `reedline_repl_rs`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_repl_context.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_repl_context.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_repl_context.rs
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

**Categories:** crates, repl, technique

**Link:** [reedline_show_bindings.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_show_bindings.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_show_bindings.rs
```

---

### Script: reedline_stdin.rs

**Description:**  Exploring `reedline` crate.

**Purpose:** explore featured crate.

**Crates:** `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_stdin.rs
```

---

### Script: reedline_transient_prompt.rs

**Description:**  Published demo from `reedline` crate. Shows use of toml block to specify `reedline`
 features referenced in the example.

 Note that this script has been known to fail with a `libsql` error refegenciny:

 `include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));`

 In this case, try running `thag --cargo <dir_path>/reedline_transient_prompt.rs -- clean`
 and then rerunning the script.


**Purpose:** Demo the use of a transient minimal prompt `! ` for returned history.

**Crates:** `nu_ansi_term`, `reedline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [reedline_transient_prompt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_transient_prompt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/reedline_transient_prompt.rs
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

**Link:** [regex_capture_toml.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/regex_capture_toml.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/regex_capture_toml.rs
```

---

### Script: repl_block.rs

**Description:**  Early proof of concept of using a different line editor for repl.rs.

**Purpose:** Exploration

**Crates:** `clap`, `lazy_static`, `regex`, `repl_block`, `strum`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [repl_block.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/repl_block.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/repl_block.rs
```

---

### Script: repl_partial_match.rs

**Description:**  Experiment with matching REPL commands with a partial match of any length. `Ctrl-d` or `quit` to exit.

**Purpose:** Usability: Accept a command as long as the user has typed in enough characters to identify it uniquely.

**Crates:** `clap`, `console`, `rustyline`, `shlex`, `strum`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [repl_partial_match.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/repl_partial_match.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/repl_partial_match.rs
```

---

### Script: repl_ryo.rs

**Description:**  A demo of a roll-your-own REPL. This one is based on `thag_(rs)`'s own `repl` module, so relies heavily on `thag(_rs)`
 as a library. Other libraries are of course available! - you just have some work to do to replace the `thag(_rs)`
 plumbing with what you want. A choice of `MIT` or `Apache 2` licences applies.

**Purpose:** Demonstrate building a `thag`-style REPL.

**Crates:** `clap`, `edit`, `nu_ansi_term`, `ratatui`, `reedline`, `regex`, `strum`, `thag_profiler`, `thag_rs`, `thag_styling`, `tui_textarea`

**Type:** Program

**Categories:** demo, repl, technique, tui

**Link:** [repl_ryo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/repl_ryo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/repl_ryo.rs
```

---

### Script: rgb_palette_comparison.rs

**Description:**  RGB vs Palette Color Comparison

 This script directly demonstrates the issue where RGB truecolor sequences
 display differently than expected compared to palette-indexed colors.
 It tests the specific color mentioned: RGB(91, 116, 116) which should be
 a dark duck-egg blue-green but appears as washed-out salmon pink.
 Find the closest 256-color palette index for an RGB color
 Calculate color distance (Manhattan distance)

**Purpose:** Demonstrate RGB vs palette color display differences on Mac

**Type:** Program

**Categories:** color, debugging, mac_os, terminal

**Link:** [rgb_palette_comparison.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/rgb_palette_comparison.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/rgb_palette_comparison.rs
```

---

### Script: rug_arbitrary_precision_nums.rs

**Description:**  Published example from the `rug` crate, showcasing the use of the crate. I added the
 last line to return a tuple of the state of the values of interest, as a quick way
 of displaying them.


 **Not compatible with Windows MSVC.**

 The `rug` crate runs blindingly fast, but be aware the rug dependency `gmp-mpfr-sys` may
 take several minutes to compile on first use or a version change.

 On Linux you may need to install the m4 package.


**Purpose:** Demo featured crate, also how we can often run an incomplete snippet "as is".

**Crates:** `rug`

**Type:** Snippet

**Categories:** crates, technique

**Link:** [rug_arbitrary_precision_nums.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/rug_arbitrary_precision_nums.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/rug_arbitrary_precision_nums.rs
```

---

### Script: rustfmt_stdin.rs

**Description:**  Read Rust source code from stdin and display the output as formatted by `rustfmt`.

**Purpose:** Format arbitrary Rust code. Does no more than `rustfmt --`.

**Type:** Program

**Categories:** crates, technique

**Link:** [rustfmt_stdin.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/rustfmt_stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/rustfmt_stdin.rs
```

---

### Script: rustlings_smart_pointers_rc1.rs

**Description:**  Published exercise solution from the `rustlings` crate.

**Purpose:** Demo one way to preserve your `rustlings` solutions, for reference or as katas.

**Type:** Program

**Categories:** learning

**Link:** [rustlings_smart_pointers_rc1.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/rustlings_smart_pointers_rc1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/rustlings_smart_pointers_rc1.rs
```

---

### Script: rustyline_compl.rs

**Description:**  Published example from the `rustyline` crate.

**Purpose:** Demo using `thag_rs` to run a basic REPL as a script.

**Crates:** `env_logger`, `rustyline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [rustyline_compl.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/rustyline_compl.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/rustyline_compl.rs
```

---

### Script: rustyline_full.rs

**Description:**  Example from `rustyline` crate readme.
 MatchingBracketValidator uses matching brackets to decide between single- and multi-line
 input.

**Purpose:** Explore `rustyline` crate.

**Crates:** `env_logger`, `rustyline`

**Type:** Program

**Categories:** crates, repl, technique

**Link:** [rustyline_full.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/rustyline_full.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/rustyline_full.rs
```

---

### Script: semver_exclude_prerelease.rs

**Description:**  Prototype of excluding pre-release crates from cargo queries.

**Purpose:** Prototype technique for `thag_rs`.

**Crates:** `semver`

**Type:** Program

**Categories:** prototype, technique

**Link:** [semver_exclude_prerelease.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/semver_exclude_prerelease.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/semver_exclude_prerelease.rs
```

---

### Script: side_by_side_diff.rs

**Description:**  Published example from `side-by-side-diff` crate.

**Purpose:** Explore integrated side by side diffs.

**Crates:** `side_by_side_diff`

**Type:** Program

**Categories:** crates, exploration

**Link:** [side_by_side_diff.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/side_by_side_diff.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/side_by_side_diff.rs
```

---

### Script: simple_osc4_test.rs

**Description:**  Simple OSC 4 Test

 A minimal test script to debug OSC 4 response capture issues.
 This script sends a single OSC 4 query and tries different methods
 to capture the response, helping identify why responses are visible
 but not being captured programmatically.
 RGB color representation
 Try to parse OSC 4 response
 Method 1: Direct stdin reading with crossterm raw mode
 Method 2: Try using shell command with script/expect
 Method 3: Use expect-like approach
 Method 4: Manual observation test
 Method 5: Redirect to file test

**Purpose:** Debug OSC 4 response capture mechanisms

**Crates:** `crossterm`

**Type:** Program

**Categories:** debugging, exploration, terminal

**Link:** [simple_osc4_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_osc4_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_osc4_test.rs
```

---

### Script: simple_reset_test.rs

**Description:**  Simple test to see if reset replacement is being called

 This creates the exact scenario from the stress test to see where
 the reset replacement logic is failing.

**Purpose:** Debug reset replacement execution

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, debugging, testing

**Link:** [simple_reset_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_reset_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_reset_test.rs
```

---

### Script: simple_stress_test.rs

**Description:**  Simple stress test with raw ANSI inspection

 This recreates the stress test scenario and shows the raw ANSI codes
 to verify if the reset replacement is working correctly.

**Purpose:** Debug stress test with raw ANSI inspection

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, debugging, testing

**Link:** [simple_stress_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_stress_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_stress_test.rs
```

---

### Script: simple_theme_test.rs

**Description:**  Simple test to debug theme loading issues

**Purpose:** Simple test for runtime theme loading functionality

**Crates:** `thag_styling`

**Type:** Program

**Categories:** demo, styling, theming

**Link:** [simple_theme_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_theme_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/simple_theme_test.rs
```

---

### Script: slog_expressions.rs

**Description:**  Published example from `slog` crate (misc/examples/expressions.rs).

**Purpose:** Demo a popular logging crate.

**Crates:** `slog`, `slog_term`

**Type:** Program

**Categories:** crates

**Link:** [slog_expressions.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/slog_expressions.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/slog_expressions.rs
```

---

### Script: smol_background_task_return.rs

**Description:**  ChatGPT-generated example of running a single task in the background.

**Purpose:** Demo.

**Crates:** `smol`

**Type:** Program

**Categories:** crates, demo

**Link:** [smol_background_task_return.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_background_task_return.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_background_task_return.rs
```

---

### Script: smol_chat_client.rs

**Description:**  Published example from `smol crate`. See also `demo/smol_chat_server.rs` and
 `demo/smol_chat_server_profile.rs`.

**Purpose:** Demo, and participant in `thag_profiler` test.

**Crates:** `smol`

**Type:** Program

**Categories:** demo

**Link:** [smol_chat_client.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_chat_client.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_chat_client.rs
```

---

### Script: smol_chat_server.rs

**Description:**  Published example from `smol crate`. See also `demo/smol_chat_client.rs` and
 `demo/smol_chat_server_profile.rs`.
 An event on the chat server.
 Dispatches events to clients.
 Reads messages from the client and forwards them to the dispatcher task.

**Purpose:** Demo, and basis for `thag_profiler` test.

**Crates:** `async_channel`, `async_dup`, `smol`

**Type:** Program

**Categories:** demo

**Link:** [smol_chat_server.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_chat_server.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_chat_server.rs
```

---

### Script: smol_chat_server_profile.rs

**Description:**  Published example from `smol crate`, instrumented for testing of `thag_profiler`, with clean shutdown
 added.

 Instrumented with that sub-crate's `thag-instrument` command, and shutdown logic added by Claude Sonnet 3.7.

 See also `demo/smol_chat_server.rs` and
 `demo/smol_chat_client.rs`.

 E.g. `thag demo/smol_chat_server_profile.rs`

 Features added by Claude for clean shutdown to preserve profiling data:

 1. **Guaranteed Shutdown**: The server now automatically shuts down after 30 seconds, ensuring that profiling data can be properly finalized.

 2. **Graceful Task Handling**: Tasks are allowed to complete with timeouts, preventing hanging processes.

 3. **Non-Blocking Accept Logic**: The server can check for termination signals while accepting connections without blocking.
 An event on the chat server.
 Dispatches events to clients.
 Reads messages from the client and forwards them to the dispatcher task.

**Purpose:** Test `thag_profiler` with `smol` async crate.

**Crates:** `async_channel`, `async_dup`, `futures_lite`, `smol`, `thag_profiler`

**Type:** Program

**Categories:** profiling, testing

**Link:** [smol_chat_server_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_chat_server_profile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/smol_chat_server_profile.rs
```

---

### Script: snippet_import_scope.rs

**Description:**  Demo scope of import statements.

**Purpose:** Prototype to confirm leaving imports in situ when wrapping snippets.

**Crates:** `ibig`

**Type:** Snippet

**Categories:** crates, learning

**Link:** [snippet_import_scope.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/snippet_import_scope.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/snippet_import_scope.rs
```

---

### Script: snippet_name_clash.rs

**Description:**  Demo scope of import statements. Two conflicting imports with the same name
 `ubig` coexisting in the same `println!` invocation. Demonstrates that when
 wrapping a snippet we can't assume it's OK to pull the imports up to the top
 level.

**Purpose:** Prototype to confirm leaving imports in situ when wrapping snippets.

**Crates:** `dashu`, `ibig`

**Type:** Snippet

**Categories:** crates, learning

**Link:** [snippet_name_clash.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/snippet_name_clash.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/snippet_name_clash.rs
```

---

### Script: sort_itertools.rs

**Description:**  Demo sorting RGB tuples using `itertools`. Generated by ChatGPT.

**Purpose:** A simple demonstration.

**Crates:** `itertools`

**Type:** Program

**Categories:** crates, learning, technique

**Link:** [sort_itertools.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/sort_itertools.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/sort_itertools.rs
```

---

### Script: sort_std.rs

**Description:**  Demo sorting RGB tuples using `std`. Generated by ChatGPT.

**Purpose:** A simple demonstration.

**Type:** Program

**Categories:** crates, learning, technique

**Link:** [sort_std.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/sort_std.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/sort_std.rs
```

---

### Script: stdin.rs

**Description:**  A version of `thag_rs`'s `stdin` module to handle standard input editor input. Like the `colors`
 module, `stdin` was originally developed here as a separate script and integrated as a module later.

 E.g. `thag demo/stdin.rs`

**Purpose:** Demo using `thag_rs` to develop a module outside of the project.

**Crates:** `anyhow`, `lazy_static`, `ratatui`, `regex`, `tui_textarea`

**Type:** Program

**Categories:** crates, prototype, technique, tui

**Link:** [stdin.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin.rs
```

---

### Script: stdin_main.rs

**Description:**  Open the history file in an editor.
 # Errors
 Will return `Err` if there is an error editing the file.

**Purpose:** Debugging and demonstration.

**Crates:** `edit`, `ratatui`, `thag_rs`

**Type:** Program

**Categories:** demo, testing, tui

**Link:** [stdin_main.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin_main.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin_main.rs
```

---

### Script: stdin_main_old_instr.rs

**Description:**  Apply highlights to the text depending on the light or dark theme as detected, configured
 or defaulted, or as toggled by the user with Ctrl-t.

**Purpose:** Debugging.

**Crates:** `lazy_static`, `mockall`, `ratatui`, `regex`, `scopeguard`, `serde`, `serde_json`, `thag_profiler`, `thag_rs`, `tui_textarea`

**Type:** Program

**Categories:** testing, tui

**Link:** [stdin_main_old_instr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin_main_old_instr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin_main_old_instr.rs
```

---

### Script: stdin_main_upd_instr.rs

**Description:**  Edit the stdin stream.


 # Examples

 ```no_run
 use thag_rs::stdin::edit;
 use thag_rs::CrosstermEventReader;
 use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers };
 use thag_rs::MockEventReader;

 let mut event_reader = MockEventReader::new();
 event_reader.expect_read_event().return_once(|| {
     Ok(Event::Key(KeyEvent::new(
         KeyCode::Char('d'),
         KeyModifiers::CONTROL,
     )))
 });
 let actual = edit(&event_reader);
 let buf = vec![""];
 assert!(matches!(actual, Ok(buf)));
 ```
 # Errors

 If the data in this stream is not valid UTF-8 then an error is returned and the read buffer is left unchanged.
 # Panics

 If the terminal cannot be reset.
 Prompt for and read Rust source code from stdin.

 # Examples

 ``` ignore
 use thag_rs::stdin::read;

 let hello = String::from("Hello world!");
 assert!(matches!(read(), Ok(hello)));
 ```
 # Errors

 If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
 Read Rust source code into a String from the provided reader (e.g., stdin or a mock reader).

 # Examples

 ``` ignore
 use thag_rs::stdin::read_to_string;

 let stdin = std::io::stdin();
 let mut input = stdin.lock();
 let hello = String::from("Hello world!");
 assert!(matches!(read_to_string(&mut input), Ok(hello)));
 ```

 # Errors

 If the data in this stream is not valid UTF-8 then an error is returned and buf is unchanged.
 Open the history file in an editor.
 # Errors
 Will return `Err` if there is an error editing the file.

**Purpose:** Debugging.

**Crates:** `edit`, `ratatui`, `thag_profiler`, `thag_rs`

**Type:** Program

**Categories:** profiling, testing, tui

**Link:** [stdin_main_upd_instr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin_main_upd_instr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/stdin_main_upd_instr.rs
```

---

### Script: string_to_static_str.rs

**Description:**  Demo: Convert a `String` to a `&'static str` at runtime, then do
 the same for a whole vector of `String`s.

 This should only be used when it's appropriate for the string
 reference to remain allocated for the life of the program.

**Purpose:** demo a handy trick.

**Type:** Program

**Categories:** learning, technique

**Link:** [string_to_static_str.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/string_to_static_str.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/string_to_static_str.rs
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

**Link:** [structopt_cli_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/structopt_cli_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/structopt_cli_gpt.rs -- -- -Vt dummy.rs 1 2 3
```

---

### Script: styleable_improvements_demo.rs

**Description:**  Demo showcasing improved Styleable trait with individual role methods

 This demonstrates:
 1. Consolidated style_with() method that works with any Styler
 2. Individual role methods: .error(), .success(), .info(), etc.
 3. Comparison with Role.paint() approach
 4. Using &self instead of self (non-consuming)

**Purpose:** Demo improved Styleable trait with role methods

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ergonomics, styling

**Link:** [styleable_improvements_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/styleable_improvements_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/styleable_improvements_demo.rs
```

---

### Script: styled_macro_enhanced.rs

**Description:**  Enhanced styled! Macro Demonstration

 This demo showcases the enhanced styled! macro with support for:
 - Basic ANSI colors (original functionality)
 - 256-color palette indices
 - True RGB colors
 - Multiple text effects

 The enhanced macro now supports three color formats:
 1. Basic colors: Red, Green, Blue, etc. (uses terminal palette)
 2. Color256(index): 256-color palette (0-255)
 3. Rgb(r, g, b): True RGB colors (0-255 per component)

**Purpose:** Demonstrate enhanced styled! macro with 256-color and RGB support

**Crates:** `thag_proc_macros`

**Type:** Program

**Categories:** styling, macros, color, demo

**Link:** [styled_macro_enhanced.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/styled_macro_enhanced.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/styled_macro_enhanced.rs
```

---

### Script: styled_string_concept.rs

**Description:**  Concept demo for StyledString that preserves outer styling context

 This demonstrates a potential StyledString type that could work like
 colored's ColoredString, automatically restoring outer styling after
 inner reset sequences.
 A styled string that preserves styling context like colored's ColoredString
 Extended Styleable trait that returns StyledString instead of plain String

**Purpose:** Concept for context-preserving styled strings

**Crates:** `thag_styling`

**Type:** Program

**Categories:** concepts, prototype, styling

**Link:** [styled_string_concept.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/styled_string_concept.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/styled_string_concept.rs
```

---

### Script: styling_demo.rs

**Description:**  Demonstrates the colour and styling options of `thag_rs`.
 Also demos the full 256-colour palette as per `demo/colors*.rs`.

 E.g. `thag demo/styling_demo.rs`

**Purpose:** Demonstrate and test the look of available colour palettes and styling settings.

**Crates:** `strum`, `thag_styling`

**Type:** Program

**Categories:** prototype, reference, testing

**Link:** [styling_demo.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/styling_demo.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/styling_demo.rs
```

---

### Script: styling_migration_guide.rs

**Description:**  Comprehensive migration guide from old embedding systems to StyledString

 This demo shows side-by-side comparisons of:
 1. sprtln_with_embeds! → StyledString with println!
 2. svprtln_with_embeds! → StyledString with vprintln!
 3. format_with_embeds → format! with StyledString
 4. Embedded struct → StyledString directly

 IMPORTANT: This guide is specifically about replacing the EMBEDDING system.
 The Styled<T> struct (.style().bold()) serves a different purpose and remains:
 - Styled<T>: General text effects (bold, italic, etc.) - KEEP USING
 - StyledString: Semantic roles + embedding/nesting - NEW PREFERRED WAY

 The new StyledString approach provides:
 - Better attribute reset handling (no bleeding)
 - More natural Rust syntax with method chaining
 - Unlimited nesting depth without pre-planning
 - Better performance (no macro overhead)
 - Cleaner, more maintainable code

**Purpose:** Migration guide from old embedding systems to StyledString

**Crates:** `thag_styling`

**Type:** Program

**Categories:** documentation, examples, migration, styling

**Link:** [styling_migration_guide.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/styling_migration_guide.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/styling_migration_guide.rs
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

**Link:** [supports_color.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/supports_color.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/supports_color.rs
```

---

### Script: supports_color_win.rs

**Description:**  Windows-friendly logic extracted from crate `supports-color`.


**Purpose:** Proof of concept for Windows environment

**Type:** Snippet

**Categories:** crates, prototype

**Link:** [supports_color_win.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/supports_color_win.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/supports_color_win.rs
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

**Link:** [syn_dump_syntax.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_dump_syntax.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_dump_syntax.rs -- demo/hello_main.rs
```

---

### Script: syn_dump_syntax_profile_syn.rs

**Description:**  A version of the published example from the `syn` crate used to demonstrate profiling a dependency with `thag_profiler`.
 Description "Parse a Rust source file into a `syn::File` and print out a debug representation of the syntax tree."

 Pass it the absolute or relative path of any Rust PROGRAM source file, e.g. its own
 path that you passed to the script runner to invoke it.

 NB: Pick a script that is a valid program (containing `fn main()` as opposed to a snippet).

 E.g.:

 ```
 THAG_PROFILER=both,,announce,true thag demo/syn_dump_syntax_profile_syn.rs -tf -- demo/hello_main.rs
 ```

 See the `README.md` for the explanation of the `THAG_PROFILER` arguments

**Purpose:** demonstrate profiling a dependency with `thag_profiler`.

**Crates:** `colored`, `syn`, `thag_profiler`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [syn_dump_syntax_profile_syn.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_dump_syntax_profile_syn.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_dump_syntax_profile_syn.rs -- demo/hello_main.rs
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

**Link:** [syn_quote.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_quote.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_quote.rs
```

---

### Script: syn_remove_attributes.rs

**Description:**  Prototype of removing an inner attribute (`#![...]`) from a syntax tree. Requires the `visit-mut'
 feature of `syn`.

**Purpose:** Demonstrate making changes to a `syn` AST.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_remove_attributes.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_remove_attributes.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_remove_attributes.rs
```

---

### Script: syn_visit_extern_crate_expr.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to determine the dependencies of a
 Rust source program passed to the script. Specifically the combination of fn `visit_item_extern_crate`
 to process the nodes representing `extern crate` statements and fn `visit_expr` to start the tree
 traversal. This version expects the script contents to consist of a Rust expression.

**Purpose:** Prototype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_extern_crate_expr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_extern_crate_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_extern_crate_expr.rs -- demo/just_a_test_expression.rs
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

**Link:** [syn_visit_extern_crate_file.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_extern_crate_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_extern_crate_file.rs -- demo/syn_visit_extern_crate_file.rs
```

---

### Script: syn_visit_node_type.rs

**Description:**  Demo of selectively modifying source code using `syn` and `quote`. This is from a solution posted by user Yandros on the Rust Playground
 in answer to a question asked on the Rust users forum. The discussion and Playground link are to be found here:
 https://users.rust-lang.org/t/writing-proc-macros-with-syn-is-there-a-way-to-visit-parts-of-the-ast-that-match-a-given-format/54733/4
 (This content is dual-licensed under the MIT and Apache 2.0 licenses according to the Rust forum terms of service.)
 I've embellished it to show how it can be formatted with `prettyplease` if parsed as a `syn::File`.

**Purpose:** Demo programmatically modifying Rust source code using `syn` and `quote`.

**Crates:** `prettyplease`, `quote`, `syn`

**Type:** Program

**Categories:** AST, crates, technique

**Link:** [syn_visit_node_type.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_node_type.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_node_type.rs
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

**Link:** [syn_visit_use_path_expr.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_path_expr.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_path_expr.rs -- demo/just_a_test_expression.rs
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

**Link:** [syn_visit_use_path_file.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_path_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_path_file.rs -- demo/syn_visit_use_path_file.rs
```

---

### Script: syn_visit_use_rename.rs

**Description:**  Prototype that uses the Visitor pattern of the `syn` crate to identify `use` statements that use "as"
 to rename a dependency so that `thag` doesn't go looking for the temporary name in the registry.
 Specifically the combination of fn `visit_use_rename` to process the nodes representing `extern crate`
 statements and fn `visit_file` to initiate the tree traversal. This version expects the script contents
 to consist of a fully-fledged Rust program.

**Purpose:** Prototype.

**Crates:** `syn`

**Type:** Program

**Categories:** AST, crates, prototype, technique

**Link:** [syn_visit_use_rename.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_rename.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_rename.rs -- demo/crossbeam_epoch_sanitize.rs
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

**Link:** [syn_visit_use_tree_file.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_tree_file.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/syn_visit_use_tree_file.rs -- demo/syn_visit_use_tree_file.rs
```

---

### Script: tempfile.rs

**Description:**  Published example from the `tempfile` readme.

**Purpose:** Demo featured crate.

**Crates:** `tempfile`

**Type:** Program

**Categories:** crates

**Link:** [tempfile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tempfile.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/tempfile.rs
```

---

### Script: termbg.rs

**Description:**  Published example from `termbg` readme.

 Detects the light or dark theme in use, as well as the colours in use.

**Purpose:** Demo theme detection with `termbg`

**Crates:** `simplelog`, `termbg`

**Type:** Program

**Categories:** crates

**Link:** [termbg.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/termbg.rs)

**Not suitable to be run from a URL.**


---

### Script: terminal_color_diagnostics.rs

**Description:**  Terminal Color Diagnostics

 This script performs comprehensive diagnostics of terminal color capabilities,
 specifically designed to investigate issues where RGB truecolor sequences
 display incorrectly while palette-indexed colors work correctly.

 The script tests multiple aspects of color handling:
 - Basic ANSI color support
 - 256-color palette support
 - RGB truecolor support
 - OSC sequence handling
 - Terminal environment detection
 - Color profile and gamma correction issues
 Test color struct for diagnostics
 Diagnostic test colors chosen to highlight color handling issues
 Terminal capability flags
 Analyze terminal environment variables and settings
 Read terminal response with timeout
 Find closest 256-color index for RGB color
 Calculate Manhattan distance between colors

**Purpose:** Comprehensive terminal color capability diagnostics and troubleshooting

**Crates:** `crossterm`

**Type:** Program

**Categories:** color, debugging, diagnosis, terminal

**Link:** [terminal_color_diagnostics.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_color_diagnostics.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_color_diagnostics.rs
```

---

### Script: terminal_light.rs

**Description:**  Demo of `terminal_light`, a crate that "answers the question "Is the terminal dark
 or light?".

**Purpose:** Demo terminal-light interrogating the background color. Results will vary with OS and terminal type.

**Crates:** `terminal_light`

**Type:** Snippet

**Categories:** crates

**Link:** [terminal_light.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_light.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_light.rs
```

---

### Script: terminal_light_fading.rs

**Description:**  A fun published example from the `terminal-light` crate. "Demonstrate mixing
 any ANSI color with the background."

**Purpose:** Mostly recreational.

**Crates:** `coolor`, `crossterm`, `terminal_light`

**Type:** Program

**Categories:** crates, recreational

**Link:** [terminal_light_fading.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_light_fading.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_light_fading.rs
```

---

### Script: terminal_light_skins.rs

**Description:**  A published example from the `terminal-light` crate. A simple example of
 choosing an appropriate skin based on the terminal theme.

**Purpose:** Demo of the `terminal-light` crate.

**Crates:** `crossterm`, `terminal_light`

**Type:** Program

**Categories:** crates

**Link:** [terminal_light_skins.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_light_skins.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_light_skins.rs
```

---

### Script: terminal_reset_test.rs

**Description:**  Demo script to test terminal reset functionality for OSC sequence corruption
 Soft terminal reset - attempts to restore normal terminal behavior
 Hard terminal reset - more aggressive reset
 Simulate the problematic OSC sequence output that causes corruption
 Test if terminal state is corrupted by checking cursor position
 Demonstrate detection of terminal corruption
 Main demo function

**Purpose:** Test terminal reset when OSC sequences cause line discipline corruption

**Type:** Program

**Categories:** debugging, terminal, xterm

**Link:** [terminal_reset_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_reset_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/terminal_reset_test.rs
```

---

### Script: test_all_formats_with_konsole.rs

**Description:**  Demo script to test all export formats including the new Konsole exporter

**Purpose:** Test all available theme export formats to verify Konsole integration

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, terminal, testing, theming

**Link:** [test_all_formats_with_konsole.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_all_formats_with_konsole.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_all_formats_with_konsole.rs
```

---

### Script: test_ansi_parsing_logic.rs

**Description:**  Simple test for ANSI parsing logic

 This tests the has_ansi_code function directly to verify it correctly
 distinguishes between color codes and text attributes.

**Purpose:** Test ANSI parsing logic for text attributes

**Crates:** `thag_styling`

**Type:** Program

**Categories:** ansi, styling, testing

**Link:** [test_ansi_parsing_logic.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_ansi_parsing_logic.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_ansi_parsing_logic.rs
```

---

### Script: test_auto_help.rs

**Description:**  This program exists to demonstrate the `thag_common` `auto_help` functionality.
 Invoking it with the argument `--help/-h` will display the doc comments as help.
 An optional `//# Purpose: ` line may be included to form the top-level help summary.
 An optional `//# Categories:` line may be used to list comma-separated categories
 to be shown at the bottom of the help screen.

**Purpose:** This is the optional `//# Purpose: ` line that becomes the help summary.

**Crates:** `thag_common`

**Type:** Program

**Categories:** demo, testing

**Link:** [test_auto_help.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_auto_help.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_auto_help.rs
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

**Link:** [test_clap_4707.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_clap_4707.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_clap_4707.rs -- --write --show-hex
```

---

### Script: test_coffee_theme.rs

**Description:**  Test script to debug color selection for the morning coffee image

 This script specifically tests the hue range assignments and fallback logic
 to understand why colors aren't matching their intended hue ranges.

**Purpose:** Debug hue range assignments in morning coffee theme generation

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [test_coffee_theme.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_coffee_theme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_coffee_theme.rs
```

---

### Script: test_color_mode_override.rs

**Description:**  Test Color Mode Override

 This script tests the THAG_COLOR_MODE environment variable override
 functionality to force specific color modes in thag_styling.
 This is particularly useful for working around terminal issues
 like Zed's RGB truecolor handling problems.

**Purpose:** Test THAG_COLOR_MODE environment variable functionality

**Crates:** `thag_styling`

**Type:** Program

**Categories:** terminal, color, testing, configuration

**Link:** [test_color_mode_override.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_color_mode_override.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_color_mode_override.rs
```

---

### Script: test_dark_theme_tuning.rs

**Description:**  Dark theme tuning previewer - shows fine-tuning effects optimized for dark themes

 This script demonstrates fine-tuning controls with parameter ranges
 optimized for dark theme generation. For light themes, use test_light_theme_tuning.rs

**Purpose:** Preview and tune dark theme generation with optimized parameters

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [test_dark_theme_tuning.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_dark_theme_tuning.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_dark_theme_tuning.rs
```

---

### Script: test_dynamic_ansi_generation.rs

**Description:**  Test dynamic ANSI generation for different color support levels

 This script verifies that our new dynamic ANSI generation approach
 correctly adapts ANSI escape sequences based on terminal color support.

**Purpose:** Test and demonstrate dynamic ANSI generation for different terminal capabilities

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, testing

**Link:** [test_dynamic_ansi_generation.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_dynamic_ansi_generation.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_dynamic_ansi_generation.rs
```

---

### Script: test_filename_handling.rs

**Description:**  Quick test to verify filename handling for different formats

**Purpose:** Test filename handling

**Crates:** `thag_styling`

**Type:** Program

**Categories:** file_handling, testing

**Link:** [test_filename_handling.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_filename_handling.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_filename_handling.rs
```

---

### Script: test_fine_tuning.rs

**Description:**  Test script to demonstrate fine-tuning controls for theme generation

 This script shows how to use the saturation multiplier, lightness adjustment,
 and contrast multiplier to fine-tune generated themes.

**Purpose:** Demonstrate fine-tuning controls for image theme generation

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [test_fine_tuning.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_fine_tuning.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_fine_tuning.rs
```

---

### Script: test_image_theme_contrast.rs

**Description:**  Test script to demonstrate improved contrast in image theme generation

 This script shows the enhanced contrast adjustment functionality
 with minimum lightness differences for better readability.

**Purpose:** Demonstrate improved contrast in image theme generation

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [test_image_theme_contrast.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_image_theme_contrast.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_image_theme_contrast.rs
```

---

### Script: test_konsole_export.rs

**Description:**  Test script for Konsole theme export functionality

**Purpose:** Test the Konsole colorscheme exporter with Catppuccin Mocha theme

**Crates:** `thag_styling`

**Type:** Program

**Categories:** terminal, testing, theming

**Link:** [test_konsole_export.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_konsole_export.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_konsole_export.rs
```

---

### Script: test_light_extreme.rs

**Description:**  Extreme parameter test for light themes to verify fine-tuning is working

 This script tests light theme generation with extreme parameter differences
 to see if the fine-tuning system is actually responsive.

**Purpose:** Test extreme light theme parameter differences

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [test_light_extreme.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_light_extreme.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_light_extreme.rs
```

---

### Script: test_light_theme_tuning.rs

**Description:**  Light theme tuning previewer - shows fine-tuning effects optimized for light themes

 This script demonstrates fine-tuning controls with parameter ranges
 optimized for light theme generation. For dark themes, use test_dark_theme_tuning.rs

**Purpose:** Preview and tune light theme generation with optimized parameters

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, tools

**Link:** [test_light_theme_tuning.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_light_theme_tuning.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_light_theme_tuning.rs
```

---

### Script: test_mintty_comprehensive.rs

**Description:**  Comprehensive test for all mintty functionality

 This demo script thoroughly tests the mintty theme exporter functionality
 including exporting themes, validating output format, and checking integration
 with the theme generation system.

**Purpose:** Comprehensive test of mintty theme functionality

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, demo

**Link:** [test_mintty_comprehensive.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_mintty_comprehensive.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_mintty_comprehensive.rs
```

---

### Script: test_mintty_export.rs

**Description:**  Test script for mintty theme exporter

 This demo script tests the mintty theme exporter functionality
 by loading a built-in theme and exporting it to mintty format.

**Purpose:** Test mintty theme export functionality

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, demo

**Link:** [test_mintty_export.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_mintty_export.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_mintty_export.rs
```

---

### Script: test_mintty_in_gen.rs

**Description:**  Test that mintty format is included in theme generator

 This demo script tests that the mintty exporter is properly integrated
 into the theme generation system by checking if it's in the list of formats.

**Purpose:** Test mintty format integration in theme generator

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, theming, demo

**Link:** [test_mintty_in_gen.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_mintty_in_gen.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_mintty_in_gen.rs
```

---

### Script: test_original_color_issue.rs

**Description:**  Test to verify the original color issue is fixed

 This script simulates the original problem where TrueColor themes
 would generate inappropriate ANSI codes for terminals with limited
 color support, and verifies that our dynamic ANSI generation fix works.

**Purpose:** Test and verify the fix for the original TrueColor/256-color compatibility issue

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, styling, terminal, testing, debugging

**Link:** [test_original_color_issue.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_original_color_issue.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_original_color_issue.rs
```

---

### Script: test_palette_optimization.rs

**Description:**  Test script to verify the palette optimization changes

 This script demonstrates the new roles (Link, Quote, Commentary) that replaced
 the old Trace role, ensuring the perfect 1:1 mapping with 16-color terminal palette.


**Purpose:** Test and demonstrate the palette optimization changes

**Crates:** `thag_styling`

**Type:** Program

**Categories:** styling, testing

**Link:** [test_palette_optimization.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_palette_optimization.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_palette_optimization.rs
```

---

### Script: test_proc_macro_examples.rs

**Description:**  Test script to validate that proc macro examples work correctly.
 This script runs a selection of proc macro examples to ensure they compile and execute properly.

**Purpose:** Test proc macro examples to ensure they work correctly

**Crates:** `thag_rs`

**Type:** Program

**Categories:** proc_macros, testing, tools

**Link:** [test_proc_macro_examples.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_proc_macro_examples.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_proc_macro_examples.rs
```

---

### Script: test_profile_extract_timestamp.rs

**Description:**  Demo of the generate_tests function-like macro for automatic test generation.

 This macro demonstrates repetitive code generation patterns by creating multiple
 test functions from a list of test data. It reduces boilerplate in test suites
 and shows how macros can automate common development tasks.

 Note that the expansion is not picked up by `cargo expand`, for reasons unknown.
 To compensate, the proc macro `generate_tests` prints the test source to `stderr`.

 Also, the expansions of the individual `generate_tests!` invocations are visible
 if the `expand` argument of the call to fn `maybe_expand_proc_macro` from the proc
 macro function fn `generate_tests` in `lib.rs` iis set to `true`. So if you prefer
 to use this, you can remove the hard-coded debugging from `generate_tests.rs`.

 To perform the tests and see the results, simply run:

 ```bash
 thag demo/proc_macro_generate_tests.rs --testing   # Short form: -T

 ```

 # Alternatively: you can run the tests via `thag_cargo`. Choose the script and the `test` subcommand.

 See also: `demo/proc_macro_generate_tests.rs`

**Purpose:** Demonstrate automatic test case generation from data

**Crates:** `thag_demo_proc_macros`, `thag_profiler`

**Type:** Snippet

**Categories:** technique, proc_macros, function_like_macros, testing, automation

**Link:** [test_profile_extract_timestamp.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_profile_extract_timestamp.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_profile_extract_timestamp.rs
```

---

### Script: test_profiler_demo_viz.rs

**Description:**  Test script to verify the profiler demo visualization feature works
 Work in progress.

**Purpose:** test if the error exists, then periodically to see if it persists.

**Crates:** `thag_profiler`

**Type:** Program

**Categories:** profiling, testing

**Link:** [test_profiler_demo_viz.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_profiler_demo_viz.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_profiler_demo_viz.rs
```

---

### Script: test_runtime_theme_loading.rs

**Description:**  Demo script that tests the new runtime theme loading functionality.

 This script demonstrates:
 1. Loading themes from user-specified directories via config
 2. Loading themes via THAG_THEME_DIR environment variable
 3. Fallback to built-in themes when user themes aren't found
 4. Proper error handling for missing directories/themes

**Purpose:** Test runtime theme loading from user-specified directories

**Crates:** `thag_common`, `thag_styling`

**Type:** Program

**Categories:** demo, styling, theming

**Link:** [test_runtime_theme_loading.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_runtime_theme_loading.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_runtime_theme_loading.rs
```

---

### Script: test_safe_print_macros.rs

**Description:**  Demo script to test the new safe print macros for terminal synchronization
 Test basic safe print functionality
 Test OSC sequences with safe_osc macro
 Test concurrent safe prints (the main use case)
 Test mixing safe and unsafe prints (demonstration)
 Test error output scenarios
 Demonstrate unit test usage pattern

**Purpose:** Test safe print macros that prevent terminal corruption in concurrent environments

**Crates:** `thag_proc_macros`

**Type:** Program

**Categories:** terminal, testing, macros, synchronization

**Link:** [test_safe_print_macros.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_safe_print_macros.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_safe_print_macros.rs
```

---

### Script: test_shared_cache.rs

**Description:**  A simple demo script to test the new shared target directory and executable cache.

 This script uses `serde_json` as a dependency to verify that:
 1. Dependencies are compiled once and shared across all scripts
 2. The executable is cached in the executable cache directory
 3. Subsequent runs are fast due to warm cache

 Run with: `thag demo/test_shared_cache.rs`
 Then run again to see the speed improvement from caching.
 Clean cache with: `thag --clean` or `thag --clean bins`

**Purpose:** Test shared target directory and executable cache functionality

**Crates:** `serde_json`

**Type:** Program

**Categories:** testing, demo

**Link:** [test_shared_cache.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_shared_cache.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_shared_cache.rs
```

---

### Script: test_sprtln.rs

**Description:**  Test script to verify sprtln macro works with both Style and Role

**Purpose:** Testing

**Crates:** `thag_styling`

**Type:** Program

**Categories:** macros, styling, technique, testing

**Link:** [test_sprtln.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_sprtln.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_sprtln.rs
```

---

### Script: test_styled_simple.rs

**Description:**  Simple test for enhanced styled! macro

 Tests the four color formats: Basic ANSI, 256-color, RGB, and Hex

**Purpose:** Simple test of enhanced styled! macro with all color formats

**Crates:** `thag_proc_macros`

**Type:** Program

**Categories:** color, macros, styling, testing

**Link:** [test_styled_simple.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_styled_simple.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_styled_simple.rs
```

---

### Script: test_urgency_hierarchy.rs

**Description:**  Urgency Hierarchy Demonstration

 This script demonstrates the new urgency-based ANSI color hierarchy where
 bright colors are used for the most critical/urgent messages, following
 established ANSI safety color standards and terminal application conventions.


**Purpose:** Demonstrate the urgency-based color hierarchy in terminal output

**Crates:** `thag_styling`

**Type:** Program

**Categories:** demo, styling, testing

**Link:** [test_urgency_hierarchy.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_urgency_hierarchy.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_urgency_hierarchy.rs
```

---

### Script: test_verbosity_setting.rs

**Description:**  Test the new verbosity setting functionality

**Purpose:** Demonstrate and test the improved verbosity setting API

**Crates:** `thag_rs`

**Type:** Program

**Categories:** debugging, testing

**Link:** [test_verbosity_setting.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/test_verbosity_setting.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/test_verbosity_setting.rs
```

---

### Script: thag_async_benchmark.rs

**Description:**  Focused async benchmark comparing tokio vs smol memory profiling with `thag_profiler` vs `dhat-rs`.
 Tests async runtime overhead and task spawning memory usage.

 # Test with tokio + thag_profiler
 THAG_PROFILER=memory,,announce,true thag --features 'full_profiling,tokio-runtime' demo/thag_async_benchmark.rs -tfm

 # Test with tokio + dhat
 thag --features 'dhat-heap,tokio-runtime' demo/thag_async_benchmark.rs -tfm

 # Test with smol + thag_profiler
 THAG_PROFILER=memory,,announce,true thag --features 'full_profiling,smol-runtime' demo/thag_async_benchmark.rs -tfm

 # Test with smol + dhat
 thag --features 'dhat-heap,smol-runtime' demo/thag_async_benchmark.rs -tfm

**Purpose:** Validate async memory profiling accuracy across different runtimes

**Crates:** `dhat`, `smol`, `thag_profiler`, `tokio`

**Type:** Program

**Categories:** async, benchmark, profiling

**Link:** [thag_async_benchmark.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_async_benchmark.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_async_benchmark.rs
```

---

### Script: thag_auto_example.rs

**Description:**  Example script demonstrating proper thag-auto usage.
 This shows how to use the thag-auto keyword for automatic dependency resolution.

 The thag-auto system allows scripts to work in different environments:
 - Development: Uses local path when THAG_DEV_PATH is set
 - Git: Uses git repository when THAG_GIT_REF is set
 - Default: Uses crates.io versions (may require published versions)

 If you get a "version not found" error, it means the specified version
 doesn't exist on crates.io yet. Set THAG_DEV_PATH or THAG_GIT_REF to use
 local or git versions instead.

**Purpose:** Demonstrate thag-auto dependency resolution system

**Type:** Program

**Categories:** demo, documentation

**Link:** [thag_auto_example.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_auto_example.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_auto_example.rs
```

---

### Script: thag_convert_themes_6422dbc.rs

**Description:**  Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.

 ## Usage examples:

 ### Convert a single theme

 ```Rust
 thag_convert_themes -i themes/wezterm/atelier_seaside_light.yaml -o themes/converted
 ```

 ### Convert a directory of themes (verbosely)

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -v
 ```

 ### Convert and also generate 256-color versions (verbosely)

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -c -v
 ```

 ### Force overwrite existing themes

 ```Rust
 thag_convert_themes -i themes/wezterm -o themes/converted -f
 ```


**Purpose:** Theme generation.

**Crates:** `clap`, `serde`, `serde_yaml_ok`, `thag_styling`, `toml`

**Type:** Program

**Categories:** tools

**Link:** [thag_convert_themes_6422dbc.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_convert_themes_6422dbc.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_convert_themes_6422dbc.rs
```

---

### Script: thag_profile_benchmark.rs

**Description:**  Benchmark comparison between thag_profiler and dhat-rs for memory profiling accuracy.
 This creates known allocation patterns and compares the results from both profilers.

**Purpose:** Validate thag_profiler accuracy against dhat-rs reference implementation

**Crates:** `dhat`, `thag_profiler`

**Type:** Program

**Categories:** benchmark, profiling

**Link:** [thag_profile_benchmark.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_profile_benchmark.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_profile_benchmark.rs
```

---

### Script: thag_prompt.rs

**Description:**  Early prototype of a front-end prompt for `thag`.

**Purpose:** Ultimately, to provide a prompt-driven front-end to the `thag` command.

**Crates:** `inquire`, `thag_styling`

**Type:** Program

**Categories:** prototype, thag_front_ends, tools

**Link:** [thag_prompt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_prompt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_prompt.rs
```

---

### Script: thag_styling_color_test.rs

**Description:**  Thag Styling Color Output Test

 This script tests what thag_styling actually outputs when THAG_COLOR_MODE
 is set. Unlike the diagnostic comparison scripts, this shows the real
 escape sequences that thag_styling generates based on the detected
 color support mode.

**Purpose:** Test actual thag_styling color output with THAG_COLOR_MODE environment variable

**Crates:** `thag_styling`

**Type:** Program

**Categories:** terminal, color, testing, styling

**Link:** [thag_styling_color_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_styling_color_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_styling_color_test.rs
```

---

### Script: thag_validate_precise.rs

**Description:**  Precise validation test with exactly known allocation sizes to verify
 thag_profiler accuracy and understand differences with dhat-rs.

**Purpose:** Validate profiler accuracy with precisely measurable allocations

**Crates:** `dhat`, `thag_profiler`

**Type:** Program

**Categories:** profiling, testing

**Link:** [thag_validate_precise.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_validate_precise.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/thag_validate_precise.rs
```

---

### Script: theme_color_mapping_comparison.rs

**Description:**  Theme Color Mapping Comparison Tool

 This tool shows exactly how the source thag-vibrant-dark theme colors
 map to the exported Alacritty format, helping debug color differences.
 Display the source theme's semantic colors with RGB values
 Show the mapping logic from semantic to ANSI colors
 Extract RGB values from a style
 Extract RGB information from a style for display

**Purpose:** Test color mapping.

**Crates:** `thag_styling`

**Type:** Program

**Categories:** color, testing, theming

**Link:** [theme_color_mapping_comparison.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_color_mapping_comparison.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_color_mapping_comparison.rs
```

---

### Script: theme_dracula_dark.rs

**Description:**  Prototype of styling with Dracula theme colours.

**Purpose:** Investigate incorporating popular themes into styling.

**Type:** Program

**Categories:** technique

**Link:** [theme_dracula_dark.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_dracula_dark.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_dracula_dark.rs
```

---

### Script: theme_gruvbox_light.rs

**Description:**  Prototype of styling with GruvBox Light theme colours.

**Purpose:** Investigate incorporating popular themes into styling.

**Type:** Program

**Categories:** technique

**Link:** [theme_gruvbox_light.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_gruvbox_light.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_gruvbox_light.rs
```

---

### Script: theme_gruvbox_light_hard.rs

**Description:**  Prototype of styling with GruvBox Light Hard theme colours.

**Purpose:** Investigate incorporating popular themes into styling.

**Type:** Program

**Categories:** technique

**Link:** [theme_gruvbox_light_hard.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_gruvbox_light_hard.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/theme_gruvbox_light_hard.rs
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

**Link:** [time_cookbook.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/time_cookbook.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/time_cookbook.rs
```

---

### Script: tlborm_callbacks.rs

**Description:**  `Callbacks` example from `The Little Book of Rust Macros`

**Purpose:** Demo macro callbacks.

**Type:** Program

**Categories:** learning, technique

**Link:** [tlborm_callbacks.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tlborm_callbacks.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/tlborm_callbacks.rs
```

---

### Script: to_relative_path.rs

**Description:**  ChatGPT 4.1-generated script expresses an absolute path relative to the current working directory.

**Purpose:** Use `pathdiff` crate to compute a relative path relative to the CWD.

**Crates:** `pathdiff`

**Type:** Program

**Categories:** crates, technique

**Link:** [to_relative_path.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/to_relative_path.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/to_relative_path.rs
```

---

### Script: tokio_hello_short.rs

**Description:**  Published example from `tokio` crate, with comments removed to work with `thag_rs` `repl` feature.
 Before running, start a background server: `ncat -l 6142 &`.

**Purpose:** Demo running `tokio` from `thag_rs`.

**Crates:** `tokio`

**Type:** Program

**Categories:** async, learning, technique

**Link:** [tokio_hello_short.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tokio_hello_short.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/tokio_hello_short.rs
```

---

### Script: tokio_hello_world.rs

**Description:**  Published example from `tokio` crate. Before running, start a server: `ncat -l 6142`
 in another terminal, or simply `ncat -l 6142 &` in the same terminal.

**Purpose:** Demo running `tokio` from `thag_rs`.

**Crates:** `tokio`

**Type:** Program

**Categories:** async, learning, technique

**Link:** [tokio_hello_world.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tokio_hello_world.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/tokio_hello_world.rs
```

---

### Script: truecolor_quantization_test.rs

**Description:**  TrueColor Quantization Detection Test

 This test detects whether a terminal silently quantizes TrueColor values
 to a 256-color palette, as suspected with Apple Terminal. The strategy:

 1. Test colors that should be identical in TrueColor but different in 256-color
 2. Test colors that fall between 256-color palette entries
 3. Use statistical analysis of multiple color tests
 4. Compare expected vs actual color distances

 If the terminal silently quantizes, we'll see:
 - Colors that should be different become identical
 - Systematic rounding to 256-color palette values
 - Loss of precision in color gradients
 RGB color representation
 Test result for a single color
 Parse hex component from OSC response
 Detect if we're running in mintty (which always supports TrueColor)
 Parse OSC 10 response
 Set and query a color with timing (supports mintty via OSC 7704)
 Convert RGB to nearest 256-color palette equivalent
 Generate test colors that reveal quantization

**Purpose:** Detect silent TrueColor quantization in terminals

**Crates:** `crossterm`

**Type:** Program

**Categories:** terminal, color, testing

**Link:** [truecolor_quantization_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/truecolor_quantization_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/truecolor_quantization_test.rs
```

---

### Script: truecolor_test.rs

**Description:**  TrueColor Detection

 This script tests TrueColor detection support by sending a TrueColor escape
 sequence and querying the result, as suggested by https://github.com/termstandard/colors.

 The approach:
 1. Query current foreground color (OSC 10)
 2. Set a specific TrueColor foreground (OSC 10 with RGB)
 3. Query the foreground color again
 4. Restore original foreground color
 5. Compare set vs queried values to determine TrueColor support
 RGB color representation
 Parse hex component from OSC response
 Detect if we're running in mintty (which always supports TrueColor)
 Parse OSC 10 (foreground color) response
 Query current foreground color using OSC 10
 Set foreground color using OSC 10
 Test TrueColor support by setting and querying

**Purpose:** Test TrueColor support using OSC sequence probing

**Crates:** `crossterm`

**Type:** Program

**Categories:** ansi, color, styling, terminal, testing, theming, tools, windows, xterm

**Link:** [truecolor_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/truecolor_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/truecolor_test.rs
```

---

### Script: tui_scrollview.rs

**Description:**  Published example from `tui-scrollview` crate. Toml entries from crate's Cargo.toml.

 Not suitable for running from a URL.

**Purpose:** Explore TUI editing

**Crates:** `color_eyre`, `lipsum`, `ratatui`, `tui_scrollview`

**Type:** Program

**Categories:** crates, exploration, technique, tui

**Link:** [tui_scrollview.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tui_scrollview.rs)

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

**Categories:** crates, exploration, technique, tui

**Link:** [tui_ta_editor.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tui_ta_editor.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_editor_profile.rs

**Description:**  A version of `tui_ta_editor_profile.rs` profiled with `thag_profiler` to demonstrate
 time profiling.

 Not suitable for running from a URL.

**Purpose:** Demo `thag_profiler`.

**Crates:** `ratatui`, `thag_profiler`, `tui_textarea`

**Type:** Program

**Categories:** crates, profiling, technique, tui

**Link:** [tui_ta_editor_profile.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tui_ta_editor_profile.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_minimal.rs

**Description:**  Demo a very minimal and not very useful TUI (text user interface) editor based on the featured crates.

 Not suitable for running from a URL.

**Purpose:** Demo TUI editor and featured crates, including `crossterm`, and the use of the `scopeguard` crate to reset the terminal when it goes out of scope.

**Crates:** `ratatui`, `scopeguard`, `tui_textarea`

**Type:** Program

**Categories:** crates, exploration, technique, tui

**Link:** [tui_ta_minimal.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tui_ta_minimal.rs)

**Not suitable to be run from a URL.**


---

### Script: tui_ta_vim.rs

**Description:**  Published basic `vim` editor example from crate `tui-textarea`. Mildly tweaked
 to use `ratatui::crossterm` re-exports instead of `crossterm` directly.

 Not suitable for running from a URL.

**Purpose:** Demo TUI `vim` editor and featured crates, including `crossterm`.

**Crates:** `ratatui`, `tui_textarea`

**Type:** Program

**Categories:** crates, tui

**Link:** [tui_ta_vim.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tui_ta_vim.rs)

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

**Categories:** async, crates, learning, exploration, technique, tui

**Link:** [tui_tokio_editor_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/tui_tokio_editor_gpt.rs)

**Not suitable to be run from a URL.**


---

### Script: type_of_at_compile_time_1.rs

**Description:**  Use a trait to determine the type of an expression at compile time, provided all cases are known in advance.

 This is a slightly embellished version of user `phicr`'s answer on `https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable-in-rust`.

 See also `demo/type_of_at_compile_time_2.rs` for an alternative implementation.

**Purpose:** Demo expression type determination for static dispatch.

**Type:** Program

**Categories:** type_identification, technique

**Link:** [type_of_at_compile_time_1.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/type_of_at_compile_time_1.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/type_of_at_compile_time_1.rs
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

**Link:** [type_of_at_compile_time_2.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/type_of_at_compile_time_2.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/type_of_at_compile_time_2.rs
```

---

### Script: type_of_at_run_time.rs

**Description:**  Typical basic (runtime) solution to expression type identification. See also `demo/determine_if_known_type_trait.rs`
 for what may be a better (compile-time) solution depending on your use case.

**Purpose:** Demo of runtime type identification.

**Crates:** `quote`, `syn`

**Type:** Program

**Categories:** type_identification, technique

**Link:** [type_of_at_run_time.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/type_of_at_run_time.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/type_of_at_run_time.rs
```

---

### Script: ubig_product_gpt.rs

**Description:**  Implement trait std::iter::Product for `ibig::UBig`. Example provided by GPT.

**Purpose:** Learning / reference.

**Crates:** `ibig`

**Type:** Program

**Categories:** big_numbers, learning, reference, technique

**Link:** [ubig_product_gpt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/ubig_product_gpt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/ubig_product_gpt.rs
```

---

### Script: unzip.rs

**Description:**  Very simple demo of the `unzip` iterator function.

**Purpose:** Demo

**Type:** Snippet

**Categories:** technique

**Link:** [unzip.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/unzip.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/unzip.rs
```

---

### Script: visual_rgb_test.rs

**Description:**  Visual RGB Rendering Test

 This script tests whether terminals actually render RGB truecolor sequences correctly
 by displaying them side-by-side with their closest 256-color palette equivalents.
 This helps detect terminals that accept RGB sequences but render them incorrectly
 (like Apple Terminal showing salmon pink instead of duck-egg blue-green).
 Test colors specifically chosen to reveal rendering issues
 Colors designed to reveal different types of rendering issues
 Find the closest 256-color palette index for an RGB color
 Calculate Manhattan distance between two RGB colors

**Purpose:** Visual test to detect accurate RGB truecolor rendering vs palette quantization

**Type:** Program

**Categories:** color, diagnosis, terminal, testing

**Link:** [visual_rgb_test.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/visual_rgb_test.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/visual_rgb_test.rs
```

---

### Script: warn_once.rs

**Description:**  This script demonstrates the usage of the `warn_once` pattern for suppressing repeated
 log messages with minimal runtime overhead.

 The dependency is `thag_profiler` because that's the only place it's used at time of writing, even though this is
 not in any way a profiling-specific function.

 Disclosure: the `thag_profiler` `warn_once` macro uses unsafe code.

 Credit to `Claude 3.7 Sonnet`.
 Simple example that shows a warning only once despite multiple calls
 Example with early return pattern
 Example using multiple warn_once! calls for different conditions
 Performance comparison between naive approach and warn_once
 Real-world example based on the record_dealloc function
 Main entry point

**Purpose:** Demo a macro I found useful, explained and benchmarked here in great detail thanks to Claude.

**Crates:** `thag_profiler`

**Type:** Program

**Categories:** demo, macros, technique

**Link:** [warn_once.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/warn_once.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/warn_once.rs
```

---

### Script: warn_once_with_id_standalone.rs

**Description:**  This script demonstrates the usage of the `warn_once_with_id` function for suppressing repeated
 log messages with minimal runtime overhead using unique IDs.

 This is a standalone implementation that doesn't require any external dependencies.
 The function uses unsafe code for maximum performance with a fast path after the first warning.

 Credit to `Claude Sonnet 4` for the implementation and comprehensive demo.
 Standalone implementation of warn_once_with_id function

 This function provides a high-performance way to suppress repeated warnings
 using unique IDs to track different warning sites independently.

 # Safety

 This function is unsafe because:
 - It uses static mutable data with UnsafeCell
 - Caller must ensure each ID is unique per call site
 - The ID should be < 128 for optimal performance (higher IDs use modulo)

 # Arguments

 * `id` - Unique identifier for this warning site (0-127 for best performance)
 * `condition` - Whether the warning condition is met
 * `warning_fn` - Closure to execute for the warning (called only once)

 # Returns

 * `true` if the condition was met (regardless of whether warning was shown)
 * `false` if the condition was not met

 # Example

 ```rust
 const MY_WARNING_ID: usize = 1;

 unsafe {
     warn_once_with_id(MY_WARNING_ID, some_error_condition, || {
         eprintln!("This warning will only appear once!");
     });
 }
 ```
 Demo showing multiple independent warnings with different IDs
 Demo showing performance characteristics
 Demo showing thread safety
 Demo showing real-world usage patterns
 Demo showing ID collision handling
 Demo showing early return pattern

**Purpose:** Standalone demo of warn_once_with_id function with embedded implementation

**Type:** Program

**Categories:** demo, macros, technique, unsafe, performance

**Link:** [warn_once_with_id_standalone.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/warn_once_with_id_standalone.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/warn_once_with_id_standalone.rs
```

---

### Script: web_safe_colors_to_256.rs

**Description:**  Map and visually test conversion of web safe colours to 256 colours, //: using the `owo-colors` crate colour names and mappings.

**Purpose:** Work out and test colour conversion.

**Crates:** `itertools`, `owo_colors`

**Type:** Program

**Categories:** demo, reference, testing

**Link:** [web_safe_colors_to_256.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/web_safe_colors_to_256.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/web_safe_colors_to_256.rs
```

---

### Script: win_test_control.rs

**Description:**  This is the "control" test for the `demo/win_test_*.rs` scripts. It seems to reliably NOT swallow the first character.

**Purpose:** Show how crates *not* sending an OSC to the terminal in Windows will *not* the first character you enter to be swallowed.

**Type:** Program

**Categories:** testing

**Link:** [win_test_control.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_control.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_control.rs
```

---

### Script: win_test_supports_color.rs

**Description:**  This seems to intermittently swallow the very first character entered in Windows, prior to `termbg` 0.6.0.

**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `supports_color`

**Type:** Program

**Categories:** testing

**Link:** [win_test_supports_color.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_supports_color.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_supports_color.rs
```

---

### Script: win_test_termbg.rs

**Description:**  This seems to "reliably" swallow the very first character entered in Windows, prior to `termbg` 0.6.0.

**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `termbg`

**Type:** Program

**Categories:** testing

**Link:** [win_test_termbg.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_termbg.rs)

**Not suitable to be run from a URL.**


---

### Script: win_test_terminal_light.rs

**Description:**  This seems to "reliably" swallow the very first character entered in Window, prior to `termbg` 0.6.0..

**Purpose:** Show how crates sending an OSC to the terminal in Windows will not get a response and will unintentionally "steal" your first character instead.

**Crates:** `terminal_light`

**Type:** Program

**Categories:** testing

**Link:** [win_test_terminal_light.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_terminal_light.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_terminal_light.rs
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
 had a PR accepted into the `termbg` crate as v0.6.1. This should substantially address the issue
 although I have not yet managed to overcome an occasional outbreak rightward march in any given
 environment. The only fix I know for this is a completely new terminal session, but
 Ensure the following is present as a dependency in the toml block or defaulted in your configuration
 file (for the Windows builds this is intended for):
 (`thag - C`): `winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }`

**Purpose:** Debug `termbg`

**Crates:** `crossterm`, `winapi`

**Type:** Program

**Categories:** testing

**Link:** [win_test_vt.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_vt.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/win_test_vt.rs
```

---

### Script: windows_detect_powershell.rs

**Description:**  Prototype of PowerShell detection

**Purpose:** Detect if we're running under PowerShell.

**Crates:** `sysinfo`

**Type:** Program

**Categories:** detection, terminal, testing, windows

**Link:** [windows_detect_powershell.rs](https://github.com/durbanlegend/thag_rs/blob/main/demo/windows_detect_powershell.rs)

**Run this example:**

```bash
thag_url https://github.com/durbanlegend/thag_rs/blob/main/demo/windows_detect_powershell.rs
```

---

