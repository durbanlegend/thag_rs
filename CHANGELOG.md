# Changelog

All notable changes to this project will be documented in this file.

## v0.1.5 (2024-10-20)

### Highlights

- Common TUI editor with file save dialog, status message and working history, basic mouse selection support, TUI history edit.
- Feature to promote script from REPL (-r) to TUI (with separate TUI history shared with -d option)
- Code quality improvements, e.g. From trait for message level to style conversions, ThagResult, Keys display build, regex! macro,
    clippy::nursery recommendations as well as clippy::pedantic.
- Fix termbg Windows behaviour (using custom version of termbg pending PR raised on termbg crate)
- Crokey-based key bindings (using custom version of crokey)
- simplelog option as alternative to env_logger
- Replaced lazy_static crate with standard Rust built-ins
- New demo and bank scripts
- Drop individual REPL builds as too expensive
- New ThagErrors: Logic and UnsupportedTerm
- Rename test functions to incorporate module name and thus allow filtering tests by module

- [Prepare for debug logging in release.](https://github.com/durbanlegend/thag_rs/commit/3b8174b13788ac5ea792d8a755404c566bd5317d)
- [Bump serde from 1.0.209 to 1.0.210](https://github.com/durbanlegend/thag_rs/commit/17eefba5eaf721ea064259429b9bd6d36ba3f1a0)
- [Stdin.rs in broken state](https://github.com/durbanlegend/thag_rs/commit/b157dd75bd08ae6a32a32f576653c90e1512faa4)
- [Working prototype of tui_repl.](https://github.com/durbanlegend/thag_rs/commit/fb67e54ab9e5024b053dd7906a8ae2698c82d49d)
- [Merge pull request #43 from durbanlegend/dependabot/cargo/serde-1.0.210](https://github.com/durbanlegend/thag_rs/commit/eaa921d11555db5737ba1275de194f581670971b)
- [Debug REPL edit_history & customize keys display.](https://github.com/durbanlegend/thag_rs/commit/65196dca535842c1c22fc0640d41dee2577e76c8)
- [Make tui_selection_bg configurable](https://github.com/durbanlegend/thag_rs/commit/cf44c68480398919c6f9d0245619f1d399aa8d40)
- [Debug & make Clippy::pedantic happy.](https://github.com/durbanlegend/thag_rs/commit/2da0eba64d87897a44a94e003d4da74c4bab9f62)
- [Bump termbg from 0.5.0 to 0.5.1](https://github.com/durbanlegend/thag_rs/commit/abb26fc9377bb67dcf02f414aab7b9df3db90c58)
- [Move resolve_term to tui_editor, add keybindings](https://github.com/durbanlegend/thag_rs/commit/656b8a536c8d1a02ca91c72456cd00687b354cdc)
- [Merge pull request #44 from durbanlegend/dependabot/cargo/termbg-0.5.1](https://github.com/durbanlegend/thag_rs/commit/ce978ec4377bf91859a6d21d397687acda217db4)
- [Order keys display in TUI editor.](https://github.com/durbanlegend/thag_rs/commit/7537cbc3cbba36706f3b60bcc7040529c106f841)
- [Fix TermTheme defaulting to Dark instead of termbg](https://github.com/durbanlegend/thag_rs/commit/0f44dd2a1a5e850ef73798ef8b59822a09af8c96)
- [Experiment with TUI file_dialog - inadequate.](https://github.com/durbanlegend/thag_rs/commit/3367493fdeb82a7a67b06f2a3bebcac1e48056d7)
- [Remove file_dialog.rs](https://github.com/durbanlegend/thag_rs/commit/e594b8b94ad3f6b172e6dee8f0982a2791999eae)
- [Bump reedline from 0.34.0 to 0.35.0](https://github.com/durbanlegend/thag_rs/commit/724b20d8c71c0a15376df4c11e3662044bae26bb)
- [Bump clap from 4.5.17 to 4.5.18](https://github.com/durbanlegend/thag_rs/commit/c4682c148d929a6d681adf442574126e56859a4c)
- [Merge pull request #46 from durbanlegend/dependabot/cargo/clap-4.5.18](https://github.com/durbanlegend/thag_rs/commit/9c8514ce3fdf4048f57faee2acf52221e7f5decb)
- [Merge pull request #45 from durbanlegend/dependabot/cargo/reedline-0.35.0](https://github.com/durbanlegend/thag_rs/commit/0aba9fc89121771cdcf7c475acdc34dd57e33787)
- [Incorporate termbg to reduce dependencies](https://github.com/durbanlegend/thag_rs/commit/b6d53b1c582f11808fdb8e022482223f214e8996)
- [Fix termbg side-effect making terminal misbehave.](https://github.com/durbanlegend/thag_rs/commit/a4fa26c902bb0acc629ddac15f60edfba98c16ba)
- [Debug rightward log drift](https://github.com/durbanlegend/thag_rs/commit/2d7c14219e9b01f4f99a1750ded339c072fb4eb0)
- [Bump tempfile from 3.12.0 to 3.13.0](https://github.com/durbanlegend/thag_rs/commit/7e948002462a4693fa1bf274ea7073cbd39fcac9)
- [Bump cargo_toml from 0.20.4 to 0.20.5](https://github.com/durbanlegend/thag_rs/commit/35db002d473cd81f6d0ff0c75c45945230e60458)
- [Bump regex from 1.10.6 to 1.11.0](https://github.com/durbanlegend/thag_rs/commit/9f687e096e411ab1c95f0ce5ad1050dc8d76d6ed)
- [Bump syn from 2.0.77 to 2.0.79](https://github.com/durbanlegend/thag_rs/commit/ade0fed23cc5d4083723c219dac5c932359b2c72)
- [Merge pull request #47 from durbanlegend/dependabot/cargo/tempfile-3.13.0](https://github.com/durbanlegend/thag_rs/commit/b3eaada75704f6838560fcdd3a66cd20377ac159)
- [Merge pull request #48 from durbanlegend/dependabot/cargo/cargo_toml-0.20.5](https://github.com/durbanlegend/thag_rs/commit/a9ce573f6556623c3022c527f03a79b64f1a31c9)
- [Merge pull request #49 from durbanlegend/dependabot/cargo/regex-1.11.0](https://github.com/durbanlegend/thag_rs/commit/7542713296b4fb80753625c226d9c6839d22e3f1)
- [Merge pull request #50 from durbanlegend/dependabot/cargo/syn-2.0.79](https://github.com/durbanlegend/thag_rs/commit/e41b6bd6b74eeaf16c35d2965d6a79f80160b277)
- [Debug compille-time message style resolution](https://github.com/durbanlegend/thag_rs/commit/6fa7eb4f3406f9cbe6038e60e7da6ad4e9abab52)
- [Clean up mod `colors`.](https://github.com/durbanlegend/thag_rs/commit/43c37abde4716da13d4014ea3fdae7297745fa7d)
- [Remove lazy_static. REPL TUI save a copy.](https://github.com/durbanlegend/thag_rs/commit/a07612562a8d73bc3218c26af5b2e7a480012adf)
- [Repl TUI history. stdin coverage. Test naming.](https://github.com/durbanlegend/thag_rs/commit/3f4cca7b34078fd59651e40f482ed62a168c89c0)
- [Add simplelog option. Minor fixes.](https://github.com/durbanlegend/thag_rs/commit/65d54ef6b4d3fe53cdbe0e722fe6530257ce20ca)
- [Embed a copy of crokey for control](https://github.com/durbanlegend/thag_rs/commit/7fc2b42a6473101d3aeb20747da2b4f33f13df45)
- [Move needed bits of crokey into project](https://github.com/durbanlegend/thag_rs/commit/6541e7a1eb62726b1d5aa3dcfda6756b4397c609)
- [Bump serde_with from 3.9.0 to 3.11.0](https://github.com/durbanlegend/thag_rs/commit/28b3cf6a5b183b232c1cd705d81d8b44f08b4f97)
- [Bump clap from 4.5.18 to 4.5.19](https://github.com/durbanlegend/thag_rs/commit/1e1ad07f9a3e2c5093854773e13be879e7ee3129)
- [Merge pull request #52 from durbanlegend/dependabot/cargo/clap-4.5.19](https://github.com/durbanlegend/thag_rs/commit/a4b309deed0deff5f20721ab385d9ebd3c982749)
- [Merge pull request #51 from durbanlegend/dependabot/cargo/serde_with-3.11.0](https://github.com/durbanlegend/thag_rs/commit/ce26779d7430943056aea28efe34177047acf03a)
- [Implement F4 clear function](https://github.com/durbanlegend/thag_rs/commit/48c823dd7205bcd7e99efb1b8ab87b5c9cb3f628)
- [Merge pull request #53 from durbanlegend/staging](https://github.com/durbanlegend/thag_rs/commit/7942b1351ae64c3a2225a003d234fec2756f2b5d)
- [Create win_test_termbg_thag.rs](https://github.com/durbanlegend/thag_rs/commit/ccb21c667da1b2d7fa26e93f0e857123be170c7c)
- [Make termbg behave on Windows](https://github.com/durbanlegend/thag_rs/commit/3ebc03c638db8adbe46d0b1c547025a3e5cb86db)
- [Bump termbg from 0.5.1 to 0.5.2](https://github.com/durbanlegend/thag_rs/commit/e73b949ad1e4e9e6d5b8b0163f5936f3ee868d0e)
- [Bump proc-macro2 from 1.0.86 to 1.0.87](https://github.com/durbanlegend/thag_rs/commit/87c2c6201c02b104d9ad60d5a3c8a6d46444a825)
- [Bump clap from 4.5.19 to 4.5.20](https://github.com/durbanlegend/thag_rs/commit/9df641ab24463be64d329e4f0a58f84d4fe29114)
- [Merge pull request #56 from durbanlegend/dependabot/cargo/clap-4.5.20](https://github.com/durbanlegend/thag_rs/commit/c3b013aa41387996328a244a74b68b517a6be024)
- [Merge pull request #55 from durbanlegend/dependabot/cargo/proc-macro2-1.0.87](https://github.com/durbanlegend/thag_rs/commit/949364ad8b985d4a8aee155fd0bace37ff256b0f)
- [Merge pull request #54 from durbanlegend/dependabot/cargo/termbg-0.5.2](https://github.com/durbanlegend/thag_rs/commit/3090ee9a79e0bd63fb9f343f33b6d16c667106a3)
- [Investigate Windows light/dark theme detection](https://github.com/durbanlegend/thag_rs/commit/942f30c47cf7311ae4dd5610dd9bbfc0bf5dcdc7)
- [Prepare for potential termbg PR for Windows issues](https://github.com/durbanlegend/thag_rs/commit/97e079d1d7c03680bb6804ad207d742003fcb26d)
- [Create termbg_bug1.rs](https://github.com/durbanlegend/thag_rs/commit/d76088fadb814f569714cb73746900d8193d368f)
- [Merge pull request #57 from durbanlegend/develop](https://github.com/durbanlegend/thag_rs/commit/bf5c3f7968de8da70b4b2162055a12008ca7eac4)

### Notes

- Also REPL alternatives to reedline.

- Bumps [serde](https://github.com/serde-rs/serde) from 1.0.209 to 1.0.210.
   [Release notes](https://github.com/serde-rs/serde/releases)
   [Commits](https://github.com/serde-rs/serde/compare/v1.0.209...v1.0.210)

  --
  updated-dependencies:
   dependency-name: serde
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Save before trying to copy repl.rs

- TODO: Instead change repl.rs edit function to call stdin::eval
  and cherry-pick changes to both modules. E.g. elimination of Context in repl is good I think.

- Bump serde from 1.0.209 to 1.0.210

- Streamline build of CMD_DESC_MAP. Remove dbg!()s interfering with TUI.

- Impl in repl.rs, TODO stdin.rs

- TODO: Implement in repl.rs to replace edit_history.

- Bumps [termbg](https://github.com/dalance/termbg) from 0.5.0 to 0.5.1.
   [Changelog](https://github.com/dalance/termbg/blob/master/CHANGELOG.md)
   [Commits](https://github.com/dalance/termbg/compare/v0.5.0...v0.5.1)

  --
  updated-dependencies:
   dependency-name: termbg
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Making keybindings explicit.

- Bump termbg from 0.5.0 to 0.5.1

- Adjust width based on key & desc lengths.

- ThagResult for repl.rs

- No save file dialog.

- Working on repl TUI feature.

- Bumps [reedline](https://github.com/nushell/reedline) from 0.34.0 to 0.35.0.
   [Release notes](https://github.com/nushell/reedline/releases)
   [Commits](https://github.com/nushell/reedline/compare/v0.34.0...v0.35.0)

  --
  updated-dependencies:
   dependency-name: reedline
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [clap](https://github.com/clap-rs/clap) from 4.5.17 to 4.5.18.
   [Release notes](https://github.com/clap-rs/clap/releases)
   [Changelog](https://github.com/clap-rs/clap/blob/master/CHANGELOG.md)
   [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.5.17...clap_complete-v4.5.18)

  --
  updated-dependencies:
   dependency-name: clap
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bump clap from 4.5.17 to 4.5.18

- Bump reedline from 0.34.0 to 0.35.0

- Replace large async-std dependency by manual loop.
  Also bump reedline to 0.35.0.

- Replace get_mappings with key-mappings! for compile-time execution and speed boost.
  colors.rs changes are probably redundant/overkill.

- Caused by crossterm::enable_raw_mode in non-interactive contexts such as testing.

- Bumps [tempfile](https://github.com/Stebalien/tempfile) from 3.12.0 to 3.13.0.
   [Changelog](https://github.com/Stebalien/tempfile/blob/master/CHANGELOG.md)
   [Commits](https://github.com/Stebalien/tempfile/compare/v3.12.0...v3.13.0)

  --
  updated-dependencies:
   dependency-name: tempfile
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [cargo_toml](https://gitlab.com/lib.rs/cargo_toml) from 0.20.4 to 0.20.5.
   [Commits](https://gitlab.com/lib.rs/cargo_toml/compare/v0.20.4...v0.20.5)

  --
  updated-dependencies:
   dependency-name: cargo_toml
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [regex](https://github.com/rust-lang/regex) from 1.10.6 to 1.11.0.
   [Release notes](https://github.com/rust-lang/regex/releases)
   [Changelog](https://github.com/rust-lang/regex/blob/master/CHANGELOG.md)
   [Commits](https://github.com/rust-lang/regex/compare/1.10.6...1.11.0)

  --
  updated-dependencies:
   dependency-name: regex
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [syn](https://github.com/dtolnay/syn) from 2.0.77 to 2.0.79.
   [Release notes](https://github.com/dtolnay/syn/releases)
   [Commits](https://github.com/dtolnay/syn/compare/2.0.77...2.0.79)

  --
  updated-dependencies:
   dependency-name: syn
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bump tempfile from 3.12.0 to 3.13.0

- Bump cargo_toml from 0.20.4 to 0.20.5

- Bump regex from 1.10.6 to 1.11.0

- Bump syn from 2.0.77 to 2.0.79

- Aiming to replace dynamic resolution while displaying with as much compile-time and up-front resolution as possible.

- Use From trait throughout for style conversions.

- Work on implementing REPL TUI history.
  Standardize use of F7/F8 keys for history scrolling.

- Add module name to test function names to allow testing individual modules.

- Log to file for TUI testing.

- Aiming to get better control over different terminal types and maybe strip out unused features.

- Update history tests to fix.

- Bumps [serde_with](https://github.com/jonasbb/serde_with) from 3.9.0 to 3.11.0.
   [Release notes](https://github.com/jonasbb/serde_with/releases)
   [Commits](https://github.com/jonasbb/serde_with/compare/v3.9.0...v3.11.0)

  --
  updated-dependencies:
   dependency-name: serde_with
    dependency-type: direct:production
    update-type: version-update:semver-minor
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [clap](https://github.com/clap-rs/clap) from 4.5.18 to 4.5.19.
   [Release notes](https://github.com/clap-rs/clap/releases)
   [Changelog](https://github.com/clap-rs/clap/blob/master/CHANGELOG.md)
   [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.5.18...clap_complete-v4.5.19)

  --
  updated-dependencies:
   dependency-name: clap
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bump clap from 4.5.18 to 4.5.19

- Bump serde_with from 3.9.0 to 3.11.0

- Minor clean-up

- Update develop branch with dependency bumps.

- Prove that thag_rs version of termbg doesn't swallow user input.

- No more swallowing first char of input. Leave it up to Microsoft to decide if and when to support Xterm interrogation.

- Bumps [termbg](https://github.com/dalance/termbg) from 0.5.1 to 0.5.2.
   [Changelog](https://github.com/dalance/termbg/blob/master/CHANGELOG.md)
   [Commits](https://github.com/dalance/termbg/compare/v0.5.1...v0.5.2)

  --
  updated-dependencies:
   dependency-name: termbg
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [proc-macro2](https://github.com/dtolnay/proc-macro2) from 1.0.86 to 1.0.87.
   [Release notes](https://github.com/dtolnay/proc-macro2/releases)
   [Commits](https://github.com/dtolnay/proc-macro2/compare/1.0.86...1.0.87)

  --
  updated-dependencies:
   dependency-name: proc-macro2
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bumps [clap](https://github.com/clap-rs/clap) from 4.5.19 to 4.5.20.
   [Release notes](https://github.com/clap-rs/clap/releases)
   [Changelog](https://github.com/clap-rs/clap/blob/master/CHANGELOG.md)
   [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.5.19...clap_complete-v4.5.20)

  --
  updated-dependencies:
   dependency-name: clap
    dependency-type: direct:production
    update-type: version-update:semver-patch
  ...

  Signed-off-by: dependabot[bot] <support@github.com>

- Bump clap from 4.5.19 to 4.5.20

- Bump proc-macro2 from 1.0.86 to 1.0.87

- Bump termbg from 0.5.1 to 0.5.2

- WIndows Terminal 1.22 preview finally introduces support for *querying* the background and foreground colours via xterm OSC. The from_winapi() only works if the colour was set via the console interface, so is largely useless.

- Temporarily re-export my GitHub fork of termbg instead of using the built-in module.
  This is to test the potential termbg PR as thoroughly as possible.

- Reinstate

- Prepare for thag 0.1.5

## v0.1.4 (2024-09-06)

- [Prepare for replacement release 0.1.4](https://github.com/durbanlegend/thag_rs/commit/57697a5073e03e4b5ef0ed28092bbc1380f6eb2d)

### Notes

- Because crates.io releases are immutable and I've made tweaks

### Notes

- Because crates.io releases are immutable and I've made tweaks
## [0.1.3] - 2024-09-05

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

## v0.1.2 (2024-08-24)

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
