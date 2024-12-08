# TODO List

## High Priority

## Medium Priority
- [ ]  More unit and integration tests. Identify new functions requiring unit tests.
- [ ]  Consider releasing a copy of repl.rs as a demo script.
- [ ]  Raise clear_screen as an issue on supports-color crate?
- [ ]  Config option for formatting main?
- [ ]  Config option for stdin -d highlighting preference, like repl.rs
- [ ]  Add conversions to and from `runner` and `cargo-script-mvs`.
- [ ]  Implement deletion of current history line with function key.
- [ ]  Look for any functions that can run at compile time.
- [ ]  Make key_handler a trait method? Or a closure?
- [ ]  Add FAQ? See Usage notes in Readme.
- [ ]  Try pre-building colour mappings
- [ ]  Try and figure out a way for thag_rs to test proc macros - and maybe to show the generated output?
        - could do an expand option that builds the file and calls cargo expand with --manifest-path <build_state.cargo_toml_path>
        - consider a dedicated installable thag_rs proc library - maybe configurable location specified in config.toml, with preferred theme.
            Source files need to import this with path in toml file or even allow a git option.
            Prototype this!
- [ ]  ?Adapt keys display to environment: Cmd only for target macos. Or just leave it because informative?
- [ ]  Try going back to derive macro with declarative macro to expand the attributes. Problem with attrib macro is the AST isn't
        editable in the way we want, it just points to spans that get resolved later. See bank/syn_parse_mappings.rs for nice format;
- [ ]  Decide whether to decommission REPL delete function - keep list
- [ ]  Test [profile.dev] optimisation level
- [ ]  Check dead code & other #[cfg[allow(...)]; look into factoring over-long gen_build_run
- [ ]  Look for code smells
- [ ]  Look into fuzzing the tests such as test_merge_manifest.
- [ ]  Testing fib scripts: ls -1 demo/fib_*.rs | grep -v basic | grep -v binet | while read f; do echo $f;  cargo run -- $f -qq -- 100 | grep 354224848179261915075 || echo "...failed"; done

- [ ]  Checklist for making releases:
       - Tip: disable ci.yml for Readme & similar tweaks that won't
              affect compilation.
       - Remember to update Cargo.toml version to the required release before tagging.
       - Do a trial release build locally to check for anomalies: cargo build --release --workspace
       - Don't upgrade thag versions in demo scripts to new release, because you get a
           catch-22 until it's on crates.io. If you absolutely need to, wait until you've
           released to crates.io a first time, then release all over again.
       - Optional: reinstall thag_rs from path. (cargo install --path .)
       - Make sure Readme images are up to date.
       - Run clippy::pedantic and clippy::nursery
       - Run cargo tests
       - Run `gen_readme`
       - Run `typos` command.
       - Run `vale README.md --no-wrap` and `vale demo/README.md --no-wrap`.
       - Run cargo msrv find, and update the MSRV in README.md.
       - NB NB: If there have been any changes to thag_proc_macros since its last published release, bump its version number
             in src/proc_macros/Cargo.toml and also in its dependency entry in the main Cargo.toml. as this will be used in
             the crates.io version
       - Use 'git changelog v0.1.<n-1>..HEAD' to generate raw release notes.
       - Leave it to cargo-dist to make the release.
       - To trigger cargo-dist:
            cargo dist init  # In case e.g. package description in Cargo.toml has changed.
            git tag v0.1.n -m "<Summary>"
            git push --tags
       - To revoke and redo:
            git tag -d v0.1.n
            git push origin --delete v0.1.n
            Tag again as above when ready.
       - Don't override release.yml, e.g. to try to add a workflow dispatch, as it's generated by cargo-dist.
       - Edit the release notes generated by cargo-dist on Github and add in
           own change log, edited as required from raw changelog output above.
       - Make sure to update all bank and demo scripts using thag to use latest release instead of develop branch if appropriate.
       - Reinstall thag_rs from tag. (cargo install --git https://github.com/durbanlegend/thag_rs --tag v0.1.<n>)
       - `Publishing to crates.io`
            - Suggest give it a day to settle before publishing to crates.io.
            - First publish the new version of src/proc_macros if applicable, same steps as below.
            - Before publishing, dry run installation with `cargo install --path /Users/donf/projects/thag_rs/`
            - First: `find . -name .DS_Store -delete`
            - Test with `cargo package --no-verify`
            - Publish for real: `cargo publish --no-verify`
            - Reinstall updated thag_rs with cargo install.
            - Keep develop branch around and bring it up to date with main branch changes such as version number in Cargo.toml
                  Use a temp staging branch like staging_temp, otherwise it will merge backwards into main for some reason while creating
                  the pull request.


## Low Priority
- [ ]  Debug Firestorm double invocation.
- [ ]  Add additional popular crates
- [ ]  Paste event in Windows slow or not happening?
- [ ]  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
- [ ]  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
- [ ]  Clap aliases not working in REPL.
- [ ]  How to navigate reedline history entry by entry instead of line by line.
- [ ]  See if with...(nu_resolve_style) methods of repl.rs can maybe use a closure to prevent lazy-static from triggering prematurely. Maybe add terminal test?
- [ ]  Simple demo https server
- [ ]  Trim dependencies, e.g. regex

## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.
