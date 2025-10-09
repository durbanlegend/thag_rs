## Running the scripts in `demo` and `src/bin`

### Commonality of scripts and tools

The scripts in src/bin are integrated `thag_rs` tools, in other words they are declared in Cargo.toml and are normally installed as commands. However, if you have cloned the `thag_rs` project you can run them like any other `thag` script.

Conversely, you can make your own tools by using the `thag -x` option to do a release build of any script, even a snippet. You could even use `thag -xe ` to buil

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
