# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - 2024-09-05

# v0.1.2..HEAD (2024-09-05)

- [Update demo dependencies: thag and reedline](https://github.com/durbanlegend/thag_rs/commit/211308f074d39cb512ac75f93cf0bb9f59a0ee9b)
- [Bump quote from 1.0.36 to 1.0.37](https://github.com/durbanlegend/thag_rs/commit/886797263559388b054c9759d430a2406987c47f)
- [Bump ratatui from 0.28.0 to 0.28.1](https://github.com/durbanlegend/thag_rs/commit/09866004157aafd5c447f5e8511f1f3219e0daa1)
- [Bump syn from 2.0.75 to 2.0.76](https://github.com/durbanlegend/thag_rs/commit/b398acba29c78e1c29ad3e1d1667be6985e2cf94)
- [Bump serde_json from 1.0.125 to 1.0.127](https://github.com/durbanlegend/thag_rs/commit/7e4af9dcd53337f00a80e1cd424e4fdec70dc90b)
- [Bump serde from 1.0.208 to 1.0.209](https://github.com/durbanlegend/thag_rs/commit/90d9de950a0456c254c12bd3443a05efaf1adcc2)
- [Add new CLI args and reorganise.](https://github.com/durbanlegend/thag_rs/commit/7c1868539b0f36370697e6e65fa6e95941becb91)
- [Revert "Build out profiling instrumentation, add feature"](https://github.com/durbanlegend/thag_rs/commit/cf362166bd834e3015253926d837259ed2b34467)
- [Merge pull request #39 from durbanlegend/dependabot/cargo/serde-1.0.209](https://github.com/durbanlegend/thag_rs/commit/1d215ce972cf1e85355281011cc9dbfbafa62cae)
- [Merge pull request #38 from durbanlegend/dependabot/cargo/serde_json-1.0.127](https://github.com/durbanlegend/thag_rs/commit/676a8c037fe9b9b1c8c0f1b35db7c5ff04a54878)
- [Merge pull request #37 from durbanlegend/dependabot/cargo/syn-2.0.76](https://github.com/durbanlegend/thag_rs/commit/dae6aed1dc3eb562485864dd959b96738db46ac0)
- [Merge pull request #36 from durbanlegend/dependabot/cargo/ratatui-0.28.1](https://github.com/durbanlegend/thag_rs/commit/376300c688c6e2c08113991aae88a6e54676278e)
- [Merge pull request #35 from durbanlegend/dependabot/cargo/quote-1.0.37](https://github.com/durbanlegend/thag_rs/commit/69d1a962a009813331d8426695c253f8bee9bff0)
- [Instrument code_utils, #[cfg(debug_assertions)]](https://github.com/durbanlegend/thag_rs/commit/a7279169b56be0d939c1e2281de17c06df724356)
- [Debugging ci.yml](https://github.com/durbanlegend/thag_rs/commit/c0f8bcd1a2f77914948238cc52456cd89fb169e3)
- [Attack on the clone()s: optimisations](https://github.com/durbanlegend/thag_rs/commit/ee1ac6322722720746746afb9a17db722283581e)
- [Attack on the clone()s: optimisations](https://github.com/durbanlegend/thag_rs/commit/8ad16edf00f102bf5001d3de1330f6bf9c32aabf)
- [Update shared.rs](https://github.com/durbanlegend/thag_rs/commit/ff58a00c99263a906d1553f39b1da5778a1ac63d)
- [Update colors.rs](https://github.com/durbanlegend/thag_rs/commit/22b12523d6d6a5d8ff68d575788e0310b2975762)
- [The only thing we have to fear is panic.](https://github.com/durbanlegend/thag_rs/commit/84e96712f170e46162818529c68d5c1151f42e06)
- [Lower your expect()ations - of the i/o subsystem](https://github.com/durbanlegend/thag_rs/commit/69db9d7c50061704a685969015e31e3de7653c8d)
- [Thag see error of his ways](https://github.com/durbanlegend/thag_rs/commit/e447ed7a89a599a059666ce0cbdfde1f1f69b3cd)
- [No expectations](https://github.com/durbanlegend/thag_rs/commit/37cf401472b73d306c8cb821c738d5365e5b14cb)
- [Build program files directly from original source.](https://github.com/durbanlegend/thag_rs/commit/d2705a8ddcdd0459525bcc9fd1d0766d73461fcd)
- [Bump syn from 2.0.76 to 2.0.77](https://github.com/durbanlegend/thag_rs/commit/01d0a86dcb31a93791ef2d109f6bd4d0a0285e83)
- [Merge pull request #40 from durbanlegend/dependabot/cargo/syn-2.0.77](https://github.com/durbanlegend/thag_rs/commit/2c0e9a2a065c499be84e82ebdf53061cdae60686)
- [Fix colors from Windows testing](https://github.com/durbanlegend/thag_rs/commit/7784891eddca4e46d977508fa93200b9b118829a)

### Notes

Upgrade manifest processing to allow arbitrary valid Cargo.toml such as profiles to be specified in toml block.
  (Catch up with inadventent over-promise in Reaadme.)
Add config.toml template and add edit config.toml option --config (-C) to CLI, renaming --cargo (-C) to --toml (-T).
Add --unquote (-u) true/false option to CLI and add user default setting to config.toml. `true` will strip quotes from string
    values returned implicitly by a snippet, false (default) will retain them. `=` sign is optional, e.g. `-u=true` and `-u true`
    are interchangeable. `--unquote` specified without a value equates to `-u true`.
    E.g. run `thag -- -e '"Hello world!"' -u` vs `thag -- -e '"Hello world!"' -u false` to see the difference.
Group related args in --help display.
Rationalise and streamline error handling and eliminate undue expects, unwraps and panics.
Run well-formed program scripts from original source for efficiency.
Reduce cloning and make other optimisations and code improvements.
Build out profiling instrumentation.
Documentation corrections and enhancements.
Demo script corrections and additions.
Bump dependencies via Dependabot.

#### Detail

- New merge_toml demo

- Workflow update cargo for demo/iced_tour.rs build error no happening locally.

- Comment out some profiler instrumentation hiding other results

- Caution: a few minor logic changes now consuming instead of cloning.
  Distinguish between choosing no colour (= None) and not having made a choice (= Default).

- Caution: a few minor logic changes now consuming instead of cloning.
  Distinguish between choosing no colour (= None) and not having made a choice (= Default).

- Windows fix from CI.

- Fix from Windows testing

- Replace inappropriate panics by error bubbling.

- Tighten up error handling by bubbling up errors, part 1.

- Create ThagError variants for all remaining error types encountered and replace Box<dyn error> with ThagError in Results

- Finish replacing expect() by ? where appropriate.

- No need to copy elsewhere if they have main and can parse to syn:File. Seems to speed things up considerably.

- Correct name of command from thag_rs to thag in demo scripts and thus gen of demo/README.md.

# v0.1.2 (2024-08-24)

- [Rename error class, start profiling.](https://github.com/durbanlegend/thag_rs/commit/3a3732935efe3adf08edf42b0203e2927b53219a)
- [Replace rustfmt by prettyprint, add changelog.](https://github.com/durbanlegend/thag_rs/commit/6f7225f1091c1df434eb59574547f420fcc99e10)

### Notes

- Prototype new cmd_args option, Readme enhancements, minor demo fixes.

- Speed and direct source formatting at the expense of comments.

## [0.1.1] - 2024-08-22

Create demo.zip and installation artifacts, minor demo script updates, e.g. replace thag git dependency with new crates.io release.

## [0.1.0] - 2024-08-22

### Manifest

- Features support. Minor enhancements

### Snippets

- Analysing return type - WIP
