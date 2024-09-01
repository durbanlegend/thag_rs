# TODO List

## High Priority
- [ ]  Clean up demos
- [ ]  Document public APIs

## Medium Priority
- [ ]  Add additional popular crates
- [ ]  More unit and integration tests
- [ ]  Document where demo subdirectory is and how to install it
- [ ]  Simple demo https server
- [ ]  >>> Decide how to distribute demo readme without demo dir, maybe in main under another name.
- [ ]  Add download demos option, see prototype demo/install_demo_gpt.rs
- [ ]  Add unquote string return values option and add to config.toml.
- [ ]  Consider releasing a copy of repl.rs as a demo script.
- [ ]  Raise clear_screen as an issue on termbg and supports-color crates?
- [ ]  Add FAQ
- [ ]  cat demo/fizz_buzz_gpt.rs | while read l; do thag_rs -qe "println!(\"{}\", \"$l\".to_uppercase());"; done
- [ ]  Add an option --config / -C to edit the config and change current --cargo / -C to --toml / -T
        or --manifest / -M
        Include an option to include or exclude quotes on returned strings.
        Maybe make it a command-line option too.
        Add option to strip symbols (default true?). LTO?
- [ ]  Config option for formatting main
- [ ]  Add conversions to and from `runner` and `cargo-script-mvs`.
- [ ]  -vv for debug mode
- [ ]  Firestorm example
- [ ]  Test [profile.dev] optimisation level
- [ ]  Check dead code & other #[cfg[allow(...)]; look into factoring over-long gen_build_run
- [ ]  Look for code smells
- [ ]  Look into fuzzing the tests such as test_merge_manifest.
- [ ]  grep png README.md | cargo run --features=debug-logs -- -v -B 'use regex::Regex; let re = Regex::new(r"\w+.png").unwrap();' -l 'println!("{}", re.find(&line).unwrap().as_str())' | sort
       grep -Eo '[a-zA-Z0-9_]+\.png' README.md
       https://download-directory.github.io/?url=https://github.com/durbanlegend/thag_rs/tree/14d31159c42249f6aa0486f500de209438b06b8f/demo
https://stackoverflow.com/questions/7106012/download-a-single-folder-or-directory-from-a-github-repository
https://test.ssgithub.com/?url=https://github.com/durbanlegend/thag_rs/tree/master/demo
- [ ]  Apply changes from demo/cmd_args.rs.
- [ ]  Document:
        /*[toml]
        [package]
        name = "dethagomizer"

        [[bin]]
        name = "dethag"
        path = "/Users/donf/projects/thag_rs/demo/dethagomizer.rs"
        */



## Low Priority
- [ ]  Paste event in Windows slow or not happening?
- [ ]  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
- [ ]  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
- [ ]  Clap aliases not working in REPL.
- [ ]  How to navigate reedline history entry by entry instead of line by line.
- [ ]  See if with...(nu_resolve_style) methods of repl.rs can maybe use a closure to prevent lazy-static from triggering prematurely. Maybe add terminal test?

## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.
- [ ]  WASM - is there a worthwhile one? - maybe Leptos if it doesn't need Node.js.
