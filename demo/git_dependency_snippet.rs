/*[toml]
[dependencies]
# https://github.com/rust-lang/cargo/issues/8555
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_support", "core", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_support", "core", "simplelog"] }
*/

/// `demo/git_dependency.rs` done as a snippet, just because.
//# Purpose: Demo `git` dependencies as a snippet.
//# Categories: technique
use thag_rs::colors;

// The colors module has a "redundant" public main method for this demo.
colors::main();
