/*[toml]
[dependencies]
# rev may be necessary to force cargo to update to the latest commit, see
# https://github.com/rust-lang/cargo/issues/8555
thag_rs = { git = "https://github.com/durbanlegend/thag_rs"}
*/

/// Demo the use of git dependencies in the toml block. Local path dependencies
/// work the same way, e.g. `thag_rs = { git = "https://github.com/durbanlegend/thag_rs" },
/// but obviously the path literal will be specific to your environment.
//# Purpose: Demo `git` dependencies, explain `path` dependencies.
use thag_rs::colors;

fn main() {
    colors::main();
}
