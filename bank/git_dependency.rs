/*[toml]
[dependencies]
# rev may be necessary to force cargo to update to the latest commit, see
# https://github.com/rust-lang/cargo/issues/8555
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_support", "core", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_support", "core", "simplelog"] }
*/

/// Demo the use of git dependencies in the toml block. Local path dependencies
/// work the same way, e.g. `thag_rs = { path = "<path/to-project>/thag_rs" },
/// but obviously the path literal will be specific to your environment.
//# Purpose: Demo `git` dependencies, explain `path` dependencies.
//# Categories: technique
use thag_rs::colors;

fn main() {
    colors::main();
}
