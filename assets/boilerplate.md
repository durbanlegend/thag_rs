## Running the scripts in `demo` and `tools`

`thag_rs` uses `clap` for a standard command-line interface. Try `thag --help` (or -h) if
you get stuck.

### In its simplest form:

  ````
  thag <path to script>`
  ````

#### E.g.:

  ````
  thag demo/hello.rs
  ````

### Passing options and arguments to a script:

Use `--` to separate options and arguments meant for the script from those meant for `thag` itself.

E.g.: `demo/fib_dashu_snippet.rs` expects to be passed an integer _n_ and will compute the _nth_ number in the Fibonacci sequence.

  ````
  thag demo/fib_dashu_snippet.rs -- 100
  ````

### Full syntax:

  ````
    thag [THAG OPTIONS] <path to script> [-- [SCRIPT OPTIONS] <script args>]
  ````

E.g.: `demo/clap_tut_builder_01.rs` is a published example from the `clap` crate.
Its command-line signature looks like this:

  ````
    clap_tut_builder_01 [OPTIONS] [name] [COMMAND]
  ````

The arguments in their short form are:

    `-c <config_file>`      an optional configuration file
    `-d` / `-dd` / `ddd`    debug, at increasing levels of verbosity
    [name]                  an optional filename
    [COMMAND]               a command (e.g. test) to run

If we were to compile `clap_tut_builder_01` as an executable (`-x` option) and then run it, we might pass
it some parameters like this:

  ````
  clap_tut_builder_01 -dd -c my.cfg my_file test -l
  ````

and get output like this:

    Value for name: my_file
    Value for config: my.cfg
    Debug mode is on
    Printing testing lists...

Running the source from `thag` looks similar, we just replace `clap_tut_builder_01` by `thag demo/clap_tut_builder_01.rs --`:

    *thag demo/clap_tut_builder_01.rs --* -dd -c my.cfg my_file test -l

Any parameters for `thag` should go before the `--`, e.g. we may choose use -qq to suppress `thag` messages:

    `thag demo/clap_tut_builder_01.rs -qq -- -dd -c my.cfg my_file test -l`

which will give identical output to the above.

#### Remember to use `--` to separate options and arguments that are intended for `thag` from those intended for the target script.
